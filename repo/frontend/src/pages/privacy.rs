use dioxus::prelude::*;
use crate::api;
use crate::auth::AUTH;
use crate::components::*;
use crate::types::*;

#[component]
pub fn Privacy() -> Element {
    let mut requests = use_signal(|| Vec::<DataRequestResponse>::new());
    let mut masked = use_signal(|| Vec::<MaskedFieldResponse>::new());
    let mut loading = use_signal(|| true);
    let mut active_tab = use_signal(|| "requests");
    let mut show_create = use_signal(|| false);
    let mut req_type = use_signal(|| "export".to_string());
    let mut req_reason = use_signal(String::new);
    let auth = AUTH.read();
    let is_admin = auth.is_admin();

    use_effect(move || {
        spawn(async move {
            loading.set(true);
            if is_admin {
                if let Ok(r) = api::list_all_data_requests().await { requests.set(r.data); }
            } else {
                if let Ok(r) = api::my_data_requests().await { requests.set(r.data); }
            }
            if let Ok(r) = api::get_masked_fields().await { masked.set(r.data); }
            loading.set(false);
        });
    });

    let handle_create = move |evt: Event<FormData>| {
        evt.prevent_default();
        let t = req_type.read().clone();
        let r = req_reason.read().clone();
        spawn(async move {
            let reason = if r.is_empty() { None } else { Some(r.as_str()) };
            match api::create_data_request(&t, reason).await {
                Ok(_) => {
                    ToastManager::success("Data request submitted");
                    show_create.set(false);
                    if let Ok(r) = api::my_data_requests().await { requests.set(r.data); }
                }
                Err(e) => ToastManager::error(format!("{}", e)),
            }
        });
    };

    let handle_review = move |uuid: String, approved: bool| {
        spawn(async move {
            let notes = if approved { Some("Approved by admin") } else { Some("Rejected") };
            match api::review_data_request(&uuid, approved, notes).await {
                Ok(_) => {
                    ToastManager::success(if approved { "Approved & processed" } else { "Rejected" });
                    if let Ok(r) = api::list_all_data_requests().await { requests.set(r.data); }
                }
                Err(e) => ToastManager::error(format!("{}", e)),
            }
        });
    };

    if *loading.read() { return rsx! { LoadingSpinner {} }; }

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div { class: "page-header-row",
                    h1 { "Privacy & Data" }
                    if !is_admin {
                        button { class: "btn btn-primary", onclick: move |_| show_create.set(true), "New Data Request" }
                    }
                }
            }

            div { class: "tabs",
                button { class: if *active_tab.read() == "requests" { "tab active" } else { "tab" }, onclick: move |_| active_tab.set("requests"), "Data Requests" }
                button { class: if *active_tab.read() == "sensitive" { "tab active" } else { "tab" }, onclick: move |_| active_tab.set("sensitive"), "Sensitive Fields" }
            }

            if *active_tab.read() == "requests" {
                if requests.read().is_empty() {
                    EmptyState { title: "No data requests".to_string() }
                } else {
                    DataTable { headers: vec!["Type".into(), "Status".into(), "Reason".into(), "Created".into(), if is_admin { "Actions".into() } else { "Notes".into() }],
                        for r in requests.read().iter() {
                            tr {
                                td { span { class: "badge badge-default", "{r.request_type}" } }
                                td { StatusBadge { status: r.status.clone() } }
                                td { "{r.reason.clone().unwrap_or_default()}" }
                                td { class: "td-muted", "{r.created_at}" }
                                td {
                                    if is_admin && r.status == "pending" {
                                        div { class: "btn-row-inline",
                                            button { class: "btn btn-xs btn-success", onclick: { let u = r.uuid.clone(); move |_| handle_review(u.clone(), true) }, "Approve" }
                                            button { class: "btn btn-xs btn-danger-outline", onclick: { let u = r.uuid.clone(); move |_| handle_review(u.clone(), false) }, "Reject" }
                                        }
                                    } else {
                                        "{r.admin_notes.clone().unwrap_or_default()}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if *active_tab.read() == "sensitive" {
                if masked.read().is_empty() {
                    EmptyState { title: "No sensitive data stored".to_string() }
                } else {
                    DataTable { headers: vec!["Field".into(), "Value (Masked)".into()],
                        for f in masked.read().iter() {
                            tr {
                                td { "{f.field_name}" }
                                td { class: "td-code", "{f.masked_value}" }
                            }
                        }
                    }
                }
            }

            if *show_create.read() {
                Modal { title: "New Data Request".to_string(), on_close: move |_| show_create.set(false),
                    form { class: "form", onsubmit: handle_create,
                        div { class: "form-group",
                            label { "Request Type" }
                            select { class: "form-input", value: "{req_type}", onchange: move |e| req_type.set(e.value()),
                                option { value: "export", "Export My Data" }
                                option { value: "delete", "Delete My Data" }
                                option { value: "rectify", "Rectify My Data" }
                            }
                        }
                        div { class: "form-group",
                            label { "Reason" }
                            textarea { class: "form-input form-textarea", placeholder: "Why are you requesting this?", value: "{req_reason}", oninput: move |e| req_reason.set(e.value()) }
                        }
                        button { r#type: "submit", class: "btn btn-primary btn-full", "Submit Request" }
                    }
                }
            }
        }
    }
}
