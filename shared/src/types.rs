use serde::{Deserialize, Serialize};

use crate::games::{carbo, tycoon};
use crate::traits::{self, GameSignal};

pub const MAX_PLAYERS: usize = 8;
pub const MAX_NAME_LENGTH: usize = 20;

pub struct ClientConnection;

#[derive(Deserialize, Serialize, Clone, Copy, Default, PartialEq, Debug)]
pub enum GameType {
    #[default]
    Tycoon,
    Carbo,
}

#[derive(Deserialize, Serialize, Clone, Copy, Default, PartialEq, Debug)]
pub enum RoomState {
    #[default]
    Lobby,
    InGame,
}

//TODO: Investigate if we can ever add Copy back to this
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Room {
    pub state: SignalType<RoomState>,
    pub game: SignalType<GameType>,
    pub host: SignalType<u8>,
    pub player_index: SignalType<u8>,
    pub carbo: carbo::CarboRoom,
    pub tycoon: tycoon::TycoonRoom,
    pub players: [SignalType<Option<Player>>; MAX_PLAYERS],
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub name: SignalType<[u8; MAX_NAME_LENGTH]>,
    pub disconnected: SignalType<bool>,
    pub carbo: carbo::CarboPlayer,
    pub tycoon: tycoon::TycoonPlayer,
}

//
// Event types
//

// TODO: Use a macro to generate the client events
// If I use the form <game::GameRoom as GameLogic>::GameServerEvent, I can probably write a macro to generate the
// Room, Player and Event types rather then having to update each one each time I add a new game
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub enum ServerEvent {
    CommonEvent(CommonServerEvent),
    CarboEvent(<carbo::CarboRoom as traits::GameLogic>::GameServerEvent),
    TycoonEvent(<tycoon::TycoonRoom as traits::GameLogic>::GameServerEvent),

    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)] //TODO: Clean up derives
pub enum CommonServerEvent {
    RoomJoined { new_room: Room, current_player: u8 },
    PlayerJoined { name: [u8; MAX_NAME_LENGTH], player_index: u8 },
    PlayerLeft { player_index: u8 },
    PlayerDisconnected { player_index: u8 },
    PlayerReconnected { player_index: u8 },
    HostChanged { player_index: u8 },
    GameChanged { game: GameType },
}

// TODO: Use a macro to generate the client events
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub enum ClientEvent {
    CommonEvent(CommonClientEvent),
    CarboEvent(<carbo::CarboRoom as traits::GameLogic>::GameClientEvent),
    TycoonEvent(<tycoon::TycoonRoom as traits::GameLogic>::GameClientEvent),

    #[default]
    Unknown,
}

// TODO: Reset current game action
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CommonClientEvent {
    JoinRoom { name: [u8; MAX_NAME_LENGTH] },
    LeaveRoom,
    ChangeGame { game: GameType },
    Disconnect,
}

//
// Signals
//

// Backend Signal
#[cfg(not(feature = "frontend"))]
#[derive(Debug, Clone, Copy)]
pub struct SignalType<T> {
    value: T,
}

#[cfg(not(feature = "frontend"))]
impl<T: Clone> GameSignal<T> for SignalType<T> {
    fn get(&self) -> &T {
        &self.value
    }

    fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    fn set(&mut self, value: T) {
        self.value = value;
    }
}

#[cfg(not(feature = "frontend"))]
impl<T: Serialize> Serialize for SignalType<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer) // Serialize just the inner value
    }
}

#[cfg(not(feature = "frontend"))]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for SignalType<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(SignalType { value }) // Wrap the deserialized value
    }
}

#[cfg(not(feature = "frontend"))]
impl<T: Default> Default for SignalType<T> {
    fn default() -> Self {
        Self { value: T::default() }
    }
}

// Frontend Signal
#[cfg(feature = "frontend")]
use leptos::prelude::{ArcRwSignal, Set};

#[cfg(feature = "frontend")]
#[derive(Debug, Clone)]
pub struct SignalType<T> {
    value: T,
    pub signal: ArcRwSignal<T>,
}

#[cfg(feature = "frontend")]
impl<T: Clone + 'static> GameSignal<T> for SignalType<T> {
    fn get(&self) -> &T {
        &self.value
    }

    fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    fn set(&mut self, value: T) {
        self.value = value.clone();
        self.signal.set(value);
    }
}

#[cfg(feature = "frontend")]
impl<T: Serialize + Clone> Serialize for SignalType<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer) // Serialize only the inner value
    }
}

#[cfg(feature = "frontend")]
impl<'de, T: Deserialize<'de> + Clone + Send + Sync + 'static> Deserialize<'de> for SignalType<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(SignalType {
            value: value.clone(),
            signal: ArcRwSignal::new(value),
        })
    }
}

#[cfg(feature = "frontend")]
impl<T: Default + Send + Sync + 'static> Default for SignalType<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            signal: ArcRwSignal::new(T::default()),
        }
    }
}
