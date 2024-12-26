use serde::{Deserialize, Serialize};

use crate::{traits::GameLogic, types::{self, ServerRoom}, ClientRoom};

#[derive(Clone, Default)]
pub struct CarboLogic {
    client_room: ClientRoom<CarboRoom, CarboPlayer>,
}

#[derive(Deserialize, Serialize, Default, Clone, Copy, PartialEq)]
pub enum CarboRoomState {
    #[default]
    Lobby,
}

#[derive(Clone, Copy, Deserialize, Serialize, Default)]
pub struct CarboRoom {
    pub turn: u8,
    pub state: CarboRoomState,
}

#[derive(Clone, Copy, Deserialize, Serialize, Default)]
pub struct CarboPlayer {
    pub hand: u64,
    pub num_cards: u8,
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

impl GameLogic for CarboLogic {
    type GameServerEvent = CarboServerEvent;
    type GameClientEvent = CarboClientEvent;
    type Room = CarboRoom;
    type Player = CarboPlayer;

    fn validate_client_game_event(&self, event: &Self::GameClientEvent, room: &Self::Room, players: &[Option<&Self::Player>; 8], player_index: usize) -> bool {
        match event {
            CarboClientEvent::StartGame => {
                let num_players = players.iter().filter(|player| player.is_some()).count();

                room.state == CarboRoomState::Lobby && num_players >= 3 // TODO: Host check
            },
            CarboClientEvent::PlayCard { card } => {
                todo!("Validate card play");
            },
        }
    }

    fn handle_client_game_event(&mut self, event: &Self::GameClientEvent, connections: &[Option<types::Connection>; 8], player_index: usize) {
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

    fn get_client_room(&self) -> &ClientRoom<Self::Room, Self::Player> {
        &self.client_room
    }

    fn get_client_room_mut(&mut self) -> &mut ClientRoom<Self::Room, Self::Player> {
        &mut self.client_room
    }
}
