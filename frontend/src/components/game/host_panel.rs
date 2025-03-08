use leptos::prelude::*;

stylance::import_crate_style!(style, "src/components/game/host_panel.module.css");

use crate::components::room::RoomContext;

#[component]
pub fn HostPanel() -> impl IntoView {
    let mut room_context = use_context::<RoomContext>().expect("RoomContext not found");

    let is_host = move || {
        room_context
            .room
            .with(|room| room.host.get() == room.player_index.get())
    };

    let (can_start, reason) = move || room_context.validate_client_event(ClientEve::StartGame);

    view! {
        <div class={"panel ".to_owned() + style::host_panel}>
            <p style="font-size: 48px;"> {"GAME SETUP"} </p>
            <button
                class="btn-green"
                type="submit"
                style="max-width: 100%;"
                disabled=move || !is_host()
            > {"Start Game"} </button>
        </div>
    }
}
