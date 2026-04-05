"""API integration tests for booking approval workflow."""
import os
import subprocess
import unittest
import urllib.request
import json
from datetime import datetime, timedelta

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8000")


def _reset_account_lockouts():
    subprocess.run(
        ["docker", "exec", "campuslearn-mysql", "mysql", "-ucampus", "-pcampus_pass",
         "campus_learn", "-e",
         "UPDATE users SET failed_login_count=0, locked_until=NULL; DELETE FROM ip_rate_limits;"],
        capture_output=True,
    )


_reset_account_lockouts()


def _accept_active_terms(token):
    """Accept the active term so booking creation is not blocked."""
    s, b = api_request("GET", "/api/v1/terms/active", token=token)
    if s == 200 and b.get("data"):
        term_uuid = b["data"]["uuid"]
        api_request("POST", f"/api/v1/terms/{term_uuid}/accept", token=token)


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


class TestBookingApproval(unittest.TestCase):
    """Tests for booking approval lifecycle based on resource.requires_approval."""

    @classmethod
    def setUpClass(cls):
        _reset_account_lockouts()
        cls.faculty_token = get_token("faculty", "Faculty@123456")
        cls.reviewer_token = get_token("reviewer", "Review@1234567")
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.student_token = get_token("student", "Student@12345")
        cls.approval_resource_uuid = None  # requires_approval = true (Clubhouse)
        cls.no_approval_resource_uuid = None  # requires_approval = false (Conference Room)

        # Accept active terms for booking creation
        if cls.faculty_token:
            _accept_active_terms(cls.faculty_token)
        if cls.student_token:
            _accept_active_terms(cls.student_token)

        # Reauth reviewer and admin (required for approve/reject endpoints)
        if cls.reviewer_token:
            api_request("POST", "/api/v1/auth/reauth", {"password": "Review@1234567"}, cls.reviewer_token)
        if cls.admin_token:
            api_request("POST", "/api/v1/auth/reauth", {"password": "Admin@12345678"}, cls.admin_token)

        # Find resources
        s, b = api_request("GET", "/api/v1/bookings/resources", token=cls.faculty_token)
        if s == 200:
            for r in b["data"]:
                if r["resource_type"] == "studio":
                    cls.approval_resource_uuid = r["uuid"]
                elif r["resource_type"] == "room" and cls.no_approval_resource_uuid is None:
                    cls.no_approval_resource_uuid = r["uuid"]

    def _create_booking(self, resource_uuid, token, day_offset=5, hour_start=10, hour_end=12):
        day = datetime.now() + timedelta(days=day_offset)
        start = day.replace(hour=hour_start, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S")
        end = day.replace(hour=hour_end, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S")
        return api_request("POST", "/api/v1/bookings", {
            "resource_uuid": resource_uuid,
            "title": f"Test Booking {day_offset}-{hour_start}",
            "start_time": start,
            "end_time": end,
        }, token)

    def test_01_approval_resource_returns_pending(self):
        """Booking a resource with requires_approval=true should return status=pending."""
        if not self.approval_resource_uuid:
            self.skipTest("No approval-required resource found")
        s, b = self._create_booking(self.approval_resource_uuid, self.faculty_token, day_offset=10, hour_start=10, hour_end=12)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")
        self.assertEqual(b["data"]["status"], "pending")
        self.__class__.pending_booking_uuid = b["data"]["uuid"]

    def test_02_no_approval_resource_returns_confirmed(self):
        """Booking a resource with requires_approval=false should auto-confirm."""
        if not self.no_approval_resource_uuid:
            self.skipTest("No non-approval resource found")
        s, b = self._create_booking(self.no_approval_resource_uuid, self.faculty_token, day_offset=11, hour_start=10, hour_end=11)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")
        self.assertEqual(b["data"]["status"], "confirmed")

    def test_03_approve_pending_booking(self):
        """Reviewer can approve a pending booking."""
        uuid = getattr(self.__class__, "pending_booking_uuid", None)
        if not uuid:
            self.skipTest("No pending booking")
        s, b = api_request("POST", f"/api/v1/bookings/{uuid}/approve", {"reason": None}, self.reviewer_token)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")

    def test_04_approve_non_pending_fails(self):
        """Approving an already-approved (confirmed) booking should fail."""
        uuid = getattr(self.__class__, "pending_booking_uuid", None)
        if not uuid:
            self.skipTest("No booking")
        s, b = api_request("POST", f"/api/v1/bookings/{uuid}/approve", {"reason": None}, self.reviewer_token)
        self.assertEqual(s, 400)

    def test_05_reject_pending_booking(self):
        """Reviewer can reject a pending booking."""
        if not self.approval_resource_uuid:
            self.skipTest("No approval-required resource")
        # Create a new pending booking to reject
        s, b = self._create_booking(self.approval_resource_uuid, self.faculty_token, day_offset=12, hour_start=14, hour_end=16)
        self.assertEqual(s, 200)
        reject_uuid = b["data"]["uuid"]
        self.assertEqual(b["data"]["status"], "pending")

        s, b = api_request("POST", f"/api/v1/bookings/{reject_uuid}/reject",
                           {"reason": "Room not available"}, self.reviewer_token)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")

    def test_06_student_cannot_approve(self):
        """Student role should be forbidden from approving bookings."""
        if not self.approval_resource_uuid:
            self.skipTest("No approval-required resource")
        # Create a pending booking
        s, b = self._create_booking(self.approval_resource_uuid, self.faculty_token, day_offset=13, hour_start=9, hour_end=11)
        if s != 200:
            self.skipTest("Could not create pending booking")
        uuid = b["data"]["uuid"]

        s, b = api_request("POST", f"/api/v1/bookings/{uuid}/approve", {"reason": None}, self.student_token)
        self.assertEqual(s, 403)

    def test_07_cancel_pending_booking(self):
        """Owner can cancel their own pending booking."""
        if not self.approval_resource_uuid:
            self.skipTest("No approval-required resource")
        # Create a pending booking
        s, b = self._create_booking(self.approval_resource_uuid, self.faculty_token, day_offset=14, hour_start=10, hour_end=12)
        if s != 200:
            self.skipTest("Could not create pending booking")
        uuid = b["data"]["uuid"]

        s, b = api_request("POST", f"/api/v1/bookings/{uuid}/cancel", token=self.faculty_token)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")

    def test_08_admin_can_approve(self):
        """Admin can also approve pending bookings."""
        if not self.approval_resource_uuid:
            self.skipTest("No approval-required resource")
        s, b = self._create_booking(self.approval_resource_uuid, self.faculty_token, day_offset=15, hour_start=10, hour_end=12)
        if s != 200:
            self.skipTest("Could not create pending booking")
        uuid = b["data"]["uuid"]

        s, b = api_request("POST", f"/api/v1/bookings/{uuid}/approve", {"reason": None}, self.admin_token)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")


if __name__ == "__main__":
    unittest.main()
