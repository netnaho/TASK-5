// Pure-Rust role-based navigation logic.
//
// This module encodes the sidebar rules rendered by `layouts/mod.rs`
// in a form that can be unit-tested off-WASM. The live sidebar mirrors
// the output of `nav_items_for_role` so tests here guard against
// accidental role-visibility regressions.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Admin,
    StaffAuthor,
    DeptReviewer,
    Faculty,
    Student,
    Unknown,
}

impl Role {
    pub fn from_str(value: &str) -> Self {
        match value {
            "admin" => Role::Admin,
            "staff_author" => Role::StaffAuthor,
            "dept_reviewer" => Role::DeptReviewer,
            "faculty" => Role::Faculty,
            "student" => Role::Student,
            _ => Role::Unknown,
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }

    pub fn can_review(&self) -> bool {
        matches!(self, Role::Admin | Role::DeptReviewer)
    }

    pub fn is_staff_or_above(&self) -> bool {
        matches!(self, Role::Admin | Role::StaffAuthor)
    }
}

/// Canonical list of nav items a role can see in the sidebar.
/// Order matches the sidebar rendered by `layouts/mod.rs`.
pub fn nav_items_for_role(role: &str) -> Vec<&'static str> {
    let r = Role::from_str(role);
    let mut items: Vec<&'static str> = Vec::new();
    items.push("Dashboard");
    items.push("Notifications");
    items.push("Courses");
    if r.can_review() {
        items.push("Approvals");
    }
    items.push("Bookings");
    if r.is_admin() {
        items.push("Risk & Compliance");
        items.push("Audit Trail");
    }
    items.push("Privacy & Data");
    items
}

/// All client-side routes registered in `main.rs :: enum Route`.
/// Kept in sync by hand; changes to the router must be mirrored here.
pub const ROUTES: &[&str] = &[
    "/",
    "/dashboard",
    "/courses",
    "/courses/:uuid",
    "/courses/:uuid/edit",
    "/approvals",
    "/bookings",
    "/risk",
    "/privacy",
    "/audit",
    "/notifications",
    "/login",
];

pub fn is_admin_only_route(path: &str) -> bool {
    matches!(path, "/risk" | "/audit")
}

pub fn is_reviewer_route(path: &str) -> bool {
    matches!(path, "/approvals")
}
