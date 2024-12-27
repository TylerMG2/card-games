use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{traits::GameLogic, RoomFields};

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
// #[derive(Deserialize, Serialize, Clone, Copy)]
// pub struct ClientPlayer<PlayerType> {
//     pub name: [u8; 20], // TODO: Move to a constant
//     pub disconnected: bool,
//     pub player: PlayerType,
// }


//
// Rooms
//

// #[derive(Deserialize, Serialize, Clone, Copy, Default)]
// pub struct ClientRoom<RoomType, PlayerType> {
//     pub players: [Option<ClientPlayer<PlayerType>>; 8], // TODO: Move to a constant
//     pub host: u8,
//     pub game: GameType,
//     pub room: RoomType,
//     pub state: RoomState,
//     pub current_player: Option<u8>, // This isn't used by the server, only needed by the client
// }

pub struct Connection {
    pub id: uuid::Uuid,
    pub sender: Option<UnboundedSender<Vec<u8>>>,
}

//
// Events
//

#[derive(Default, Serialize, Clone, Deserialize)]
#[serde(bound(deserialize = "Logic: GameLogic + DeserializeOwned"))] // Prevent serde adding Deserialize bounds to Logic
pub enum ServerEvent<Logic: GameLogic> {
    RoomJoined { room: Logic, current_player: u8 },
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