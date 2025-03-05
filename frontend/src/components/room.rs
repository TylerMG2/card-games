use futures::channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use gloo::storage::{LocalStorage, Storage};
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared::{
    logic,
    traits::{GameSignal, ToFromBytes},
    types::{self, MAX_NAME_LENGTH},
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    ErrorEvent, MessageEvent, WebSocket, js_sys,
    wasm_bindgen::{JsCast, prelude::Closure},
};

use crate::components::{game::Game, join_room::JoinRoom};

#[derive(Clone, PartialEq)]
pub enum WebsocketState {
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Clone)]
pub struct RoomContext {
    pub room: RwSignal<types::Room>,
    set_ws_state: WriteSignal<WebsocketState>,
    sender: UnboundedSender<Vec<u8>>,
    connection: types::ClientConnection,
}

impl RoomContext {
    fn validate_client_event(&self, event: &types::ClientEvent) -> bool {
        self.room.with(|room| {
            logic::validate_client_event(room, event, *room.player_index.value() as usize)
        })
    }

    fn handle_client_event(&mut self, event: &types::ClientEvent) {
        self.room.update(|room| {
            logic::handle_client_event(
                room,
                event,
                &mut self.connection,
                *room.player_index.value() as usize,
            )
        });
    }

    pub fn send_event(&mut self, event: types::ClientEvent) {
        console_log(format!("Sending event: {:?}", event).as_str());

        if self.validate_client_event(&event) {
            self.handle_client_event(&event);

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
    let (tx_signal, set_tx_signal) = signal(None::<UnboundedSender<Vec<u8>>>);
    let room = RwSignal::new(types::Room::default());

    let params = use_params_map();
    let code = move || params.read().get("code");

    let id = get_player_id();

    // TODO: I think we need to clean up the websocket connection when the component is unmounted
    let join_room = move |name: Option<[u8; MAX_NAME_LENGTH]>| {
        let Some(code) = code() else {
            return;
        }; // TODO: Redirect to home page, this should never happen
        if ws_state.get_untracked() != WebsocketState::Disconnected {
            return;
        } // Return if we are already connected

        set_ws_state.set(WebsocketState::Connecting);

        // Start the websocket connection
        let ws = match WebSocket::new(build_ws_url(&code, &id, &name).as_str()) {
            Ok(ws) => ws,
            Err(_) => {
                console_log("Failed to connect");
                set_ws_state.set(WebsocketState::Disconnected); // Todo maybe get the error type for better logging
                return;
            }
        };
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let (tx, mut rx) = futures::channel::mpsc::unbounded::<Vec<u8>>();
        set_tx_signal.set(Some(tx));

        // On error
        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |_e: ErrorEvent| {
            set_ws_state.set(WebsocketState::Disconnected);
            set_in_room.set(false);
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // On open
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            console_log("Connected");
            set_ws_state.set(WebsocketState::Connected);
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
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
                let event = types::ServerEvent::from_bytes(&bytes);
                console_log(format!("Received event: {:?}", event).as_str());

                // Untracked because we don't want to rerender everything, only to changes for signals within the room
                // Only potential issue is if *room = new_room, but that should only happen on room joined event, which should
                // trigger rerendering anyways
                room.update_untracked(|room| {
                    let player_index = *room.player_index.value() as usize;
                    logic::handle_server_event(room, &event, Some(player_index), false);
                });

                // For tracking if we are in the room
                if matches!(
                    event,
                    types::ServerEvent::CommonEvent(types::CommonServerEvent::RoomJoined { .. })
                ) {
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

    // Attempt initial connection on mount
    Effect::new(move || {
        // So its reactive to code changes
        if code().is_some() && ws_state.get_untracked() == WebsocketState::Disconnected {
            let name = None;
            join_room(name);
        }
    });

    view! {
        <Show
            when=move || !in_room.get() || tx_signal.read().is_none()
            fallback=move || {
                provide_context(RoomContext {
                    room,
                    set_ws_state,
                    sender: tx_signal.get().expect("Sender should be set"),
                    connection: types::ClientConnection,
                });
                view! { <Game /> }
            }
        >
            <Show
                when=move || ws_state.read() == WebsocketState::Disconnected
                fallback=|| view! { <div class="loading-room">"Joining Room"</div> }
            >
                <JoinRoom join_room=join_room />
            </Show>
        </Show>
    }
}

// Check if their is a player id in local storage, if not create one and store it
pub fn get_player_id() -> uuid::Uuid {
    LocalStorage::get::<String>("player_id")
        .ok()
        .and_then(|id| uuid::Uuid::parse_str(&id).ok())
        .unwrap_or_else(|| {
            let id = uuid::Uuid::new_v4();
            let _ = LocalStorage::set("player_id", id.to_string()); // TODO: Handle error if necessary
            id
        })
}

fn build_ws_url(code: &str, id: &uuid::Uuid, name: &Option<[u8; MAX_NAME_LENGTH]>) -> String {
    if let Some(name) = name {
        format!(
            "ws://localhost:3000/ws?code={}&id={}&name={}",
            code,
            id,
            String::from_utf8_lossy(name)
        )
    } else {
        format!("ws://localhost:3000/ws?code={}&id={}", code, id)
    }
}
