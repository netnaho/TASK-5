"""API integration tests for the two-step approval workflow."""
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


class TestApprovalWorkflow(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.author_token = get_token("author", "Author@1234567")
        cls.reviewer_token = get_token("reviewer", "Review@1234567")
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.course_uuid = None
        cls.approval_uuid = None

    def test_01_create_course_for_approval(self):
        if not self.author_token:
            self.skipTest("Author login failed")
        code = f"APR-{uuid_mod.uuid4().hex[:6].upper()}"
        s, b = api_request("POST", "/api/v1/courses", {
            "title": "Approval Test Course",
            "code": code,
        }, self.author_token)
        self.assertEqual(s, 200)
        self.__class__.course_uuid = b["data"]["uuid"]

    def test_02_submit_for_approval(self):
        if not self.course_uuid:
            self.skipTest("No course")
        future = (datetime.now() + timedelta(days=30)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request("POST", f"/api/v1/approvals/{self.course_uuid}/submit", {
            "release_notes": "Initial release",
            "effective_date": future,
        }, self.author_token)
        self.assertEqual(s, 200)
        self.__class__.approval_uuid = b["data"]["approval_uuid"]

    def test_03_get_approval(self):
        if not self.approval_uuid:
            self.skipTest("No approval")
        s, b = api_request("GET", f"/api/v1/approvals/{self.approval_uuid}", token=self.reviewer_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "pending_step1")
        self.assertEqual(len(b["data"]["steps"]), 2)

    def test_04_self_approval_prevented(self):
        """Author cannot approve their own submission."""
        if not self.approval_uuid:
            self.skipTest("No approval")
        s, b = api_request("POST", f"/api/v1/approvals/{self.approval_uuid}/review", {
            "approved": True,
            "comments": "Self-approving!",
        }, self.author_token)
        # Author is staff_author, not a reviewer, so should get 403
        self.assertEqual(s, 403)

    def test_05_step1_approve_by_reviewer(self):
        if not self.approval_uuid:
            self.skipTest("No approval")
        s, b = api_request("POST", f"/api/v1/approvals/{self.approval_uuid}/review", {
            "approved": True,
            "comments": "Looks good",
        }, self.reviewer_token)
        self.assertEqual(s, 200)

    def test_06_step2_approve_by_admin(self):
        if not self.approval_uuid:
            self.skipTest("No approval")
        s, b = api_request("POST", f"/api/v1/approvals/{self.approval_uuid}/review", {
            "approved": True,
            "comments": "Final approval",
        }, self.admin_token)
        self.assertEqual(s, 200)

    def test_07_verify_course_approved_scheduled(self):
        if not self.course_uuid:
            self.skipTest("No course")
        s, b = api_request("GET", f"/api/v1/courses/{self.course_uuid}", token=self.admin_token)
        self.assertEqual(s, 200)
        # Should be approved_scheduled since effective_date is in the future
        self.assertIn(b["data"]["status"], ["approved_scheduled", "published"])

    def test_08_approval_queue(self):
        s, b = api_request("GET", "/api/v1/approvals/queue", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertIsInstance(b["data"], list)

    def test_09_course_versions_exist(self):
        if not self.course_uuid:
            self.skipTest("No course")
        s, b = api_request("GET", f"/api/v1/courses/{self.course_uuid}/versions", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertGreaterEqual(len(b["data"]), 1)


class TestApprovalRejection(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.author_token = get_token("author", "Author@1234567")
        cls.reviewer_token = get_token("reviewer", "Review@1234567")

    def test_rejection_flow(self):
        if not self.author_token or not self.reviewer_token:
            self.skipTest("Login failed")

        # Create course
        code = f"REJ-{uuid_mod.uuid4().hex[:6].upper()}"
        s, b = api_request("POST", "/api/v1/courses", {"title": "Reject Course", "code": code}, self.author_token)
        if s != 200:
            self.skipTest("Could not create course")
        course_uuid = b["data"]["uuid"]

        # Submit
        future = (datetime.now() + timedelta(days=10)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request("POST", f"/api/v1/approvals/{course_uuid}/submit", {
            "release_notes": "Test release",
            "effective_date": future,
        }, self.author_token)
        if s != 200:
            self.skipTest("Could not submit")
        approval_uuid = b["data"]["approval_uuid"]

        # Reject at step 1
        s, b = api_request("POST", f"/api/v1/approvals/{approval_uuid}/review", {
            "approved": False,
            "comments": "Needs revision",
        }, self.reviewer_token)
        self.assertEqual(s, 200)

        # Verify course is rejected
        s, b = api_request("GET", f"/api/v1/courses/{course_uuid}", token=self.author_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "rejected")


if __name__ == "__main__":
    unittest.main()
