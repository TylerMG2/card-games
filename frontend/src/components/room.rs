use std::sync::{Arc, RwLock};

use futures::channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos_router::{hooks::use_params_map, params};
use shared::{logic, traits::{GameSignal, ToFromBytes}, types::{self, MAX_NAME_LENGTH}};
use wasm_bindgen_futures::spawn_local;
use web_sys::{js_sys, wasm_bindgen::{prelude::Closure, JsCast}, ErrorEvent, MessageEvent, SubmitEvent, WebSocket};
use gloo::storage::{LocalStorage, Storage, errors::StorageError};

use crate::components::{games::game::Game, join_room::JoinRoom};

#[derive(Clone, PartialEq)]
pub enum WebsocketState {
    Connecting,
    Connected,
    Disconnected,
}


#[derive(Clone)]
pub struct RoomContext {
    pub room: Arc<RwLock<types::Room>>,
    set_ws_state: WriteSignal<WebsocketState>,
    sender: UnboundedSender<Vec<u8>>,
}

impl RoomContext {
    pub fn validate_client_event(&self, event: &types::ClientEvent) -> bool {
        let room = self.room.read().unwrap();
        logic::validate_client_event(&room, event, *room.player_index.value() as usize)
    }

    pub fn send_event(&self, event: types::ClientEvent) {
        console_log(format!("Sending event: {:?}", event).as_str());

        if self.validate_client_event(&event) {
            let bytes = event.to_bytes();
            if self.sender.unbounded_send(bytes).is_err() {
                self.set_ws_state.set(WebsocketState::Disconnected);
            }
        }
    }
}

#[component]
pub fn Room() -> impl IntoView {
    let (ws_state, set_ws_state) = signal(WebsocketState::Disconnected);
    let (in_room, set_in_room) = signal(false);
    let room = Arc::new(RwLock::new(types::Room::default()));

    let params = use_params_map();
    let code = move || params.read().get("code");
    
    let id = get_player_id();

    // TODO: I think we need to clean up the websocket connection when the component is unmounted
    let join_room = move |name: [u8; MAX_NAME_LENGTH]| {
        let Some(code) = code() else { return; }; // TODO: Redirect to home page, this should never happen
        if ws_state.get_untracked() != WebsocketState::Disconnected { return; } // Return if we are already connected

        set_ws_state.set(WebsocketState::Connecting);

        // Start the websocket connection
        let ws = match WebSocket::new(&format!("ws://localhost:3000/ws?code={}&id={}", code, id)) {
            Ok(ws) => ws,
            Err(_) => {
                console_log("Failed to connect");
                set_ws_state.set(WebsocketState::Disconnected); // Todo maybe get the error type for better logging
                return;
            }
        };
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<Vec<u8>>();
        
        // Provide the context (THIS DOESN'T WORK AT THE MOMENT)
        provide_context(RoomContext {
            room: room.clone(),
            set_ws_state,
            sender: tx.clone(),
        });

        // On error
        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |_e: ErrorEvent| {
            set_ws_state.set(WebsocketState::Disconnected);
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // On open
        let open_tx = tx.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            console_log("Connected");
            set_ws_state.set(WebsocketState::Connected);

            // Send join room event
            let event = types::ClientEvent::CommonEvent(types::CommonClientEvent::JoinRoom { name });
            if open_tx.unbounded_send(event.to_bytes()).is_err() {
                set_ws_state.set(WebsocketState::Disconnected);
                return;
            }
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // On close
        let onclose_callback = Closure::<dyn FnMut()>::new(move || {
            console_log("Disconnected");
            set_ws_state.set(WebsocketState::Disconnected); // TODO: this should only really be failed on error, not on close
            set_in_room.set(false);
        });
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        // On message
        let room_clone = room.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let bytes = js_sys::Uint8Array::new(&buffer);
                let mut vec = vec![0; bytes.length() as usize]; 
                bytes.copy_to(&mut vec);
                let event = types::ServerEvent::from_bytes(&vec);
                console_log(format!("Received event: {:?}", event).as_str());

                let mut room = room_clone.write().unwrap(); //TODO: Handle error, maybe just return and close the connection
                let player_index = *room.player_index.value() as usize;
                logic::handle_server_event(&mut room, &event, Some(player_index), false);

                // Room joined event
                if let types::ServerEvent::CommonEvent(types::CommonServerEvent::RoomJoined { new_room: _, current_player: _ }) = event {
                    set_in_room.set(true);
                }
            } else {
                console_log("Received unknown message");
            }
        });
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // Set up the sender
        spawn_local(async move {
            while let Some(msg) = rx.next().await {
                if ws.send_with_u8_array(&msg).is_err() {
                    set_ws_state.set(WebsocketState::Disconnected);
                }
            }
        });
    };

    view! {
        <Show 
            when=move || !in_room.get()
            fallback=move || view! { <Game /> }
        >
            <JoinRoom join_room=join_room.clone() />
        </Show>
    }
}


// Check if their is a player id in local storage, if not create one and store it
pub fn get_player_id() -> uuid::Uuid {
    let storage: Result<String, StorageError> = LocalStorage::get("player_id");
    match storage {
        Ok(id) => {
            match uuid::Uuid::parse_str(&id) {
                Ok(id) => id,
                Err(_) => {
                    let id = uuid::Uuid::new_v4();
                    let _ = LocalStorage::set("player_id", &id.to_string()); // TODO: Handle error, maybe dont need to handle since we at least have a new uuid
                    id
                }
            }
        }
        Err(_) => {
            let id = uuid::Uuid::new_v4();
            let _ = LocalStorage::set("player_id", &id.to_string()); // TODO: Handle error
            id
        }
    }
}

