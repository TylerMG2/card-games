use crate::{
    helpers::*,
    traits::{self, GameSignal},
    types::{self, SignalType},
};
use serde::{Deserialize, Serialize};

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
    //TODO: Maybe pub eliminated: SignalType<bool>,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct CoupRoom {
    pub turn: SignalType<u8>,
    pub deck: [Card; 15],
    pub last_action: SignalType<Option<PlayerAction>>,
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
    Action { player: u8, action: PlayerAction },
    Counteraction { player: u8, claim: Role },
    Challenge { player: u8 },
    CardRevealed { player: u8, card: u8 }, // 0 or 1
    // Counters last action
    // Person who was challenged reveals a card, clients can work out if they were correct
    // I would like to allow the person being challenge to lie about not having the card
    // which is why the person being challenge should be able to choose which card to reveal
    // regardless of the wether they have the correct card or not
    ChallengeRevealed { player: u8, card: u8 },
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub struct PlayerAction {
    action: ActionType,
    player: u8,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum ActionType {
    Action(PlayerActionType),
    Counteraction { claim: Role, against: u8 },
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum PlayerActionType {
    Income,                     // Gain 1 coin
    ForeignAid,                 // Gain 2 coins
    Coup,                       // Pay 7 coins to eliminate a player
    Tax,                        // Duke
    Assassinate { target: u8 }, // Assassin
    Exchange,                   // Ambassador
    Steal { target: u8 },       // Captain
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum CoupClientEvent {
    Action { action: PlayerAction },
    Counteraction { claim: Role },
    Challenge,
    ResolveChallenge { card: u8 },
    RevealCard { card: u8 },
}

impl traits::GameLogic for CoupRoom {
    type GameServerEvent = CoupServerEvent;
    type GameClientEvent = CoupClientEvent;
    type Room = CoupRoom;
    type Player = CoupPlayer;

    fn validate_client_game_event(
        room: &types::Room,
        event: &Self::GameClientEvent,
        player_index: usize,
    ) -> bool {
        if !is_player_alive(room, player_index) {
            return false;
        }

        match event {
            CoupClientEvent::Action { action: _ } => {
                *room.coup.turn.value() == player_index as u8 && !has_unresolved_challenge(room)
            }
            CoupClientEvent::Counteraction { claim } => {
                if has_unresolved_challenge(room) {
                    return false;
                }

                let last_action = match room.coup.last_action.value() {
                    Some(action) => action,
                    None => return false,
                };

                if last_action.player == player_index as u8 {
                    return false; // Can't counter if the last action was taken by us
                }

                let last_action = match last_action.action {
                    ActionType::Action(action) => action,
                    _ => return false,
                };

                // Check if the action is counterable and if the claim is correct
                match last_action {
                    PlayerActionType::ForeignAid => *claim == Role::Duke,
                    PlayerActionType::Steal { target } => {
                        (*claim == Role::Captain || *claim == Role::Ambassador)
                            && target == player_index as u8
                    }
                    PlayerActionType::Assassinate { target } => {
                        *claim == Role::Contessa && target == player_index as u8
                    }
                    _ => false,
                }
            }

            CoupClientEvent::Challenge => {
                if has_unresolved_challenge(room) {
                    return false;
                }

                // Check if the last action was not taken by us and is challengeable by us
                let last_action = match room.coup.last_action.value() {
                    Some(action) => action,
                    None => return false,
                };

                if last_action.player == player_index as u8 {
                    return false; // Can't challenge if the last action was taken by us
                };

                match last_action.action {
                    ActionType::Action(action) => match action {
                        PlayerActionType::Exchange | PlayerActionType::Tax => true,
                        PlayerActionType::Assassinate { target }
                        | PlayerActionType::Steal { target } => target == player_index as u8,
                        _ => false,
                    },
                    // If its a counteraction, we can only challenge if the counteraction is against us
                    ActionType::Counteraction { claim: _, against } => {
                        against == player_index as u8
                    }
                }
            }
            CoupClientEvent::RevealCard { card: _ } => {
                todo!("Validate reveal card");
            }
            CoupClientEvent::ResolveChallenge { card } => {
                // If we were the player that took the last action and there is a challenge
                // check if we have the card we are revealing
                todo!("Validate resolve challenge");
            }
        }
    }

    fn handle_client_game_event(
        room: &mut types::Room,
        event: &Self::GameClientEvent,
        connections: &mut impl traits::Networking,
        player_index: usize,
    ) {
        match event {
            CoupClientEvent::Action { action } => {
                connections.send_to_all_except_origin_game_event::<Self>(
                    room,
                    CoupServerEvent::Action {
                        player: player_index as u8,
                        action: *action,
                    },
                    player_index,
                );
            }
            CoupClientEvent::Counteraction { claim } => {
                connections.send_to_all_except_origin_game_event::<Self>(
                    room,
                    CoupServerEvent::Counteraction {
                        player: player_index as u8,
                        claim: *claim,
                    },
                    player_index,
                );
            }
            CoupClientEvent::Challenge => {
                connections.send_to_all_except_origin_game_event::<Self>(
                    room,
                    CoupServerEvent::Challenge {
                        player: player_index as u8,
                    },
                    player_index,
                );
            }
            CoupClientEvent::RevealCard { card } => {
                todo!("Handle reveal card");
            }
            CoupClientEvent::ResolveChallenge { card } => {
                todo!("Handle resolve challenge");
            }
        }
    }

    fn handle_server_game_event(
        room: &mut types::Room,
        event: &Self::GameServerEvent,
        player_index: Option<usize>,
        is_server_side: bool,
    ) {
        match event {
            CoupServerEvent::GameStarted { turn, cards } => {
                todo!("Handle game started");
            }
            CoupServerEvent::Action { player, action } => {
                todo!("Handle action taken");
            }
            CoupServerEvent::Counteraction { player, claim } => {
                todo!("Handle counteraction");
            }
            CoupServerEvent::Challenge { player } => {
                todo!("Handle challenge");
            }
            CoupServerEvent::CardRevealed { player, card } => {
                todo!("Handle card revealed");
            }
            CoupServerEvent::ChallengeRevealed { player, card } => {
                todo!("Handle challenge revealed");
            }
        }
    }

    fn validate_start_game(room: &types::Room, _: usize) -> bool {
        num_players(room) >= 3
    }

    fn handle_start_game(room: &mut types::Room, connections: &mut impl traits::Networking) {
        todo!("Handle start game");
    }

    fn wrap_game_event(event: Self::GameServerEvent) -> types::ServerEvent {
        types::ServerEvent::CoupEvent(event)
    }
}

fn is_player_alive(room: &types::Room, player_index: usize) -> bool {
    room.players
        .get(player_index)
        .and_then(|player| player.value().as_ref())
        .map(|player| has_unrevealed_cards(&player.coup))
        .unwrap_or_default()
}

fn has_unrevealed_cards(player: &CoupPlayer) -> bool {
    player.cards.iter().any(|card| !card.revealed.value())
}

fn has_unresolved_challenge(room: &types::Room) -> bool {
    room.coup.challenge.value().is_some()
}
