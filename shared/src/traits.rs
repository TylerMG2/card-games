use serde::{de::DeserializeOwned, Serialize};

use crate::{logic::{handle_client_event, handle_server_event, validate_client_event}, types};

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
    fn validate_client_game_event(&self, event: &Self::GameClientEvent, room: &Self::Room, players: &[Option<&Self::Player>; 8], player_index: usize) -> bool;

    // The server should not update the room directly, instead it should send server events to the clients which will update their rooms and the
    // server room using the handle_server_event method
    fn handle_client_game_event(&self, event: &Self::GameClientEvent, room: &mut types::ServerRoom<Self>, player_index: usize);

    // Player index should be provided all the time from the client, for the server its only provided when the server is sending an event to a specific player
    fn handle_server_event(&self, event: &Self::GameServerEvent, room: &mut Self::Room, players: &mut [Option<&mut Self::Player>; 8], player_index: Option<usize>);

    fn default_room(&self) -> Self::Room {
        Self::Room::default()
    }

    fn default_player(&self) -> Self::Player {
        Self::Player::default()
    }

    fn get_client_event(&self, bytes: &[u8]) -> types::ClientEvent<Self::GameClientEvent> {
        types::ClientEvent::from_bytes(bytes)
    }

    fn get_server_event(&self, bytes: &[u8]) -> types::ServerEvent<Self> {
        types::ServerEvent::from_bytes(bytes)
    }

    fn validate_client_event(&self, room: &types::ClientRoom<Self::Room, Self::Player>, event: &types::ClientEvent<Self::GameClientEvent>, player_index: usize) -> bool {
        validate_client_event(self, room, event, player_index)
    }

    // TODO: Should return a enum/bool to indicate if the connection should be closed (basically if player left)
    fn handle_client_event(&self, room: &mut types::ServerRoom<Self>, event: &types::ClientEvent<Self::GameClientEvent>, player_index: usize) {
        handle_client_event(self, room, event, player_index);
    }

    //
    // Networking methods
    //
    fn send_to_all_game_event(&self, event: &Self::GameServerEvent, room: &mut types::ServerRoom<Self>) {
        self.send_to_all(&types::ServerEvent::GameEvent(*event), room);
    }

    fn send_to_all(&self, event: &types::ServerEvent<Self>, room: &mut types::ServerRoom<Self>) {
        handle_server_event(&mut room.logic, &mut room.client_room, event, None);

        for connection in room.connections.iter() {
            send(event, connection);
        }
    }

    fn send_to_all_except_game_event(&self, event: &Self::GameServerEvent, except: usize, room: &mut types::ServerRoom<Self>) {
        self.send_to_all_except(&types::ServerEvent::GameEvent(*event), except, room);
    }

    fn send_to_all_except(&self, event: &types::ServerEvent<Self>, except: usize, room: &mut types::ServerRoom<Self>) {
        handle_server_event(&mut room.logic, &mut room.client_room, event, None);

        for (index, connection) in room.connections.iter().enumerate() {
            if index != except {
                send(event, connection);
            }
        }
    }

    fn send_to_game_event(&self, event: &Self::GameServerEvent, player_index: usize, room: &mut types::ServerRoom<Self>) {
        self.send_to(&types::ServerEvent::GameEvent(*event), player_index, room);
    }

    fn send_to(&self, event: &types::ServerEvent<Self>, player_index: usize, room: &mut types::ServerRoom<Self>) {
        if let Some(connection) = room.connections.get(player_index) {
            send(event, connection);
        } else {
            println!("Tried to send to a connection that doesn't exist");
            return;
        }

        // Only need to handle the event if we actually sent it to a player
        handle_server_event(&mut room.logic, &mut room.client_room, event, Some(player_index)); 
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

fn send<T>(event: &types::ServerEvent<T>, connection: &Option<types::Connection>)
where
    T: GameLogic,
{
    if let Some(connection) = connection {
        if let Some(sender) = &connection.sender {
            sender.send(event.to_bytes()).unwrap();
        }
    }
}