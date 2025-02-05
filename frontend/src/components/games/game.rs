use leptos::prelude::*;

use crate::components::room::RoomContext;

#[component]
pub fn Game() -> impl IntoView {
    let room_context = use_context::<RoomContext>().expect("RoomContext not found");

    // TODO: Render different game views, maybe also we could have common details displayed here
    view! {
        <div style="display: flex; flex-direction: column; align-items: center; height: 100%; justify-content: center;">
            {"Test"}
        </div>
    }
}