use dioxus::prelude::*;

#[component]
pub fn Modal(title: String, on_close: EventHandler<()>, children: Element) -> Element {
    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div { class: "modal-content", onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h2 { class: "modal-title", "{title}" }
                    button { class: "modal-close-btn", onclick: move |_| on_close.call(()), "×" }
                }
                div { class: "modal-body", {children} }
            }
        }
    }
}
