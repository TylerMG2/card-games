use games::GameLogicType;
use shared_core::Connection;
use tokio::sync::mpsc::UnboundedSender;

pub mod games {
    mod games;

    pub use games::create_logic_from;
    pub use games::GameLogicType;
    pub mod tycoon;
    pub mod carbo;
}

pub use shared_core::*;

#[derive(Default)]
pub struct ServerRoom {
    pub connections: [Option<Connection>; 8], // TODO: Move to a constant
    pub logic: GameLogicType,
}

impl ServerRoom {
    pub fn add_connection(&mut self, tx: UnboundedSender<Vec<u8>>, id: uuid::Uuid) -> Option<usize> {
        let mut first_free: Option<usize> = None;

        // Look for player id while keeping track of the first free slot
        for (index, connection) in self.connections.iter_mut().enumerate() {
            if connection.is_none() && first_free.is_none() {
                first_free = Some(index);
            }

            if let Some(connection) = connection {
                if connection.id == id {
                    connection.sender = Some(tx);
                    return Some(index);
                }
            }
        }

        if let Some(index) = first_free {
            self.connections[index] = Some(Connection { id, sender: Some(tx) });
            Some(index)
        } else {
            None
        }
    }
}
