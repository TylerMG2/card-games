use crate::{types, ServerRoom};
use crate::{traits::GameLogic, types::GameType};
use crate::games::tycoon::TycoonLogic;
use crate::games::carbo::CarboLogic;    

pub fn get_logic(game: GameType) -> GameLogicType {
    match game {
        GameType::Tycoon => GameLogicType::Tycoon(TycoonLogic::default()),
        GameType::Carbo => GameLogicType::Carbo(CarboLogic::default()),
    }
}

pub enum GameLogicType {
    Tycoon(TycoonLogic),
    Carbo(CarboLogic),
}

impl Default for GameLogicType {
    fn default() -> Self {
        GameLogicType::Tycoon(TycoonLogic::default())
    }
}

// Lots of duplicated code here, but it's the best way I can think of to handle the different game types
impl GameLogicType {
    // Returns true if the connection was a reconnection
    pub fn handle_connection(&mut self, connections: &[Option<types::Connection>; 8], player_index: usize) -> bool {
        match self {
            GameLogicType::Tycoon(logic) => logic.handle_connection(connections, player_index),
            GameLogicType::Carbo(logic) => logic.handle_connection(connections, player_index),
        }
    }

    pub fn handle_disconnection(&mut self, connections: &[Option<types::Connection>; 8], player_index: usize) {
        match self {
            GameLogicType::Tycoon(logic) => logic.handle_disconnection(connections, player_index),
            GameLogicType::Carbo(logic) => logic.handle_disconnection(connections, player_index),
        }
    }

    pub fn has_player(&self, player_index: usize) -> bool {
        match self {
            GameLogicType::Tycoon(logic) => logic.get_client_room().players[player_index].is_some(),
            GameLogicType::Carbo(logic) => logic.get_client_room().players[player_index].is_some(),
        }
    }
}