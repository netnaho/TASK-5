use dioxus::prelude::*;
use crate::api;
use crate::types::NotificationItem;

pub static NOTIF_COUNT: GlobalSignal<i64> = Signal::global(|| 0);

#[component]
pub fn NotificationBell() -> Element {
    let mut open = use_signal(|| false);
    let mut notifications: Signal<Vec<NotificationItem>> = use_signal(Vec::new);

    // Fetch unread count on mount
    use_effect(move || {
        spawn(async move {
            if let Ok(resp) = api::get_unread_count().await {
                *NOTIF_COUNT.write() = resp.data.count;
            }
        });
    });

    let count = NOTIF_COUNT.read();

    let toggle = move |_| {
        let was_open = *open.read();
        open.set(!was_open);
        if !was_open {
            spawn(async move {
                if let Ok(resp) = api::get_notifications().await {
                    notifications.set(resp.data);
                }
            });
        }
    };

    let mark_all = move |_| {
        spawn(async move {
            let _ = api::mark_all_notifications_read().await;
            *NOTIF_COUNT.write() = 0;
            if let Ok(resp) = api::get_notifications().await {
                notifications.set(resp.data);
            }
        });
    };

    rsx! {
        div { class: "notif-bell-wrapper",
            button {
                class: "notif-bell-btn",
                onclick: toggle,
                title: "Notifications",
                span { class: "notif-bell-icon", "🔔" }
                if *count > 0 {
                    span { class: "notif-badge", "{count}" }
                }
            }

            if *open.read() {
                div { class: "notif-dropdown",
                    div { class: "notif-dropdown-header",
                        span { class: "notif-dropdown-title", "Notifications" }
                        button {
                            class: "btn btn-sm btn-outline",
                            onclick: mark_all,
                            "Mark all read"
                        }
                    }
                    div { class: "notif-dropdown-list",
                        if notifications.read().is_empty() {
                            div { class: "notif-empty", "No notifications" }
                        } else {
                            for notif in notifications.read().iter().take(10) {
                                {
                                    let uuid = notif.uuid.clone();
                                    let uuid_key = uuid.clone();
                                    let is_read = notif.is_read;
                                    rsx! {
                                        div {
                                            key: "{uuid_key}",
                                            class: if is_read { "notif-item notif-read" } else { "notif-item notif-unread" },
                                            onclick: move |_| {
                                                if !is_read {
                                                    let uuid2 = uuid.clone();
                                                    spawn(async move {
                                                        let _ = api::mark_notification_read(&uuid2).await;
                                                        if let Ok(resp) = api::get_unread_count().await {
                                                            *NOTIF_COUNT.write() = resp.data.count;
                                                        }
                                                        if let Ok(resp) = api::get_notifications().await {
                                                            notifications.set(resp.data);
                                                        }
                                                    });
                                                }
                                            },
                                            p { class: "notif-item-title", "{notif.title}" }
                                            p { class: "notif-item-msg", "{notif.message}" }
                                            span { class: "notif-item-time", "{notif.created_at}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "notif-dropdown-footer",
                        Link { to: crate::Route::Notifications {}, onclick: move |_| open.set(false),
                            "View all notifications →"
                        }
                    }
                }
            }
        }
    }
}
