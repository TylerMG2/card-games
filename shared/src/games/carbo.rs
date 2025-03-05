use crate::{traits, types};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone, Copy, PartialEq, Debug)]
pub enum CarboRoomState {
    #[default]
    Lobby,
}

#[derive(Clone, Copy, Deserialize, Serialize, Default, Debug)]
pub struct CarboRoom {
    pub turn: u8,
    pub state: CarboRoomState,
}

#[derive(Clone, Copy, Deserialize, Serialize, Default, Debug)]
pub struct CarboPlayer {
    pub visible_cards: u64,
    pub num_cards: u8,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum CarboServerEvent {
    GameStarted { turn: u8, cards: u64 },
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum CarboClientEvent {
    StartGame,
    PlayCard { card: u64 },
}

impl traits::GameLogic for CarboRoom {
    type GameServerEvent = CarboServerEvent;
    type GameClientEvent = CarboClientEvent;
    type Room = CarboRoom;
    type Player = CarboPlayer;

    fn validate_client_game_event(
        room: &types::Room,
        event: &CarboClientEvent,
        player_index: usize,
    ) -> bool {
        match event {
            CarboClientEvent::StartGame => room.carbo.state == CarboRoomState::Lobby,
            CarboClientEvent::PlayCard { card } => {
                todo!("Validate card played");
            }
        }
    }

    fn handle_client_game_event(
        room: &mut types::Room,
        event: &Self::GameClientEvent,
        connections: &mut impl traits::Networking,
        player_index: usize,
    ) {
        if Self::validate_client_game_event(room, event, player_index) {
            match event {
                CarboClientEvent::StartGame => {
                    todo!("Handle game started");
                }
                CarboClientEvent::PlayCard { card } => {
                    todo!("Handle card played");
                }
            }
        }
    }

    fn handle_server_game_event(
        room: &mut types::Room,
        event: &CarboServerEvent,
        player_index: Option<usize>,
        is_server_side: bool,
    ) {
        match event {
            CarboServerEvent::GameStarted { turn, cards } => {
                todo!("Handle game started");
            }
        }
    }

    fn wrap_game_event(event: Self::GameServerEvent) -> types::ServerEvent {
        types::ServerEvent::CarboEvent(event)
    }
}
