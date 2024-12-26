mod types;
mod traits;
mod logic;

pub mod games {
    mod games;

    pub use games::get_logic;
    pub use games::GameLogicType;
    pub mod tycoon;
    pub mod carbo;
}

pub use types::*;
pub use traits::*;
pub use logic::*;