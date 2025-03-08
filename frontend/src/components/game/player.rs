//TODO: Generate this component with a proc macro
use leptos::prelude::*;

use crate::components::room::RoomContext;

#[component]
pub fn Player(player_index: usize) -> impl IntoView {
    view! {
        <div class="tycoon">
            "Player"
        </div>
    }
}
