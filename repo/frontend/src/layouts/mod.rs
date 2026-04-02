use dioxus::prelude::*;
use crate::auth::AUTH;
use crate::api;
use crate::components::LoadingSpinner;

#[component]
pub fn MainLayout() -> Element {
    let nav = use_navigator();

    // Reactive guard: reads AUTH inside the closure so Dioxus re-runs this
    // effect whenever AUTH changes (login, logout, token expiry).
    use_effect(move || {
        let auth = AUTH.read();
        if !auth.is_authenticated && !api::is_logged_in() {
            nav.replace(crate::Route::Login {});
        }
    });

    let auth = AUTH.read();
    let has_token = api::is_logged_in();

    // Token exists but AUTH not yet hydrated (get_me in flight after page reload).
    // Show spinner so the effect above doesn't fire a premature redirect.
    if has_token && !auth.is_authenticated {
        return rsx! {
            div { class: "app-container",
                main { class: "main-content",
                    LoadingSpinner { message: "Loading session..." }
                }
            }
        };
    }

    // No token, not authenticated — effect above will call nav.replace(Login).
    if !auth.is_authenticated {
        return rsx! {
            div { class: "app-container",
                main { class: "main-content",
                    LoadingSpinner {}
                }
            }
        };
    }

    rsx! {
        div { class: "app-container",
            nav { class: "sidebar",
                div { class: "sidebar-header",
                    h2 { class: "sidebar-title", "CampusLearn" }
                    p { class: "sidebar-subtitle", "Operations Suite" }
                }
                div { class: "sidebar-nav",
                    Link { to: crate::Route::Dashboard {}, class: "nav-item", "Dashboard" }

                    div { class: "nav-section-label", "ACADEMIC" }
                    Link { to: crate::Route::Courses {}, class: "nav-item", "Courses" }
                    if auth.role() == "admin" || auth.role() == "dept_reviewer" {
                        Link { to: crate::Route::Approvals {}, class: "nav-item", "Approvals" }
                    }

                    div { class: "nav-section-label", "RESOURCES" }
                    Link { to: crate::Route::Bookings {}, class: "nav-item", "Bookings" }

                    if auth.is_admin() {
                        div { class: "nav-section-label", "ADMIN" }
                        Link { to: crate::Route::Risk {}, class: "nav-item", "Risk & Compliance" }
                        Link { to: crate::Route::Audit {}, class: "nav-item", "Audit Trail" }
                    }

                    div { class: "nav-section-label", "ACCOUNT" }
                    Link { to: crate::Route::Privacy {}, class: "nav-item", "Privacy & Data" }
                }
                div { class: "sidebar-footer",
                    if let Some(user) = &auth.user {
                        div { class: "user-info",
                            span { class: "user-name", "{user.full_name}" }
                            span { class: "user-role", "{user.role}" }
                        }
                    }
                    button {
                        class: "btn btn-outline btn-sm btn-full",
                        onclick: move |_| crate::auth::logout(),
                        "Sign Out"
                    }
                }
            }
            main { class: "main-content with-sidebar",
                Outlet::<crate::Route> {}
            }
        }
    }
}
