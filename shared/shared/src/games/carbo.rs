use serde::{Deserialize, Serialize};
use shared_core::GameLogic;
use shared_macros::{PlayerFields, RoomFields};


#[derive(Deserialize, Serialize, Default, Clone, Copy, PartialEq)]
pub enum CarboRoomState {
    #[default]
    Lobby,
}

#[derive(RoomFields, Clone, Copy, Deserialize, Serialize, Default)]
pub struct CarboRoom {
    pub turn: u8,
    pub state: CarboRoomState,

    #[host] 
    pub host: u8,
    #[players] 
    pub players: [Option<CarboPlayer>; 8],
    #[player_index] 
    pub player_index: u8,
}

#[derive(PlayerFields, Clone, Copy, Deserialize, Serialize, Default)]
pub struct CarboPlayer {
    pub visible_cards: u64,
    pub num_cards: u8,

    #[name] 
    pub name: [u8; 20], 
    #[disconnected] 
    pub disconnected: bool,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum CarboServerEvent {
    GameStarted { turn: u8, cards: u64 },
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum CarboClientEvent {
    StartGame,
    PlayCard { card: u64 },
}

impl GameLogic for CarboRoom {
    type GameServerEvent = CarboServerEvent;
    type GameClientEvent = CarboClientEvent;

    fn validate_client_game_event(&self, event: &Self::GameClientEvent, player_index: usize) -> bool {
        match event {
            CarboClientEvent::StartGame => {
                let num_players = self.players.iter().filter(|player| player.is_some()).count();

                self.state == CarboRoomState::Lobby && num_players >= 3 && player_index == self.host as usize
            },
            CarboClientEvent::PlayCard { card } => {
                todo!("Validate card play");
            },
        }
    }

    fn handle_client_game_event(&mut self, event: &Self::GameClientEvent, connections: &[Option<shared_core::Connection>; 8], player_index: usize) {
        match event {
            CarboClientEvent::StartGame => {
                todo!("Start game");
            },
            CarboClientEvent::PlayCard { card } => {
                todo!("Play card");
            },
        }
    }

    fn handle_server_game_event(&mut self, event: &Self::GameServerEvent, player_index: Option<usize>) {
        match event {
            CarboServerEvent::GameStarted { turn, cards } => {
                todo!("Game started");
            },
        }
    }
}
