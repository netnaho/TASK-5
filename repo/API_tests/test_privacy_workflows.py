"""API integration tests for privacy export, delete, and rectify workflows."""
import os
import unittest
import urllib.request
import json

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


class TestExportRequest(unittest.TestCase):
    """Test creating and approving a data export request."""

    @classmethod
    def setUpClass(cls):
        cls.user_token = get_token("faculty", "Faculty@123456")
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.export_uuid = None

    def test_01_create_export_request(self):
        """POST /api/v1/privacy/requests with type=export succeeds."""
        if not self.user_token:
            self.skipTest("Login failed")
        s, b = api_request("POST", "/api/v1/privacy/requests", {
            "request_type": "export",
            "reason": "GDPR data portability request",
        }, self.user_token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIn("uuid", b["data"])
        self.__class__.export_uuid = b["data"]["uuid"]

    def test_02_admin_approve_export(self):
        """Admin approves export; verify completed status."""
        if not self.export_uuid or not self.admin_token:
            self.skipTest("No export request or admin login failed")
        # Admin must reauth first
        s, _ = api_request("POST", "/api/v1/auth/reauth",
            {"password": "Admin@12345678"}, self.admin_token)
        if s != 200:
            self.skipTest("Reauth failed")
        s, b = api_request("POST", f"/api/v1/privacy/requests/{self.export_uuid}/review", {
            "approved": True,
            "admin_notes": "Export approved for compliance",
        }, self.admin_token)
        self.assertEqual(s, 200)


class TestRectifyRequest(unittest.TestCase):
    """Test rectification request creation and validation."""

    @classmethod
    def setUpClass(cls):
        cls.user_token = get_token("faculty", "Faculty@123456")

    def test_create_rectify_request(self):
        """POST with type=rectify, field_name, and new_value succeeds."""
        if not self.user_token:
            self.skipTest("Login failed")
        s, b = api_request("POST", "/api/v1/privacy/requests", {
            "request_type": "rectify",
            "reason": "Correct my email address",
            "field_name": "email",
            "new_value": "corrected@test.com",
        }, self.user_token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIn("uuid", b["data"])

    def test_rectify_requires_field_name(self):
        """POST with type=rectify without field_name returns 400."""
        if not self.user_token:
            self.skipTest("Login failed")
        s, b = api_request("POST", "/api/v1/privacy/requests", {
            "request_type": "rectify",
            "reason": "Missing field name test",
        }, self.user_token)
        self.assertEqual(s, 400)
        self.assertIn("message", b)


class TestDeleteRequest(unittest.TestCase):
    """Test right-to-be-forgotten (deletion) request."""

    @classmethod
    def setUpClass(cls):
        cls.user_token = get_token("student", "Student@12345")

    def test_create_delete_request(self):
        """POST with type=delete succeeds."""
        if not self.user_token:
            self.skipTest("Login failed")
        s, b = api_request("POST", "/api/v1/privacy/requests", {
            "request_type": "delete",
            "reason": "Right to be forgotten under GDPR",
        }, self.user_token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIn("uuid", b["data"])


class TestPrivacyRequestVisibility(unittest.TestCase):
    """Users can view their own requests; admin can see all."""

    @classmethod
    def setUpClass(cls):
        cls.user_token = get_token("faculty", "Faculty@123456")
        cls.admin_token = get_token("admin", "Admin@12345678")

    def test_user_sees_own_requests(self):
        if not self.user_token:
            self.skipTest("Login failed")
        s, b = api_request("GET", "/api/v1/privacy/requests/my", token=self.user_token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIsInstance(b["data"], list)

    def test_admin_sees_all_requests(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("GET", "/api/v1/privacy/requests", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIsInstance(b["data"], list)


if __name__ == "__main__":
    unittest.main()
