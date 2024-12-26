use std::{collections::HashMap, sync::Arc};
use axum::{extract::{ws::WebSocket, Query, State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use serde::Deserialize;
use shared::{games::{tycoon::TycoonLogic, GameLogicType}, GameLogic, ServerRoom};
use tokio::{net::TcpListener, sync::RwLock};

#[derive(Clone)]
struct AppState<Logic: GameLogic> {
    rooms: Arc<RwLock<HashMap<String, ServerRoom<Logic>>>>,
}

#[derive(Debug, Clone, Deserialize)]
struct QueryParams {
    id: String,
    code: String,
}

#[tokio::main]
async fn main() {
    let state = create_app_state();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("localhost:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler<Logic: GameLogic>(ws: WebSocketUpgrade, query: Query<QueryParams>, State(state): State<AppState<Logic>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, query.0, state))
}

async fn handle_socket<Logic: GameLogic>(socket: WebSocket, query: QueryParams, state: AppState<Logic>) {
    

}

fn create_app_state() -> AppState<impl GameLogic> {
    AppState::<TycoonLogic> {
        rooms: Arc::new(RwLock::new(HashMap::new())),
    }
}