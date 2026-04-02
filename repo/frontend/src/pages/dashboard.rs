use dioxus::prelude::*;
use crate::api;
use crate::auth::AUTH;

#[component]
pub fn Dashboard() -> Element {
    let auth = AUTH.read();
    let mut course_count = use_signal(|| 0usize);
    let mut booking_count = use_signal(|| 0usize);
    let mut approval_count = use_signal(|| 0usize);

    let greeting = auth.user.as_ref()
        .map(|u| format!("Welcome back, {}", u.full_name))
        .unwrap_or_else(|| "Welcome".to_string());

    let role = auth.role().to_string();
    let role_for_effect = role.clone();

    use_effect(move || {
        let r = role_for_effect.clone();
        spawn(async move {
            if let Ok(resp) = api::list_courses().await { course_count.set(resp.data.len()); }
            if let Ok(resp) = api::my_bookings().await { booking_count.set(resp.data.iter().filter(|b| b.status == "confirmed").count()); }
            if r == "admin" || r == "dept_reviewer" {
                if let Ok(resp) = api::get_approval_queue().await { approval_count.set(resp.data.len()); }
            }
        });
    });

    rsx! {
        div { class: "page",
            div { class: "page-header",
                h1 { "{greeting}" }
                p { class: "text-secondary", "CampusLearn Operations Suite" }
            }
            div { class: "dashboard-grid",
                Link { to: crate::Route::Courses {}, class: "stat-card",
                    h3 { "Courses" }
                    p { class: "stat-value", "{course_count}" }
                    p { class: "stat-label", "Available courses" }
                }
                Link { to: crate::Route::Bookings {}, class: "stat-card",
                    h3 { "Bookings" }
                    p { class: "stat-value", "{booking_count}" }
                    p { class: "stat-label", "Active bookings" }
                }
                if auth.can_review() {
                    Link { to: crate::Route::Approvals {}, class: "stat-card",
                        h3 { "Approvals" }
                        p { class: "stat-value", "{approval_count}" }
                        p { class: "stat-label", "Pending review" }
                    }
                }
                Link { to: crate::Route::Privacy {}, class: "stat-card",
                    h3 { "Privacy" }
                    p { class: "stat-value", "--" }
                    p { class: "stat-label", "Data requests" }
                }
            }
        }
    }
}
