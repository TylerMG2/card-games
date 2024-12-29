use serde::{de::DeserializeOwned, Serialize};

use crate::{logic::{self, handle_server_event}, types::{self, MAX_PLAYERS}};

pub trait GameLogic {
    type GameServerEvent: Serialize + DeserializeOwned;
    type GameClientEvent: Serialize + DeserializeOwned;
    type Room: Serialize + DeserializeOwned + Clone + Copy + Default;
    type Player: Serialize + DeserializeOwned + Clone + Copy + Default;

    // Maybe in the future ill make it so that the room is the Room type of the logic
    fn validate_client_game_event(room: &types::Room, event: &Self::GameClientEvent, player_index: usize) -> bool;
    fn handle_client_game_event(room: &mut types::Room, event: &Self::GameClientEvent, connections: &impl Networking, player_index: usize);
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
    fn send_to_all_game_event<Logic: GameLogic>(&self, room: &mut types::Room, event: Logic::GameServerEvent) {
        self.send_to_all(room, Logic::wrap_game_event(event));
    }
    fn send_to_all(&self, room: &mut types::Room, event: types::ServerEvent);

    fn send_to_all_except_game_event<Logic: GameLogic>(&self, room: &mut types::Room, event: Logic::GameServerEvent, except: usize) {
        self.send_to_all_except(room, Logic::wrap_game_event(event), except);
    }
    fn send_to_all_except(&self, room: &mut types::Room, event: types::ServerEvent, except: usize);

    fn send_to_game_event<Logic: GameLogic>(&self, room: &mut types::Room, event: Logic::GameServerEvent, player_index: usize) {
        self.send_to(room, Logic::wrap_game_event(event), player_index);
    }
    fn send_to(&self, room: &mut types::Room, event: types::ServerEvent, player_index: usize);

    // Used for deterministic events where the origin client can handle the event instantly
    fn send_to_all_game_event_deterministic<Logic: GameLogic>(&self, room: &mut types::Room, event: Logic::GameServerEvent, origin: usize) {
        self.send_to_all_deterministic(room, Logic::wrap_game_event(event), origin);
    }
    fn send_to_all_deterministic(&self, room: &mut types::Room, event: types::ServerEvent, origin: usize);
}

pub trait NetworkingSend {
    fn send(&self, event: &types::ServerEvent);
}

impl<T> Networking for [Option<T>; MAX_PLAYERS] 
where 
    T: NetworkingSend
{
    fn send_to_all(&self, room: &mut types::Room, event: types::ServerEvent) {
        logic::handle_server_event(room, &event, None, true);

        for connection in self.iter() {
            if let Some(connection) = connection {
                connection.send(&event);
            }
        }
    }

    fn send_to_all_except(&self, room: &mut types::Room, event: types::ServerEvent, except: usize) {
        logic::handle_server_event(room, &event, None, true);

        for (index, connection) in self.iter().enumerate() {
            if index != except {
                if let Some(connection) = connection {
                    connection.send(&event);
                }
            }
        }
    }

    fn send_to(&self, room: &mut types::Room, event: types::ServerEvent, player_index: usize) {
        if let Some(Some(connection)) = self.get(player_index) {
            connection.send(&event);
        } else {
            println!("Tried to send to a connection that doesn't exist");
            return;
        }

        logic::handle_server_event(room, &event, Some(player_index), true); // Only need to handle the event if we actually sent it to a player
    }

    fn send_to_all_deterministic(&self, room: &mut types::Room, event: types::ServerEvent, origin: usize) {
        self.send_to_all_except(room, event, origin);
    }
}

// Used for client side instant updates
//
// This is useful is alot of games for instant feedback, for example, in a card game like tycoon millionaire, the client
// the outcome of playing a card is deterministic and the client can update its state instantly without waiting for the server
impl Networking for types::ClientConnection {
    fn send_to_all(&self, _room: &mut types::Room, _event: types::ServerEvent) { } // Do nothing
    fn send_to_all_except(&self, _room: &mut types::Room, _event: types::ServerEvent, _except: usize) { } // Do nothing
    fn send_to(&self, _room: &mut types::Room, _event: types::ServerEvent, _player_index: usize) { } // Do nothing

    fn send_to_all_deterministic(&self, room: &mut types::Room, event: types::ServerEvent, origin: usize) {
        handle_server_event(room, &event, Some(origin), false);
    }
}