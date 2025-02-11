use serde::{de::DeserializeOwned, Serialize};

use crate::{games::{carbo, tycoon}, logic::{self, handle_server_event}, types::{self, MAX_PLAYERS}};

pub trait GameLogic {
    type GameServerEvent: Serialize + DeserializeOwned;
    type GameClientEvent: Serialize + DeserializeOwned;
    type Room: Serialize + DeserializeOwned + Clone + Copy + Default;
    type Player: Serialize + DeserializeOwned + Clone + Copy + Default;

    // Maybe in the future ill make it so that the room is the Room type of the logic
    fn validate_client_game_event(room: &types::Room, event: &Self::GameClientEvent, player_index: usize) -> bool;
    fn handle_client_game_event(room: &mut types::Room, event: &Self::GameClientEvent, connections: &mut impl Networking, player_index: usize);
    fn handle_server_game_event(room: &mut types::Room, event: &Self::GameServerEvent, player_index: Option<usize>, is_server_side: bool);

    fn wrap_game_event(event: Self::GameServerEvent) -> types::ServerEvent;
}

pub trait ToFromBytes {
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}

impl<T> ToFromBytes for T
where
    T: Serialize + DeserializeOwned + Default,
{
    fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap_or_default()
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }
}

pub trait Networking {
    fn send_to_all_game_event<Logic: GameLogic>(&mut self, room: &mut types::Room, event: Logic::GameServerEvent) {
        self.send_to_all(room, Logic::wrap_game_event(event));
    }
    fn send_to_all(&mut self, room: &mut types::Room, event: types::ServerEvent);

    fn send_to_all_except_game_event<Logic: GameLogic>(&mut self, room: &mut types::Room, event: Logic::GameServerEvent, except: usize) {
        self.send_to_all_except(room, Logic::wrap_game_event(event), except);
    }
    fn send_to_all_except(&mut self, room: &mut types::Room, event: types::ServerEvent, except: usize);

    fn send_to_game_event<Logic: GameLogic>(&mut self, room: &mut types::Room, event: Logic::GameServerEvent, player_index: usize) {
        self.send_to(room, Logic::wrap_game_event(event), player_index);
    }
    fn send_to(&mut self, room: &mut types::Room, event: types::ServerEvent, player_index: usize);

    // Used for deterministic events where the origin client can handle the event instantly
    fn send_to_all_except_origin_game_event<Logic: GameLogic>(&mut self, room: &mut types::Room, event: Logic::GameServerEvent, origin: usize) {
        self.send_to_all_except_origin(room, Logic::wrap_game_event(event), origin);
    }
    fn send_to_all_except_origin(&mut self, room: &mut types::Room, event: types::ServerEvent, origin: usize);
}

pub trait NetworkingSend {
    fn send(&mut self, event: &types::ServerEvent);
}

impl<T> Networking for [Option<T>; MAX_PLAYERS] 
where 
    T: NetworkingSend
{
    fn send_to_all(&mut self, room: &mut types::Room, event: types::ServerEvent) {
        logic::handle_server_event(room, &event, None, true);

        println!("Sending {:?} to all", event);

        for connection in self.iter_mut() {
            if let Some(connection) = connection {
                connection.send(&event);
            }
        }
    }

    fn send_to_all_except(&mut self, room: &mut types::Room, event: types::ServerEvent, except: usize) {
        logic::handle_server_event(room, &event, None, true);

        println!("Sending {:?} to all except {}", event, except);

        for (index, connection) in self.iter_mut().enumerate() {
            if index != except {
                if let Some(connection) = connection {
                    connection.send(&event);
                }
            }
        }
    }

    fn send_to(&mut self, room: &mut types::Room, event: types::ServerEvent, player_index: usize)  {
        println!("Sending {:?} to {}", event, player_index);

        if let Some(Some(connection)) = self.get_mut(player_index) {
            connection.send(&event);
        } else {
            println!("Tried to send to a connection that doesn't exist");
            return;
        }

        logic::handle_server_event(room, &event, Some(player_index), true); // Only need to handle the event if we actually sent it to a player
    }

    fn send_to_all_except_origin(&mut self, room: &mut types::Room, event: types::ServerEvent, origin: usize) {
        self.send_to_all_except(room, event, origin);
    }
}

// Used for client side instant updates
//
// This is useful is alot of games for instant feedback, for example, in a card game like tycoon millionaire, the outcome
// of playing a card is deterministic and the client can update its state instantly without waiting for the server
impl Networking for types::ClientConnection {
    fn send_to_all(&mut self, _room: &mut types::Room, _event: types::ServerEvent) { } // Do nothing
    fn send_to_all_except(&mut self, _room: &mut types::Room, _event: types::ServerEvent, _except: usize) { } // Do nothing
    fn send_to(&mut self, _room: &mut types::Room, _event: types::ServerEvent, _player_index: usize) { } // Do nothing

    fn send_to_all_except_origin(&mut self, room: &mut types::Room, event: types::ServerEvent, origin: usize) {
        handle_server_event(room, &event, Some(origin), false);
    }
}

// Trait for rooms, this makes it so the frontend can use more precise signals for updating the UI
// rather then a single room signal.
// As boilerplate as this is, it allows both the frontend and backend to use the same logic for updating the room state
// without having the frontend rerender for every single event.
pub trait RoomTrait {
    fn get_host(&self) -> u8;
    fn set_host(&mut self, host: u8);

    fn get_player_index(&self) -> u8;
    fn set_player_index(&mut self, player_index: u8);

    fn get_state(&self) -> types::RoomState;
    fn set_state(&mut self, state: types::RoomState);

    fn get_game(&self) -> types::GameType;
    fn set_game(&mut self, game: types::GameType);

    fn get_players(&self) -> [Option<types::Player>; MAX_PLAYERS];
    fn get_players_mut(&mut self) -> &mut [Option<types::Player>; MAX_PLAYERS];

    fn get_player(&self, index: usize) -> Option<&types::Player>;
    fn get_player_mut(&mut self, index: usize) -> Option<&mut types::Player>;
    fn set_player(&mut self, index: usize, player: Option<types::Player>);

    fn get_tycoon(&self) -> &tycoon::TycoonRoom; // TODO: This and carbo should return a trait
    fn get_tycoon_mut(&mut self) -> &mut tycoon::TycoonRoom;

    fn get_carbo(&self) -> &carbo::CarboRoom;
    fn get_carbo_mut(&mut self) -> &mut carbo::CarboRoom;
}

impl RoomTrait for types::Room {
    fn get_host(&self) -> u8 { self.host }
    fn set_host(&mut self, host: u8) { self.host = host; }

    fn get_player_index(&self) -> u8 { self.player_index }
    fn set_player_index(&mut self, player_index: u8) { self.player_index = player_index; }

    fn get_state(&self) -> types::RoomState { self.state }
    fn set_state(&mut self, state: types::RoomState) { self.state = state; }

    fn get_game(&self) -> types::GameType { self.game }
    fn set_game(&mut self, game: types::GameType) { self.game = game; }

    fn get_players(&self) -> [Option<types::Player>; MAX_PLAYERS] { self.players }
    fn get_players_mut(&mut self) -> &mut [Option<types::Player>; MAX_PLAYERS] { &mut self.players }

    fn get_player(&self, index: usize) -> Option<&types::Player> { self.players.get(index)?.as_ref() }
    fn get_player_mut(&mut self, index: usize) -> Option<&mut types::Player> { self.players.get_mut(index)?.as_mut() }
    fn set_player(&mut self, index: usize, player: Option<types::Player>) { self.players[index] = player; }

    fn get_tycoon(&self) -> &tycoon::TycoonRoom { &self.tycoon }
    fn get_tycoon_mut(&mut self) -> &mut tycoon::TycoonRoom { &mut self.tycoon }

    fn get_carbo(&self) -> &carbo::CarboRoom { &self.carbo }
    fn get_carbo_mut(&mut self) -> &mut carbo::CarboRoom { &mut self.carbo }
}