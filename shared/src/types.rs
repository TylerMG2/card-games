use tokio::sync::mpsc::UnboundedSender;

use crate::{events::{ServerEvent, ToFromBytes}, traits::{GameLogic, Player}};

pub enum GameType {
    Tycoon,
    Carbo,
    Default,
}

pub struct ClientRoom<RoomType: Default + Copy, PlayerType: Player + Default + Copy> {
    pub players: [Option<PlayerType>; 8], // TODO: Move to a constant
    pub host: u8,
    pub game: GameType,
    pub room: RoomType,
}

pub struct ServerRoom<Logic: GameLogic> {
    pub connections: [Option<Connection>; 8], // TODO: Move to a constant
    pub client_room: ClientRoom<Logic::Room, Logic::Player>,
    pub logic: Logic,
}

pub struct Connection {
    pub id: uuid::Uuid,
    pub sender: Option<UnboundedSender<Vec<u8>>>,
}

impl<Logic> ServerRoom<Logic>
where
    Logic: GameLogic,
{
    pub fn new(logic: Logic) -> Self {
        Self {
            connections: [const { None }; 8],
            client_room: ClientRoom {
                players: [None; 8],
                host: 0,
                game: GameType::Default,
                room: Logic::Room::default(),
            },
            logic,
        }
    }

    pub fn send_to_all(&mut self, event: &ServerEvent<Logic::GameServerEvent>) {
        handle_server_event(self, event, None);

        for connection in self.connections.iter() {
            if let Some(connection) = connection {
                if let Some(sender) = &connection.sender {
                    sender.send(event.to_bytes()).unwrap(); // TODO: Handle send error
                }
            }
        }
    }

    pub fn send_except(&mut self, event: &ServerEvent<Logic::GameServerEvent>, except: usize) {
        handle_server_event(self, event, None);

        for (index, connection) in self.connections.iter().enumerate() {
            if index != except {
                if let Some(connection) = connection {
                    if let Some(sender) = &connection.sender {
                        sender.send(event.to_bytes()).unwrap();
                    }
                }
            }
        }
    }

    pub fn send_to(&mut self, event: &ServerEvent<Logic::GameServerEvent>, to: usize) {
        handle_server_event(self, event, Some(to));

        if let Some(connection) = self.connections[to].as_ref() {
            if let Some(sender) = &connection.sender {
                sender.send(event.to_bytes()).unwrap();
            }
        }
    }
}

fn handle_server_event<Logic: GameLogic>(room: &mut ServerRoom<Logic>, event: &ServerEvent<Logic::GameServerEvent>, player_index: Option<usize>) {
    match event {
        ServerEvent::GameEvent(event) => {
            room.logic.handle_server_event(&event, &mut room.client_room.room, &mut room.client_room.players, player_index);
        },
        ServerEvent::Unknown => panic!("Should never send a ServerEvent::Unknown"), 
        _ => {}, // TODO: Implement normal event handling
    }
}