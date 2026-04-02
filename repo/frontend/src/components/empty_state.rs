use dioxus::prelude::*;

#[component]
pub fn EmptyState(title: String, #[props(optional)] description: Option<String>) -> Element {
    rsx! {
        div { class: "empty-state",
            h3 { class: "empty-state-title", "{title}" }
            if let Some(desc) = description {
                p { class: "empty-state-desc", "{desc}" }
            }
        }
    }
}
