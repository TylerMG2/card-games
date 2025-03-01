use serde::{Deserialize, Serialize};
use crate::{traits::{self, GameSignal}, types::{self, SignalType}};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Duke,
    Assassin,
    Captain,
    Ambassador,
    Contessa,

    #[default]
    Unknown,    
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Card {
    pub role: SignalType<Role>,
    pub revealed: SignalType<bool>,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct CoupRoom {
    pub turn: SignalType<u8>,
    pub deck: [Card; 15],
    pub last_action: SignalType<Option<CoupAction>>,
    pub last_counteraction: Option<Role>,
    pub challenge: SignalType<Option<(u8, u8)>>, // (player, challenger)
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct CoupPlayer {
    pub coins: SignalType<u8>,
    pub cards: [Card; 2],
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum CoupServerEvent {
    GameStarted { turn: u8, cards: [Card; 2] },
    ActionTaken { player: u8, action: CoupAction },
    ActionChallenged { player: u8, challenger: u8 },
    CardRevealed { player: u8, card: u8 }, // 0 or 1
    Counteraction { player: u8, claim: Role }, // Counters last action
    CounteractionChallenged { player: u8, challenger: u8 }, // Challenger challenges counteraction
    // Person who was challenged reveals a card, clients can work out if they were correct
    // I would like to allow the person being challenge to lie about not having the card
    // which is why the person being challenge should be able to choose which card to reveal 
    // regardless of the wether they have the correct card or not
    ChallengeRevealed { player: u8, card: u8 }, 

}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum CoupAction {
    Income, // Gain 1 coin
    ForeignAid, // Gain 2 coins
    Coup, // Pay 7 coins to eliminate a player
    Tax { target: u8 }, // Duke
    Assassinate { target: u8 }, // Assassin
    Exchange, // Ambassador
    Steal { target: u8 }, // Captain
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum CoupClientEvent {
    StartGame,
    TakeAction { action: CoupAction },
    Counteraction { claim: Role },
    ChallengeAction,
    RevealCard { card: u8 },
}

impl traits::GameLogic for CoupRoom {
    type GameServerEvent = CoupServerEvent;
    type GameClientEvent = CoupClientEvent;
    type Room = CoupRoom;
    type Player = CoupPlayer;

    fn validate_client_game_event(room: &types::Room, event: &Self::GameClientEvent, player_index: usize) -> bool {
        match event {
            CoupClientEvent::StartGame => {
                *room.host.value() == player_index as u8
            },
            CoupClientEvent::TakeAction { action } => {
                *room.coup.turn.value() == player_index as u8
            },
            CoupClientEvent::Counteraction { claim } => {
                todo!("Validate counteraction");
            },
            CoupClientEvent::ChallengeAction => {
                if let Some(player) = room.players.get(player_index) {
                    if let Some(player) = player.value() {
                        player.coup.cards.iter().any(|card| !card.revealed.value())
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            CoupClientEvent::RevealCard { card } => {
                *room.coup.turn.value() == player_index as u8
                    && matches!(room.coup.challenge.value(), Some((challenger, _)) if *challenger == player_index as u8)
            }
            
        }
    }

    fn handle_client_game_event(room: &mut types::Room, event: &Self::GameClientEvent, connections: &mut impl traits::Networking, player_index: usize) {
        if Self::validate_client_game_event(room, event, player_index) {
            match event {
                CoupClientEvent::StartGame => {
                    todo!("Handle game started");
                },
                CoupClientEvent::TakeAction { action } => {
                    todo!("Handle action taken");
                },
                CoupClientEvent::Counteraction { claim } => {
                    todo!("Handle counteraction");
                },
                CoupClientEvent::ChallengeAction => {
                    todo!("Handle challenge action");
                },
                CoupClientEvent::RevealCard { card } => {
                    todo!("Handle reveal card");
                }
            }
        }
    }

    fn handle_server_game_event(room: &mut types::Room, event: &Self::GameServerEvent, player_index: Option<usize>, is_server_side: bool) {
        match event {
            CoupServerEvent::GameStarted { turn, cards } => {
                todo!("Handle game started");
            }
            CoupServerEvent::ActionTaken { player, action } => {
                todo!("Handle action taken");
            }
            CoupServerEvent::ActionChallenged { player, challenger } => {
                todo!("Handle action challenged");
            }
            CoupServerEvent::CardRevealed { player, card } => {
                todo!("Handle card revealed");
            }
            CoupServerEvent::Counteraction { player, claim } => {
                todo!("Handle counteraction");
            }
            CoupServerEvent::CounteractionChallenged { player, challenger } => {
                todo!("Handle counteraction challenged");
            }
            CoupServerEvent::ChallengeRevealed { player, card } => {
                todo!("Handle challenge revealed");
            }
        }
    }

    fn wrap_game_event(event: Self::GameServerEvent) -> types::ServerEvent {
        types::ServerEvent::CoupEvent(event)
    }
}
