use dioxus::prelude::*;
use crate::api;
use crate::auth::AUTH;
use crate::components::*;
use crate::types::*;

#[component]
pub fn CourseDetail(uuid: String) -> Element {
    let mut course = use_signal(|| None::<CourseResponse>);
    let mut sections = use_signal(|| Vec::<SectionResponse>::new());
    let mut versions = use_signal(|| Vec::<VersionResponse>::new());
    let mut loading = use_signal(|| true);
    let mut show_submit = use_signal(|| false);
    let mut release_notes = use_signal(String::new);
    let mut effective_date = use_signal(String::new);
    let mut active_tab = use_signal(|| "content");
    let auth = AUTH.read();
    let can_author = auth.role() == "admin" || auth.role() == "staff_author";
    let uuid_clone = uuid.clone();

    use_effect(move || {
        let u = uuid_clone.clone();
        spawn(async move {
            loading.set(true);
            if let Ok(r) = api::get_course(&u).await { course.set(Some(r.data)); }
            if let Ok(r) = api::get_sections(&u).await { sections.set(r.data); }
            if let Ok(r) = api::get_versions(&u).await { versions.set(r.data); }
            loading.set(false);
        });
    });

    let handle_submit_approval = {
        let uuid = uuid.clone();
        move |evt: Event<FormData>| {
            evt.prevent_default();
            let rn = release_notes.read().clone();
            let ed = effective_date.read().clone();
            let u = uuid.clone();
            spawn(async move {
                match api::submit_for_approval(&u, &rn, &ed).await {
                    Ok(_) => {
                        ToastManager::success("Submitted for approval");
                        show_submit.set(false);
                        if let Ok(r) = api::get_course(&u).await { course.set(Some(r.data)); }
                    }
                    Err(e) => ToastManager::error(format!("Failed: {}", e)),
                }
            });
        }
    };

    if *loading.read() {
        return rsx! { LoadingSpinner {} };
    }

    let c = match course.read().as_ref() {
        Some(c) => c.clone(),
        None => return rsx! { EmptyState { title: "Course not found".to_string() } },
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div { class: "page-header-row",
                    div {
                        h1 { "{c.title}" }
                        div { class: "page-meta",
                            span { class: "meta-code", "{c.code}" }
                            StatusBadge { status: c.status.clone() }
                            span { class: "meta-version", "v{c.current_version}" }
                            if let Some(ref upd) = c.updated_on {
                                span { class: "meta-muted", "Updated: {upd}" }
                            }
                        }
                    }
                    div { class: "page-actions",
                        if can_author && (c.status == "draft" || c.status == "rejected") {
                            Link { to: crate::Route::CourseEditor { uuid: c.uuid.clone() }, class: "btn btn-outline", "Edit" }
                            button { class: "btn btn-primary", onclick: move |_| show_submit.set(true), "Submit for Approval" }
                        }
                    }
                }
            }

            if let Some(ref desc) = c.description {
                p { class: "course-description", "{desc}" }
            }

            if !c.tags.is_empty() {
                div { class: "tag-list",
                    for tag in c.tags.iter() {
                        span { class: "tag", "{tag.name}" }
                    }
                }
            }

            // Tabs
            div { class: "tabs",
                button { class: if *active_tab.read() == "content" { "tab active" } else { "tab" }, onclick: move |_| active_tab.set("content"), "Content" }
                button { class: if *active_tab.read() == "versions" { "tab active" } else { "tab" }, onclick: move |_| active_tab.set("versions"), "Versions ({versions.read().len()})" }
            }

            if *active_tab.read() == "content" {
                if sections.read().is_empty() {
                    EmptyState { title: "No sections yet".to_string() }
                } else {
                    for sec in sections.read().iter() {
                        div { class: "section-card",
                            h3 { class: "section-title", "{sec.title}" }
                            if sec.lessons.is_empty() {
                                p { class: "text-muted", "No lessons" }
                            }
                            for lesson in sec.lessons.iter() {
                                div { class: "lesson-row",
                                    span { class: "lesson-type badge badge-default", "{lesson.content_type}" }
                                    span { class: "lesson-title", "{lesson.title}" }
                                    if let Some(dur) = lesson.duration_minutes {
                                        span { class: "lesson-duration", "{dur} min" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if *active_tab.read() == "versions" {
                if versions.read().is_empty() {
                    EmptyState { title: "No versions yet".to_string() }
                } else {
                    DataTable { headers: vec!["Version".into(), "Summary".into(), "Created".into(), "Expires".into()],
                        for v in versions.read().iter() {
                            tr {
                                td { "v{v.version_number}" }
                                td { "{v.change_summary.clone().unwrap_or_default()}" }
                                td { class: "td-muted", "{v.created_at}" }
                                td { class: "td-muted", "{v.expires_at.clone().unwrap_or_else(|| \"--\".to_string())}" }
                            }
                        }
                    }
                }
            }

            if *show_submit.read() {
                Modal { title: "Submit for Approval".to_string(), on_close: move |_| show_submit.set(false),
                    form { class: "form", onsubmit: handle_submit_approval,
                        div { class: "form-group",
                            label { "Release Notes (required)" }
                            textarea { class: "form-input form-textarea", required: true, placeholder: "Describe what changed...", value: "{release_notes}", oninput: move |e| release_notes.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "Effective Date (MM/DD/YYYY HH:MM AM/PM)" }
                            input { class: "form-input", required: true, placeholder: "e.g. 06/15/2025 09:00 AM", value: "{effective_date}", oninput: move |e| effective_date.set(e.value()) }
                        }
                        button { r#type: "submit", class: "btn btn-primary btn-full", "Submit" }
                    }
                }
            }
        }
    }
}
