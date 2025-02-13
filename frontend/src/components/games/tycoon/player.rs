use leptos::{leptos_dom::logging::console_log, prelude::*};
use shared::traits::GameSignal;

use crate::components::room::RoomContext;

#[component]
pub fn Player(player_index: usize) -> impl IntoView {
    let room_context = use_context::<RoomContext>().expect("RoomContext not found");
    let room = room_context.room;

    let player_signal = room.read().unwrap().players[player_index].clone();
    

    let player_is_some = move || room.clone().read().unwrap().players[player_index].get().is_some();
    //let player = move || room.clone().read().unwrap().players[player_index].get();

    view! {
        <Show when=move || player_signal.get().is_some() fallback=|| view! { <div></div> }>
        {
            console_log(format!("Rerendering player: {:?}", player_index).as_str());
        }
            <div class="player">

            </div>
        </Show>
    }
}

// <div class="player-name">{player().unwrap().name.signal.get()}</div>