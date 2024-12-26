use serde::{de::DeserializeOwned, Serialize};

use crate::{logic::{handle_client_event, handle_server_event, validate_client_event}, types};

pub trait GameLogic 
where 
    Self: Sized + Send + Sync + Default + 'static,
{
    type GameServerEvent: DeserializeOwned + Serialize + Copy;
    type GameClientEvent: DeserializeOwned + Serialize;
    type Room: Default + Copy + Serialize + DeserializeOwned + Send + Sync;
    type Player: Default + Copy + Serialize + DeserializeOwned + Send + Sync;

    // Shared validation for the client and server, in theory in the future if all validation is shared between the client and server
    // we could update the client immediately with changes expected from the server since we know the server will accept the event
    fn validate_client_game_event(&self, event: &Self::GameClientEvent, room: &Self::Room, players: &[Option<&Self::Player>; 8], player_index: usize) -> bool;

    // The server should not update the room directly, instead it should send server events to the clients which will update their rooms and the
    // server room using the handle_server_event method
    fn handle_client_game_event(&mut self, event: &Self::GameClientEvent, connections: &[Option<types::Connection>; 8], player_index: usize);

    // Player index should be provided all the time from the client, for the server its only provided when the server is sending an event to a specific player
    fn handle_server_game_event(&mut self, event: &Self::GameServerEvent, player_index: Option<usize>);

    // Both client room methods are a little extra boiler plate but required
    fn get_client_room(&self) -> &types::ClientRoom<Self::Room, Self::Player>;
    fn get_client_room_mut(&mut self) -> &mut types::ClientRoom<Self::Room, Self::Player>;

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
    fn handle_client_event(&mut self, connections: &[Option<types::Connection>; 8], event: &types::ClientEvent<Self::GameClientEvent>, player_index: usize) {
        handle_client_event(self, connections, event, player_index);
    }

    fn handle_connection(&mut self, connections: &[Option<types::Connection>; 8], player_index: usize) -> bool {
        let mut reconnection = false;
        if let Some(Some(_player)) = self.get_client_room().players.get(player_index) {
            self.send_to_all_except(&types::ServerEvent::PlayerReconnected { player_index: player_index as u8 }, player_index, connections);
            reconnection = true;
        }

        if reconnection {
            self.send_to(&types::ServerEvent::RoomJoined { room: *self.get_client_room(), current_player: player_index as u8 }, player_index, connections);
        }
        reconnection
    }

    fn handle_disconnection(&mut self, connections: &[Option<types::Connection>; 8], player_index: usize) {
        self.send_to_all_except(&types::ServerEvent::PlayerLeft { player_index: player_index as u8 }, player_index, connections);
    }

    //
    // Networking methods
    //
    fn send_to_all_game_event(&mut self, event: &Self::GameServerEvent, connections: &[Option<types::Connection>; 8]) {
        self.send_to_all(&types::ServerEvent::GameEvent(*event), connections);
    }

    fn send_to_all(&mut self, event: &types::ServerEvent<Self>, connections: &[Option<types::Connection>; 8]) {
        handle_server_event(self, event, None);

        for connection in connections.iter() {
            send(event, connection);
        }
    }

    fn send_to_all_except_game_event(&mut self, event: &Self::GameServerEvent, except: usize, connections: &[Option<types::Connection>; 8]) {
        self.send_to_all_except(&types::ServerEvent::GameEvent(*event), except, connections);
    }

    fn send_to_all_except(&mut self, event: &types::ServerEvent<Self>, except: usize, connections: &[Option<types::Connection>; 8]) {
        handle_server_event(self, event, None);

        for (index, connection) in connections.iter().enumerate() {
            if index != except {
                send(event, connection);
            }
        }
    }

    fn send_to_game_event(&mut self, event: &Self::GameServerEvent, player_index: usize, connections: &[Option<types::Connection>; 8]) {
        self.send_to(&types::ServerEvent::GameEvent(*event), player_index, connections);
    }

    fn send_to(&mut self, event: &types::ServerEvent<Self>, player_index: usize, connections: &[Option<types::Connection>; 8]) {
        if let Some(connection) = connections.get(player_index) {
            send(event, connection);
        } else {
            println!("Tried to send to a connection that doesn't exist");
            return;
        }

        // Only need to handle the event if we actually sent it to a player
        handle_server_event(self, event, Some(player_index)); 
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