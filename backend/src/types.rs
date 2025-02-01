use shared::{traits::{NetworkingSend, ToFromBytes}, types};
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
                println!("Failed to send event to player {}, closing connection", self.id);
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
    pub fn add_connection(&mut self, tx: UnboundedSender<Vec<u8>>, id: uuid::Uuid) -> Option<usize> {
        let mut first_free: Option<usize> = None;

        // Look for player id while keeping track of the first free slot
        for (index, connection) in self.connections.iter_mut().enumerate() {
            if connection.is_none() && first_free.is_none() {
                first_free = Some(index);
            }

            if let Some(connection) = connection {
                if connection.id == id {
                    connection.sender = Some(tx);
                    return Some(index);
                }
            }
        }

        if let Some(index) = first_free {
            self.connections[index] = Some(Connection { id, sender: Some(tx) });
            Some(index)
        } else {
            None
        }
    }
}