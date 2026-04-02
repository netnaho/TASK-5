use dioxus::prelude::*;
use crate::api;
use crate::auth::{AuthState, AUTH};
use crate::components::ToastManager;

#[component]
pub fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let mut api_status = use_signal(|| "Checking...".to_string());
    let nav = use_navigator();

    use_effect(move || {
        spawn(async move {
            match api::check_health().await {
                Ok(h) => api_status.set(format!("Connected ({})", h.status)),
                Err(e) => api_status.set(format!("Offline: {}", e)),
            }
        });
    });

    let handle_login = move |_evt: Event<FormData>| {
        spawn(async move {
            loading.set(true);
            error.set(None);

            match api::login(&username.read(), &password.read()).await {
                Ok(resp) => {
                    api::set_token(&resp.data.token);
                    *AUTH.write() = AuthState::logged_in(resp.data.user);
                    nav.push(crate::Route::Dashboard {});
                }
                Err(e) => {
                    error.set(Some(format!("Login failed: {}", e)));
                    loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "login-page",
            div { class: "login-card",
                div { class: "login-header",
                    h1 { class: "login-title", "CampusLearn" }
                    p { class: "login-subtitle", "Operations Suite" }
                }
                form { class: "login-form", onsubmit: handle_login, prevent_default: "onsubmit",
                    div { class: "form-group",
                        label { r#for: "username", "Username" }
                        input {
                            r#type: "text", id: "username", class: "form-input",
                            placeholder: "Enter your username", required: true,
                            value: "{username}", oninput: move |e| username.set(e.value()),
                        }
                    }
                    div { class: "form-group",
                        label { r#for: "password", "Password" }
                        input {
                            r#type: "password", id: "password", class: "form-input",
                            placeholder: "Enter your password", required: true,
                            value: "{password}", oninput: move |e| password.set(e.value()),
                        }
                    }
                    if let Some(err) = error.read().as_ref() {
                        div { class: "form-error", "{err}" }
                    }
                    button {
                        r#type: "submit", class: "btn btn-primary btn-full",
                        disabled: *loading.read(),
                        if *loading.read() { "Signing in..." } else { "Sign In" }
                    }
                }
                div { class: "login-footer",
                    span { class: "api-status", "API: {api_status}" }
                }
            }
        }
    }
}
