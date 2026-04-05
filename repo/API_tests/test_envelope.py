"""API integration tests for response envelope consistency."""
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


class TestSuccessEnvelope(unittest.TestCase):
    """All success responses must use the standard envelope: {"success": true, "data": ...}."""

    def test_health_returns_envelope(self):
        """GET /health returns envelope with status=ok."""
        s, b = api_request("GET", "/health")
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIn("data", b)
        self.assertEqual(b["data"]["status"], "ok")
        self.assertIn("service", b["data"])

    def test_info_returns_envelope(self):
        """GET /api/v1/info returns envelope with application metadata."""
        s, b = api_request("GET", "/api/v1/info")
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIn("data", b)
        self.assertIn("name", b["data"])
        self.assertIn("version", b["data"])
        self.assertIn("api_version", b["data"])

    def test_auth_me_returns_envelope(self):
        """GET /api/v1/auth/me with valid token returns envelope with user data."""
        token = get_token("admin", "Admin@12345678")
        if not token:
            self.skipTest("Login failed")
        s, b = api_request("GET", "/api/v1/auth/me", token=token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIn("data", b)
        self.assertIn("uuid", b["data"])
        self.assertIn("username", b["data"])
        self.assertIn("role", b["data"])

    def test_login_success_returns_envelope(self):
        """POST /api/v1/auth/login success returns envelope with token."""
        s, b = api_request("POST", "/api/v1/auth/login",
            {"username": "faculty", "password": "Faculty@123456"})
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIn("data", b)
        self.assertIn("token", b["data"])


class TestErrorEnvelope(unittest.TestCase):
    """Error responses must use the error envelope: {"status": N, "error": "...", "message": "..."}."""

    def test_error_returns_error_envelope(self):
        """GET /api/v1/audit without auth returns 401 error envelope."""
        s, b = api_request("GET", "/api/v1/audit")
        self.assertEqual(s, 401)
        self.assertIn("status", b)
        self.assertIn("error", b)
        self.assertIn("message", b)
        self.assertEqual(b["status"], 401)

    def test_invalid_login_returns_error_envelope(self):
        """POST /api/v1/auth/login with bad credentials returns error envelope."""
        s, b = api_request("POST", "/api/v1/auth/login",
            {"username": "admin", "password": "WrongPassword!"})
        self.assertIn(s, [400, 401])
        self.assertIn("status", b)
        self.assertIn("error", b)
        self.assertIn("message", b)

    def test_forbidden_returns_error_envelope(self):
        """Accessing admin-only endpoint as non-admin returns 403 error envelope."""
        token = get_token("author", "Author@1234567")
        if not token:
            self.skipTest("Login failed")
        s, b = api_request("GET", "/api/v1/risk/rules", token=token)
        self.assertEqual(s, 403)
        self.assertIn("status", b)
        self.assertIn("error", b)
        self.assertIn("message", b)

    def test_not_found_returns_error_envelope(self):
        """GET for a nonexistent resource returns error envelope."""
        token = get_token("admin", "Admin@12345678")
        if not token:
            self.skipTest("Login failed")
        s, b = api_request("GET", "/api/v1/courses/00000000-0000-0000-0000-000000000000", token=token)
        self.assertIn(s, [404, 400])
        self.assertIn("status", b)
        self.assertIn("error", b)
        self.assertIn("message", b)


if __name__ == "__main__":
    unittest.main()
