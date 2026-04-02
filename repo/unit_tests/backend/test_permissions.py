"""Unit tests for RBAC permission checks."""
import unittest


ROLES = ["admin", "staff_author", "dept_reviewer", "faculty", "student", "integration"]

# Mirrors UserRole methods from backend/src/models/user.rs
def can_author_courses(role: str) -> bool:
    return role in ("admin", "staff_author")

def can_review_approvals(role: str) -> bool:
    return role in ("admin", "dept_reviewer")

def can_view_published_courses(role: str) -> bool:
    return role in ("faculty", "student", "admin", "staff_author", "dept_reviewer")

# Mirrors self-approval prevention from approval_service.rs
def can_approve_own_submission(requester_id: int, reviewer_id: int) -> bool:
    return requester_id != reviewer_id

# Mirrors role guard logic
def is_allowed_by_guard(user_role: str, required_roles: list[str]) -> bool:
    return user_role in required_roles or user_role == "admin"


class TestRolePermissions(unittest.TestCase):
    def test_admin_can_author(self):
        self.assertTrue(can_author_courses("admin"))

    def test_staff_author_can_author(self):
        self.assertTrue(can_author_courses("staff_author"))

    def test_reviewer_cannot_author(self):
        self.assertFalse(can_author_courses("dept_reviewer"))

    def test_faculty_cannot_author(self):
        self.assertFalse(can_author_courses("faculty"))

    def test_student_cannot_author(self):
        self.assertFalse(can_author_courses("student"))

    def test_admin_can_review(self):
        self.assertTrue(can_review_approvals("admin"))

    def test_dept_reviewer_can_review(self):
        self.assertTrue(can_review_approvals("dept_reviewer"))

    def test_staff_author_cannot_review(self):
        self.assertFalse(can_review_approvals("staff_author"))

    def test_faculty_cannot_review(self):
        self.assertFalse(can_review_approvals("faculty"))

    def test_all_main_roles_can_view_published(self):
        for role in ["admin", "staff_author", "dept_reviewer", "faculty", "student"]:
            self.assertTrue(can_view_published_courses(role), f"{role} should see published courses")

    def test_integration_cannot_view_published(self):
        self.assertFalse(can_view_published_courses("integration"))


class TestSelfApprovalPrevention(unittest.TestCase):
    def test_cannot_approve_own(self):
        self.assertFalse(can_approve_own_submission(1, 1))

    def test_can_approve_others(self):
        self.assertTrue(can_approve_own_submission(1, 2))

    def test_different_ids(self):
        self.assertTrue(can_approve_own_submission(100, 200))


class TestRoleGuards(unittest.TestCase):
    def test_admin_passes_all_guards(self):
        for required in [["staff_author"], ["dept_reviewer"], ["faculty"]]:
            self.assertTrue(is_allowed_by_guard("admin", required))

    def test_staff_author_passes_own_guard(self):
        self.assertTrue(is_allowed_by_guard("staff_author", ["staff_author"]))

    def test_student_blocked_by_author_guard(self):
        self.assertFalse(is_allowed_by_guard("student", ["staff_author"]))

    def test_faculty_blocked_by_reviewer_guard(self):
        self.assertFalse(is_allowed_by_guard("faculty", ["dept_reviewer"]))


if __name__ == "__main__":
    unittest.main()
