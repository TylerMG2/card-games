use crate::types::GameType;

pub struct GameConfig {
    pub max_players: u16,
    pub min_players: u16,
}

pub const fn game_config(game_type: GameType) -> GameConfig {
    match game_type {
        GameType::Tycoon => GameConfig {
            max_players: 8,
            min_players: 3,
        },
        GameType::Carbo => GameConfig {
            max_players: 8,
            min_players: 3,
        },
        GameType::Coup => GameConfig {
            max_players: 6,
            min_players: 3,
        },
    }
}
