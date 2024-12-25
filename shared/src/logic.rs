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
        types::ServerEvent::Unknown => panic!("Should never send a types::ServerEvent::Unknown"), 
    }
}