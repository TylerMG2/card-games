use leptos::prelude::*;

use crate::components::room::RoomContext;
use player::Player;

mod player;

#[component]
pub fn Tycoon() -> impl IntoView {
    view! {
        <div class="tycoon">
            { (0..8).map(|i| view! { <Player player_index=i /> }).collect::<Vec<_>>() }
        </div>
    }
}
