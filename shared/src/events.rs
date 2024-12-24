use serde::{de::DeserializeOwned, Deserialize, Serialize};

/*
 * General Events
 */
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub enum ServerEvent<EventType> {
    RoomJoined { room: u8 },
    PlayerJoined { player: u8, player_index: u8 },
    PlayerLeft { player_index: u8 },
    PlayerDisconnected { player_index: u8 },
    PlayerReconnected { player_index: u8 },
    HostChanged { player_index: u8 },
    GameEvent(EventType),

    #[default]
    Unknown,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub enum ClientEvent<T> {
    JoinRoom { room: u8 },
    LeaveRoom,
    StartGame,
    GameEvent(T),

    #[default]
    Unknown,
}

/*
 * Tycoon Millionaire
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TycoonServerEvent {
    //GameStarted { state: RoomState, turn: u8, cards: u64, other_hands: [u8; MAX_PLAYERS] },
    CardsPlayed { cards: u64 },
    Pass,
    ReceiveCards { cards: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TycoonClientEvent {
    PlayCards { cards: u64 },
    Pass,
    ExchangeCards { cards: u64 },
}

/*
 * Carbo
 */


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