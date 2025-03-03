use leptos::{leptos_dom::logging::console_log, prelude::*};
use shared::types::MAX_PLAYERS;

use crate::components::{
    games::tycoon::{player, tycoon::Seats},
    room::RoomContext,
};

#[component]
pub fn Player(player_index: usize) -> impl IntoView {
    let room_context = use_context::<RoomContext>().expect("RoomContext not found");
    let room = room_context.room;

    let player = move || room.get().players[player_index].get();

    let room_context = use_context::<RoomContext>().expect("RoomContext not found");

    // Get all current players in the room
    let seat = move |player_index: usize| {
        room_context.room.with(|room| {
            let current_player = room.player_index.get();
            let mut current_player_local = 0;
            let mut local_player_index = 0;

            let mut num_players = 0;
            for (index, player) in room.players.iter().enumerate() {
                if player.get().is_some() {
                    if index == current_player as usize {
                        current_player_local = num_players;
                    }

                    if index == player_index {
                        local_player_index = num_players;
                    }

                    num_players += 1;
                }
            }

            get_seat_position(current_player_local, local_player_index, num_players)
        })
    };

    view! {
        <div class="player" style={move || seat(player_index).as_style().to_owned()}>
            {move || player().map(|player| {
                    view! {
                        <div class="player-name">
                            {String::from_utf8_lossy(&player.name.get()).trim_end_matches('\0').to_string()}
                        </div>
                    }.into_any()
            })}
        </div>
    }
}

// Return the seat of an other player based on the current player
// the current player will always be at the bottom
fn get_seat_position(current_player: i8, player_index: i8, num_players: i8) -> Seats {
    console_log(
        format!(
            "current_player: {}, player_index: {}, num_players: {}",
            current_player, player_index, num_players
        )
        .as_str(),
    );

    if num_players as usize > MAX_PLAYERS || num_players == 0 {
        return Seats::None;
    }

    let relative_position = (player_index - current_player).rem_euclid(num_players) as u8;
    NEW_SEATS[num_players as usize - 1][relative_position as usize].clone()
}

const NEW_SEATS: [[Seats; MAX_PLAYERS]; MAX_PLAYERS] = [
    [
        Seats::CurrentPlayer,
        Seats::None,
        Seats::None,
        Seats::None,
        Seats::None,
        Seats::None,
        Seats::None,
        Seats::None,
    ],
    [
        Seats::CurrentPlayer,
        Seats::TopMiddle,
        Seats::None,
        Seats::None,
        Seats::None,
        Seats::None,
        Seats::None,
        Seats::None,
    ],
    [
        Seats::CurrentPlayer,
        Seats::MiddleLeft,
        Seats::MiddleRight,
        Seats::None,
        Seats::None,
        Seats::None,
        Seats::None,
        Seats::None,
    ],
    [
        Seats::CurrentPlayer,
        Seats::MiddleLeft,
        Seats::TopMiddle,
        Seats::MiddleRight,
        Seats::None,
        Seats::None,
        Seats::None,
        Seats::None,
    ],
    [
        Seats::CurrentPlayer,
        Seats::MiddleLeft,
        Seats::TopLeft,
        Seats::TopRight,
        Seats::MiddleRight,
        Seats::None,
        Seats::None,
        Seats::None,
    ],
    [
        Seats::CurrentPlayer,
        Seats::MiddleLeft,
        Seats::TopLeft,
        Seats::TopMiddle,
        Seats::TopRight,
        Seats::MiddleRight,
        Seats::None,
        Seats::None,
    ],
    [
        Seats::CurrentPlayer,
        Seats::BottomLeft,
        Seats::MiddleTopLeft,
        Seats::TopLeft,
        Seats::TopRight,
        Seats::MiddleTopRight,
        Seats::BottomRight,
        Seats::None,
    ],
    [
        Seats::CurrentPlayer,
        Seats::BottomLeft,
        Seats::MiddleTopLeft,
        Seats::TopLeft,
        Seats::TopMiddle,
        Seats::TopRight,
        Seats::MiddleTopRight,
        Seats::BottomRight,
    ],
];
