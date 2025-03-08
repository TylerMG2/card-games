use crate::{traits::GameSignal, types};

pub fn is_host(room: &types::Room, player_index: usize) -> bool {
    *room.host.value() == player_index as u8
}

pub fn is_lobby(room: &types::Room) -> bool {
    *room.state.value() == types::RoomState::Lobby
}

pub fn get_player(room: &types::Room, player_index: usize) -> Option<&types::Player> {
    room.players
        .get(player_index)
        .and_then(|player| player.value().as_ref())
}

pub fn get_player_mut(room: &mut types::Room, player_index: usize) -> Option<&mut types::Player> {
    room.players
        .get_mut(player_index)
        .and_then(|player| player.value_mut().as_mut())
}

pub fn num_players(room: &types::Room) -> usize {
    room.players
        .iter()
        .filter(|player| player.value().is_some())
        .count()
}

// pub fn reset_room<T: GameLogic>(room: &mut T::Room) {

// }
