use leptos::prelude::*;
use shared::types::{self, MAX_PLAYERS};

use crate::components::{games::tycoon::player::Player, room::RoomContext};


pub enum Seats {
    CurrentPlayer,
    TopLeft,
    TopMiddle,
    TopRight,
    MiddleLeft,
    MiddleRight,
    MiddleTopRight,
    MiddleTopLeft,
    BottomLeft,
    BottomRight,
    None,
}

impl Seats {
    pub fn as_style(&self) -> &'static str {
        match self {
            Seats::CurrentPlayer => "top: 100%; left: 50%;",
            Seats::TopLeft => "top: 15%; left: 27%;",
            Seats::TopMiddle => "top: 10%; left: 50%;",
            Seats::TopRight => "top: 15%; left: 73%;",
            Seats::MiddleTopRight => "top: 35%; left: 90%;",
            Seats::MiddleTopLeft => "top: 35%; left: 10%;",
            Seats::MiddleLeft => "top: 50%; left: 10%;",
            Seats::MiddleRight => "top: 50%; left: 90%;",
            Seats::BottomLeft => "top: 70%; left: 12%;",
            Seats::BottomRight => "top: 70%; left: 88%;",
            Seats::None => "",
        }
    }
}

#[component]
pub fn Tycoon() -> impl IntoView {
    let room_context = use_context::<RoomContext>().expect("RoomContext not found");
    
    // Get all current players in the room
    

    // Render all the players
    view! {
        <div class="tycoon">
            <Player player_index=0 />
            <Player player_index=1 />
            <Player player_index=2 />
            <Player player_index=3 />
            <Player player_index=4 />
            <Player player_index=5 />
            <Player player_index=6 />
            <Player player_index=7 />
        </div>
    }
}

// Return the seat of an other player based on the current player
// the current player will always be at the bottom
fn get_seat_position(current_player: u8, player_index: u8, num_players: u8) -> &'static str {
    if num_players as usize > MAX_PLAYERS || num_players == 0 {
        return Seats::None.as_style();
    }

    let relative_position = (player_index as i8 - current_player as i8).rem_euclid(num_players as i8) as u8;
    &NEW_SEATS[num_players as usize - 1][relative_position as usize].as_style()
}

const NEW_SEATS: [[Seats; MAX_PLAYERS]; MAX_PLAYERS] = [
    [Seats::CurrentPlayer, Seats::None, Seats::None, Seats::None, Seats::None, Seats::None, Seats::None, Seats::None],
    [Seats::CurrentPlayer, Seats::TopMiddle, Seats::None, Seats::None, Seats::None, Seats::None, Seats::None, Seats::None],
    [Seats::CurrentPlayer, Seats::MiddleLeft, Seats::MiddleRight, Seats::None, Seats::None, Seats::None, Seats::None, Seats::None],
    [Seats::CurrentPlayer, Seats::MiddleLeft, Seats::TopMiddle, Seats::MiddleRight, Seats::None, Seats::None, Seats::None, Seats::None],
    [Seats::CurrentPlayer, Seats::MiddleLeft, Seats::TopLeft, Seats::TopRight, Seats::MiddleRight, Seats::None, Seats::None, Seats::None],
    [Seats::CurrentPlayer, Seats::MiddleLeft, Seats::TopLeft, Seats::TopMiddle, Seats::TopRight, Seats::MiddleRight, Seats::None, Seats::None],
    [Seats::CurrentPlayer, Seats::BottomLeft, Seats::MiddleTopLeft, Seats::TopLeft, Seats::TopRight, Seats::MiddleTopRight, Seats::BottomRight, Seats::None],
    [Seats::CurrentPlayer, Seats::BottomLeft, Seats::MiddleTopLeft, Seats::TopLeft, Seats::TopMiddle, Seats::TopRight, Seats::MiddleTopRight, Seats::BottomRight],
];