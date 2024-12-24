use tokio::sync::mpsc::UnboundedSender;

use crate::traits::Player;

pub enum GameType {
    Tycoon,
    Carbo,
}

pub struct ClientRoom<RoomType: Default + Copy, PlayerType: Player + Default + Copy> {
    pub players: [Option<PlayerType>; 8], // TODO: Move to a constant
    pub host: u8,
    pub game: GameType,
    pub room: RoomType,
}

pub struct ServerRoom<RoomType: Default + Copy, PlayerType: Player + Default + Copy> {
    pub connections: [Option<Connection>; 8], // TODO: Move to a constant
    pub client_room: ClientRoom<RoomType, PlayerType>,
}

pub struct Connection {
    pub id: uuid::Uuid,
    pub sender: UnboundedSender<Vec<u8>>,
}