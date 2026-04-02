use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub uuid: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub department_id: Option<i64>,
    pub is_active: bool,
    pub last_login_at: Option<NaiveDateTime>,
    pub last_reauth_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    StaffAuthor,
    DeptReviewer,
    Faculty,
    Student,
    Integration,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::StaffAuthor => "staff_author",
            UserRole::DeptReviewer => "dept_reviewer",
            UserRole::Faculty => "faculty",
            UserRole::Student => "student",
            UserRole::Integration => "integration",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(UserRole::Admin),
            "staff_author" => Some(UserRole::StaffAuthor),
            "dept_reviewer" => Some(UserRole::DeptReviewer),
            "faculty" => Some(UserRole::Faculty),
            "student" => Some(UserRole::Student),
            "integration" => Some(UserRole::Integration),
            _ => None,
        }
    }

    pub fn can_author_courses(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::StaffAuthor)
    }

    pub fn can_review_approvals(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::DeptReviewer)
    }

    pub fn can_view_published_courses(&self) -> bool {
        matches!(self, UserRole::Faculty | UserRole::Student | UserRole::Admin | UserRole::StaffAuthor | UserRole::DeptReviewer)
    }
}
