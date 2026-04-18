//! Strict frontend unit tests.
//!
//! These tests execute real frontend Rust modules — they use `#[path = ...]`
//! to include the actual source files from `../../frontend/src/*` rather than
//! mirroring their contents. Any change to those real modules is immediately
//! reflected in this test crate.
//!
//! Run with:
//!     cargo test --manifest-path unit_tests/frontend_rs/Cargo.toml

#![allow(unused_imports)]
#![allow(dead_code)]

#[path = "../../../frontend/src/types/mod.rs"]
pub mod frontend_types;

#[path = "../../../frontend/src/role_nav.rs"]
pub mod role_nav;

#[cfg(test)]
mod types_tests {
    use super::frontend_types::*;

    #[test]
    fn user_info_serde_round_trip() {
        let user = UserInfo {
            uuid: "u-123".into(),
            username: "jdoe".into(),
            email: "jdoe@example.com".into(),
            full_name: "Jane Doe".into(),
            role: "admin".into(),
            department_id: Some(7),
        };
        let serialized = serde_json::to_string(&user).expect("serialize");
        assert!(serialized.contains("\"role\":\"admin\""));
        let parsed: UserInfo = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(parsed, user);
        assert_eq!(parsed.department_id, Some(7));
    }

    #[test]
    fn login_response_defaults_and_nested_user() {
        let body = r#"{
            "token":"tkn",
            "token_type":"Bearer",
            "expires_in":3600,
            "user":{
                "uuid":"u1","username":"admin","email":"a@b.c",
                "full_name":"A","role":"admin","department_id":null
            }
        }"#;
        let parsed: LoginResponse = serde_json::from_str(body).expect("parse");
        assert_eq!(parsed.token, "tkn");
        assert_eq!(parsed.token_type, "Bearer");
        assert_eq!(parsed.expires_in, 3600);
        assert_eq!(parsed.user.role, "admin");
        assert!(parsed.user.department_id.is_none());
    }

    #[test]
    fn api_response_envelope_shape() {
        // Mirrors the backend envelope: { success, data, message? }
        let raw = r#"{"success":true,"data":{"status":"ok","service":"api"},"message":null}"#;
        let env: ApiResponse<HealthResponse> = serde_json::from_str(raw).expect("parse");
        assert!(env.success);
        assert_eq!(env.data.status, "ok");
        assert_eq!(env.data.service, "api");
        assert!(env.message.is_none());
    }

    #[test]
    fn course_response_includes_tags_vec() {
        let raw = r#"{
            "uuid":"c1","title":"T","code":"C","description":null,
            "department_id":null,"term_id":null,"instructor_id":null,
            "status":"draft","visibility":"internal","max_enrollment":null,
            "current_version":1,"release_notes":null,"effective_date":null,
            "updated_on":null,"tags":[{"id":1,"uuid":"t1","name":"tag","slug":"tag"}],
            "created_at":"2024-01-01","updated_at":"2024-01-01"
        }"#;
        let c: CourseResponse = serde_json::from_str(raw).expect("parse");
        assert_eq!(c.title, "T");
        assert_eq!(c.status, "draft");
        assert_eq!(c.current_version, 1);
        assert_eq!(c.tags.len(), 1);
        assert_eq!(c.tags[0].slug, "tag");
    }

    #[test]
    fn masked_field_response_parses() {
        let raw = r#"{"field_name":"ssn","masked_value":"***-**-####"}"#;
        let m: MaskedFieldResponse = serde_json::from_str(raw).expect("parse");
        assert_eq!(m.field_name, "ssn");
        assert_eq!(m.masked_value, "***-**-####");
    }

    #[test]
    fn unread_count_parses_int() {
        let raw = r#"{"count":42}"#;
        let u: UnreadCount = serde_json::from_str(raw).expect("parse");
        assert_eq!(u.count, 42);
    }

    #[test]
    fn tag_response_round_trip() {
        let t = TagResponse { id: 1, uuid: "t1".into(), name: "Data".into(), slug: "data".into() };
        let json = serde_json::to_string(&t).unwrap();
        let back: TagResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, 1);
        assert_eq!(back.uuid, "t1");
        assert_eq!(back.name, "Data");
        assert_eq!(back.slug, "data");
    }

    #[test]
    fn count_response_optional_fields() {
        let only_events = r#"{"events_created":3}"#;
        let c: CountResponse = serde_json::from_str(only_events).unwrap();
        assert_eq!(c.events_created, Some(3));
        assert!(c.transitions_processed.is_none());

        let only_trans = r#"{"transitions_processed":5}"#;
        let c2: CountResponse = serde_json::from_str(only_trans).unwrap();
        assert_eq!(c2.transitions_processed, Some(5));
    }
}

#[cfg(test)]
mod role_nav_tests {
    use super::role_nav::*;

    #[test]
    fn role_from_str_recognizes_all_known() {
        assert_eq!(Role::from_str("admin"), Role::Admin);
        assert_eq!(Role::from_str("staff_author"), Role::StaffAuthor);
        assert_eq!(Role::from_str("dept_reviewer"), Role::DeptReviewer);
        assert_eq!(Role::from_str("faculty"), Role::Faculty);
        assert_eq!(Role::from_str("student"), Role::Student);
    }

    #[test]
    fn role_from_str_unknown_is_unknown() {
        assert_eq!(Role::from_str(""), Role::Unknown);
        assert_eq!(Role::from_str("integration"), Role::Unknown);
        assert_eq!(Role::from_str("guest"), Role::Unknown);
    }

    #[test]
    fn admin_predicate() {
        assert!(Role::Admin.is_admin());
        assert!(!Role::StaffAuthor.is_admin());
        assert!(!Role::Student.is_admin());
    }

    #[test]
    fn can_review_permits_admin_and_dept_reviewer() {
        assert!(Role::Admin.can_review());
        assert!(Role::DeptReviewer.can_review());
        assert!(!Role::StaffAuthor.can_review());
        assert!(!Role::Faculty.can_review());
        assert!(!Role::Student.can_review());
    }

    #[test]
    fn is_staff_or_above_permits_admin_and_staff_author() {
        assert!(Role::Admin.is_staff_or_above());
        assert!(Role::StaffAuthor.is_staff_or_above());
        assert!(!Role::DeptReviewer.is_staff_or_above());
        assert!(!Role::Faculty.is_staff_or_above());
    }

    #[test]
    fn admin_sees_admin_nav_sections() {
        let items = nav_items_for_role("admin");
        assert!(items.contains(&"Dashboard"));
        assert!(items.contains(&"Courses"));
        assert!(items.contains(&"Approvals"));
        assert!(items.contains(&"Bookings"));
        assert!(items.contains(&"Risk & Compliance"));
        assert!(items.contains(&"Audit Trail"));
        assert!(items.contains(&"Privacy & Data"));
        assert!(items.contains(&"Notifications"));
    }

    #[test]
    fn student_has_no_admin_sections() {
        let items = nav_items_for_role("student");
        assert!(items.contains(&"Dashboard"));
        assert!(items.contains(&"Courses"));
        assert!(items.contains(&"Bookings"));
        assert!(items.contains(&"Privacy & Data"));
        assert!(!items.contains(&"Approvals"));
        assert!(!items.contains(&"Risk & Compliance"));
        assert!(!items.contains(&"Audit Trail"));
    }

    #[test]
    fn dept_reviewer_sees_approvals_not_admin_sections() {
        let items = nav_items_for_role("dept_reviewer");
        assert!(items.contains(&"Approvals"));
        assert!(!items.contains(&"Risk & Compliance"));
        assert!(!items.contains(&"Audit Trail"));
    }

    #[test]
    fn staff_author_does_not_see_approvals_queue() {
        let items = nav_items_for_role("staff_author");
        assert!(items.contains(&"Courses"));
        assert!(!items.contains(&"Approvals"));
        assert!(!items.contains(&"Risk & Compliance"));
    }

    #[test]
    fn faculty_minimal_nav() {
        let items = nav_items_for_role("faculty");
        for required in ["Dashboard", "Courses", "Bookings", "Privacy & Data"] {
            assert!(items.contains(&required), "faculty missing {required}");
        }
        for forbidden in ["Approvals", "Risk & Compliance", "Audit Trail"] {
            assert!(!items.contains(&forbidden), "faculty should not see {forbidden}");
        }
    }

    #[test]
    fn unknown_role_has_base_nav_only() {
        let items = nav_items_for_role("ghost");
        for required in ["Dashboard", "Courses", "Bookings", "Privacy & Data"] {
            assert!(items.contains(&required));
        }
        assert!(!items.contains(&"Approvals"));
        assert!(!items.contains(&"Risk & Compliance"));
        assert!(!items.contains(&"Audit Trail"));
    }

    #[test]
    fn nav_items_have_no_duplicates() {
        for role in ["admin", "staff_author", "dept_reviewer", "faculty", "student", "unknown"] {
            let items = nav_items_for_role(role);
            let mut sorted = items.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(items.len(), sorted.len(), "duplicate nav item for {role}: {items:?}");
        }
    }

    #[test]
    fn dashboard_is_first_for_all_roles() {
        for role in ["admin", "staff_author", "dept_reviewer", "faculty", "student"] {
            let items = nav_items_for_role(role);
            assert_eq!(items.first(), Some(&"Dashboard"), "Dashboard must be first for {role}");
        }
    }

    #[test]
    fn routes_registry_includes_login_and_dashboard() {
        assert!(ROUTES.contains(&"/"));
        assert!(ROUTES.contains(&"/login"));
        assert!(ROUTES.contains(&"/dashboard"));
        assert!(ROUTES.contains(&"/courses"));
        assert!(ROUTES.contains(&"/courses/:uuid"));
        assert!(ROUTES.contains(&"/courses/:uuid/edit"));
        assert!(ROUTES.contains(&"/approvals"));
        assert!(ROUTES.contains(&"/bookings"));
        assert!(ROUTES.contains(&"/risk"));
        assert!(ROUTES.contains(&"/privacy"));
        assert!(ROUTES.contains(&"/audit"));
        assert!(ROUTES.contains(&"/notifications"));
    }

    #[test]
    fn all_routes_start_with_slash() {
        for r in ROUTES {
            assert!(r.starts_with('/'), "route must start with '/': {r}");
        }
    }

    #[test]
    fn admin_only_routes_flagged() {
        assert!(is_admin_only_route("/risk"));
        assert!(is_admin_only_route("/audit"));
        assert!(!is_admin_only_route("/courses"));
        assert!(!is_admin_only_route("/bookings"));
        assert!(!is_admin_only_route("/dashboard"));
    }

    #[test]
    fn reviewer_route_flag() {
        assert!(is_reviewer_route("/approvals"));
        assert!(!is_reviewer_route("/risk"));
        assert!(!is_reviewer_route("/audit"));
        assert!(!is_reviewer_route("/courses"));
    }
}
