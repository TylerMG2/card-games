use leptos::prelude::*;
use shared::types;
use web_sys::SubmitEvent;


use super::room::RoomContext;

#[component]
pub fn JoinRoom() -> impl IntoView {
    let room_context = use_context::<RoomContext>().expect("RoomContext not found");
    let name = RwSignal::new("".to_string());

    let join_game = move |ev: SubmitEvent| {
        ev.prevent_default();
        let name_string = name.get();
        let name_slice = name_string.as_bytes();

        let mut name_bytes = [0u8; 20];
        let len = name_slice.len().min(20);
        name_bytes[..len].copy_from_slice(&name_slice[..len]);

        let event = types::ClientEvent::CommonEvent(types::CommonClientEvent::JoinRoom {
            name: name_bytes
        });
        room_context.send_event(event);
    };

    view! {
        <div style="display: flex; flex-direction: column; align-items: center; height: 100%; justify-content: center;">
            <div class="d-flex flex-column panel panel-1">
                <form class="d-flex flex-column gap-16" on:submit=join_game>
                    <div style="font-size: 64px;"> {"JOIN ROOM"} </div>
                    <input 
                        type="text"
                        value=name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                        placeholder="Enter name"
                        maxlength="20"
                        class="input-class"
                    />
                    <button 
                        class="btn-green"
                        type="submit"
                        style="width: 100%;"
                    > {"Join Room"} </button>
                    <p>{ "Created by Tyler" }</p>
                </form>
            </div>
        </div>
    }
}