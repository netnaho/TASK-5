use dioxus::prelude::*;
use crate::api;
use crate::components::*;
use crate::types::*;

#[component]
pub fn Bookings() -> Element {
    let mut resources = use_signal(|| Vec::<ResourceResponse>::new());
    let mut bookings = use_signal(|| Vec::<BookingResponse>::new());
    let mut breaches = use_signal(|| Vec::<BreachResponse>::new());
    let mut restrictions = use_signal(|| Vec::<RestrictionResponse>::new());
    let mut loading = use_signal(|| true);
    let mut active_tab = use_signal(|| "bookings");
    let mut show_book = use_signal(|| false);
    let mut sel_resource = use_signal(String::new);
    let mut book_title = use_signal(String::new);
    let mut book_start = use_signal(String::new);
    let mut book_end = use_signal(String::new);
    let mut show_reschedule = use_signal(|| None::<String>);
    let mut new_start = use_signal(String::new);
    let mut new_end = use_signal(String::new);
    let mut reschedule_reason = use_signal(String::new);

    let reload = move || {
        spawn(async move {
            loading.set(true);
            if let Ok(r) = api::list_resources().await { resources.set(r.data); }
            if let Ok(r) = api::my_bookings().await { bookings.set(r.data); }
            if let Ok(r) = api::my_breaches().await { breaches.set(r.data); }
            if let Ok(r) = api::my_restrictions().await { restrictions.set(r.data); }
            loading.set(false);
        });
    };

    use_effect(move || { reload(); });

    let handle_book = move |evt: Event<FormData>| {
        evt.prevent_default();
        let r_uuid = sel_resource.read().clone();
        let t = book_title.read().clone();
        let s = book_start.read().clone();
        let e = book_end.read().clone();
        spawn(async move {
            match api::create_booking(&r_uuid, &t, &s, &e).await {
                Ok(_) => {
                    ToastManager::success("Booking created!");
                    show_book.set(false);
                    book_title.set(String::new());
                    book_start.set(String::new());
                    book_end.set(String::new());
                    if let Ok(r) = api::my_bookings().await { bookings.set(r.data); }
                }
                Err(e) => ToastManager::error(format!("{}", e)),
            }
        });
    };

    let handle_cancel = move |uuid: String| {
        spawn(async move {
            match api::cancel_booking(&uuid).await {
                Ok(_) => {
                    ToastManager::success("Booking cancelled");
                    if let Ok(r) = api::my_bookings().await { bookings.set(r.data); }
                    if let Ok(r) = api::my_breaches().await { breaches.set(r.data); }
                }
                Err(e) => ToastManager::error(format!("{}", e)),
            }
        });
    };

    let handle_reschedule = move |_: Event<MouseData>| {
        let uuid = show_reschedule.read().clone();
        let ns = new_start.read().clone();
        let ne = new_end.read().clone();
        let reason = reschedule_reason.read().clone();
        spawn(async move {
            if let Some(u) = uuid {
                let r = if reason.is_empty() { None } else { Some(reason.as_str()) };
                match api::reschedule_booking(&u, &ns, &ne, r).await {
                    Ok(_) => {
                        ToastManager::success("Rescheduled");
                        show_reschedule.set(None);
                        if let Ok(r) = api::my_bookings().await { bookings.set(r.data); }
                    }
                    Err(e) => ToastManager::error(format!("{}", e)),
                }
            }
        });
    };

    if *loading.read() {
        return rsx! { LoadingSpinner {} };
    }

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div { class: "page-header-row",
                    h1 { "Bookings" }
                    button { class: "btn btn-primary", onclick: move |_| show_book.set(true), "+ New Booking" }
                }
                if !restrictions.read().is_empty() {
                    div { class: "alert alert-danger",
                        "You have active booking restrictions. Some actions may be unavailable."
                    }
                }
            }

            div { class: "tabs",
                button { class: if *active_tab.read() == "bookings" { "tab active" } else { "tab" }, onclick: move |_| active_tab.set("bookings"), "My Bookings" }
                button { class: if *active_tab.read() == "resources" { "tab active" } else { "tab" }, onclick: move |_| active_tab.set("resources"), "Resources" }
                button { class: if *active_tab.read() == "breaches" { "tab active" } else { "tab" }, onclick: move |_| active_tab.set("breaches"),
                    "Breaches"
                    if !breaches.read().is_empty() {
                        span { class: "tab-badge", "{breaches.read().len()}" }
                    }
                }
            }

            if *active_tab.read() == "bookings" {
                if bookings.read().is_empty() {
                    EmptyState { title: "No bookings".to_string(), description: Some("Book a resource to get started.".to_string()) }
                } else {
                    DataTable { headers: vec!["Resource".into(), "Title".into(), "Start".into(), "End".into(), "Status".into(), "Reschedules".into(), "Actions".into()],
                        for b in bookings.read().iter() {
                            tr {
                                td { "{b.resource_name.clone().unwrap_or_default()}" }
                                td { "{b.title}" }
                                td { class: "td-muted", "{b.start_time}" }
                                td { class: "td-muted", "{b.end_time}" }
                                td { StatusBadge { status: b.status.clone() } }
                                td { "{b.reschedule_count}/2" }
                                td {
                                    if b.status == "confirmed" {
                                        div { class: "btn-row-inline",
                                            if b.reschedule_count < 2 {
                                                button {
                                                    class: "btn btn-xs btn-outline",
                                                    onclick: { let u = b.uuid.clone(); move |_| show_reschedule.set(Some(u.clone())) },
                                                    "Reschedule"
                                                }
                                            }
                                            button {
                                                class: "btn btn-xs btn-danger-outline",
                                                onclick: { let u = b.uuid.clone(); move |_| handle_cancel(u.clone()) },
                                                "Cancel"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if *active_tab.read() == "resources" {
                div { class: "resource-grid",
                    for r in resources.read().iter() {
                        div { class: "resource-card",
                            h3 { "{r.name}" }
                            div { class: "resource-meta",
                                span { class: "badge badge-default", "{r.resource_type}" }
                                if let Some(ref loc) = r.location { span { class: "meta-muted", "{loc}" } }
                            }
                            if let Some(cap) = r.capacity { p { class: "meta-muted", "Capacity: {cap}" } }
                            p { class: "meta-muted", "Hours: {r.open_time} - {r.close_time}" }
                            p { class: "meta-muted", "Max booking: {r.max_booking_hours}h" }
                        }
                    }
                }
            }

            if *active_tab.read() == "breaches" {
                if breaches.read().is_empty() {
                    EmptyState { title: "No breaches".to_string(), description: Some("Clean record!".to_string()) }
                } else {
                    DataTable { headers: vec!["Type".into(), "Severity".into(), "Description".into(), "Status".into(), "Date".into()],
                        for b in breaches.read().iter() {
                            tr {
                                td { "{b.breach_type}" }
                                td { StatusBadge { status: b.severity.clone() } }
                                td { "{b.description}" }
                                td { StatusBadge { status: b.status.clone() } }
                                td { class: "td-muted", "{b.created_at}" }
                            }
                        }
                    }
                }
            }

            if *show_book.read() {
                Modal { title: "New Booking".to_string(), on_close: move |_| show_book.set(false),
                    form { class: "form", onsubmit: handle_book,
                        div { class: "form-group",
                            label { "Resource" }
                            select { class: "form-input", required: true, value: "{sel_resource}", onchange: move |e| sel_resource.set(e.value()),
                                option { value: "", "Select resource..." }
                                for r in resources.read().iter() {
                                    option { value: "{r.uuid}", "{r.name} ({r.resource_type})" }
                                }
                            }
                        }
                        div { class: "form-group",
                            label { "Title" }
                            input { class: "form-input", required: true, placeholder: "Meeting title", value: "{book_title}", oninput: move |e| book_title.set(e.value()) }
                        }
                        div { class: "form-row",
                            div { class: "form-group form-half",
                                label { "Start (YYYY-MM-DD HH:MM:SS)" }
                                input { class: "form-input", required: true, placeholder: "2025-06-15 10:00:00", value: "{book_start}", oninput: move |e| book_start.set(e.value()) }
                            }
                            div { class: "form-group form-half",
                                label { "End" }
                                input { class: "form-input", required: true, placeholder: "2025-06-15 12:00:00", value: "{book_end}", oninput: move |e| book_end.set(e.value()) }
                            }
                        }
                        div { class: "form-help", "Rules: max 90 days ahead, max 2 active per resource, max {resources.read().first().map(|r| r.max_booking_hours).unwrap_or(4)}h for rooms" }
                        button { r#type: "submit", class: "btn btn-primary btn-full", "Book" }
                    }
                }
            }

            if show_reschedule.read().is_some() {
                Modal { title: "Reschedule Booking".to_string(), on_close: move |_| show_reschedule.set(None),
                    div { class: "form",
                        div { class: "form-group",
                            label { "New Start" }
                            input { class: "form-input", placeholder: "YYYY-MM-DD HH:MM:SS", value: "{new_start}", oninput: move |e| new_start.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "New End" }
                            input { class: "form-input", placeholder: "YYYY-MM-DD HH:MM:SS", value: "{new_end}", oninput: move |e| new_end.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "Reason" }
                            input { class: "form-input", placeholder: "Optional", value: "{reschedule_reason}", oninput: move |e| reschedule_reason.set(e.value()) }
                        }
                        button { class: "btn btn-primary btn-full", onclick: handle_reschedule, "Reschedule" }
                    }
                }
            }
        }
    }
}
