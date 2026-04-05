mod api;
mod auth;
mod components;
mod forms;
mod hooks;
mod layouts;
mod pages;
mod theme;
mod types;

use dioxus::prelude::*;
use pages::login::Login;
use pages::dashboard::Dashboard;
use pages::courses::Courses;
use pages::course_detail::CourseDetail;
use pages::course_editor::CourseEditor;
use pages::approvals::Approvals;
use pages::bookings::Bookings;
use pages::risk::Risk;
use pages::privacy::Privacy;
use pages::audit::Audit;
use pages::notifications::Notifications;

fn main() {
    tracing_wasm::set_as_global_default();
    launch(App);
}

#[component]
fn App() -> Element {
    // Hydrate auth state from localStorage token on app startup.
    // After a hard redirect (login → /dashboard), the page reloads and
    // the AUTH signal is empty. This fetches /auth/me to restore it.
    use_effect(move || {
        if api::is_logged_in() && !auth::AUTH.read().is_authenticated {
            spawn(async move {
                match api::get_me().await {
                    Ok(resp) => {
                        *auth::AUTH.write() = auth::AuthState::logged_in(resp.data);
                    }
                    Err(_) => {
                        // Token invalid or expired — clear it and reset signal
                        api::clear_token();
                        *auth::AUTH.write() = auth::AuthState::logged_out();
                    }
                }
            });
        }
    });

    rsx! {
        components::ToastContainer {}
        Router::<Route> {}
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(layouts::MainLayout)]
    #[route("/")]
    Home {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/courses")]
    Courses {},
    #[route("/courses/:uuid")]
    CourseDetail { uuid: String },
    #[route("/courses/:uuid/edit")]
    CourseEditor { uuid: String },
    #[route("/approvals")]
    Approvals {},
    #[route("/bookings")]
    Bookings {},
    #[route("/risk")]
    Risk {},
    #[route("/privacy")]
    Privacy {},
    #[route("/audit")]
    Audit {},
    #[route("/notifications")]
    Notifications {},
    #[end_layout]
    #[route("/login")]
    Login {},
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

#[component]
fn Home() -> Element {
    let nav = navigator();
    if api::is_logged_in() {
        nav.replace(Route::Dashboard {});
    } else {
        nav.replace(Route::Login {});
    }
    rsx! {
        div { class: "loading-container",
            p { "Redirecting..." }
        }
    }
}

#[component]
fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        div { class: "not-found-page",
            div { class: "not-found-content",
                h1 { "404" }
                p { "Page not found." }
                Link { to: Route::Dashboard {}, class: "btn btn-primary", "Go to Dashboard" }
            }
        }
    }
}
