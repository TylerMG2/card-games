mod tycoon;

use crate::components::room::RoomContext;
use leptos::prelude::*;
use shared::types;
use tycoon::Tycoon;

#[component]
pub fn Game() -> impl IntoView {
    let mut room_context = use_context::<RoomContext>().expect("RoomContext not found");

    let gamemode = move || room_context.room.read().game.get();

    // TODO: Check if it only rerenders when the game type changes as opposed to the room changing in any way
    view! {
        <div>
            {move || match gamemode() {
                types::GameType::Carbo => view! {
                    <div>
                        <h2>{"Carbo"}</h2>
                        <p>{"Carbo is a game where you have to guess the number of carbs in a food item"}</p>
                    </div>
                }.into_any(),
                types::GameType::Tycoon => view! { <Tycoon /> }.into_any(),
                types::GameType::Coup => view! {
                    <div>
                        <h2>{"Coup"}</h2>
                        <p>{"Coup is a game where you have to bluff your way to victory"}</p>
                    </div>
                }.into_any(),
            }}

            <button
                class="btn-red"
                type="submit"
                style="width: 100%;"
                on:click=move |_| room_context.send_event(types::ClientEvent::CommonEvent(types::CommonClientEvent::LeaveRoom))
            > {"Leave Room"} </button>
        </div>
    }
}
