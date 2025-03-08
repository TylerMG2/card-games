//TODO: Generate this component with a proc macro
use leptos::prelude::*;

use crate::components::room::RoomContext;

#[component]
pub fn OtherPlayer(player_index: usize) -> impl IntoView {
    view! {
        <div class="tycoon">
            "Other Player"
        </div>
    }
}
