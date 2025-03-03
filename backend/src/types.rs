use shared::{
    traits::{Networking, NetworkingSend, ToFromBytes},
    types::{self, MAX_NAME_LENGTH},
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct Connection {
    pub id: uuid::Uuid,
    pub sender: Option<UnboundedSender<Vec<u8>>>,
}

impl NetworkingSend for Connection {
    fn send(&mut self, event: &types::ServerEvent) {
        if let Some(sender) = &self.sender {
            if sender.send(event.to_bytes()).is_err() {
                // Should force rx.recv() to return None as long as there are no other references to the sender
                println!(
                    "Failed to send event to player {}, closing connection",
                    self.id
                );
                self.sender = None;
            }
        }
        // Player already disconnected
    }
}

#[derive(Default)]
pub struct ServerRoom {
    pub connections: [Option<Connection>; types::MAX_PLAYERS],
    pub room: types::Room,
}

impl ServerRoom {
    //TODO: let's make this return a Result instead of an Option, so we can return an error if the room is full
    pub fn handle_connection(
        &mut self,
        tx: UnboundedSender<Vec<u8>>,
        id: uuid::Uuid,
        name: Option<[u8; MAX_NAME_LENGTH]>,
    ) -> Option<usize> {
        let mut first_free: Option<usize> = None;

        // First we check if the player is already in the room, otherwise we add them at the first free spot
        for (index, connection) in self.connections.iter_mut().enumerate() {
            if let Some(connection) = connection {
                if connection.id == id {
                    connection.sender = Some(tx);
                    println!("Player {} reconnected", id);
                    self.connections.send_to_all_except(
                        &mut self.room,
                        types::ServerEvent::CommonEvent(
                            types::CommonServerEvent::PlayerReconnected {
                                player_index: index as u8,
                            },
                        ),
                        index,
                    );
                    return Some(index);
                }
            } else if first_free.is_none() {
                first_free = Some(index);
            }
        }

        if let Some(index) = first_free {
            if let Some(name) = name {
                self.connections[index] = Some(Connection {
                    id,
                    sender: Some(tx),
                });
                self.connections.send_to_all_except(
                    &mut self.room,
                    types::ServerEvent::CommonEvent(types::CommonServerEvent::PlayerJoined {
                        name,
                        player_index: index as u8,
                    }),
                    index,
                );
                return Some(index);
            }
        }

        None
    }
}
