use leptos::{leptos_dom::logging::console_log, prelude::*};

use crate::components::room::RoomContext;

#[component]
pub fn Player(player_index: usize) -> impl IntoView {
    let room_context = use_context::<RoomContext>().expect("RoomContext not found");

    let player = Signal::derive(move || room_context.room.get().players[player_index]);

    view! {
        <Show when=move || player.get().is_some() fallback=|| view! { <div></div> }>
        {
            console_log(format!("Rerendering player: {:?}", player_index).as_str());
        }
            <div class="player">
                <div class="player-name">{player.get().unwrap().common.name}</div>
            </div>
        </Show>
    }
}