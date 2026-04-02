use dioxus::prelude::*;

#[component]
pub fn LoadingSpinner(#[props(optional)] message: Option<String>) -> Element {
    let msg = message.unwrap_or_else(|| "Loading...".to_string());
    rsx! {
        div { class: "loading-container",
            div { class: "spinner" }
            p { class: "loading-text", "{msg}" }
        }
    }
}
