use leptos::prelude::*;
use shared::types::MAX_NAME_LENGTH;
use web_sys::SubmitEvent;

#[component]
pub fn JoinRoom(mut join_room: impl FnMut(Option<[u8; MAX_NAME_LENGTH]>) + 'static) -> impl IntoView {
    let name = RwSignal::new("".to_string());

    let join_game = move |ev: SubmitEvent| {
        ev.prevent_default();
        let name_string = name.get().trim().to_string();
        let name_slice = name_string.as_bytes();

        let mut name_bytes = [0u8; 20];
        let len = name_slice.len().min(20);
        name_bytes[..len].copy_from_slice(&name_slice[..len]);

        join_room(Some(name_bytes));
    };

    //TODO: Does input value need to be set?
    view! {
        <div style="display: flex; flex-direction: column; align-items: center; height: 100%; justify-content: center;">
            <div class="d-flex flex-column panel panel-1">
                <form class="d-flex flex-column gap-16" on:submit=join_game>
                    <div style="font-size: 64px;"> {"JOIN ROOM"} </div>
                    <input 
                        type="text"
                        value={move || name.get()}
                        on:input=move |ev| name.set(event_target_value(&ev))
                        placeholder="Enter name"
                        maxlength="20"
                        class="input-class"
                        required
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