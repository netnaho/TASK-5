"""API integration tests for terms acceptance workflow."""
import os
import unittest
import urllib.request
import json
from datetime import datetime, timedelta

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8000")


def api_request(method, path, data=None, token=None):
    url = f"{BASE_URL}{path}"
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    body = json.dumps(data).encode() if data else None
    try:
        req = urllib.request.Request(url, data=body, headers=headers, method=method)
        with urllib.request.urlopen(req, timeout=10) as resp:
            return resp.status, json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        body = json.loads(e.read().decode()) if e.fp else {}
        return e.code, body
    except Exception as e:
        raise ConnectionError(f"Cannot reach {url}: {e}")


def get_token(username, password):
    s, b = api_request("POST", "/api/v1/auth/login", {"username": username, "password": password})
    return b["data"]["token"] if s == 200 else None


class TestTermsAcceptance(unittest.TestCase):
    """Tests for terms acceptance workflow and enforcement."""

    @classmethod
    def setUpClass(cls):
        cls.faculty_token = get_token("faculty", "Faculty@123456")
        cls.student_token = get_token("student", "Student@12345")
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.active_term_uuid = None

        # Get active term
        s, b = api_request("GET", "/api/v1/terms/active", token=cls.faculty_token)
        if s == 200 and b.get("data"):
            cls.active_term_uuid = b["data"]["uuid"]

    def test_01_accept_active_term(self):
        """User can accept the active term."""
        if not self.active_term_uuid:
            self.skipTest("No active term")
        s, b = api_request("POST", f"/api/v1/terms/{self.active_term_uuid}/accept", token=self.faculty_token)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")

    def test_02_accept_idempotent(self):
        """Accepting the same term twice should succeed (idempotent)."""
        if not self.active_term_uuid:
            self.skipTest("No active term")
        s, b = api_request("POST", f"/api/v1/terms/{self.active_term_uuid}/accept", token=self.faculty_token)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")

    def test_03_my_acceptances(self):
        """After accepting, the acceptance should appear in my-acceptances."""
        if not self.active_term_uuid:
            self.skipTest("No active term")
        s, b = api_request("GET", "/api/v1/terms/my-acceptances", token=self.faculty_token)
        self.assertEqual(s, 200)
        self.assertIsInstance(b["data"], list)
        self.assertGreater(len(b["data"]), 0, "Expected at least one acceptance")

    def test_04_accept_nonexistent_term(self):
        """Accepting a nonexistent term should return 404."""
        s, b = api_request("POST", "/api/v1/terms/00000000-0000-0000-0000-000000000000/accept", token=self.faculty_token)
        self.assertEqual(s, 404)

    def test_05_booking_after_acceptance(self):
        """After accepting terms, user should be able to create a booking."""
        if not self.active_term_uuid:
            self.skipTest("No active term")
        # Ensure student accepts terms first
        api_request("POST", f"/api/v1/terms/{self.active_term_uuid}/accept", token=self.student_token)

        # Get a room resource
        s, b = api_request("GET", "/api/v1/bookings/resources", token=self.student_token)
        if s != 200:
            self.skipTest("Cannot list resources")
        rooms = [r for r in b["data"] if r["resource_type"] == "room"]
        if not rooms:
            self.skipTest("No room resources")

        tomorrow = datetime.now() + timedelta(days=20)
        start = tomorrow.replace(hour=10, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S")
        end = tomorrow.replace(hour=11, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S")
        s, b = api_request("POST", "/api/v1/bookings", {
            "resource_uuid": rooms[0]["uuid"],
            "title": "Post-Terms Booking",
            "start_time": start,
            "end_time": end,
        }, self.student_token)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")

    def test_06_unauthenticated_accept_fails(self):
        """Accepting without auth should return 401."""
        if not self.active_term_uuid:
            self.skipTest("No active term")
        s, b = api_request("POST", f"/api/v1/terms/{self.active_term_uuid}/accept")
        self.assertEqual(s, 401)


if __name__ == "__main__":
    unittest.main()
