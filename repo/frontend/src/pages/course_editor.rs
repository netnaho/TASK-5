use dioxus::prelude::*;
use crate::api;
use crate::auth::AUTH;
use crate::components::*;
use crate::types::*;

#[component]
pub fn CourseEditor(uuid: String) -> Element {
    // RBAC: only authors and admins can edit courses
    let auth = AUTH.read();
    let can_author = auth.role() == "admin" || auth.role() == "staff_author";

    if !can_author {
        ToastManager::error("Unauthorized access");
        let nav = navigator();
        nav.replace(crate::Route::Dashboard {});
        return rsx! { LoadingSpinner {} };
    }

    let mut course = use_signal(|| None::<CourseResponse>);
    let mut sections = use_signal(|| Vec::<SectionResponse>::new());
    let mut loading = use_signal(|| true);
    let mut title = use_signal(String::new);
    let mut desc = use_signal(String::new);
    let mut new_section_title = use_signal(String::new);
    let mut new_lesson_title = use_signal(String::new);
    let mut new_lesson_section = use_signal(String::new);
    let mut new_lesson_body = use_signal(String::new);
    let mut show_add_lesson = use_signal(|| false);
    let uuid_clone = uuid.clone();

    use_effect(move || {
        let u = uuid_clone.clone();
        spawn(async move {
            loading.set(true);
            if let Ok(r) = api::get_course(&u).await {
                title.set(r.data.title.clone());
                desc.set(r.data.description.clone().unwrap_or_default());
                course.set(Some(r.data));
            }
            if let Ok(r) = api::get_sections(&u).await { sections.set(r.data); }
            loading.set(false);
        });
    });

    let save_course = {
        let uuid = uuid.clone();
        move |_: Event<MouseData>| {
            let t = title.read().clone();
            let d = desc.read().clone();
            let u = uuid.clone();
            spawn(async move {
                let d_opt = if d.is_empty() { None } else { Some(d.as_str()) };
                match api::update_course(&u, Some(&t), d_opt).await {
                    Ok(_) => ToastManager::success("Course saved"),
                    Err(e) => ToastManager::error(format!("Save failed: {}", e)),
                }
            });
        }
    };

    let add_section = {
        let uuid = uuid.clone();
        move |_: Event<MouseData>| {
            let st = new_section_title.read().clone();
            let u = uuid.clone();
            let count = sections.read().len() as i32;
            spawn(async move {
                if st.is_empty() { ToastManager::error("Section title required"); return; }
                match api::create_section(&u, &st, count + 1).await {
                    Ok(_) => {
                        ToastManager::success("Section added");
                        new_section_title.set(String::new());
                        if let Ok(r) = api::get_sections(&u).await { sections.set(r.data); }
                    }
                    Err(e) => ToastManager::error(format!("Failed: {}", e)),
                }
            });
        }
    };

    let add_lesson = move |_: Event<MouseData>| {
        let sec_uuid = new_lesson_section.read().clone();
        let lt = new_lesson_title.read().clone();
        let lb = new_lesson_body.read().clone();
        let u = uuid.clone();
        spawn(async move {
            if lt.is_empty() || sec_uuid.is_empty() { ToastManager::error("Title and section required"); return; }
            match api::create_lesson(&sec_uuid, &lt, "text", &lb).await {
                Ok(_) => {
                    ToastManager::success("Lesson added");
                    show_add_lesson.set(false);
                    new_lesson_title.set(String::new());
                    new_lesson_body.set(String::new());
                    if let Ok(r) = api::get_sections(&u).await { sections.set(r.data); }
                }
                Err(e) => ToastManager::error(format!("Failed: {}", e)),
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
                    h1 { "Edit Course" }
                    if can_author {
                        button { class: "btn btn-primary", onclick: save_course, "Save Changes" }
                    }
                }
            }

            div { class: "editor-grid",
                div { class: "editor-main",
                    div { class: "card",
                        h3 { class: "card-title", "Course Details" }
                        div { class: "form-group",
                            label { "Title" }
                            input { class: "form-input", value: "{title}", oninput: move |e| title.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "Description" }
                            textarea { class: "form-input form-textarea", rows: "4", value: "{desc}", oninput: move |e| desc.set(e.value()) }
                        }
                    }

                    div { class: "card",
                        div { class: "card-header-row",
                            h3 { class: "card-title", "Sections & Lessons" }
                            if can_author {
                                button { class: "btn btn-sm btn-outline", onclick: move |_| show_add_lesson.set(true), "+ Lesson" }
                            }
                        }
                        for sec in sections.read().iter() {
                            div { class: "section-card",
                                h4 { class: "section-title", "{sec.title}" }
                                for lesson in sec.lessons.iter() {
                                    div { class: "lesson-row",
                                        span { class: "badge badge-default", "{lesson.content_type}" }
                                        span { "{lesson.title}" }
                                    }
                                }
                            }
                        }
                        if can_author {
                            div { class: "add-section-row",
                                input { class: "form-input", placeholder: "New section title", value: "{new_section_title}", oninput: move |e| new_section_title.set(e.value()) }
                                button { class: "btn btn-sm btn-primary", onclick: add_section, "+ Section" }
                            }
                        }
                    }
                }

                div { class: "editor-sidebar",
                    div { class: "card",
                        h3 { class: "card-title", "Status" }
                        if let Some(ref c) = *course.read() {
                            div { class: "sidebar-info",
                                StatusBadge { status: c.status.clone() }
                                p { class: "meta-muted", "Version: v{c.current_version}" }
                            }
                        }
                    }
                }
            }

            if *show_add_lesson.read() {
                Modal { title: "Add Lesson".to_string(), on_close: move |_| show_add_lesson.set(false),
                    div { class: "form",
                        div { class: "form-group",
                            label { "Section" }
                            select { class: "form-input", value: "{new_lesson_section}", onchange: move |e| new_lesson_section.set(e.value()),
                                option { value: "", "Select section..." }
                                for sec in sections.read().iter() {
                                    option { value: "{sec.uuid}", "{sec.title}" }
                                }
                            }
                        }
                        div { class: "form-group",
                            label { "Title" }
                            input { class: "form-input", value: "{new_lesson_title}", oninput: move |e| new_lesson_title.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "Content" }
                            textarea { class: "form-input form-textarea", rows: "6", value: "{new_lesson_body}", oninput: move |e| new_lesson_body.set(e.value()) }
                        }
                        button { class: "btn btn-primary btn-full", onclick: add_lesson, "Add Lesson" }
                    }
                }
            }
        }
    }
}
