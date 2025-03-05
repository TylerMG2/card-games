use components::{home::Home, room::Room};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

pub mod components {
    pub mod game;
    pub mod home;
    pub mod join_room;
    pub mod room;
}

pub mod ui {
    pub mod input;
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <div>{"404"}</div> }>
                <Route path=path!("/") view=Home />
                <Route path=path!("/room/:code") view=Room />
            </Routes>
        </Router>
    }
}
