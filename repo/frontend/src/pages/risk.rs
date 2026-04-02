use dioxus::prelude::*;
use crate::api;
use crate::auth::AUTH;
use crate::components::*;
use crate::types::*;

#[component]
pub fn Risk() -> Element {
    // RBAC: admin-only page
    let auth = AUTH.read();
    if !auth.is_admin() {
        ToastManager::error("Unauthorized access");
        let nav = navigator();
        nav.replace(crate::Route::Dashboard {});
        return rsx! { LoadingSpinner {} };
    }

    let mut events = use_signal(|| Vec::<RiskEventResponse>::new());
    let mut rules = use_signal(|| Vec::<RiskRuleResponse>::new());
    let mut loading = use_signal(|| true);
    let mut active_tab = use_signal(|| "events");

    use_effect(move || {
        spawn(async move {
            loading.set(true);
            if let Ok(r) = api::list_risk_events().await { events.set(r.data); }
            if let Ok(r) = api::list_risk_rules().await { rules.set(r.data); }
            loading.set(false);
        });
    });

    let run_eval = move |_: Event<MouseData>| {
        spawn(async move {
            match api::run_risk_evaluation().await {
                Ok(r) => {
                    ToastManager::success(format!("Evaluation complete: {} events", r.data.events_created.unwrap_or(0)));
                    if let Ok(r) = api::list_risk_events().await { events.set(r.data); }
                }
                Err(e) => ToastManager::error(format!("{}", e)),
            }
        });
    };

    let handle_dismiss = move |uuid: String| {
        spawn(async move {
            match api::update_risk_event(&uuid, "acknowledged", Some("Reviewed by admin")).await {
                Ok(_) => {
                    ToastManager::success("Event acknowledged");
                    if let Ok(r) = api::list_risk_events().await { events.set(r.data); }
                }
                Err(e) => ToastManager::error(format!("{}", e)),
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
                    h1 { "Risk & Compliance" }
                    button { class: "btn btn-primary", onclick: run_eval, "Run Evaluation" }
                }
            }

            div { class: "tabs",
                button { class: if *active_tab.read() == "events" { "tab active" } else { "tab" }, onclick: move |_| active_tab.set("events"), "Risk Events ({events.read().len()})" }
                button { class: if *active_tab.read() == "rules" { "tab active" } else { "tab" }, onclick: move |_| active_tab.set("rules"), "Rules" }
            }

            if *active_tab.read() == "events" {
                if events.read().is_empty() {
                    EmptyState { title: "No risk events".to_string() }
                } else {
                    DataTable { headers: vec!["Rule".into(), "Score".into(), "Status".into(), "Entity".into(), "Date".into(), "Actions".into()],
                        for ev in events.read().iter() {
                            tr {
                                td { "{ev.rule_name.clone().unwrap_or_default()}" }
                                td {
                                    {
                                        let score_class = if ev.risk_score >= 80.0 { "text-danger".to_string() } else if ev.risk_score >= 50.0 { "text-warning".to_string() } else { "text-muted".to_string() };
                                        rsx! { span { class: "{score_class}", "{ev.risk_score:.0}" } }
                                    }
                                }
                                td { StatusBadge { status: ev.status.clone() } }
                                td { class: "td-muted", "{ev.entity_type.clone().unwrap_or_default()}" }
                                td { class: "td-muted", "{ev.created_at}" }
                                td {
                                    if ev.status == "new" {
                                        button {
                                            class: "btn btn-xs btn-outline",
                                            onclick: { let u = ev.uuid.clone(); move |_| handle_dismiss(u.clone()) },
                                            "Acknowledge"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if *active_tab.read() == "rules" {
                DataTable { headers: vec!["Name".into(), "Type".into(), "Severity".into(), "Interval".into(), "Last Run".into()],
                    for rule in rules.read().iter() {
                        tr {
                            td { "{rule.name}" }
                            td { class: "td-code", "{rule.rule_type}" }
                            td { StatusBadge { status: rule.severity.clone() } }
                            td { "{rule.schedule_interval_minutes} min" }
                            td { class: "td-muted", "{rule.last_run_at.clone().unwrap_or_else(|| \"Never\".to_string())}" }
                        }
                    }
                }
            }
        }
    }
}
