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

    def test_01_create_deletion_request(self):
        if not self.user_token:
            self.skipTest("Login failed")
        s, b = api_request("POST", "/api/v1/privacy/requests", {
            "request_type": "delete",
            "reason": "Right to be forgotten",
        }, self.user_token)
        self.assertEqual(s, 200)
        self.__class__.request_uuid = b["data"]["uuid"]

    def test_02_admin_approves_deletion(self):
        if not self.request_uuid or not self.admin_token:
            self.skipTest("No request or admin")
        s, b = api_request("POST", f"/api/v1/privacy/requests/{self.request_uuid}/review", {
            "approved": True,
            "admin_notes": "Deletion approved",
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


if __name__ == "__main__":
    unittest.main()
