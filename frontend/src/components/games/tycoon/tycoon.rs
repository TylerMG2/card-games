use leptos::prelude::*;

use crate::components::{games::tycoon::player::Player, room::RoomContext};


#[derive(Clone, PartialEq)]
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
            Seats::None => "top: 50%; left: 90%;",
        }
    }
}

#[component]
pub fn Tycoon() -> impl IntoView {
    view! {
        <div class="tycoon">
            { (0..8).map(|i| view! { <Player player_index=i /> }).collect::<Vec<_>>() }
        </div>
    }
}