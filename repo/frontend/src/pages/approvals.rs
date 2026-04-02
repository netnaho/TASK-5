use dioxus::prelude::*;
use crate::api;
use crate::auth::AUTH;
use crate::components::*;
use crate::types::*;

#[component]
pub fn Approvals() -> Element {
    let mut queue = use_signal(|| Vec::<ApprovalQueueItem>::new());
    let mut loading = use_signal(|| true);
    let mut selected = use_signal(|| None::<ApprovalQueueItem>);
    let mut comments = use_signal(String::new);
    let auth = AUTH.read();
    let can_review = auth.role() == "admin" || auth.role() == "dept_reviewer";

    // RBAC: only reviewers and admins can access this page
    if !can_review {
        ToastManager::error("Unauthorized access");
        let nav = navigator();
        nav.replace(crate::Route::Dashboard {});
        return rsx! { LoadingSpinner {} };
    }

    use_effect(move || {
        spawn(async move {
            loading.set(true);
            if let Ok(r) = api::get_approval_queue().await { queue.set(r.data); }
            loading.set(false);
        });
    });

    let handle_review = move |approved: bool| {
        let sel = selected.read().clone();
        let c = comments.read().clone();
        spawn(async move {
            if let Some(item) = sel {
                let c_opt = if c.is_empty() { None } else { Some(c.as_str()) };
                match api::review_approval(&item.approval.uuid, approved, c_opt).await {
                    Ok(_) => {
                        ToastManager::success(if approved { "Approved" } else { "Rejected" });
                        selected.set(None);
                        comments.set(String::new());
                        if let Ok(r) = api::get_approval_queue().await { queue.set(r.data); }
                    }
                    Err(e) => ToastManager::error(format!("Failed: {}", e)),
                }
            }
        });
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                h1 { "Approval Queue" }
            }

            if *loading.read() {
                LoadingSpinner {}
            } else if queue.read().is_empty() {
                EmptyState { title: "No pending approvals".to_string(), description: Some("All caught up!".to_string()) }
            } else {
                DataTable { headers: vec!["Course".into(), "Code".into(), "Requester".into(), "Status".into(), "Effective".into(), "Actions".into()],
                    for item in queue.read().iter() {
                        tr {
                            td { "{item.course_title}" }
                            td { class: "td-code", "{item.course_code}" }
                            td { "{item.requester_name}" }
                            td { StatusBadge { status: item.approval.status.clone() } }
                            td { class: "td-muted", "{item.approval.effective_date.clone().unwrap_or_default()}" }
                            td {
                                if can_review {
                                    button {
                                        class: "btn btn-sm btn-outline",
                                        onclick: { let i = item.clone(); move |_| selected.set(Some(i.clone())) },
                                        "Review"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(item) = selected.read().as_ref() {
                Modal { title: format!("Review: {}", item.course_title), on_close: move |_| selected.set(None),
                    div { class: "review-detail",
                        div { class: "detail-row",
                            span { class: "detail-label", "Status:" }
                            StatusBadge { status: item.approval.status.clone() }
                        }
                        if let Some(ref rn) = item.approval.release_notes {
                            div { class: "detail-row",
                                span { class: "detail-label", "Release Notes:" }
                                p { "{rn}" }
                            }
                        }
                        div { class: "detail-row",
                            span { class: "detail-label", "Steps:" }
                            div { class: "steps-list",
                                for step in item.approval.steps.iter() {
                                    div { class: "step-row",
                                        span { "Step {step.step_order}: " }
                                        StatusBadge { status: step.status.clone() }
                                        if let Some(ref role) = step.reviewer_role {
                                            span { class: "meta-muted", " ({role})" }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "form-group",
                            label { "Comments" }
                            textarea { class: "form-input form-textarea", value: "{comments}", oninput: move |e| comments.set(e.value()) }
                        }
                        div { class: "btn-row",
                            button { class: "btn btn-success", onclick: move |_| handle_review(true), "Approve" }
                            button { class: "btn btn-danger", onclick: move |_| handle_review(false), "Reject" }
                        }
                    }
                }
            }
        }
    }
}
