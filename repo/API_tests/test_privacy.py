"""API integration tests for privacy/sensitive data workflows."""
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


class TestDataExportFlow(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.user_token = get_token("faculty", "Faculty@123456")
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.request_uuid = None
        # Reauth admin so the review endpoint (now reauth-guarded) passes
        if cls.admin_token:
            api_request("POST", "/api/v1/auth/reauth",
                {"password": "Admin@12345678"}, cls.admin_token)

    def test_01_create_export_request(self):
        if not self.user_token:
            self.skipTest("Login failed")
        s, b = api_request("POST", "/api/v1/privacy/requests", {
            "request_type": "export",
            "reason": "I want a copy of my data",
        }, self.user_token)
        self.assertEqual(s, 200)
        self.__class__.request_uuid = b["data"]["uuid"]

    def test_02_view_my_requests(self):
        if not self.user_token:
            self.skipTest("Login failed")
        s, b = api_request("GET", "/api/v1/privacy/requests/my", token=self.user_token)
        self.assertEqual(s, 200)
        self.assertGreater(len(b["data"]), 0)

    def test_03_admin_sees_pending_requests(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("GET", "/api/v1/privacy/requests", token=self.admin_token)
        self.assertEqual(s, 200)

    def test_04_admin_approves_export(self):
        if not self.request_uuid or not self.admin_token:
            self.skipTest("No request or admin")
        s, b = api_request("POST", f"/api/v1/privacy/requests/{self.request_uuid}/review", {
            "approved": True,
            "admin_notes": "Approved for GDPR compliance",
        }, self.admin_token)
        self.assertEqual(s, 200)


class TestDataDeletionFlow(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.user_token = get_token("student", "Student@12345")
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.request_uuid = None
        # Reauth admin so the review endpoint (now reauth-guarded) passes
        if cls.admin_token:
            api_request("POST", "/api/v1/auth/reauth",
                {"password": "Admin@12345678"}, cls.admin_token)

    def test_01_create_deletion_request(self):
        if not self.user_token:
            self.skipTest("Login failed")
        s, b = api_request("POST", "/api/v1/privacy/requests", {
            "request_type": "delete",
            "reason": "Right to be forgotten",
        }, self.user_token)
        self.assertEqual(s, 200)
        self.__class__.request_uuid = b["data"]["uuid"]

    def test_02_admin_reviews_deletion(self):
        """Admin rejects the deletion to avoid destroying the student account for other tests."""
        if not self.request_uuid or not self.admin_token:
            self.skipTest("No request or admin")
        s, b = api_request("POST", f"/api/v1/privacy/requests/{self.request_uuid}/review", {
            "approved": False,
            "admin_notes": "Rejected in test to preserve account",
        }, self.admin_token)
        self.assertEqual(s, 200)


class TestSensitiveDataMasking(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.token = get_token("faculty", "Faculty@123456")

    def test_01_store_sensitive_data(self):
        if not self.token:
            self.skipTest("Login failed")
        s, b = api_request("POST", "/api/v1/privacy/sensitive", {
            "field_name": "ssn",
            "value": "123-45-6789",
        }, self.token)
        self.assertEqual(s, 200)

    def test_02_get_masked_fields(self):
        if not self.token:
            self.skipTest("Login failed")
        s, b = api_request("GET", "/api/v1/privacy/sensitive", token=self.token)
        self.assertEqual(s, 200)
        if b["data"]:
            ssn_field = next((f for f in b["data"] if f["field_name"] == "ssn"), None)
            if ssn_field:
                self.assertIn("***", ssn_field["masked_value"])
                self.assertNotIn("123-45-6789", ssn_field["masked_value"])


class TestReauthPrivacyReview(unittest.TestCase):
    """Verify that privacy request review requires recent reauth."""

    @classmethod
    def setUpClass(cls):
        # Fresh login only — no reauth, so last_reauth_at is NULL
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.user_token = get_token("faculty", "Faculty@123456")
        s, b = api_request("POST", "/api/v1/privacy/requests",
            {"request_type": "rectify", "reason": "Reauth guard test"}, cls.user_token)
        cls.request_uuid = b["data"]["uuid"] if s == 200 else None

    def test_review_without_reauth_returns_403(self):
        if not self.request_uuid:
            self.skipTest("No privacy request created")
        s, b = api_request("POST", f"/api/v1/privacy/requests/{self.request_uuid}/review",
            {"approved": False, "admin_notes": "rejected without reauth"}, self.admin_token)
        self.assertEqual(s, 403)
        self.assertIn("reauth", b.get("message", "").lower())

    def test_review_after_reauth_succeeds(self):
        if not self.request_uuid:
            self.skipTest("No privacy request created")
        s, _ = api_request("POST", "/api/v1/auth/reauth",
            {"password": "Admin@12345678"}, self.admin_token)
        if s != 200:
            self.skipTest("Reauth failed")
        s, _ = api_request("POST", f"/api/v1/privacy/requests/{self.request_uuid}/review",
            {"approved": True, "admin_notes": "Approved after reauth"}, self.admin_token)
        self.assertEqual(s, 200)


if __name__ == "__main__":
    unittest.main()
