use std::{collections::HashMap, sync::Arc, time::Duration};
use axum::{extract::{ws::{Message, WebSocket}, Query, State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use serde::Deserialize;

use shared::{logic::handle_client_event, traits::{Networking, ToFromBytes}, types::{ClientEvent, CommonClientEvent, CommonServerEvent, ServerEvent, MAX_NAME_LENGTH}, ServerRoom};
use tokio::{net::TcpListener, sync::{mpsc::UnboundedSender, RwLock}, time::{sleep, timeout}};
use futures::{sink::SinkExt, stream::{SplitStream, StreamExt}};

#[derive(Clone)]
struct AppState {
    rooms: Arc<RwLock<HashMap<String, ServerRoom>>>,
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
    println!("{} attemping to connect to {}", query.id, query.code);

    let player_index = match handle_connection(&state, &query, tx, id, &mut receiver).await {
        Some(player_index) => player_index,
        None => return,
    };

    println!("{} connected to {}", query.id, query.code);

    let recv_state = state.clone();
    let recv_query = query.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    let mut rooms = recv_state.rooms.write().await;
                    let room = match rooms.get_mut(&recv_query.code) {
                        Some(room) => room,
                        None => {
                            println!("{} is in a room that no longer exists.", recv_query.id);
                            break;
                        },
                    };
                    let event = ClientEvent::from_bytes(&data);
                    
                    handle_client_event(&mut room.room, &event, &room.connections, player_index);

                    // Special case for leaving the room
                    if let ClientEvent::CommonEvent(CommonClientEvent::LeaveRoom) = event {
                        room.connections[player_index] = None;
                        println!("{} left {}", recv_query.id, recv_query.code);
                    }
                },
                Ok(Message::Close(_)) => break,
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Binary(msg)).await.is_err() {
                println!("Failed to send message to {}", query.id);
                break;
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
    let room = rooms.get_mut(&query.code).unwrap();

    // If the connection wasn't removed (player leaving the room) then disconnect the player
    if let Some(Some(connection)) = room.connections.get_mut(player_index) {
        connection.sender = None;
        room.connections.send_to_all(&mut room.room, ServerEvent::CommonEvent(CommonServerEvent::PlayerDisconnected { player_index: player_index as u8 }));
        println!("{} disconnected from {}", id, query.code);
    }

    // Remove the room if all disconnected (None or sender is None) dont unwrap
    if room.connections.iter().all(|connection| {
        match connection {
            Some(connection) => connection.sender.is_none(),
            None => true,
        }
    }) {
        rooms.remove(&query.code);
        println!("Room {} removed", query.code);
    }
}

async fn handle_connection(state: &AppState, query: &QueryParams, tx: UnboundedSender<Vec<u8>>, id: uuid::Uuid, receiver: &mut SplitStream<WebSocket>) -> Option<usize> {
    let player_index = {
        let mut rooms = state.rooms.write().await;
        let room = rooms.entry(query.code.clone()).or_insert(ServerRoom::default());

        if let Some(player_index) = room.add_connection(tx, id) {
            let join_event = ServerEvent::CommonEvent(CommonServerEvent::RoomJoined { new_room: room.room, current_player: player_index as u8 });
            room.connections.send_to(&mut room.room, join_event, player_index);

            // Check if the player is reconnecting
            if let Some(Some(_)) = room.room.common.players.get_mut(player_index) {
                room.connections.send_to_all(&mut room.room, ServerEvent::CommonEvent(CommonServerEvent::PlayerReconnected { player_index: player_index as u8 }));
                return Some(player_index);
            }
            player_index
        } else {
            println!("{} failed to connect to {}, REASON: Room full", query.id, query.code);
            return None;
        }
    };
    
    // If we get here it means the player is connecting for the first time
    match timeout(Duration::from_secs(300), wait_for_name_and_code(receiver)).await {
        Ok(Some(name)) => {
            let mut rooms = state.rooms.write().await;
            let room = rooms.get_mut(&query.code).unwrap();
            room.connections.send_to_all(&mut room.room, ServerEvent::CommonEvent(CommonServerEvent::PlayerJoined { name, player_index: player_index as u8 }));
            Some(player_index)
        },
        Ok(None) => None,
        Err(_) => {
            println!("{} failed to connect to {}, REASON: Timed out", query.id, query.code);
            None
        },
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
                    return Some(name);
                }
            }
            _ => {}
        }
    }
    None
}