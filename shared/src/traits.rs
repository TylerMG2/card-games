use serde::{de::DeserializeOwned, Serialize};

use crate::{logic::handle_server_event, types};

pub trait GameLogic 
where 
    Self: Sized
{
    type GameServerEvent: DeserializeOwned + Serialize + Copy;
    type GameClientEvent: DeserializeOwned + Serialize;
    type Room: Default + Copy + Serialize + DeserializeOwned;
    type Player: Default + Copy + Serialize + DeserializeOwned;

    // Shared validation for the client and server, in theory in the future if all validation is shared between the client and server
    // we could update the client immediately with changes expected from the server since we know the server will accept the event
    fn validate_event(&self, event: &Self::GameClientEvent, room: &Self::Room, players: &[Option<Self::Player>; 8], player_index: usize) -> bool;

    // The server should not update the room directly, instead it should send server events to the clients which will update their rooms and the
    // server room using the handle_server_event method
    fn handle_client_event(&self, event: &Self::GameClientEvent, room: &mut types::ServerRoom<Self>, player_index: usize);

    // Player index should be provided all the time from the client, for the server its only provided when the server is sending an event to a specific player
    fn handle_server_event(&self, event: &Self::GameServerEvent, room: &mut Self::Room, players: &mut [Option<&mut Self::Player>; 8], player_index: Option<usize>);

    fn default_room(&self) -> Self::Room {
        Self::Room::default()
    }

    fn default_player(&self) -> Self::Player {
        Self::Player::default()
    }
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

pub trait Networked {
    type Logic: GameLogic;

    fn send_to_all_game_event(&self, event: &<<Self as Networked>::Logic as GameLogic>::GameServerEvent, room: &mut types::ServerRoom<Self::Logic>) {
        self.send_to_all(&types::ServerEvent::GameEvent(*event), room);
    }

    fn send_to_all(&self, event: &types::ServerEvent<Self::Logic>, room: &mut types::ServerRoom<Self::Logic>) {
        handle_server_event(&mut room.logic, &mut room.client_room, event, None);

        for connection in room.connections.iter() {
            self.send(event, connection);
        }
    }

    fn send_to_all_except_game_event(&self, event: &<<Self as Networked>::Logic as GameLogic>::GameServerEvent, except: usize, room: &mut types::ServerRoom<Self::Logic>) {
        self.send_to_all_except(&types::ServerEvent::GameEvent(*event), except, room);
    }
    
    fn send_to_all_except(&self, event: &types::ServerEvent<Self::Logic>, except: usize, room: &mut types::ServerRoom<Self::Logic>) {
        handle_server_event(&mut room.logic, &mut room.client_room, event, None);

        for (index, connection) in room.connections.iter().enumerate() {
            if index != except {
                self.send(event, connection);
            }
        }
    }

    fn send_to_game_event(&self, event: &<<Self as Networked>::Logic as GameLogic>::GameServerEvent, player_index: usize, room: &mut types::ServerRoom<Self::Logic>) {
        self.send_to(&types::ServerEvent::GameEvent(*event), player_index, room);
    }

    fn send_to(&self, event: &types::ServerEvent<Self::Logic>, player_index: usize, room: &mut types::ServerRoom<Self::Logic>) {
        if let Some(connection) = room.connections.get(player_index) {
            self.send(event, connection);
        } else {
            println!("Tried to send to a connection that doesn't exist");
            return;
        }

        handle_server_event(&mut room.logic, &mut room.client_room, event, Some(player_index)); // Only need to handle the event if we actually sent it
    }

    fn send(&self, event: &types::ServerEvent<Self::Logic>, connection: &Option<types::Connection>) {
        if let Some(connection) = connection {
            if let Some(sender) = &connection.sender {
                sender.send(event.to_bytes()).unwrap();
            }
        }
    }
}

impl<T> Networked for T
where
    T: GameLogic,
{
    type Logic = T;
}