use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{games::get_logic, traits::{GameLogic, Networked, ToFromBytes}};

#[derive(Deserialize, Serialize, Clone, Copy, Default)]
pub enum GameType {

    #[default]
    Tycoon,
    Carbo,
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

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct ClientRoom<RoomType, PlayerType> {
    pub players: [Option<ClientPlayer<PlayerType>>; 8], // TODO: Move to a constant
    pub host: u8,
    pub game: GameType,
    pub room: RoomType,
    pub current_player: Option<u8>, // This isn't used by the server, only needed by the client
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

//
// Events
//

#[derive(Default, Serialize, Deserialize, Clone)]
pub enum ServerEvent<Logic: GameLogic> {
    RoomJoined { room: ClientRoom<Logic::Room, Logic::Player>, current_player: u8 },
    PlayerJoined { player: ClientPlayer<Logic::Player>, player_index: u8 },
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
    StartGame,
    ChangeGame { game: GameType },
    GameEvent(T),

    #[default]
    Unknown,
}

fn new_server_room(game_type: GameType) -> ServerRoom<impl GameLogic> {
    let logic = get_logic(game_type);

    ServerRoom {
        connections: [const { None }; 8],
        client_room: ClientRoom {
            players: [None; 8],
            host: 0,
            game: game_type,
            room: logic.default_room(),
            current_player: None,
        },
        logic,
    }
}

// Take ownership of the room (to avoid reuse) and return a new room with the updated game type
fn switch_game_mode(room: ServerRoom<impl GameLogic>, game: GameType) -> ServerRoom<impl GameLogic> {
    let logic = get_logic(game);

    // Create new players array replacing the player field with the default player of the new logic type
    let mut players = [None; 8];
    for (index, player) in room.client_room.players.iter().enumerate() {
        if let Some(player) = player {
            players[index] = Some(ClientPlayer {
                name: player.name,
                disconnected: player.disconnected,
                player: logic.default_player(),
            });
        }
    }

    ServerRoom {
        connections: room.connections,
        client_room: ClientRoom {
            players,
            host: room.client_room.host,
            game,
            room: logic.default_room(),
            current_player: room.client_room.current_player,
        },
        logic,
    }
}