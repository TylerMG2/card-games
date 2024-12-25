use crate::{games::get_logic, traits::GameLogic, types::{self, ClientPlayer, ClientRoom}};

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
            if let Some(player) = client_room.players.get_mut(*player_index as usize) { // TODO, maybe centralize player fetching so we can better log when no player is found
                if let Some(player) = player {
                    player.disconnected = true;
                }
            }
        },
        types::ServerEvent::PlayerJoined { player, player_index } => {
            client_room.players[*player_index as usize] = Some(*player);
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
            *client_room = *room; // This is kinda inefficient from the servers perspective since it would copy the entire room each time a player joins which the server doesn't need to do
            client_room.current_player = Some(*current_player);
        },
        types::ServerEvent::GameChanged { game: _ } => { }, // Should not be handled here
        types::ServerEvent::Unknown => panic!("Should never send a types::ServerEvent::Unknown"), // TODO: Either ignore or force the client to disconnect
    }
}

pub fn validate_client_event<Logic: GameLogic>(logic: &Logic, room: &types::ClientRoom<Logic::Room, Logic::Player>, event: &types::ClientEvent<Logic::GameClientEvent>, player_index: usize) -> bool {
    match event {
        types::ClientEvent::GameEvent(event) => {
            let mut players = [const { None }; 8];
            for (index, player) in room.players.iter().enumerate() {
                if let Some(player) = player {
                    players[index] = Some(&player.player);
                }
            }

            logic.validate_client_game_event(event, &room.room, &players, player_index)
        },
        types::ClientEvent::JoinRoom { name } => {
            if let Some(Some(player)) = room.players.get(player_index) {
                player.name == [0; 20]
            } else {
                false
            }
        },
        types::ClientEvent::LeaveRoom => true,
        types::ClientEvent::ChangeGame { game } => room.host as usize == player_index, // TODO: Check if the current game is in progress
        types::ClientEvent::Unknown => false,
    }
}

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
            if let Some(player) = room.client_room.players.get_mut(player_index) {
                if let Some(player) = player {
                    player.name = *name;
                }
            }
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

