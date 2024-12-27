
use shared_core::{Connection, GameLogic, GameType, PlayerFields, RoomFields};

use super::{carbo::CarboRoom, tycoon::TycoonRoom};

pub fn get_logic(game: GameType) -> GameLogicType {
    match game {
        GameType::Tycoon => GameLogicType::Tycoon(TycoonRoom::default()),
        GameType::Carbo => GameLogicType::Carbo(CarboRoom::default()),
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
impl GameLogicType {
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
}