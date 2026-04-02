use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ToastLevel { Success, Error, Warning, Info }

#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub id: u32,
    pub level: ToastLevel,
    pub text: String,
}

pub static TOASTS: GlobalSignal<Vec<ToastMessage>> = Signal::global(Vec::new);
static TOAST_ID: GlobalSignal<u32> = Signal::global(|| 0);

pub struct ToastManager;

impl ToastManager {
    pub fn show(level: ToastLevel, text: impl Into<String>) {
        let mut counter = TOAST_ID.write();
        *counter += 1;
        let id = *counter;
        TOASTS.write().push(ToastMessage { id, level, text: text.into() });
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(4000).await;
            TOASTS.write().retain(|t| t.id != id);
        });
    }
    pub fn success(text: impl Into<String>) { Self::show(ToastLevel::Success, text); }
    pub fn error(text: impl Into<String>) { Self::show(ToastLevel::Error, text); }
}

#[component]
pub fn ToastContainer() -> Element {
    let toasts = TOASTS.read();
    rsx! {
        div { class: "toast-container",
            for toast in toasts.iter() {
                div {
                    key: "{toast.id}",
                    class: match toast.level {
                        ToastLevel::Success => "toast toast-success",
                        ToastLevel::Error => "toast toast-error",
                        ToastLevel::Warning => "toast toast-warning",
                        ToastLevel::Info => "toast toast-info",
                    },
                    span { class: "toast-text", "{toast.text}" }
                    button {
                        class: "toast-close",
                        onclick: { let id = toast.id; move |_| { TOASTS.write().retain(|t| t.id != id); } },
                        "×"
                    }
                }
            }
        }
    }
}
