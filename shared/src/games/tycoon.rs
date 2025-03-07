use crate::{
    traits::{self, GameSignal},
    types,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone, Copy, PartialEq, Debug)]
pub enum RoomState {
    #[default]
    Lobby,
    CardExchange,
    Game,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct TycoonRoom {
    pub turn: types::SignalType<u8>,
    pub last_played: types::SignalType<u8>,
    pub last_played_player: u8,
    pub revolution: types::SignalType<bool>,
    pub state: types::SignalType<RoomState>,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct TycoonPlayer {
    pub hand: types::SignalType<u64>,
    pub num_cards: types::SignalType<u8>,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum TycoonServerEvent {
    GameStarted {
        turn: u8,
        cards: u64,
        other_hands: [u8; 8],
    }, // TODO: Move to a constant
    CardsPlayed {
        cards: u64,
    },
    Pass,
    ReceiveCards {
        cards: u64,
    },
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum TycoonClientEvent {
    PlayCards { cards: u64 },
    Pass,
    ExchangeCards { cards: u64 },
}

impl traits::GameLogic for TycoonRoom {
    type GameServerEvent = TycoonServerEvent;
    type GameClientEvent = TycoonClientEvent;
    type Room = TycoonRoom;
    type Player = TycoonPlayer;

    fn validate_client_game_event(
        room: &types::Room,
        event: &TycoonClientEvent,
        player_index: usize,
    ) -> bool {
        match event {
            TycoonClientEvent::PlayCards { cards } => {
                todo!("Validate cards played");
            }
            TycoonClientEvent::Pass => {
                todo!("Validate pass");
            }
            TycoonClientEvent::ExchangeCards { cards } => {
                todo!("Validate exchange cards");
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
                TycoonClientEvent::PlayCards { cards } => {
                    connections.send_to_all_except_origin_game_event::<Self>(
                        room,
                        TycoonServerEvent::CardsPlayed { cards: *cards },
                        player_index,
                    );
                }
                TycoonClientEvent::Pass => {
                    connections.send_to_all_except_origin_game_event::<Self>(
                        room,
                        TycoonServerEvent::Pass,
                        player_index,
                    );
                }
                TycoonClientEvent::ExchangeCards { cards } => {
                    todo!("Handle exchange cards");
                }
            }
        }
    }

    fn handle_server_game_event(
        room: &mut types::Room,
        event: &Self::GameServerEvent,
        as_player: Option<usize>,
        is_server_side: bool,
    ) {
        match event {
            TycoonServerEvent::GameStarted {
                turn,
                cards,
                other_hands,
            } => {
                room.tycoon.turn.set(*turn);
                room.tycoon.state.set(RoomState::Game);

                for (index, hand) in other_hands.iter().enumerate() {
                    if let Some(player) = room.players.get_mut(index) {
                        if let Some(player) = player.value_mut() {
                            player.tycoon.num_cards.set(*hand);

                            if let Some(player_index) = as_player {
                                if index == player_index {
                                    player.tycoon.hand.set(*cards);
                                }
                            }
                        }
                    }
                }
            }
            TycoonServerEvent::CardsPlayed { cards } => {
                todo!("Handle cards played");
            }
            TycoonServerEvent::Pass => {
                todo!("Handle pass");
            }
            TycoonServerEvent::ReceiveCards { cards } => {
                todo!("Handle receive cards");
            }
        }
    }

    fn handle_start_game(room: &mut types::Room, connections: &mut impl traits::Networking) {
        todo!("Handle start game, tycoon");
    }

    fn validate_start_game(room: &types::Room, player_index: usize) -> bool {
        *room.tycoon.state.value() == RoomState::Lobby
    }

    fn wrap_game_event(event: Self::GameServerEvent) -> types::ServerEvent {
        types::ServerEvent::TycoonEvent(event)
    }
}
