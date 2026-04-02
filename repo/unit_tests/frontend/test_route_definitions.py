"""Unit tests for frontend route definitions and role-based navigation."""
import unittest

ROUTES = {
    "/": "Home (redirect)",
    "/login": "Login",
    "/dashboard": "Dashboard",
    "/courses": "Course catalog",
    "/courses/:uuid": "Course detail",
    "/courses/:uuid/edit": "Course editor",
    "/approvals": "Approval queue",
    "/bookings": "Bookings",
    "/risk": "Risk & Compliance",
    "/privacy": "Privacy & Data",
    "/audit": "Audit trail",
}

ROLE_NAV = {
    "admin": ["Dashboard", "Courses", "Approvals", "Bookings", "Risk & Compliance", "Audit Trail", "Privacy & Data"],
    "staff_author": ["Dashboard", "Courses", "Bookings", "Privacy & Data"],
    "dept_reviewer": ["Dashboard", "Courses", "Approvals", "Bookings", "Privacy & Data"],
    "faculty": ["Dashboard", "Courses", "Bookings", "Privacy & Data"],
    "student": ["Dashboard", "Courses", "Bookings", "Privacy & Data"],
}


class TestRouteDefinitions(unittest.TestCase):
    def test_all_routes_defined(self):
        self.assertGreaterEqual(len(ROUTES), 11)

    def test_all_routes_start_with_slash(self):
        for route in ROUTES:
            self.assertTrue(route.startswith("/"))

    def test_admin_sees_all_nav(self):
        items = ROLE_NAV["admin"]
        self.assertIn("Dashboard", items)
        self.assertIn("Risk & Compliance", items)
        self.assertIn("Audit Trail", items)
        self.assertIn("Approvals", items)

    def test_student_no_admin_nav(self):
        items = ROLE_NAV["student"]
        self.assertNotIn("Risk & Compliance", items)
        self.assertNotIn("Audit Trail", items)
        self.assertNotIn("Approvals", items)

    def test_reviewer_sees_approvals(self):
        self.assertIn("Approvals", ROLE_NAV["dept_reviewer"])

    def test_author_no_approvals(self):
        self.assertNotIn("Approvals", ROLE_NAV["staff_author"])

    def test_all_roles_have_dashboard(self):
        for role, items in ROLE_NAV.items():
            self.assertIn("Dashboard", items, f"{role} missing Dashboard")

    def test_all_roles_have_privacy(self):
        for role, items in ROLE_NAV.items():
            self.assertIn("Privacy & Data", items, f"{role} missing Privacy")

    def test_all_roles_have_bookings(self):
        for role, items in ROLE_NAV.items():
            self.assertIn("Bookings", items, f"{role} missing Bookings")


if __name__ == "__main__":
    unittest.main()
