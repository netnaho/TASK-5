use dioxus::prelude::*;
use crate::api;
use crate::components::{EmptyState, LoadingSpinner, NOTIF_COUNT};
use crate::types::NotificationItem;

#[component]
pub fn Notifications() -> Element {
    let mut items: Signal<Vec<NotificationItem>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            match api::get_notifications().await {
                Ok(resp) => {
                    items.set(resp.data);
                }
                Err(_) => {}
            }
            loading.set(false);
        });
    });

    let mark_all = move |_| {
        spawn(async move {
            let _ = api::mark_all_notifications_read().await;
            *NOTIF_COUNT.write() = 0;
            if let Ok(resp) = api::get_notifications().await {
                items.set(resp.data);
            }
        });
    };

    rsx! {
        div { class: "page-container",
            div { class: "page-header",
                h1 { class: "page-title", "Notifications" }
                if !items.read().is_empty() {
                    button {
                        class: "btn btn-outline",
                        onclick: mark_all,
                        "Mark all read"
                    }
                }
            }

            if *loading.read() {
                LoadingSpinner { message: "Loading notifications..." }
            } else if items.read().is_empty() {
                EmptyState {
                    title: "No notifications",
                    description: "You have no notifications yet.",
                }
            } else {
                div { class: "notif-list",
                    for notif in items.read().iter() {
                        {
                            let uuid = notif.uuid.clone();
                            let uuid_key = uuid.clone();
                            let is_read = notif.is_read;
                            let title = notif.title.clone();
                            let message = notif.message.clone();
                            let ntype = notif.notification_type.clone();
                            let created_at = notif.created_at.clone();
                            rsx! {
                                div {
                                    key: "{uuid_key}",
                                    class: if is_read { "notif-row notif-row-read" } else { "notif-row notif-row-unread" },
                                    onclick: move |_| {
                                        if !is_read {
                                            let uuid2 = uuid.clone();
                                            spawn(async move {
                                                let _ = api::mark_notification_read(&uuid2).await;
                                                if let Ok(resp) = api::get_unread_count().await {
                                                    *NOTIF_COUNT.write() = resp.data.count;
                                                }
                                                if let Ok(resp) = api::get_notifications().await {
                                                    items.set(resp.data);
                                                }
                                            });
                                        }
                                    },
                                    div { class: "notif-row-content",
                                        div { class: "notif-row-header",
                                            span { class: "notif-row-title", "{title}" }
                                            span { class: format!("badge badge-{}", ntype), "{ntype}" }
                                        }
                                        p { class: "notif-row-message", "{message}" }
                                        span { class: "notif-row-time", "{created_at}" }
                                    }
                                    if !is_read {
                                        span { class: "notif-unread-dot", title: "Unread" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
