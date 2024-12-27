
use shared_core::{Connection, GameLogic, GameType};

use super::{carbo::CarboRoom, tycoon::TycoonRoom};

pub fn create_logic_from(game: GameType, previous: GameLogicType) -> GameLogicType {
    match game {
        GameType::Tycoon => GameLogicType::Tycoon(Default::default()),
        GameType::Carbo => GameLogicType::Carbo(Default::default()),
    }
}

pub enum GameLogicType {
    Tycoon(TycoonRoom),
    Carbo(CarboRoom),
}

impl Default for GameLogicType {
    fn default() -> Self {
        GameLogicType::Tycoon(TycoonRoom::default())
    }
}

// Lots of duplicated code here, but it's the best way I can think of to handle the different game types
// I think we can use a proc macro to generate this code for each variant of GameLogicType
// Best to do this once I have one game working so I can see what the common code is
impl GameLogicType {
    pub fn change_game(&mut self, game: GameType) {
        *self = match self {
            GameLogicType::Tycoon(logic) => GameLogicType::from_previous_logic(game, logic),
            GameLogicType::Carbo(logic) => GameLogicType::from_previous_logic(game, logic),
        };
    }

    pub fn from_previous_logic(game: GameType, logic: &impl GameLogic) -> Self {
        match game {
            GameType::Tycoon => GameLogicType::Tycoon(TycoonRoom::from_logic(logic)),
            GameType::Carbo => GameLogicType::Carbo(CarboRoom::from_logic(logic)),
        }
    }

    // Returns true if the connection was a reconnection
    pub fn handle_connection(&mut self, connections: &[Option<Connection>; 8], player_index: usize) -> bool {
        match self {
            GameLogicType::Tycoon(logic) => logic.handle_connection(connections, player_index),
            GameLogicType::Carbo(logic) => logic.handle_connection(connections, player_index),
        }   
    }

    pub fn handle_disconnection(&mut self, connections: &[Option<Connection>; 8], player_index: usize) {
        match self {
            GameLogicType::Tycoon(logic) => logic.handle_disconnection(connections, player_index),
            GameLogicType::Carbo(logic) => logic.handle_disconnection(connections, player_index),
        }
    }

    pub fn has_player(&self, player_index: usize) -> bool {
        match self {
            GameLogicType::Tycoon(logic) => logic.players[player_index].is_some(),
            GameLogicType::Carbo(logic) => logic.players[player_index].is_some(),
        }
    }

    pub fn process_client_event(&mut self, connections: &[Option<Connection>; 8], bytes: &[u8], player_index: usize) -> Option<shared_core::ProcessEventResult> {
        match self {
            GameLogicType::Tycoon(logic) => logic.process_client_event(connections, bytes, player_index),
            GameLogicType::Carbo(logic) => logic.process_client_event(connections, bytes, player_index),
        }
    }
}