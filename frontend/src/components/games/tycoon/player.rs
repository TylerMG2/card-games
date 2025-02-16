use leptos::{leptos_dom::logging::console_log, prelude::*, tachys::view};

use crate::components::room::RoomContext;

#[component]
pub fn Player(player_index: usize) -> impl IntoView {
    let room_context = use_context::<RoomContext>().expect("RoomContext not found");
    let room = room_context.room;

    let player = move || room.read().unwrap().players[player_index].get();

    view! {
        {
            move || {
                let player = player();
                if let Some(player) = player {
                    view! {
                        <div class="player">
                            <div class="player-name">{move || {
                                String::from_utf8_lossy(&player.name.get()).trim_end_matches('\0').to_string()
                            }}</div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div></div>
                    }.into_any()
                }
            }
        }
    }
}

// <div class="player-name">{player().unwrap().name.signal.get()}</div>