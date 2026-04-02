"""API integration tests for audit endpoints."""
import os
import unittest
import urllib.request
import json

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8000")


def api_get(path: str, token: str = None) -> tuple[int, dict]:
    url = f"{BASE_URL}{path}"
    headers = {"Accept": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    try:
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=10) as resp:
            return resp.status, json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        body = json.loads(e.read().decode()) if e.fp else {}
        return e.code, body


def api_post(path: str, data: dict, token: str = None) -> tuple[int, dict]:
    url = f"{BASE_URL}{path}"
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    try:
        req = urllib.request.Request(url, data=json.dumps(data).encode(), headers=headers, method="POST")
        with urllib.request.urlopen(req, timeout=10) as resp:
            return resp.status, json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        body = json.loads(e.read().decode()) if e.fp else {}
        return e.code, body


def get_token(username: str, password: str) -> str:
    s, b = api_post("/api/v1/auth/login", {"username": username, "password": password})
    return b["data"]["token"] if s == 200 else None


class TestAuditEndpoint(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.student_token = get_token("student", "Student@12345")

    def test_admin_can_view_audit_logs(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_get("/api/v1/audit", self.admin_token)
        self.assertEqual(s, 200)
        self.assertIsInstance(b["data"], list)

    def test_student_cannot_view_audit_logs(self):
        if not self.student_token:
            self.skipTest("Student login failed")
        s, _ = api_get("/api/v1/audit", self.student_token)
        self.assertEqual(s, 403)

    def test_audit_log_has_expected_fields(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_get("/api/v1/audit?limit=5", self.admin_token)
        self.assertEqual(s, 200)
        if b["data"]:
            log = b["data"][0]
            for field in ["uuid", "action", "entity_type", "created_at"]:
                self.assertIn(field, log)


if __name__ == "__main__":
    unittest.main()
