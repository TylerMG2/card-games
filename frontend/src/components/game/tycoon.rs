use leptos::prelude::*;
use shared::types::MAX_PLAYERS;

use crate::components::room::RoomContext;
use player::Player;

mod player;

#[component]
pub fn Tycoon() -> impl IntoView {
    view! {
        <div class="tycoon">
            { (0..MAX_PLAYERS).map(|i| view! { <Player player_index=i /> }).collect::<Vec<_>>() }
        </div>
    }
}
