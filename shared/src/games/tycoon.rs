use serde::{Deserialize, Serialize};

use crate::{traits::GameLogic, types::ServerRoom};

#[derive(Clone)]
pub struct TycoonLogic {
}

#[derive(Deserialize, Serialize, Default, Clone, Copy, PartialEq)]
pub enum RoomState {
    #[default]
    Lobby,
    CardExchange,
    Game,
}

#[derive(Clone, Copy, Deserialize, Serialize, Default)]
pub struct TycoonRoom {
    pub turn: u8,
    pub last_played: u64,
    pub last_played_player: u8,
    pub revolution: bool,
    pub state: RoomState,
}

#[derive(Clone, Copy, Deserialize, Serialize, Default)]
pub struct TycoonPlayer {
    pub hand: u64,
    pub num_cards: u8,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum TycoonServerEvent {
    GameStarted { turn: u8, cards: u64, other_hands: [u8; 8] }, // TODO: Move to a constant
    CardsPlayed { cards: u64 },
    Pass,
    ReceiveCards { cards: u64 },
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum TycoonClientEvent {
    StartGame,
    PlayCards { cards: u64 },
    Pass,
    ExchangeCards { cards: u64 },
}

impl GameLogic for TycoonLogic {
    type GameServerEvent = TycoonServerEvent;
    type GameClientEvent = TycoonClientEvent;
    type Room = TycoonRoom;
    type Player = TycoonPlayer;

    fn validate_client_game_event(&self, event: &Self::GameClientEvent, room: &Self::Room, players: &[Option<&Self::Player>; 8], player_index: usize) -> bool {
        match event {
            TycoonClientEvent::StartGame => {
                let num_players = players.iter().filter(|player| player.is_some()).count();

                room.state == RoomState::Lobby && num_players >= 3 //TODO: player_index == room.host as usize && 
            },
            TycoonClientEvent::PlayCards { cards } => {
                todo!("Validate cards played");
            }
            TycoonClientEvent::Pass => {
                todo!("Validate Pass");
            }
            TycoonClientEvent::ExchangeCards { cards } => {
                todo!("Validate Exchange cards");
            }
        }
    }

    fn handle_client_game_event(&self, event: &Self::GameClientEvent, room: &mut ServerRoom<Self>, player_index: usize) {
        match event {
            TycoonClientEvent::StartGame => {
                todo!("Start game");
            },
            TycoonClientEvent::PlayCards { cards } => {
                self.send_to_all_game_event(&TycoonServerEvent::CardsPlayed { cards: *cards }, room);
            },
            TycoonClientEvent::Pass => {
                self.send_to_all_game_event(&TycoonServerEvent::Pass, room);
            },
            TycoonClientEvent::ExchangeCards { cards } => {
                todo!("Exchange cards");
            },
        }
    }

    fn handle_server_event(&self, event: &Self::GameServerEvent, room: &mut Self::Room, players: &mut [Option<&mut Self::Player>; 8], player_index: Option<usize>) {
        match event {
            TycoonServerEvent::GameStarted { turn, cards, other_hands } => {
                room.turn = *turn;
                room.last_played = 0;
                room.last_played_player = 0;
                room.revolution = false;

                for (index, player) in players.iter_mut().enumerate() {
                    if let Some(player) = player.as_deref_mut() {
                        player.hand = 0;
                        player.num_cards = other_hands[index];
                    }
                }

                if let Some(player_index) = player_index {
                    if let Some(Some(player)) = players.get_mut(player_index) {
                        player.hand = *cards;
                        player.num_cards = player.hand.count_ones() as u8;
                    }
                } else {
                    println!("Player index not found for game start event");
                }
            },
            TycoonServerEvent::CardsPlayed { cards } => {
                todo!("Cards played");
            },
            TycoonServerEvent::Pass => {
                todo!("Pass");
            },
            TycoonServerEvent::ReceiveCards { cards } => {
                todo!("Receive cards");
            },
        }
    }
}
