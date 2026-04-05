"""API integration tests for re-authentication enforcement on sensitive endpoints."""
import os
import unittest
import urllib.request
import json
import uuid as uuid_mod
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


class TestApprovalReviewReauthEnforcement(unittest.TestCase):
    """Approval review endpoint requires ReauthReviewerGuard."""

    @classmethod
    def setUpClass(cls):
        # Fresh tokens -- no reauth performed
        cls.reviewer_token = get_token("reviewer", "Review@1234567")
        cls.author_token = get_token("author", "Author@1234567")
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.approval_uuid = None

        # Author creates a course and submits for approval so we have a valid UUID
        if cls.author_token:
            code = f"RAE-{uuid_mod.uuid4().hex[:6].upper()}"
            s, b = api_request("POST", "/api/v1/courses",
                {"title": "Reauth Enforcement Test", "code": code}, cls.author_token)
            if s == 200:
                course_uuid = b["data"]["uuid"]
                future = (datetime.now() + timedelta(days=30)).strftime("%m/%d/%Y %I:%M %p")
                s, b = api_request("POST", f"/api/v1/approvals/{course_uuid}/submit",
                    {"release_notes": "Reauth test", "effective_date": future}, cls.author_token)
                if s == 200:
                    cls.approval_uuid = b["data"]["approval_uuid"]

    def test_approval_review_requires_reauth(self):
        """Reviewer trying POST /api/v1/approvals/<uuid>/review without reauth gets 403."""
        if not self.approval_uuid or not self.reviewer_token:
            self.skipTest("Setup failed")
        s, b = api_request("POST", f"/api/v1/approvals/{self.approval_uuid}/review",
            {"approved": True, "comments": "Trying without reauth"}, self.reviewer_token)
        self.assertEqual(s, 403)
        self.assertIn("reauth", b.get("message", "").lower())

    def test_approval_review_after_reauth(self):
        """Reviewer who has performed reauth can review an approval."""
        if not self.approval_uuid or not self.reviewer_token:
            self.skipTest("Setup failed")
        # Perform reauth
        s, _ = api_request("POST", "/api/v1/auth/reauth",
            {"password": "Review@1234567"}, self.reviewer_token)
        if s != 200:
            self.skipTest("Reauth failed")
        # Now the review should succeed (not 403)
        s, b = api_request("POST", f"/api/v1/approvals/{self.approval_uuid}/review",
            {"approved": True, "comments": "After reauth"}, self.reviewer_token)
        self.assertEqual(s, 200)


class TestBookingApproveRejectReauthEnforcement(unittest.TestCase):
    """Booking approve/reject endpoints require ReauthReviewerGuard."""

    @classmethod
    def setUpClass(cls):
        # Fresh tokens -- no reauth
        cls.reviewer_token = get_token("reviewer", "Review@1234567")
        cls.faculty_token = get_token("faculty", "Faculty@123456")
        cls.booking_uuid = None

        # Faculty creates a booking to get a valid UUID
        if cls.faculty_token:
            s, b = api_request("GET", "/api/v1/bookings/resources", token=cls.faculty_token)
            if s == 200:
                rooms = [r for r in b["data"] if r["resource_type"] == "room"]
                if rooms:
                    resource_uuid = rooms[0]["uuid"]
                    tomorrow = datetime.now() + timedelta(days=5)
                    start = tomorrow.replace(hour=8, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S")
                    end = tomorrow.replace(hour=9, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S")
                    s, b = api_request("POST", "/api/v1/bookings", {
                        "resource_uuid": resource_uuid,
                        "title": "Reauth Booking Test",
                        "start_time": start,
                        "end_time": end,
                    }, cls.faculty_token)
                    if s == 200:
                        cls.booking_uuid = b["data"]["uuid"]

    def test_booking_approve_requires_reauth(self):
        """Reviewer trying POST /api/v1/bookings/<uuid>/approve without reauth gets 403."""
        if not self.booking_uuid or not self.reviewer_token:
            self.skipTest("Setup failed")
        s, b = api_request("POST", f"/api/v1/bookings/{self.booking_uuid}/approve",
            {}, self.reviewer_token)
        self.assertEqual(s, 403)
        self.assertIn("reauth", b.get("message", "").lower())

    def test_booking_reject_requires_reauth(self):
        """Reviewer trying POST /api/v1/bookings/<uuid>/reject without reauth gets 403."""
        if not self.booking_uuid or not self.reviewer_token:
            self.skipTest("Setup failed")
        s, b = api_request("POST", f"/api/v1/bookings/{self.booking_uuid}/reject",
            {"reason": "Test rejection"}, self.reviewer_token)
        self.assertEqual(s, 403)
        self.assertIn("reauth", b.get("message", "").lower())


class TestAuditAccessReauthEnforcement(unittest.TestCase):
    """Audit log endpoint requires ReauthAdminGuard."""

    @classmethod
    def setUpClass(cls):
        # Fresh admin token -- no reauth
        cls.admin_token = get_token("admin", "Admin@12345678")

    def test_audit_access_requires_reauth(self):
        """Admin trying GET /api/v1/audit without reauth gets 403."""
        if not self.admin_token:
            self.skipTest("Login failed")
        s, b = api_request("GET", "/api/v1/audit", token=self.admin_token)
        self.assertEqual(s, 403)
        self.assertIn("reauth", b.get("message", "").lower())

    def test_audit_access_after_reauth(self):
        """Admin who has reauthenticated can access audit logs."""
        if not self.admin_token:
            self.skipTest("Login failed")
        s, _ = api_request("POST", "/api/v1/auth/reauth",
            {"password": "Admin@12345678"}, self.admin_token)
        if s != 200:
            self.skipTest("Reauth failed")
        s, b = api_request("GET", "/api/v1/audit", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIsInstance(b["data"], list)


if __name__ == "__main__":
    unittest.main()
