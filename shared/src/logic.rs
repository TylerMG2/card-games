use crate::{games::{carbo, coup, tycoon}, helpers::{is_host, is_lobby}, traits::{GameLogic, GameSignal, Networking}, types::{self, ClientEvent, CommonClientEvent, CommonServerEvent, ServerEvent}};

pub fn handle_server_event(room: &mut types::Room, event: &ServerEvent, as_player: Option<usize>, is_server_side: bool) {
    match event {
        ServerEvent::TycoonEvent(event) => {
            tycoon::TycoonRoom::handle_server_game_event(room, event, as_player, is_server_side);
        },
        ServerEvent::CarboEvent(event) => {
            carbo::CarboRoom::handle_server_game_event(room, event, as_player, is_server_side);
        },
        ServerEvent::CoupEvent(event) => {
            coup::CoupRoom::handle_server_game_event(room, event, as_player, is_server_side);
        },
        ServerEvent::CommonEvent(event) => {
            match event {
                CommonServerEvent::HostChanged { player_index } => { // TODO: After finishing the game, we may be able to remove this event and just have the host change when the host disconnects
                    room.host.set(*player_index);
                },
                CommonServerEvent::PlayerDisconnected { player_index } => {
                    if let Some(player) = room.players.get_mut(*player_index as usize) { // TODO, maybe centralize player fetching so we can better log when no player is found since this should never happen in this case
                        if let Some(player) = player.get_mut() {
                            player.disconnected.set(true);
                        }
                    }
        
                    // Change host if the host disconnected
                    if player_index == room.host.value() {
                        let mut new_host = None;
                        for (index, player) in room.players.iter().enumerate() {
                            if let Some(player) = player.value() {
                                if !player.disconnected.value() {
                                    new_host = Some(index as u8);
                                    break;
                                }
                            }
                        }
        
                        if let Some(new_host) = new_host {
                            room.host.set(new_host);
                        }
                    }
                },
                CommonServerEvent::PlayerJoined { name, player_index } => {
                    let mut new_player = types::Player::default();
                    new_player.name.set(*name);
                    room.players[*player_index as usize].set(Some(new_player));
                },
                CommonServerEvent::PlayerLeft { player_index } => {
                    room.players[*player_index as usize].set(None);
                    //TODO: I think move all players after the player that left to the left
                },
                CommonServerEvent::PlayerReconnected { player_index } => {
                    if let Some(player) = room.players.get_mut(*player_index as usize) {
                        if let Some(player) = player.get_mut() {
                            player.disconnected.set(false);
                        }
                    }
                },
                CommonServerEvent::RoomJoined { new_room, current_player } => {
                    if !is_server_side {
                        *room = new_room.clone();
                        room.player_index.set(*current_player);
                    }
                },
                CommonServerEvent::GameChanged { game } => {
                    room.game.set(*game);
                },
                CommonServerEvent::ResetGame => {
                    // TODO: this is a little verbose, maybe we can make a function to reset the game
                    match room.game.value() {
                        types::GameType::Carbo => {
                            room.carbo = carbo::CarboRoom::default();
                            room.players.iter_mut().for_each(|player| {
                                if let Some(player) = player.get_mut() {
                                    player.carbo = carbo::CarboPlayer::default();
                                }
                            });
                        },
                        types::GameType::Tycoon => {
                            room.tycoon = tycoon::TycoonRoom::default();
                            room.players.iter_mut().for_each(|player| {
                                if let Some(player) = player.get_mut() {
                                    player.tycoon = tycoon::TycoonPlayer::default();
                                }
                            });
                        },
                        types::GameType::Coup => {
                            room.coup = coup::CoupRoom::default();
                            room.players.iter_mut().for_each(|player| {
                                if let Some(player) = player.get_mut() {
                                    player.coup = coup::CoupPlayer::default();
                                }
                            });
                        },
                    }
                },
            }
        },
        ServerEvent::Unknown => {}, // TODO: Either ignore or force the client to disconnect
    }
}

pub fn validate_client_event(room: &types::Room, event: &ClientEvent, player_index: usize) -> bool {
    match event {
        ClientEvent::TycoonEvent(event) => {
            tycoon::TycoonRoom::validate_client_game_event(room, event, player_index)
        },
        ClientEvent::CarboEvent(event) => {
            carbo::CarboRoom::validate_client_game_event(room, event, player_index)
        },
        ClientEvent::CoupEvent(event) => {
            coup::CoupRoom::validate_client_game_event(room, event, player_index)
        },
        ClientEvent::CommonEvent(event) => {
            match event {
                CommonClientEvent::LeaveRoom => true,
                CommonClientEvent::ChangeGame { game: _ } => is_host(room, player_index) && is_lobby(room),
                CommonClientEvent::Disconnect => true,
                CommonClientEvent::ResetGame => is_host(room, player_index) && is_lobby(room), // TODO: reconsider when they should be able to reset
            }
        },
        ClientEvent::Unknown => false,
    }
}

// The room should never be mutated in this function, in order to keep the client and server in sync while minimizing data transfer,
// all that should take place here is converting client events to server events where then the room is mutated by the server and client.
// This means we have a central place where both the client and server update their state from the same events and same logic.
// We can now send way less data between the client and server since the server can send the client the events that caused the state change.
// For example playing cards in a card game could have much greater impact on the game state then just the cards played, traditionally, the server
// would have to send a game state containing all the things that could have changed, but now the server can just send the client the event.
// This also means that the client can now predict the game state without having to wait for the server to send the game state.
pub fn handle_client_event(room: &mut types::Room, event: &ClientEvent, connections: &mut impl Networking, player_index: usize) {
    match event {
        //TODO: Ignore events that are not for the current game
        ClientEvent::TycoonEvent(event) => {
            tycoon::TycoonRoom::handle_client_game_event(room, event, connections, player_index);
        },
        ClientEvent::CarboEvent(event) => {
            carbo::CarboRoom::handle_client_game_event(room, event, connections, player_index);
        },
        ClientEvent::CoupEvent(event) => {
            coup::CoupRoom::handle_client_game_event(room, event, connections, player_index);
        },
        ClientEvent::CommonEvent(event) => {
            match event {
                CommonClientEvent::LeaveRoom => {
                    connections.send_to_all_except_origin(room, ServerEvent::CommonEvent(CommonServerEvent::PlayerLeft { player_index: player_index as u8 }), player_index);
                },
                CommonClientEvent::ChangeGame { game } => {
                    connections.send_to_all_except_origin(room, ServerEvent::CommonEvent(CommonServerEvent::GameChanged { game: *game }), player_index);
                },
                CommonClientEvent::Disconnect => {
                    connections.send_to_all_except_origin(room, ServerEvent::CommonEvent(CommonServerEvent::PlayerDisconnected { player_index: player_index as u8 }), player_index);
                },
                CommonClientEvent::ResetGame => {
                    connections.send_to_all_except_origin(room, ServerEvent::CommonEvent(CommonServerEvent::ResetGame), player_index);
                }
            }
        },
        ClientEvent::Unknown => {},
    }
}

// Note to future tyler: I think I can make this a macro.
// Additionally, I would like to remove the mutability of the room and connections, instead returning a list of events to send
// in order, this way I can't accidently edit the room or connections here when it really should be done in the handle_client_event