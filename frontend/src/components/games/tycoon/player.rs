use leptos::{leptos_dom::logging::console_log, prelude::*};

use crate::components::room::RoomContext;

#[component]
pub fn Player(player_index: usize) -> impl IntoView {
    let room_context = use_context::<RoomContext>().expect("RoomContext not found");

    let player = move || room_context.room.read().unwrap().players[player_index].signal.get();

    view! {
        <Show when=move || player().is_some() fallback=|| view! { <div></div> }>
        {
            console_log(format!("Rerendering player: {:?}", player_index).as_str());
        }
            <div class="player">
                
            </div>
        </Show>
    }
}

// <div class="player-name">{player().unwrap().name.signal.get()}</div>