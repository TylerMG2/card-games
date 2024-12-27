use crate::{traits::GameLogic, types, PlayerFields, RoomFields};

pub fn handle_server_event<Logic: GameLogic>(logic: &mut Logic, event: &types::ServerEvent<Logic>, as_player: Option<usize>, is_server_side: bool) {
    match event {
        types::ServerEvent::GameEvent(event) => {
            logic.handle_server_game_event(&event, as_player, is_server_side);
        },
        types::ServerEvent::HostChanged { player_index } => {
            logic.set_host(*player_index);
        },
        types::ServerEvent::PlayerDisconnected { player_index } => {
            if let Some(player) = logic.players_mut().get_mut(*player_index as usize) { // TODO, maybe centralize player fetching so we can better log when no player is found since this should never happen in this case
                if let Some(player) = player {
                    player.set_disconnected(true);
                }
            }

            if *player_index == logic.host() {
                let mut new_host = None;
                for (index, player) in logic.players().iter().enumerate() {
                    if let Some(player) = player {
                        if !player.disconnected() {
                            new_host = Some(index as u8);
                            break;
                        }
                    }
                }

                if let Some(new_host) = new_host {
                    logic.set_host(new_host);
                }
            }
        },
        types::ServerEvent::PlayerJoined { name, player_index } => {
            let mut new_player = <Logic as RoomFields>::Player::default();
            new_player.set_name(name);
            logic.players_mut()[*player_index as usize] = Some(new_player);
        },
        types::ServerEvent::PlayerLeft { player_index } => {
            logic.players_mut()[*player_index as usize] = None;
        },
        types::ServerEvent::PlayerReconnected { player_index } => {
            if let Some(player) = logic.players_mut().get_mut(*player_index as usize) {
                if let Some(player) = player {
                    player.set_disconnected(false);
                }
            }
        },
        types::ServerEvent::RoomJoined { room, current_player } => {
            if !is_server_side {
                *logic = *room;
                logic.set_player_index(*current_player);
            }
        },
        types::ServerEvent::GameChanged { game: _ } => { }, // Should not be handled here
        types::ServerEvent::Unknown => panic!("Should never send a types::ServerEvent::Unknown"), // TODO: Either ignore or force the client to disconnect
    }
}

pub fn validate_client_event<Logic: GameLogic>(logic: &Logic, event: &types::ClientEvent<Logic::GameClientEvent>, player_index: usize) -> bool {
    match event {
        types::ClientEvent::GameEvent(event) => {
            logic.validate_client_game_event(event, player_index)
        },
        types::ClientEvent::JoinRoom { name: _ } => {
            if let Some(player) = logic.players().get(player_index) {
                return player.is_none();
            }
           false
        },
        types::ClientEvent::LeaveRoom => true,
        types::ClientEvent::ChangeGame { game: _ } => logic.host() as usize == player_index, // TODO: check if room is in lobby && room.state == types::RoomState::Lobby,
        types::ClientEvent::Unknown => false,
    }
}

// The goal of this function is too avoid any state changes at all, it should all be handled in handle_server event which is called
// by both the server and client meaning they should always be in sync if nothing else changes the state.
pub fn handle_client_event<Logic: GameLogic>(logic: &mut Logic, connections: &[Option<types::Connection>; 8], event: &types::ClientEvent<Logic::GameClientEvent>, player_index: usize) {
    match event {
        types::ClientEvent::GameEvent(event) => {
            logic.handle_client_game_event(event, connections, player_index);
        },
        types::ClientEvent::JoinRoom { name } => {
            logic.send_to_all_except(&types::ServerEvent::PlayerJoined { name: *name, player_index: player_index as u8 }, player_index, connections);

            // send to all except should create the player in the room before sending the event (a bit hacky but it works)
            logic.send_to(&types::ServerEvent::RoomJoined { room: *logic, current_player: player_index as u8 }, player_index, connections);
        },
        types::ClientEvent::LeaveRoom => {
            logic.send_to_all_except(&types::ServerEvent::PlayerLeft { player_index: player_index as u8 }, player_index, connections);
        },
        types::ClientEvent::ChangeGame { game } => {
            logic.send_to_all(&types::ServerEvent::GameChanged { game: *game }, connections);
        },
        types::ClientEvent::Unknown => (),
    }
}

