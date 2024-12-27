use std::{collections::HashMap, sync::Arc, time::Duration};
use axum::{extract::{ws::{Message, WebSocket}, Query, State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use serde::Deserialize;

use shared::{ProcessEventResult, ServerRoom};
use tokio::{net::TcpListener, sync::RwLock, time::sleep};
use futures::{sink::SinkExt, stream::StreamExt};

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
    let player_index: usize;

    // Register the connection (Scoped to avoid holding the lock for too long)
    let reconnected = {
        let mut rooms = state.rooms.write().await;
        let room = rooms.entry(query.code.clone()).or_insert(ServerRoom::default());
        let possible_index = room.add_connection(tx, id);

        match possible_index {
            Some(possible_index) => {
                player_index = possible_index;
                room.logic.handle_connection(&room.connections, possible_index)
            },
            None => {
                println!("{} failed to connect to {}, no room", query.id, query.code);
                return;
            }
        }
    };

    if reconnected {
        println!("{} reconnecting to {}", query.id, query.code);
    }

    // If user is not reconnected automatically, wait 15 seconds then check if the player at the index is still None
    let connection_result = if !reconnected {
        let code = query.code.clone();
        let state = state.clone();
        tokio::spawn(async move {
            sleep(Duration::from_secs(15)).await;

            let mut rooms = state.rooms.write().await;
            let room = rooms.get_mut(&code).unwrap();
            room.logic.has_player(player_index)
        }).await.unwrap()
    } else {
        true
    };

    if connection_result {
        println!("{} connected to {}", query.id, query.code);
        let recv_state = state.clone();
        let recv_query = query.clone();

        let mut recv_task = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(_) => break, // Close the connection if receiving fails (force the player to reconnect)
                };

                match msg {
                    Message::Binary(data) => {
                        let mut rooms = recv_state.rooms.write().await;
                        let room = rooms.get_mut(&recv_query.code).expect("Room should only be removed if all players are disconnected");
    
                        match room.logic.process_client_event(&room.connections, &data, player_index) {
                            Some(ProcessEventResult::LeaveRoom) => { // Doesn't really need to be an Option, can just add my own None variant
                                room.connections[player_index] = None;
                                break;
                            },
                            Some(ProcessEventResult::ChangeGame(game)) => {
                                room.logic.change_game(game);
                            },
                            None => {},
                        }
                    }
                    _ => {}
                }
            }
        });

        let mut send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if sender.send(Message::Binary(msg)).await.is_err() {
                    break;
                }
            }
        });

        // Abort the tasks if one of them fails
        tokio::select! {
            _ = &mut send_task => recv_task.abort(),
            _ = &mut recv_task => send_task.abort(),
        };
    } else {
        println!("{} failed to connect to {}, didn't provide name in time", query.id, query.code);
    }

    // Disconnect
    let mut rooms = state.rooms.write().await;
    let room = rooms.get_mut(&query.code).unwrap();
    println!("{} disconnected from {}", query.id, query.code);

    // If the connection wasn't removed (player leaving the room) then disconnect the player
    if let Some(Some(connection)) = room.connections.get_mut(player_index) {
        connection.sender = None;
        room.logic.handle_disconnection(&room.connections, player_index);
    }

    // Remove the room if all disconnected (None or sender is None) dont unwrap
    if room.connections.iter().all(|connection| {
        match connection {
            Some(connection) => connection.sender.is_none(),
            None => true,
        }
    }) {
        rooms.remove(&query.code);
    }
}