use leptos::prelude::*;

stylance::import_crate_style!(style, "src/ui/panel.module.css");

#[component]
pub fn Panel(
    level: u8,
    children: Children,
) -> impl IntoView {

    // TODO: Add level classes
    let _level = if level > 4 { 4 } else { level };

    view! {
        <div
            class=style::panel
        >
            {children()}
        </div>
    }
}
