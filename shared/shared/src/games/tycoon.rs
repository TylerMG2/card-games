use serde::{Deserialize, Serialize};
use shared_core::{Connection, GameLogic};
use shared_macros::{PlayerFields, RoomFields};

#[derive(Deserialize, Serialize, Default, Clone, Copy, PartialEq)]
pub enum RoomState {
    #[default]
    Lobby,
    CardExchange,
    Game,
}

// I don't know how much I like this at the moment, the previous way was a ClientRoom struct that had a room field
// but that required a generic type. This was is more verbose to define initially but less verbose to use (no need)
// to access everything through client_room.room
#[derive(RoomFields, Clone, Copy, Deserialize, Serialize, Default)]
pub struct TycoonRoom {
    pub turn: u8,
    pub last_played: u64,
    pub last_played_player: u8,
    pub revolution: bool,
    pub state: RoomState,

    #[host]
    pub host: u8,
    #[players] 
    pub players: [Option<TycoonPlayer>; 8],
    #[player_index] 
    pub player_index: u8,
}

#[derive(PlayerFields, Clone, Copy, Deserialize, Serialize, Default)]
pub struct TycoonPlayer {
    pub hand: u64,
    pub num_cards: u8,

    #[name] 
    pub name: [u8; 20], 
    #[disconnected] 
    pub disconnected: bool,
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

impl GameLogic for TycoonRoom {
    type GameServerEvent = TycoonServerEvent;
    type GameClientEvent = TycoonClientEvent;

    fn validate_client_game_event(&self, event: &Self::GameClientEvent, player_index: usize) -> bool {
        match event {
            TycoonClientEvent::StartGame => {
                let num_players = self.players.iter().filter(|player| player.is_some()).count();

                self.state == RoomState::Lobby && num_players >= 3 && player_index == self.host as usize
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

    fn handle_client_game_event(&mut self, event: &Self::GameClientEvent, connections: &[Option<Connection>; 8], player_index: usize) {
        match event {
            TycoonClientEvent::StartGame => {
                todo!("Start game");
            },
            TycoonClientEvent::PlayCards { cards } => {
                self.send_to_all_game_event(&TycoonServerEvent::CardsPlayed { cards: *cards }, connections);
            },
            TycoonClientEvent::Pass => {
                self.send_to_all_game_event(&TycoonServerEvent::Pass, connections);
            },
            TycoonClientEvent::ExchangeCards { cards } => {
                todo!("Exchange cards");
            },
        }
    }

    fn handle_server_game_event(&mut self, event: &Self::GameServerEvent, player_index: Option<usize>, is_server_side: bool) {;
        match event {
            TycoonServerEvent::GameStarted { turn, cards, other_hands } => {
                self.turn = *turn;
                self.last_played = 0;
                self.last_played_player = 0;
                self.revolution = false;

                for (index, player) in self.players.iter_mut().enumerate() {
                    if let Some(player) = player.as_mut() {
                        player.hand = 0;
                        player.num_cards = other_hands[index];
                    }
                }

                if let Some(player_index) = player_index {
                    if let Some(Some(player)) = self.players.get_mut(player_index) {
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
