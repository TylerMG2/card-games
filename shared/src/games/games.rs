use crate::{traits::GameLogic, types::GameType};
use crate::games::tycoon::TycoonLogic;

pub fn get_logic(game: GameType) -> impl GameLogic {
    match game {
        GameType::Tycoon => TycoonLogic {},
        _ => panic!("Game not implemented"),
    }
}
