use serde::{Deserialize, Serialize};

use crate::games::{carbo, tycoon};
use crate::traits;

pub const MAX_PLAYERS: usize = 8;
pub const MAX_NAME_LENGTH: usize = 20;

pub struct ClientConnection;

#[derive(Deserialize, Serialize, Clone, Copy, Default, PartialEq, Debug)]
pub enum GameType {
    #[default]
    Tycoon,
    Carbo,
}

#[derive(Deserialize, Serialize, Clone, Copy, Default, PartialEq, Debug)]
pub enum RoomState {
    #[default]
    Lobby,
    InGame,
}

#[derive(Default, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Room {
    pub state: RoomState,
    pub game: GameType,
    pub host: u8,
    pub player_index: u8,
    pub carbo: carbo::CarboRoom,
    pub tycoon: tycoon::TycoonRoom,
    pub players: [Option<Player>; MAX_PLAYERS],
}

#[derive(Default, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Player {
    pub common: CommonPlayer,
    pub carbo: carbo::CarboPlayer,
    pub tycoon: tycoon::TycoonPlayer,
}

#[derive(Default, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CommonPlayer {
    pub name: [u8; MAX_NAME_LENGTH],
    pub disconnected: bool,
}

//
// Event types
//

// TODO: Use a macro to generate the client events
// If I use the form <game::GameRoom as GameLogic>::GameServerEvent, I can probably write a macro to generate the
// Room, Player and Event types rather then having to update each one each time I add a new game
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub enum ServerEvent {
    CommonEvent(CommonServerEvent),
    CarboEvent(<carbo::CarboRoom as traits::GameLogic>::GameServerEvent),
    TycoonEvent(<tycoon::TycoonRoom as traits::GameLogic>::GameServerEvent),

    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)] //TODO: Clean up derives
pub enum CommonServerEvent {
    RoomJoined { new_room: Room, current_player: u8 },
    PlayerJoined { name: [u8; MAX_NAME_LENGTH], player_index: u8 },
    PlayerLeft { player_index: u8 },
    PlayerDisconnected { player_index: u8 },
    PlayerReconnected { player_index: u8 },
    HostChanged { player_index: u8 },
    GameChanged { game: GameType },
}

// TODO: Use a macro to generate the client events
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub enum ClientEvent {
    CommonEvent(CommonClientEvent),
    CarboEvent(<carbo::CarboRoom as traits::GameLogic>::GameClientEvent),
    TycoonEvent(<tycoon::TycoonRoom as traits::GameLogic>::GameClientEvent),

    #[default]
    Unknown,
}

// TODO: Reset current game action
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CommonClientEvent {
    JoinRoom { name: [u8; MAX_NAME_LENGTH] },
    LeaveRoom,
    ChangeGame { game: GameType },
    Disconnect,
}