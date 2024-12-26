use crate::{traits::GameLogic, types::GameType};
use crate::games::tycoon::TycoonLogic;
use crate::games::carbo::CarboLogic;    

pub fn get_logic(game: GameType) -> impl GameLogic {
    match game {
        GameType::Tycoon => TycoonLogic {},
        _ => panic!("Game not implemented"),
    }
}

pub enum GameLogicType {
    Tycoon(TycoonLogic),
    Carbo(CarboLogic),
}