use crate::{traits::GameLogic, types::{self, ClientPlayer}};

// I think in the future I should add a boolean argument thats like "is_server" to the handle_server_event function
// then I can avoid updating state that doesn't need to be updated on the server, i.e a player joining a room where
// we just send them the current room and update the current player (only needed by the client).
pub fn handle_server_event<Logic: GameLogic>(logic: &mut Logic, client_room: &mut types::ClientRoom<Logic::Room, Logic::Player>, event: &types::ServerEvent<Logic>, player_index: Option<usize>) {
    match event {
        types::ServerEvent::GameEvent(event) => {
            // Map Client players to just the .player field (Individual game logic shouldn't need to modify the ClientPlayer struct)
            let mut players = [const { None }; 8];
            for (index, player) in client_room.players.iter_mut().enumerate() {
                if let Some(player) = player.as_mut() {
                    players[index] = Some(&mut player.player);
                }
            }

            logic.handle_server_event(&event, &mut client_room.room, &mut players, player_index);
        },
        types::ServerEvent::HostChanged { player_index } => {
            client_room.host = *player_index;
        },
        types::ServerEvent::PlayerDisconnected { player_index } => {
            if let Some(player) = client_room.players.get_mut(*player_index as usize) { // TODO, maybe centralize player fetching so we can better log when no player is found since this should never happen in this case
                if let Some(player) = player {
                    player.disconnected = true;
                }
            }
        },
        types::ServerEvent::PlayerJoined { name, player_index } => {
            client_room.players[*player_index as usize] = Some(
                ClientPlayer {
                    name: *name,
                    disconnected: false,
                    player: Default::default(),
                }
            );
        },
        types::ServerEvent::PlayerLeft { player_index } => {
            client_room.players[*player_index as usize] = None;
        },
        types::ServerEvent::PlayerReconnected { player_index } => {
            if let Some(player) = client_room.players.get_mut(*player_index as usize) {
                if let Some(player) = player {
                    player.disconnected = false;
                }
            }
        },
        types::ServerEvent::RoomJoined { room, current_player } => {
            // Hacky way to avoid the server resetting the room when a player joins for no reason (no state change)
            if player_index.is_some() {
                *client_room = *room;
                client_room.current_player = Some(*current_player);
            }
        },
        types::ServerEvent::GameChanged { game: _ } => { }, // Should not be handled here
        types::ServerEvent::Unknown => panic!("Should never send a types::ServerEvent::Unknown"), // TODO: Either ignore or force the client to disconnect
    }
}

pub fn validate_client_event<Logic: GameLogic>(logic: &Logic, room: &types::ClientRoom<Logic::Room, Logic::Player>, event: &types::ClientEvent<Logic::GameClientEvent>, player_index: usize) -> bool {
    match event {
        types::ClientEvent::GameEvent(event) => {
            let mut players = [const { None }; 8]; // TODO: Move to a constant
            for (index, player) in room.players.iter().enumerate() {
                if let Some(player) = player {
                    players[index] = Some(&player.player);
                }
            }

            logic.validate_client_game_event(event, &room.room, &players, player_index)
        },
        types::ClientEvent::JoinRoom { name: _ } => {
            if let Some(player) = room.players.get(player_index) {
                return player.is_none();
            }
           false
        },
        types::ClientEvent::LeaveRoom => true,
        types::ClientEvent::ChangeGame { game: _ } => room.host as usize == player_index && room.state == types::RoomState::Lobby,
        types::ClientEvent::Unknown => false,
    }
}

// The goal of this function is too avoid any state changes at all, it should all be handled in handle_server event which is called
// by both the server and client meaning they should always be in sync if nothing else changes the state.
pub fn handle_client_event<Logic: GameLogic>(logic: &Logic, room: &mut types::ServerRoom<Logic>, event: &types::ClientEvent<Logic::GameClientEvent>, player_index: usize) {
    match event {
        types::ClientEvent::GameEvent(event) => {
            let mut players = [const { None }; 8];
            for (index, player) in room.client_room.players.iter_mut().enumerate() {
                if let Some(player) = player.as_mut() {
                    players[index] = Some(&mut player.player);
                }
            }

            logic.handle_client_game_event(event, room, player_index);
        },
        types::ClientEvent::JoinRoom { name } => {
            logic.send_to_all_except(&types::ServerEvent::PlayerJoined { name: *name, player_index: player_index as u8 }, player_index, room);

            // send to all except should create the player in the room before sending the event (a bit hacky but it works)
            logic.send_to(&types::ServerEvent::RoomJoined { room: room.client_room, current_player: player_index as u8 }, player_index, room);
        },
        types::ClientEvent::LeaveRoom => {
            logic.send_to_all_except(&types::ServerEvent::PlayerLeft { player_index: player_index as u8 }, player_index, room);
        },
        types::ClientEvent::ChangeGame { game } => {
            logic.send_to_all(&types::ServerEvent::GameChanged { game: *game }, room);
        },
        types::ClientEvent::Unknown => (),
    }
}

