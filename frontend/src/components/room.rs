use std::sync::{Arc, RwLock};

use futures::channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared::{logic, traits::{GameSignal, ToFromBytes}, types};
use wasm_bindgen_futures::spawn_local;
use web_sys::{js_sys, wasm_bindgen::{prelude::Closure, JsCast}, ErrorEvent, MessageEvent, WebSocket};
use gloo::storage::{LocalStorage, Storage, errors::StorageError};

use crate::components::{games::game::Game, join_room::JoinRoom};

#[derive(Clone, PartialEq)]
pub enum WebsocketState {
    Connecting,
    Connected,
    Disconnected,
    Failed,
}


#[derive(Clone)]
pub struct RoomContext {
    pub room: Arc<RwLock<types::Room>>,
    pub state: ReadSignal<WebsocketState>,
    set_state: WriteSignal<WebsocketState>,
    sender: ReadSignal<Option<UnboundedSender<Vec<u8>>>>,
}

impl RoomContext {
    pub fn validate_client_event(&self, event: &types::ClientEvent) -> bool {
        let room = self.room.read().unwrap();
        logic::validate_client_event(&room, event, room.player_index.get() as usize)
    }

    pub fn send_event(&self, event: types::ClientEvent) {
        console_log(format!("Sending event: {:?}", event).as_str());

        if self.validate_client_event(&event) {
            if let Some(sender) = self.sender.get() {
                let bytes = event.to_bytes();
                if sender.unbounded_send(bytes).is_err() {
                    self.set_state.set(WebsocketState::Failed);
                }
            }
        }
    }
}

#[component]
pub fn Room() -> impl IntoView {
    let id = get_player_id();
    let (code, set_code) = signal(None);
    let (state, set_state) = signal(WebsocketState::Disconnected);
    let room = Arc::new(RwLock::new(types::Room::default()));
    let room_websocket = room.clone();
    let (sender, set_sender) = signal(None);
    let (in_room, set_in_room) = signal(false);
    
    Effect::new(move |_| {
        let params = use_params_map();
        if let Some(code_param) = params.read().get("code") {
            set_code.set(Some(code_param));
        }
    });

    // TODO: I think we need to clean up the websocket connection when the component is unmounted
    Effect::new(move || {
        let room_clone = room_websocket.clone();

        // TODO: We should probably add some better reconnect logic to avoid spamming the server
        if state.get() != WebsocketState::Disconnected { return; }

        if let Some(code) = code.get() {
            set_state.set(WebsocketState::Connecting);
            let ws = match WebSocket::new(&format!("ws://localhost:3000/ws?code={}&id={}", code, id)) {
                Ok(ws) => ws,
                Err(_) => {
                    console_log("Failed to connect");
                    set_state.set(WebsocketState::Failed); // Todo maybe get the error type for better logging
                    return;
                }
            };
            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            // On error
            let onerror_callback = Closure::<dyn FnMut(_)>::new(move |_e: ErrorEvent| {
                set_state.set(WebsocketState::Failed);
            });
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            // On open
            let onopen_callback = Closure::<dyn FnMut()>::new(move || {
                console_log("Connected");
                set_state.set(WebsocketState::Connected);
            });
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();

            // On close
            let onclose_callback = Closure::<dyn FnMut()>::new(move || {
                console_log("Disconnected");
                set_state.set(WebsocketState::Failed); // TODO: this should only really be failed on error, not on close
                set_in_room.set(false);
            });
            ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();

            // On message
            let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                if let Ok(buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                    let bytes = js_sys::Uint8Array::new(&buffer);
                    let mut vec = vec![0; bytes.length() as usize]; 
                    bytes.copy_to(&mut vec);
                    let event = types::ServerEvent::from_bytes(&vec);
                    console_log(format!("Received event: {:?}", event).as_str());

                    let mut room = room_clone.write().unwrap(); //TODO: Handle error, maybe just return and close the connection
                    let player_index = room.player_index.get() as usize;
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

            let (tx, mut rx) = futures::channel::mpsc::unbounded::<Vec<u8>>();
            set_sender.set(Some(tx));

            spawn_local(async move {
                while let Some(msg) = rx.next().await {
                    if ws.send_with_u8_array(&msg).is_err() {
                        set_state.set(WebsocketState::Failed);
                    }
                }
            });
        }
    });

    let context = RoomContext {
        room,
        state,
        set_state,
        sender,
    };

    provide_context(context);

    view! {
        {move || {
            match state.get() {
                WebsocketState::Disconnected => {
                    view! { <div> {"Disconnected"} </div> }.into_any() // TODO: Add a reconnect button
                },
                WebsocketState::Connecting => {
                    view! { <div> {"Connecting"} </div> }.into_any() // TODO: Fix ui
                },
                WebsocketState::Failed => {
                    view! { <div> {"Failed to connect, something went really wrong..."} </div> }.into_any() // TODO: Fix ui
                },
                WebsocketState::Connected => {
                    view! {
                        <Show 
                            when=move || in_room.get() 
                            fallback=|| view! { <JoinRoom /> }
                        >
                            <Game />
                        </Show>
                    }.into_any()
                },
            }
        }}
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

