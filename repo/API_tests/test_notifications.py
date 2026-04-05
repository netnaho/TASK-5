"""API integration tests for the persistent in-app notification center."""
import os
import unittest
import urllib.request
import json
import uuid as uuid_mod
from datetime import datetime, timedelta

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8000")


def api_request(method: str, path: str, data: dict = None, token: str = None) -> tuple[int, dict]:
    url = f"{BASE_URL}{path}"
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    body_bytes = json.dumps(data).encode() if data else None
    try:
        req = urllib.request.Request(url, data=body_bytes, headers=headers, method=method)
        with urllib.request.urlopen(req, timeout=10) as resp:
            return resp.status, json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        body = json.loads(e.read().decode()) if e.fp else {}
        return e.code, body
    except Exception as e:
        raise ConnectionError(f"Cannot reach {url}: {e}")


def get_token(username: str, password: str) -> str:
    status, body = api_request("POST", "/api/v1/auth/login", {"username": username, "password": password})
    return body["data"]["token"] if status == 200 else None


class TestNotificationEndpoints(unittest.TestCase):
    """Basic CRUD operations on the notifications endpoints."""

    @classmethod
    def setUpClass(cls):
        cls.author_token = get_token("author", "Author@1234567")
        cls.admin_token = get_token("admin", "Admin@12345678")

    def test_01_list_returns_200(self):
        if not self.author_token:
            self.skipTest("Author login failed")
        s, b = api_request("GET", "/api/v1/notifications/", token=self.author_token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIsInstance(b["data"], list)

    def test_02_unread_count_returns_integer(self):
        if not self.author_token:
            self.skipTest("Author login failed")
        s, b = api_request("GET", "/api/v1/notifications/unread-count", token=self.author_token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIn("count", b["data"])
        self.assertIsInstance(b["data"]["count"], int)

    def test_03_mark_all_read_returns_200(self):
        if not self.author_token:
            self.skipTest("Author login failed")
        s, b = api_request("PUT", "/api/v1/notifications/read-all", token=self.author_token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        # Unread count should now be 0
        s2, b2 = api_request("GET", "/api/v1/notifications/unread-count", token=self.author_token)
        self.assertEqual(s2, 200)
        self.assertEqual(b2["data"]["count"], 0)

    def test_04_unauthenticated_returns_401(self):
        s, _ = api_request("GET", "/api/v1/notifications/")
        self.assertEqual(s, 401)

    def test_05_unread_count_unauthenticated_returns_401(self):
        s, _ = api_request("GET", "/api/v1/notifications/unread-count")
        self.assertEqual(s, 401)


class TestNotificationTriggeredByApproval(unittest.TestCase):
    """Verify that approval workflow events generate notifications."""

    @classmethod
    def setUpClass(cls):
        cls.author_token = get_token("author", "Author@1234567")
        cls.reviewer_token = get_token("reviewer", "Review@1234567")
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.course_uuid = None
        cls.approval_uuid = None

        if not cls.author_token:
            return

        # Create a course
        code = f"NOTIF-{uuid_mod.uuid4().hex[:6].upper()}"
        s, b = api_request("POST", "/api/v1/courses", {
            "title": "Notification Test Course",
            "code": code,
        }, cls.author_token)
        if s == 200:
            cls.course_uuid = b["data"]["uuid"]

        if not cls.course_uuid:
            return

        # Submit for approval (past effective date so it executes immediately on step2 approval)
        past_date = (datetime.now() - timedelta(days=1)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request(
            "POST", f"/api/v1/approvals/{cls.course_uuid}/submit",
            {"release_notes": "Notification test", "effective_date": past_date},
            cls.author_token,
        )
        if s == 200:
            cls.approval_uuid = b["data"]["approval_uuid"]

    def test_06_reviewer_receives_notification_on_submission(self):
        if not self.reviewer_token or not self.approval_uuid:
            self.skipTest("Prerequisites not met")
        s, b = api_request("GET", "/api/v1/notifications/", token=self.reviewer_token)
        self.assertEqual(s, 200)
        notifications = b["data"]
        approval_notifs = [n for n in notifications if n["notification_type"] == "approval"]
        self.assertTrue(len(approval_notifs) > 0, "Reviewer should have at least one approval notification")

    def test_07_author_receives_notification_on_rejection(self):
        if not self.reviewer_token or not self.admin_token or not self.author_token:
            self.skipTest("Prerequisites not met")

        # Clear author notifications first
        api_request("PUT", "/api/v1/notifications/read-all", token=self.author_token)

        # Create a fresh course and submit
        code = f"REJ-{uuid_mod.uuid4().hex[:6].upper()}"
        s, b = api_request("POST", "/api/v1/courses", {
            "title": "Rejection Notification Test",
            "code": code,
        }, self.author_token)
        if s != 200:
            self.skipTest("Could not create course")
        course_uuid = b["data"]["uuid"]

        past_date = (datetime.now() - timedelta(days=1)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request(
            "POST", f"/api/v1/approvals/{course_uuid}/submit",
            {"release_notes": "Rejection test", "effective_date": past_date},
            self.author_token,
        )
        if s != 200:
            self.skipTest("Could not submit for approval")
        approval_uuid = b["data"]["approval_uuid"]

        # Reviewer rejects step 1
        s, b = api_request(
            "POST", f"/api/v1/approvals/{approval_uuid}/review",
            {"approved": False, "comments": "Rejected in test"},
            self.reviewer_token,
        )
        self.assertEqual(s, 200)

        # Author should have a rejection notification
        s, b = api_request("GET", "/api/v1/notifications/", token=self.author_token)
        self.assertEqual(s, 200)
        rejection_notifs = [
            n for n in b["data"]
            if "reject" in n["title"].lower() or "rejected" in n["message"].lower()
        ]
        self.assertTrue(len(rejection_notifs) > 0, "Author should receive rejection notification")

    def test_08_author_receives_notification_on_full_approval(self):
        if not self.reviewer_token or not self.admin_token or not self.author_token:
            self.skipTest("Prerequisites not met")

        api_request("PUT", "/api/v1/notifications/read-all", token=self.author_token)
        api_request("POST", "/api/v1/auth/reauth", {"password": "Admin@12345678"}, self.admin_token)

        # Create a fresh course and submit
        code = f"APP-{uuid_mod.uuid4().hex[:6].upper()}"
        s, b = api_request("POST", "/api/v1/courses", {
            "title": "Approval Notification Test",
            "code": code,
        }, self.author_token)
        if s != 200:
            self.skipTest("Could not create course")
        course_uuid = b["data"]["uuid"]

        past_date = (datetime.now() - timedelta(days=1)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request(
            "POST", f"/api/v1/approvals/{course_uuid}/submit",
            {"release_notes": "Full approval test", "effective_date": past_date},
            self.author_token,
        )
        if s != 200:
            self.skipTest("Could not submit for approval")
        approval_uuid = b["data"]["approval_uuid"]

        # Step 1: reviewer approves
        s, _ = api_request(
            "POST", f"/api/v1/approvals/{approval_uuid}/review",
            {"approved": True, "comments": "Step 1 approved"},
            self.reviewer_token,
        )
        if s != 200:
            self.skipTest("Step 1 approval failed")

        # Step 2: admin approves
        s, _ = api_request(
            "POST", f"/api/v1/approvals/{approval_uuid}/review",
            {"approved": True, "comments": "Step 2 approved"},
            self.admin_token,
        )
        self.assertEqual(s, 200)

        # Author should have an approval notification
        s, b = api_request("GET", "/api/v1/notifications/", token=self.author_token)
        self.assertEqual(s, 200)
        approved_notifs = [
            n for n in b["data"]
            if "approved" in n["title"].lower() or "approved" in n["message"].lower()
        ]
        self.assertTrue(len(approved_notifs) > 0, "Author should receive approval notification")


class TestNotificationTriggeredByBooking(unittest.TestCase):
    """Verify that booking events generate notifications."""

    @classmethod
    def setUpClass(cls):
        cls.faculty_token = get_token("faculty", "Faculty@123456")
        cls.resource_uuid = None

        if not cls.faculty_token:
            return

        # Get an available resource
        s, b = api_request("GET", "/api/v1/bookings/resources", token=cls.faculty_token)
        if s == 200 and b["data"]:
            cls.resource_uuid = b["data"][0]["uuid"]

    def test_09_booking_confirmation_notification(self):
        if not self.faculty_token or not self.resource_uuid:
            self.skipTest("Prerequisites not met")

        # Clear notifications
        api_request("PUT", "/api/v1/notifications/read-all", token=self.faculty_token)

        # Create a booking
        tomorrow = (datetime.now() + timedelta(days=1)).strftime("%Y-%m-%d")
        s, b = api_request(
            "POST", "/api/v1/bookings",
            {
                "resource_uuid": self.resource_uuid,
                "title": "Notification Test Booking",
                "start_time": f"{tomorrow} 10:00:00",
                "end_time": f"{tomorrow} 11:00:00",
            },
            self.faculty_token,
        )
        if s != 200:
            self.skipTest(f"Could not create booking: {b}")

        # Check for booking confirmation notification
        s, b = api_request("GET", "/api/v1/notifications/", token=self.faculty_token)
        self.assertEqual(s, 200)
        booking_notifs = [
            n for n in b["data"]
            if n["notification_type"] == "booking"
               and "confirmed" in n["title"].lower()
        ]
        self.assertTrue(len(booking_notifs) > 0, "User should receive booking confirmation notification")

    def test_10_mark_single_notification_read(self):
        if not self.faculty_token:
            self.skipTest("Faculty login failed")

        # Get notifications and mark first unread one as read
        s, b = api_request("GET", "/api/v1/notifications/", token=self.faculty_token)
        self.assertEqual(s, 200)
        unread = [n for n in b["data"] if not n["is_read"]]
        if not unread:
            self.skipTest("No unread notifications to test")

        uuid = unread[0]["uuid"]
        s, b = api_request("PUT", f"/api/v1/notifications/{uuid}/read", token=self.faculty_token)
        self.assertEqual(s, 200)

        # Confirm it's now read
        s, b = api_request("GET", "/api/v1/notifications/", token=self.faculty_token)
        notif = next((n for n in b["data"] if n["uuid"] == uuid), None)
        self.assertIsNotNone(notif)
        self.assertTrue(notif["is_read"])


if __name__ == "__main__":
    unittest.main()
