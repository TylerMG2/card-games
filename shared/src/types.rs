use serde::{Deserialize, Serialize};

use crate::games::{carbo, coup, tycoon};
use crate::traits::{self, GameSignal};

pub const MAX_PLAYERS: usize = 8;
pub const MAX_NAME_LENGTH: usize = 20;

#[derive(Default, Clone, Copy, Debug)]
pub struct ClientConnection;

#[derive(Deserialize, Serialize, Clone, Copy, Default, PartialEq, Debug)]
pub enum GameType {
    #[default]
    Tycoon,
    Carbo,
    Coup,
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
    pub player_index: SignalType<u8>, // TODO: If we ever make spectating possible, this will need to be a SignalType<Option<u8>>
    pub carbo: carbo::CarboRoom,
    pub tycoon: tycoon::TycoonRoom,
    pub coup: coup::CoupRoom,
    pub players: [SignalType<Option<Player>>; MAX_PLAYERS],
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub name: SignalType<[u8; MAX_NAME_LENGTH]>,
    pub disconnected: SignalType<bool>,
    pub carbo: carbo::CarboPlayer,
    pub tycoon: tycoon::TycoonPlayer,
    pub coup: coup::CoupPlayer,
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
    CoupEvent(<coup::CoupRoom as traits::GameLogic>::GameServerEvent),

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
    ResetGame,
}

// TODO: Use a macro to generate the client events
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub enum ClientEvent {
    CommonEvent(CommonClientEvent),
    CarboEvent(<carbo::CarboRoom as traits::GameLogic>::GameClientEvent),
    TycoonEvent(<tycoon::TycoonRoom as traits::GameLogic>::GameClientEvent),
    CoupEvent(<coup::CoupRoom as traits::GameLogic>::GameClientEvent),

    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CommonClientEvent {
    LeaveRoom,
    ChangeGame { game: GameType },
    ResetGame,
    Disconnect,
}

//
// Signals
//

#[cfg(feature = "frontend")]
use leptos::prelude::{ArcRwSignal, Set, Get};

#[derive(Debug, Clone, Serialize)]
pub struct SignalType<T> {
    value: T,

    #[cfg(feature = "frontend")]
    #[serde(skip)]
    pub signal: ArcRwSignal<T>,
}

// By making this specifically available for the frontend, we avoid any of the logic in shared
// accidentally using a frontend signal getter instead of just the value getter which can be 
// inconsistent
#[cfg(feature = "frontend")]
impl<T: Clone + 'static> SignalType<T> {
    pub fn get(&self) -> T {
        self.signal.get()
    }
}

impl<T: Clone + 'static> GameSignal<T> for SignalType<T> { 
    fn value(&self) -> &T {
        &self.value
    }

    fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    // For future Tyler, there could be a weird bug here when the value T has other Signals inside it,
    // since we are setting the value to another value, the old value will be dropped and their signals
    // will be dropped, this could be an issue if the signal is cloned and used elsewhere.
    //
    // I think we are fine as long as we never allow cloning of the internal .signal
    fn set(&mut self, value: T) {
        self.value = value.clone();

        #[cfg(feature = "frontend")]
        self.signal.set(value);
    }
}

impl<'de, T: Deserialize<'de> + Clone> Deserialize<'de> for SignalType<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(SignalType { 
            value: value.clone(),
            #[cfg(feature = "frontend")]
            signal: ArcRwSignal::new(value),
        }) // Wrap the deserialized value
    }
}

impl<T: Default> Default for SignalType<T> {
    fn default() -> Self {
        Self { 
            value: T::default(),
            #[cfg(feature = "frontend")]
            signal: ArcRwSignal::new(T::default()),
        }
    }
}