use dioxus::prelude::*;
use crate::api;
use crate::auth::AUTH;
use crate::components::*;
use crate::types::*;

#[component]
pub fn Audit() -> Element {
    // RBAC: admin-only page
    let auth = AUTH.read();
    if !auth.is_admin() {
        ToastManager::error("Unauthorized access");
        let nav = navigator();
        nav.replace(crate::Route::Dashboard {});
        return rsx! { LoadingSpinner {} };
    }

    let mut logs = use_signal(|| Vec::<AuditLogEntry>::new());
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            loading.set(true);
            if let Ok(r) = api::list_audit_logs(Some(200)).await { logs.set(r.data); }
            loading.set(false);
        });
    });

    if *loading.read() { return rsx! { LoadingSpinner {} }; }

    rsx! {
        div { class: "page",
            div { class: "page-header",
                h1 { "Audit Trail" }
                p { class: "text-secondary", "Immutable log of all system actions. 7-year retention." }
            }

            if logs.read().is_empty() {
                EmptyState { title: "No audit entries".to_string() }
            } else {
                DataTable { headers: vec!["Action".into(), "Entity".into(), "User".into(), "Correlation".into(), "Timestamp".into()],
                    for log in logs.read().iter() {
                        tr {
                            td { span { class: "td-code", "{log.action}" } }
                            td {
                                "{log.entity_type}"
                                if let Some(eid) = log.entity_id {
                                    span { class: "meta-muted", " #{eid}" }
                                }
                            }
                            td {
                                if let Some(uid) = log.user_id {
                                    "User #{uid}"
                                } else {
                                    "System"
                                }
                            }
                            td { class: "td-muted td-truncate", "{log.correlation_id.clone().unwrap_or_default()}" }
                            td { class: "td-muted", "{log.created_at}" }
                        }
                    }
                }
            }
        }
    }
}
