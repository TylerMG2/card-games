use serde::{de::DeserializeOwned, Serialize};

use crate::types;

pub trait GameLogic 
where 
    Self: Sized
{
    type GameServerEvent: DeserializeOwned + Serialize;
    type GameClientEvent: DeserializeOwned + Serialize;
    type Room: Default + Copy;
    type Player: Player + Default + Copy;

    fn validate_event(&self, event: &Self::GameClientEvent, player_index: usize) -> bool;

    // The server should not update the room directly, instead it should send server events to the clients which will update their rooms and the
    // server room using the handle_server_event method
    fn handle_client_event(&self, event: &Self::GameClientEvent, room: &mut types::ServerRoom<Self>, player_index: usize);

    // Player index should be provided all the time from the client, for the server its only provided when the server is sending an event to a specific player
    fn handle_server_event(&self, event: &Self::GameServerEvent, room: &mut Self::Room, players: &mut [Option<Self::Player>; 8], player_index: Option<usize>);
}

pub trait Player {
    fn get_name(&self) -> [u8; 20]; // TODO: Move to a constant
    fn set_name(&mut self, name: [u8; 20]);

    fn get_disconnected(&self) -> bool;
    fn set_disconnected(&mut self, disconnected: bool);
}