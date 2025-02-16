use crate::{traits::{GameLogic, GameSignal}, types};

pub fn is_host(room: &types::Room, player_index: usize) -> bool {
    *room.host.value() == player_index as u8
}

pub fn is_lobby(room: &types::Room) -> bool {
    *room.state.value() == types::RoomState::Lobby
}

// pub fn reset_room<T: GameLogic>(room: &mut T::Room) {
    
// }