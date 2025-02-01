use leptos::*; //TODO: Remove *
use leptos::prelude::*;
use web_sys::Event;

stylance::import_crate_style!(style, "src/ui/input.module.css");

#[component]
pub fn Input(
    #[prop(into)] value: RwSignal<String>,
    #[prop(optional)] placeholder: String,
    #[prop(optional)] style: String,
    #[prop(optional)] max_length: Option<u8>,
) -> impl IntoView {
    view! {
        <input
            type="text"
            placeholder=placeholder
            value=value.get()
            class=style::input
            maxlength=max_length
            style=style
            on:input=move |ev: Event| {
                let input_element = event_target_value(&ev);
                value.set(input_element);
            }
        />
    }
}
