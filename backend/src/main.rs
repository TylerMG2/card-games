use std::{collections::HashMap, sync::Arc, time::Duration};
use axum::{extract::{ws::{Message, WebSocket}, Query, State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use serde::Deserialize;

use shared::{logic::{handle_client_event, validate_client_event}, traits::{Networking, ToFromBytes}, types::{ClientEvent, CommonClientEvent, CommonServerEvent, ServerEvent, MAX_NAME_LENGTH}};
use tokio::{net::TcpListener, sync::{mpsc::UnboundedSender, RwLock}, time::timeout};
use futures::{sink::SinkExt, stream::{SplitStream, StreamExt}};
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
}

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
    let player_index = match handle_connection(&state, &query, tx, id, &mut receiver).await {
        Some(player_index) => {
            let mut rooms = state.rooms.write().await;

            // I wonder if theres a nice way to refactor this since this should all be guaranteed to exist
            if let Some(room) = rooms.get_mut(&query.code) {
                let join_event = ServerEvent::CommonEvent(CommonServerEvent::RoomJoined { new_room: room.room, current_player: player_index as u8 });
                room.connections.send_to(&mut room.room, join_event, player_index);
            } else {
                // This should never happen
                return;
            }
            player_index
        },
        None => return, // TODO: Handle failed connection (close room if needed), might want to refactor a little
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

// Attempt to automatically reconnect the player if they already have a connection
// otherwise wait for the player to send their name.
async fn handle_connection(state: &AppState, query: &QueryParams, tx: UnboundedSender<Vec<u8>>, id: uuid::Uuid, receiver: &mut SplitStream<WebSocket>) -> Option<usize> {
    let mut rooms = state.rooms.write().await;
    let room = rooms.entry(query.code.clone()).or_insert(types::ServerRoom::default());

    if let Some(player_index) = room.add_connection(tx, id) {
        // Check if the player is reconnecting
        if let Some(Some(_)) = room.room.common.players.get_mut(player_index) {
            println!("({}) {} reconnected", query.code, query.id);
            room.connections.send_to_all_except(&mut room.room, ServerEvent::CommonEvent(CommonServerEvent::PlayerReconnected { player_index: player_index as u8 }), player_index);
            Some(player_index)
        } else {
            drop(rooms); // Drop the lock to prevent deadlock

            match timeout(Duration::from_secs(300), wait_for_name_and_code(receiver)).await {
                Ok(Some(name)) => {
                    let mut rooms = state.rooms.write().await;
                    let room = rooms.get_mut(&query.code).unwrap();
                    room.connections.send_to_all_except(&mut room.room, ServerEvent::CommonEvent(CommonServerEvent::PlayerJoined { name, player_index: player_index as u8 }), player_index);
                    Some(player_index)
                },
                Ok(None) => {
                    println!("({}) {} failed to connect, REASON: Invalid name", query.code, query.id);
                    None
                },
                Err(_) => {
                    println!("({}) {} failed to connect, REASON: Timeout", query.code, query.id);
                    None
                },
            }
        }
    } else {
        println!("({}) {} failed to connect, REASON: Room is full", query.code, query.id);
        None
    }
}

async fn wait_for_name_and_code(receiver: &mut SplitStream<WebSocket>) -> Option<[u8; MAX_NAME_LENGTH]> {
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => break,
        };

        match msg {
            Message::Binary(data) => {
                let event = ClientEvent::from_bytes(&data);
                if let ClientEvent::CommonEvent(CommonClientEvent::JoinRoom { name }) = event {
                    // TODO: Validate and sanitize the name
                    return Some(name);
                }
            }
            _ => {}
        }
    }
    None
}