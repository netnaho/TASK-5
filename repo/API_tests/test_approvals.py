"""API integration tests for the two-step approval workflow."""
import os
import subprocess
import unittest
import urllib.request
import json
import uuid as uuid_mod
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
        # Reauth reviewer and admin for approval actions
        if cls.reviewer_token:
            api_request("POST", "/api/v1/auth/reauth", {"password": "Review@1234567"}, cls.reviewer_token)
        if cls.admin_token:
            api_request("POST", "/api/v1/auth/reauth", {"password": "Admin@12345678"}, cls.admin_token)

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
        # Reauth reviewer for review actions
        if cls.reviewer_token:
            api_request("POST", "/api/v1/auth/reauth", {"password": "Review@1234567"}, cls.reviewer_token)

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


class TestApprovalVisibility(unittest.TestCase):
    """Only the submitter, admins, and relevant dept reviewers may read an approval by UUID."""

    @classmethod
    def setUpClass(cls):
        cls.admin_token   = get_token("admin",   "Admin@12345678")
        cls.author_token  = get_token("author",  "Author@1234567")
        cls.student_token = get_token("student", "Student@12345")
        cls.faculty_token = get_token("faculty", "Faculty@123456")

        # Author creates a fresh course and submits it for approval
        cls.approval_uuid = None
        if not cls.author_token:
            return
        code = f"APV-{uuid_mod.uuid4().hex[:5].upper()}"
        s, b = api_request("POST", "/api/v1/courses",
            {"title": "Approval Visibility Course", "code": code}, cls.author_token)
        if s != 200:
            return
        course_uuid = b["data"]["uuid"]
        future = (datetime.now() + timedelta(days=30)).strftime("%m/%d/%Y 09:00 AM")
        s, b = api_request("POST", f"/api/v1/approvals/{course_uuid}/submit",
            {"release_notes": "Visibility test submission", "effective_date": future},
            cls.author_token)
        if s == 200:
            cls.approval_uuid = b["data"]["approval_uuid"]

    def test_student_cannot_view_approval(self):
        if not self.approval_uuid or not self.student_token:
            self.skipTest("Setup failed")
        s, _ = api_request("GET", f"/api/v1/approvals/{self.approval_uuid}",
            token=self.student_token)
        self.assertEqual(s, 403)

    def test_faculty_cannot_view_approval(self):
        if not self.approval_uuid or not self.faculty_token:
            self.skipTest("Setup failed")
        s, _ = api_request("GET", f"/api/v1/approvals/{self.approval_uuid}",
            token=self.faculty_token)
        self.assertEqual(s, 403)

    def test_author_can_view_own_approval(self):
        if not self.approval_uuid or not self.author_token:
            self.skipTest("Setup failed")
        s, b = api_request("GET", f"/api/v1/approvals/{self.approval_uuid}",
            token=self.author_token)
        self.assertEqual(s, 200)
        self.assertIn("uuid", b["data"])

    def test_admin_can_view_any_approval(self):
        if not self.approval_uuid or not self.admin_token:
            self.skipTest("Setup failed")
        s, b = api_request("GET", f"/api/v1/approvals/{self.approval_uuid}",
            token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertIn("uuid", b["data"])


def create_and_publish_course(author_token, reviewer_token, admin_token, code_prefix="PUB") -> str | None:
    """Helper: create a course and run it through the full publish approval flow.
    Returns the course_uuid if published successfully, else None."""
    code = f"{code_prefix}-{uuid_mod.uuid4().hex[:5].upper()}"
    s, b = api_request("POST", "/api/v1/courses",
        {"title": f"Published Course {code}", "code": code}, author_token)
    if s != 200:
        return None
    course_uuid = b["data"]["uuid"]

    # Submit with a past effective_date so admin approval triggers immediate publish
    past = (datetime.now() - timedelta(hours=1)).strftime("%m/%d/%Y %I:%M %p")
    s, b = api_request("POST", f"/api/v1/approvals/{course_uuid}/submit",
        {"release_notes": "Test publish", "effective_date": past}, author_token)
    if s != 200:
        return None
    pub_approval_uuid = b["data"]["approval_uuid"]

    # Reviewer approves step 1
    s, _ = api_request("POST", f"/api/v1/approvals/{pub_approval_uuid}/review",
        {"approved": True, "comments": "Step 1 OK"}, reviewer_token)
    if s != 200:
        return None

    # Admin approves step 2 → immediate publish
    s, _ = api_request("POST", f"/api/v1/approvals/{pub_approval_uuid}/review",
        {"approved": True, "comments": "Step 2 OK"}, admin_token)
    if s != 200:
        return None

    return course_uuid


class TestUnpublishApprovalFlow(unittest.TestCase):
    """End-to-end two-step unpublish approval with scheduled effective date."""

    @classmethod
    def setUpClass(cls):
        cls.admin_token    = get_token("admin",    "Admin@12345678")
        cls.author_token   = get_token("author",   "Author@1234567")
        cls.reviewer_token = get_token("reviewer", "Review@1234567")
        cls.course_uuid    = None
        cls.unpublish_uuid = None

        if not (cls.admin_token and cls.author_token and cls.reviewer_token):
            return

        # Reauth reviewer and admin for review actions
        api_request("POST", "/api/v1/auth/reauth", {"password": "Review@1234567"}, cls.reviewer_token)
        api_request("POST", "/api/v1/auth/reauth", {"password": "Admin@12345678"}, cls.admin_token)

        cls.course_uuid = create_and_publish_course(
            cls.author_token, cls.reviewer_token, cls.admin_token, code_prefix="UNPS"
        )

    def test_01_published_course_exists(self):
        if not self.course_uuid:
            self.skipTest("Setup failed — course not published")
        s, b = api_request("GET", f"/api/v1/courses/{self.course_uuid}", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "published")

    def test_02_cannot_unpublish_non_published_course(self):
        """Draft course must be rejected by the unpublish endpoint."""
        code = f"DFT-{uuid_mod.uuid4().hex[:5].upper()}"
        s, b = api_request("POST", "/api/v1/courses",
            {"title": "Draft Only", "code": code}, self.author_token)
        if s != 200:
            self.skipTest("Could not create draft")
        draft_uuid = b["data"]["uuid"]
        future = (datetime.now() + timedelta(days=7)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request("POST", f"/api/v1/approvals/{draft_uuid}/unpublish",
            {"release_notes": "Should fail", "effective_date": future}, self.author_token)
        self.assertEqual(s, 400)

    def test_03_submit_unpublish_request(self):
        if not self.course_uuid:
            self.skipTest("Setup failed")
        future = (datetime.now() + timedelta(days=7)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request("POST", f"/api/v1/approvals/{self.course_uuid}/unpublish",
            {"release_notes": "Deprecating this course", "effective_date": future}, self.author_token)
        self.assertEqual(s, 200)
        self.assertIn("approval_uuid", b["data"])
        self.__class__.unpublish_uuid = b["data"]["approval_uuid"]

    def test_04_course_status_is_pending_unpublish(self):
        if not self.course_uuid:
            self.skipTest("Setup failed")
        s, b = api_request("GET", f"/api/v1/courses/{self.course_uuid}", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "pending_unpublish")

    def test_05_duplicate_unpublish_blocked(self):
        if not self.course_uuid or not self.unpublish_uuid:
            self.skipTest("Setup failed")
        future = (datetime.now() + timedelta(days=7)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request("POST", f"/api/v1/approvals/{self.course_uuid}/unpublish",
            {"release_notes": "Duplicate", "effective_date": future}, self.author_token)
        self.assertEqual(s, 400)

    def test_06_unpublish_appears_in_reviewer_queue(self):
        if not self.unpublish_uuid:
            self.skipTest("Setup failed")
        s, b = api_request("GET", "/api/v1/approvals/queue", token=self.reviewer_token)
        self.assertEqual(s, 200)
        uuids = [item["approval"]["uuid"] for item in b["data"]]
        self.assertIn(self.unpublish_uuid, uuids)

    def test_07_self_approval_prevented(self):
        if not self.unpublish_uuid:
            self.skipTest("Setup failed")
        # author tries to approve their own unpublish request
        s, _ = api_request("POST", f"/api/v1/approvals/{self.unpublish_uuid}/review",
            {"approved": True, "comments": "Self-approve"}, self.author_token)
        self.assertEqual(s, 403)

    def test_08_reviewer_approves_step1(self):
        if not self.unpublish_uuid:
            self.skipTest("Setup failed")
        s, _ = api_request("POST", f"/api/v1/approvals/{self.unpublish_uuid}/review",
            {"approved": True, "comments": "Step 1 approved"}, self.reviewer_token)
        self.assertEqual(s, 200)

    def test_09_admin_approves_step2_scheduled(self):
        if not self.unpublish_uuid:
            self.skipTest("Setup failed")
        s, _ = api_request("POST", f"/api/v1/approvals/{self.unpublish_uuid}/review",
            {"approved": True, "comments": "Unpublish approved, scheduled"}, self.admin_token)
        self.assertEqual(s, 200)

    def test_10_approval_status_is_approved_scheduled(self):
        if not self.unpublish_uuid:
            self.skipTest("Setup failed")
        s, b = api_request("GET", f"/api/v1/approvals/{self.unpublish_uuid}", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "approved_scheduled")
        self.assertEqual(b["data"]["entity_type"], "course_unpublish")

    def test_11_course_stays_pending_unpublish_until_scheduled_date(self):
        if not self.course_uuid:
            self.skipTest("Setup failed")
        s, b = api_request("GET", f"/api/v1/courses/{self.course_uuid}", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "pending_unpublish")

    def test_12_missing_release_notes_rejected(self):
        future = (datetime.now() + timedelta(days=7)).strftime("%m/%d/%Y %I:%M %p")
        # Use a dummy UUID — validation happens before DB lookup
        s, b = api_request("POST", "/api/v1/approvals/00000000-0000-0000-0000-000000000000/unpublish",
            {"release_notes": "", "effective_date": future}, self.author_token)
        self.assertEqual(s, 400)

    def test_13_invalid_effective_date_format_rejected(self):
        if not self.course_uuid:
            self.skipTest("Setup failed")
        s, b = api_request("POST", f"/api/v1/approvals/{self.course_uuid}/unpublish",
            {"release_notes": "Test", "effective_date": "not-a-date"}, self.author_token)
        self.assertEqual(s, 400)


class TestUnpublishImmediateFlow(unittest.TestCase):
    """Immediate unpublish when effective_date is in the past."""

    @classmethod
    def setUpClass(cls):
        cls.admin_token    = get_token("admin",    "Admin@12345678")
        cls.author_token   = get_token("author",   "Author@1234567")
        cls.reviewer_token = get_token("reviewer", "Review@1234567")
        cls.course_uuid    = None
        cls.unpublish_uuid = None

        if not (cls.admin_token and cls.author_token and cls.reviewer_token):
            return

        # Reauth reviewer and admin for review actions
        api_request("POST", "/api/v1/auth/reauth", {"password": "Review@1234567"}, cls.reviewer_token)
        api_request("POST", "/api/v1/auth/reauth", {"password": "Admin@12345678"}, cls.admin_token)

        cls.course_uuid = create_and_publish_course(
            cls.author_token, cls.reviewer_token, cls.admin_token, code_prefix="UNPI"
        )
        if not cls.course_uuid:
            return

        # Submit unpublish with a past effective_date → should execute immediately on final approval
        past = (datetime.now() - timedelta(hours=1)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request("POST", f"/api/v1/approvals/{cls.course_uuid}/unpublish",
            {"release_notes": "Immediate unpublish", "effective_date": past}, cls.author_token)
        if s == 200:
            cls.unpublish_uuid = b["data"]["approval_uuid"]

    def test_01_reviewer_approves_step1(self):
        if not self.unpublish_uuid:
            self.skipTest("Setup failed")
        s, _ = api_request("POST", f"/api/v1/approvals/{self.unpublish_uuid}/review",
            {"approved": True}, self.reviewer_token)
        self.assertEqual(s, 200)

    def test_02_admin_approves_step2_immediate(self):
        if not self.unpublish_uuid:
            self.skipTest("Setup failed")
        s, _ = api_request("POST", f"/api/v1/approvals/{self.unpublish_uuid}/review",
            {"approved": True}, self.admin_token)
        self.assertEqual(s, 200)

    def test_03_course_is_immediately_unpublished(self):
        if not self.course_uuid:
            self.skipTest("Setup failed")
        s, b = api_request("GET", f"/api/v1/courses/{self.course_uuid}", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "unpublished")

    def test_04_approval_status_is_approved(self):
        if not self.unpublish_uuid:
            self.skipTest("Setup failed")
        s, b = api_request("GET", f"/api/v1/approvals/{self.unpublish_uuid}", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "approved")


class TestUnpublishRejection(unittest.TestCase):
    """Rejection at step 1 or 2 restores course to 'published'."""

    @classmethod
    def setUpClass(cls):
        cls.admin_token    = get_token("admin",    "Admin@12345678")
        cls.author_token   = get_token("author",   "Author@1234567")
        cls.reviewer_token = get_token("reviewer", "Review@1234567")
        cls.course_uuid    = None
        cls.unpublish_uuid = None

        if not (cls.admin_token and cls.author_token and cls.reviewer_token):
            return

        # Reauth reviewer and admin for review actions
        api_request("POST", "/api/v1/auth/reauth", {"password": "Review@1234567"}, cls.reviewer_token)
        api_request("POST", "/api/v1/auth/reauth", {"password": "Admin@12345678"}, cls.admin_token)

        cls.course_uuid = create_and_publish_course(
            cls.author_token, cls.reviewer_token, cls.admin_token, code_prefix="UNPR"
        )
        if not cls.course_uuid:
            return

        future = (datetime.now() + timedelta(days=7)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request("POST", f"/api/v1/approvals/{cls.course_uuid}/unpublish",
            {"release_notes": "Rejection test", "effective_date": future}, cls.author_token)
        if s == 200:
            cls.unpublish_uuid = b["data"]["approval_uuid"]

    def test_01_reviewer_rejects_step1(self):
        if not self.unpublish_uuid:
            self.skipTest("Setup failed")
        s, _ = api_request("POST", f"/api/v1/approvals/{self.unpublish_uuid}/review",
            {"approved": False, "comments": "Unnecessary unpublish"}, self.reviewer_token)
        self.assertEqual(s, 200)

    def test_02_approval_status_is_rejected(self):
        if not self.unpublish_uuid:
            self.skipTest("Setup failed")
        s, b = api_request("GET", f"/api/v1/approvals/{self.unpublish_uuid}", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "rejected")

    def test_03_course_restored_to_published(self):
        if not self.course_uuid:
            self.skipTest("Setup failed")
        s, b = api_request("GET", f"/api/v1/courses/{self.course_uuid}", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "published")

    def test_04_can_resubmit_unpublish_after_rejection(self):
        """Once rejected, a fresh unpublish request can be submitted."""
        if not self.course_uuid:
            self.skipTest("Setup failed")
        future = (datetime.now() + timedelta(days=14)).strftime("%m/%d/%Y %I:%M %p")
        s, b = api_request("POST", f"/api/v1/approvals/{self.course_uuid}/unpublish",
            {"release_notes": "Second attempt", "effective_date": future}, self.author_token)
        self.assertEqual(s, 200)


if __name__ == "__main__":
    unittest.main()
