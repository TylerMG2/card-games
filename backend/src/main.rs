use std::{collections::HashMap, sync::Arc};
use axum::{extract::{ws::{Message, WebSocket}, Query, State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use serde::Deserialize;

use shared::{logic::{handle_client_event, validate_client_event}, traits::{Networking, ToFromBytes}, types::{ClientEvent, CommonClientEvent, CommonServerEvent, ServerEvent, MAX_NAME_LENGTH}};
use tokio::{net::TcpListener, sync::RwLock};
use futures::{sink::SinkExt, stream::StreamExt};
use types::ServerRoom;

mod types;

#[derive(Clone)]
struct AppState {
    rooms: Arc<RwLock<HashMap<String, types::ServerRoom>>>,
}

#[derive(Debug, Clone, Deserialize)]
struct QueryParams {
    id: String,
    code: String,
    name: Option<[u8; MAX_NAME_LENGTH]>,
}

//TODO: It might be worth adding a way to generate a unique room code i.e 'create_room' endpoint
// rather the run the risk of a collision by having the client generate it. Although the risk is low with
// a 6-8 character code and low number of players.
#[tokio::main]
async fn main() {
    let state = AppState {
        rooms: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("localhost:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, query: Query<QueryParams>, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, query.0, state))
}

async fn handle_socket(socket: WebSocket, query: QueryParams, state: AppState) {
    if query.code.len() != 6 { return; }
    let id = {
        match uuid::Uuid::parse_str(&query.id) {
            Ok(id) => id,
            Err(_) => return,
        }
    };

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    let (mut sender, mut receiver) = socket.split();
    println!("({}) {} attempting to connect", query.code, query.id);

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Binary(msg)).await.is_err() {
                break;
            }
        }
    });

    // Important for handle_connection to take ownership of receiver so no other references are held
    let player_index = {
        let mut rooms = state.rooms.write().await;
        let room = rooms.entry(query.code.clone()).or_insert(types::ServerRoom::default());
        match room.handle_connection(tx, id, query.name) {
            Some(player_index) => player_index,
            None => {
                if is_room_empty(room) {
                    rooms.remove(&query.code);
                    println!("({}) Room closed", query.code);
                }

                println!("({}) {} failed to connect", query.code, query.id);
                return; // TODO: Send a message to the client
            },
        }
    };

    println!("({}) {} connected", query.code, query.id);

    let recv_state = state.clone();
    let recv_query = query.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    if data.len() < 1 || data.len() > 1000 { continue; } // Nothing should be this small or large

                    let mut rooms = recv_state.rooms.write().await;
                    let room = match rooms.get_mut(&recv_query.code) {
                        Some(room) => room,
                        None => {
                            println!("{} is in a room that no longer exists.", recv_query.id);
                            break;
                        },
                    };
                    let event = ClientEvent::from_bytes(&data);
                    println!("({}) Received {:?} from {}", recv_query.code, event, recv_query.id);
                    
                    // We don't need to validate the player_id since its associated with the connection
                    if validate_client_event(&room.room, &event, player_index) {
                        handle_client_event(&mut room.room, &event, &mut room.connections, player_index);
                    } else {
                        println!("({}) {} sent an invalid event: {:?}", recv_query.code, recv_query.id, event);
                    }
                    
                    // Special case for leaving the room
                    if let ClientEvent::CommonEvent(CommonClientEvent::LeaveRoom) = event {
                        room.connections[player_index] = None;
                        println!("({}) {} left the room", recv_query.code, recv_query.id);
                    }
                },
                Ok(Message::Close(_)) => break,
                Ok(_) => continue,
                Err(_) => break, // TODO: More explicit error handling
            }
        }
    });

    // Abort the tasks if one of them fails
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    // Disconnect
    let mut rooms = state.rooms.write().await;
    if let Some(room) = rooms.get_mut(&query.code) {

        // If the connection wasn't removed (player leaving the room) then disconnect the player
        if let Some(Some(connection)) = room.connections.get_mut(player_index) {
            connection.sender = None;
            room.connections.send_to_all(&mut room.room, ServerEvent::CommonEvent(CommonServerEvent::PlayerDisconnected { player_index: player_index as u8 }));
            println!("({}) {} disconnected", query.code, query.id);
        }

        // Close the room if no players are left
        if is_room_empty(room) {
            rooms.remove(&query.code);
            println!("({}) Room closed", query.code);
        }
    }
}

fn is_room_empty(room: &ServerRoom) -> bool {
    room.connections.iter().all(|connection| {
        match connection {
            Some(connection) => connection.sender.is_none(),
            None => true,
        }
    })
}