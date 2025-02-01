use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use futures_util::StreamExt;
use leptos::ev::MouseEvent;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared::{logic, types, traits::ToFromBytes};
use wasm_bindgen_futures::spawn_local;
use web_sys::{js_sys, wasm_bindgen::{prelude::Closure, JsCast}, ErrorEvent, MessageEvent, WebSocket};
use gloo::storage::{LocalStorage, Storage, errors::StorageError};

use crate::ui::{button::{Button, ButtonType}, input::Input, panel::Panel};

#[derive(Clone, PartialEq)]
pub enum WebsocketState {
    Connecting,
    Connected,
    Disconnected,
    Failed,
}


#[derive(Clone, Copy)]
pub struct RoomContext {
    pub room: ReadSignal<types::Room>,
    set_room: WriteSignal<types::Room>,
    pub state: ReadSignal<WebsocketState>,
    set_state: WriteSignal<WebsocketState>,
    sender: ReadSignal<Option<UnboundedSender<Vec<u8>>>>,
}

impl RoomContext {
    pub fn validate_client_event(&self, event: &types::ClientEvent) -> bool {
        let room = self.room.get();
        logic::validate_client_event(&room, event, room.common.player_index as usize)
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
    let name = RwSignal::new("".to_string());
    let (state, set_state) = signal(WebsocketState::Disconnected);
    let (room, set_room) = signal(types::Room::default());
    let (sender, set_sender) = signal(None);
    
    let params = use_params_map();
    set_code.set(params.read().get("code"));

    // TODO: I think we need to clean up the websocket connection when the component is unmounted
    Effect::new(move |_| {
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
            let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
                console_log(format!("Error: {:?}", e).as_str());
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
                set_state.set(WebsocketState::Disconnected);
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

                    set_room.update(|room| {
                        logic::handle_server_event(room, &event, Some(room.common.player_index as usize), false);
                    });

                    // Log current room state
                    //console_log(format!("Room: {:?}", room.get()).as_str());

                    // Room joined event
                    if let types::ServerEvent::CommonEvent(types::CommonServerEvent::RoomJoined { new_room: _, current_player: _ }) = event {
                        // TODO: Update a signal to indicate that the player has joined the room
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
        set_room,
        state,
        set_state,
        sender,
    };

    let join_game = {
        move |_: MouseEvent| {
            let name_string = name.get();
            let name_slice = name_string.as_bytes();

            let mut name_bytes = [0u8; 20];
            let len = name_slice.len().min(20);
            name_bytes[..len].copy_from_slice(&name_slice[..len]);

            let event = types::ClientEvent::CommonEvent(types::CommonClientEvent::JoinRoom {
                name: name_bytes
            });
            context.send_event(event);
        }
    };

    provide_context(context);

    // Return a view rendering a form if the code is not set, otherwise render a form with name input is not connected, otherwise render the room
    view! {
        //TODO: We need a check to see if the player is in the room (post join_game) and render the room if they are
        //this render should choose different components based on the game type: room.common.game
        {move || {
            match state.get() {
                WebsocketState::Disconnected => {
                    view! { <div> {"Disconnected"} </div> }.into_any()
                },
                WebsocketState::Connecting => {
                    view! { <div> {"Connecting"} </div> }.into_any()
                },
                WebsocketState::Failed => {
                    view! { <div> {"Failed to connect, something went really wrong..."} </div> }.into_any()
                },
                WebsocketState::Connected => {
                    view! {
                        <div style="display: flex; flex-direction: column; align-items: center; height: 100%; justify-content: center;">
                            <Panel level=1>
                                <div class="d-flex flex-column gap-16">
                                    <div style="font-size: 48px;"> {"Join Room"} </div>
                                    <Input value={name.clone()} placeholder={"Enter name".to_string()} max_length=20/>
                                    <Button on_click={join_game.clone()} button_type={ButtonType::Green} style="width: 100%".to_owned()> {"Join Room"} </Button>
                                </div>
                            </Panel>
                        </div>
                    }.into_any()
                },
            }
        }}
    }
}

// Function to check if their is a player id in local storage, if not create one and store it
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

