use leptos::{ev::MouseEvent, *};
use leptos::prelude::*;

stylance::import_crate_style!(style, "src/ui/button.module.css");

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonType {
    #[default]
    Blue,
    Green,
    Red,
}

impl ButtonType {
    fn to_class(&self) -> &'static str {
        match self {
            ButtonType::Blue => style::blue,
            ButtonType::Green => style::green,
            ButtonType::Red => style::red,
        }
    }
}

#[component]
pub fn Button(
    #[prop(optional)] button_type: ButtonType,
    #[prop(optional)] style: String,
    on_click: impl FnMut(MouseEvent) + 'static,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class={format!("{} {}", style::button, button_type.to_class())} 
            on:click=on_click
            style={style}
        >
            {children()}
        </button>
    }
}
