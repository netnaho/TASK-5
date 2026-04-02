use dioxus::prelude::*;
use crate::types::UserInfo;
use crate::api;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AuthState {
    pub user: Option<UserInfo>,
    pub is_authenticated: bool,
    pub is_loading: bool,
}

impl AuthState {
    pub fn logged_in(user: UserInfo) -> Self {
        Self { user: Some(user), is_authenticated: true, is_loading: false }
    }

    pub fn logged_out() -> Self {
        Self { user: None, is_authenticated: false, is_loading: false }
    }

    pub fn role(&self) -> &str {
        self.user.as_ref().map(|u| u.role.as_str()).unwrap_or("")
    }

    pub fn is_admin(&self) -> bool {
        self.role() == "admin"
    }

    pub fn is_staff_or_above(&self) -> bool {
        matches!(self.role(), "admin" | "staff_author")
    }

    pub fn can_review(&self) -> bool {
        matches!(self.role(), "admin" | "dept_reviewer")
    }
}

pub static AUTH: GlobalSignal<AuthState> = Signal::global(AuthState::default);

pub fn logout() {
    api::clear_token();
    *AUTH.write() = AuthState::logged_out();
    // Full page reload to wipe all in-memory signals/cache
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}

#[cfg(test)]
mod tests {
    use crate::types::UserInfo;
    use super::AuthState;

    fn mock_user(role: &str) -> UserInfo {
        UserInfo {
            uuid: "test-uuid".to_string(),
            username: "testuser".to_string(),
            email: "test@test.com".to_string(),
            full_name: "Test User".to_string(),
            role: role.to_string(),
            department_id: None,
        }
    }

    #[test]
    fn test_auth_state_initialization() {
        let state = AuthState::default();
        assert!(!state.is_authenticated);
        assert!(state.user.is_none());
        assert!(!state.is_loading);
        assert_eq!(state.role(), "");
        assert!(!state.is_admin());
        assert!(!state.is_staff_or_above());
        assert!(!state.can_review());

        let logged_out = AuthState::logged_out();
        assert!(!logged_out.is_authenticated);
        assert!(logged_out.user.is_none());
    }

    #[test]
    fn test_auth_state_admin_role() {
        let state = AuthState::logged_in(mock_user("admin"));
        assert!(state.is_authenticated);
        assert!(state.user.is_some());
        assert_eq!(state.role(), "admin");
        assert!(state.is_admin());
        assert!(state.is_staff_or_above());
        assert!(state.can_review());
    }

    #[test]
    fn test_auth_state_student_role() {
        let state = AuthState::logged_in(mock_user("student"));
        assert!(state.is_authenticated);
        assert_eq!(state.role(), "student");
        assert!(!state.is_admin());
        assert!(!state.is_staff_or_above());
        assert!(!state.can_review());
    }

    #[test]
    fn test_auth_state_staff_author_role() {
        let state = AuthState::logged_in(mock_user("staff_author"));
        assert!(state.is_authenticated);
        assert_eq!(state.role(), "staff_author");
        assert!(!state.is_admin());
        assert!(state.is_staff_or_above());
        assert!(!state.can_review());
    }

    #[test]
    fn test_auth_state_dept_reviewer_role() {
        let state = AuthState::logged_in(mock_user("dept_reviewer"));
        assert!(state.is_authenticated);
        assert_eq!(state.role(), "dept_reviewer");
        assert!(!state.is_admin());
        assert!(!state.is_staff_or_above());
        assert!(state.can_review());
    }

    #[test]
    fn test_auth_state_faculty_role() {
        let state = AuthState::logged_in(mock_user("faculty"));
        assert!(state.is_authenticated);
        assert_eq!(state.role(), "faculty");
        assert!(!state.is_admin());
        assert!(!state.is_staff_or_above());
        assert!(!state.can_review());
    }
}
