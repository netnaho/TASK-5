use dioxus::prelude::*;
use crate::api;
use crate::auth::AUTH;
use crate::components::*;
use crate::types::*;

#[component]
pub fn Courses() -> Element {
    let mut courses = use_signal(|| Vec::<CourseResponse>::new());
    let mut loading = use_signal(|| true);
    let mut show_create = use_signal(|| false);
    let mut new_title = use_signal(String::new);
    let mut new_code = use_signal(String::new);
    let mut new_desc = use_signal(String::new);
    let auth = AUTH.read();
    let can_author = auth.role() == "admin" || auth.role() == "staff_author";

    let load = move || {
        spawn(async move {
            loading.set(true);
            if let Ok(resp) = api::list_courses().await {
                courses.set(resp.data);
            }
            loading.set(false);
        });
    };

    use_effect(move || { load(); });

    let handle_create = move |evt: Event<FormData>| {
        evt.prevent_default();
        let title = new_title.read().clone();
        let code = new_code.read().clone();
        let desc = new_desc.read().clone();
        spawn(async move {
            let d = if desc.is_empty() { None } else { Some(desc.as_str()) };
            match api::create_course(&title, &code, d).await {
                Ok(_) => {
                    ToastManager::success("Course created");
                    show_create.set(false);
                    new_title.set(String::new());
                    new_code.set(String::new());
                    new_desc.set(String::new());
                    if let Ok(r) = api::list_courses().await { courses.set(r.data); }
                }
                Err(e) => ToastManager::error(format!("Failed: {}", e)),
            }
        });
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div { class: "page-header-row",
                    h1 { "Courses" }
                    if can_author {
                        button { class: "btn btn-primary", onclick: move |_| show_create.set(true), "+ New Course" }
                    }
                }
            }

            if *loading.read() {
                LoadingSpinner {}
            } else if courses.read().is_empty() {
                EmptyState { title: "No courses yet".to_string(), description: Some("Create your first course to get started.".to_string()) }
            } else {
                DataTable { headers: vec!["Code".into(), "Title".into(), "Status".into(), "Version".into(), "Updated".into(), "Actions".into()],
                    for course in courses.read().iter() {
                        tr {
                            td { class: "td-code", "{course.code}" }
                            td { Link { to: crate::Route::CourseDetail { uuid: course.uuid.clone() }, "{course.title}" } }
                            td { StatusBadge { status: course.status.clone() } }
                            td { "v{course.current_version}" }
                            td { class: "td-muted", "{course.updated_at}" }
                            td {
                                if can_author && (course.status == "draft" || course.status == "rejected") {
                                    Link { to: crate::Route::CourseEditor { uuid: course.uuid.clone() }, class: "btn btn-sm btn-outline", "Edit" }
                                }
                            }
                        }
                    }
                }
            }

            if *show_create.read() {
                Modal { title: "Create Course".to_string(), on_close: move |_| show_create.set(false),
                    form { class: "form", onsubmit: handle_create,
                        div { class: "form-group",
                            label { "Course Code" }
                            input { class: "form-input", required: true, placeholder: "e.g. CS101", value: "{new_code}", oninput: move |e| new_code.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "Title" }
                            input { class: "form-input", required: true, placeholder: "Course title", value: "{new_title}", oninput: move |e| new_title.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "Description" }
                            textarea { class: "form-input form-textarea", placeholder: "Optional description", value: "{new_desc}", oninput: move |e| new_desc.set(e.value()) }
                        }
                        button { r#type: "submit", class: "btn btn-primary btn-full", "Create Course" }
                    }
                }
            }
        }
    }
}
