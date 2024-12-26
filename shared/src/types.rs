use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{games::{get_logic, GameLogicType}, traits::GameLogic};

#[derive(Deserialize, Serialize, Clone, Copy, Default)]
pub enum GameType {

    #[default]
    Tycoon,
    Carbo,
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Default)]
pub enum RoomState {
    #[default]
    Lobby,
    InGame,
}

//
// Players
//
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct ClientPlayer<PlayerType> {
    pub name: [u8; 20], // TODO: Move to a constant
    pub disconnected: bool,
    pub player: PlayerType,
}


//
// Rooms
//

#[derive(Deserialize, Serialize, Clone, Copy, Default)]
pub struct ClientRoom<RoomType, PlayerType> {
    pub players: [Option<ClientPlayer<PlayerType>>; 8], // TODO: Move to a constant
    pub host: u8,
    pub game: GameType,
    pub room: RoomType,
    pub state: RoomState,
    pub current_player: Option<u8>, // This isn't used by the server, only needed by the client
}

#[derive(Default)]
pub struct ServerRoom {
    pub connections: [Option<Connection>; 8], // TODO: Move to a constant
    pub logic: GameLogicType,
}

pub struct Connection {
    pub id: uuid::Uuid,
    pub sender: Option<UnboundedSender<Vec<u8>>>,
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

//
// Events
//

#[derive(Default, Serialize, Deserialize, Clone)]
pub enum ServerEvent<Logic: GameLogic> {
    RoomJoined { room: ClientRoom<Logic::Room, Logic::Player>, current_player: u8 },
    PlayerJoined { name: [u8; 20], player_index: u8 }, // TODO: Move to a constant
    PlayerLeft { player_index: u8 },
    PlayerDisconnected { player_index: u8 },
    PlayerReconnected { player_index: u8 },
    HostChanged { player_index: u8 },
    GameChanged { game: GameType },
    GameEvent(Logic::GameServerEvent),

    #[default]
    Unknown,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub enum ClientEvent<T> {
    JoinRoom { name: [u8; 20] }, // TODO: Move to a constant
    LeaveRoom,
    ChangeGame { game: GameType },
    GameEvent(T),

    #[default]
    Unknown,
}