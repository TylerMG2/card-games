use serde::{de::DeserializeOwned, Serialize};

use crate::{logic::{handle_client_event, handle_server_event, validate_client_event}, types, ClientEvent, ProcessEventResult};

// TODO: Move all shared logic (like handle_connection) to a seperate outside function to keep the trait clean
pub trait GameLogic : Sized + RoomFields + 'static {
    type GameServerEvent: DeserializeOwned + Serialize + Copy;
    type GameClientEvent: DeserializeOwned + Serialize;

    // Shared validation for the client and server, in theory in the future if all validation is shared between the client and server
    // we could update the client immediately with changes expected from the server since we know the server will accept the event
    fn validate_client_game_event(&self, event: &Self::GameClientEvent, player_index: usize) -> bool;

    // The server should not update the room directly, instead it should send server events to the clients which will update their rooms and the
    // server room using the handle_server_event method
    fn handle_client_game_event(&mut self, event: &Self::GameClientEvent, connections: &[Option<types::Connection>; 8], player_index: usize);

    // Player index should be provided all the time from the client, for the server its only provided when the server is sending an event to a specific player
    fn handle_server_game_event(&mut self, event: &Self::GameServerEvent, player_index: Option<usize>, is_server_side: bool);
    // TODO: Add a function like get_game_name() to return the name of the game

    fn process_server_event(&mut self, bytes: &[u8], player_index: Option<usize>) {
        let event = types::ServerEvent::from_bytes(bytes);
        handle_server_event(self, &event, player_index, false);
    }

    fn process_client_event(&mut self, connections: &[Option<types::Connection>; 8], bytes: &[u8], player_index: usize) -> Option<ProcessEventResult> {
        let event = types::ClientEvent::from_bytes(bytes);
        if validate_client_event(self, &event, player_index) {
            handle_client_event(self, connections, &event, player_index);
        }

        match event {
            ClientEvent::LeaveRoom => Some(ProcessEventResult::LeaveRoom),
            ClientEvent::ChangeGame { game } => Some(ProcessEventResult::ChangeGame(game)),
            _ => None,
        }
    }

    fn handle_connection(&mut self, connections: &[Option<types::Connection>; 8], player_index: usize) -> bool {
        let mut reconnection = false;
        if let Some(Some(_player)) = self.players().get(player_index) {
            self.send_to_all_except(&types::ServerEvent::PlayerReconnected { player_index: player_index as u8 }, player_index, connections);
            reconnection = true;
        }

        if reconnection {
            self.send_to(&types::ServerEvent::RoomJoined { room: *self, current_player: player_index as u8 }, player_index, connections);
        }
        reconnection
    }

    fn handle_disconnection(&mut self, connections: &[Option<types::Connection>; 8], player_index: usize) {
        self.send_to_all_except(&types::ServerEvent::PlayerLeft { player_index: player_index as u8 }, player_index, connections);
    }

    // Build from another logic type
    fn from_logic(logic: &impl GameLogic) -> Self {
        let mut new = Self::default();
        let new_players = new.players_mut();

        for (index, player) in logic.players().iter().enumerate() {
            if let Some(player) = player {
                let mut new_player = Self::Player::default();
                new_player.set_name(player.name());
                new_player.set_disconnected(player.disconnected());
                new_players[index] = Some(new_player);
            }
        }
        new.set_host(logic.host());
        new.set_player_index(logic.player_index());
        new
    }

    //
    // Networking methods
    //
    fn send_to_all_game_event(&mut self, event: &Self::GameServerEvent, connections: &[Option<types::Connection>; 8]) {
        self.send_to_all(&types::ServerEvent::GameEvent(*event), connections);
    }

    fn send_to_all(&mut self, event: &types::ServerEvent<Self>, connections: &[Option<types::Connection>; 8]) {
        handle_server_event(self, event, None, true);

        for connection in connections.iter() {
            send(event, connection);
        }
    }

    fn send_to_all_except_game_event(&mut self, event: &Self::GameServerEvent, except: usize, connections: &[Option<types::Connection>; 8]) {
        self.send_to_all_except(&types::ServerEvent::GameEvent(*event), except, connections);
    }

    fn send_to_all_except(&mut self, event: &types::ServerEvent<Self>, except: usize, connections: &[Option<types::Connection>; 8]) {
        handle_server_event(self, event, None, true);

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
        handle_server_event(self, event, Some(player_index), true); 
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

// Object traits

pub trait PlayerFields: Sized + Default + Clone + Copy + Serialize + DeserializeOwned {
    fn name(&self) -> &[u8; 20]; // TODO: Maybe make this more flexible although I think name is ok to always be fixed length per game
    fn set_name(&mut self, name: &[u8; 20]);
    fn disconnected(&self) -> bool;
    fn set_disconnected(&mut self, disconnected: bool);
}

pub trait RoomFields: Default + Clone + Copy + Serialize + DeserializeOwned {
    type Player: PlayerFields;

    fn players(&self) -> &[Option<Self::Player>; 8]; // TODO: Make this more flexible, for now lets just have it match exactly the length of connections. Maybe a vec would be better?
    fn players_mut(&mut self) -> &mut [Option<Self::Player>; 8];
    fn host(&self) -> u8;
    fn set_host(&mut self, host: u8);
    fn player_index(&self) -> u8;
    fn set_player_index(&mut self, player_index: u8);
}
