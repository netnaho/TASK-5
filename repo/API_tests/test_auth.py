"""API integration tests for authentication endpoints."""
import os
import subprocess
import unittest
import urllib.request
import json

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8000")


def _reset_account_lockouts():
    """Reset failed login counts and IP rate limits so tests start clean."""
    subprocess.run(
        ["docker", "exec", "campuslearn-mysql", "mysql", "-ucampus", "-pcampus_pass",
         "campus_learn", "-e",
         "UPDATE users SET failed_login_count=0, locked_until=NULL; DELETE FROM ip_rate_limits;"],
        capture_output=True,
    )


# Reset lockouts once at module load, before any test class runs
_reset_account_lockouts()


def api_post(path: str, data: dict, token: str = None) -> tuple[int, dict]:
    url = f"{BASE_URL}{path}"
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    try:
        body_bytes = json.dumps(data).encode()
        req = urllib.request.Request(url, data=body_bytes, headers=headers, method="POST")
        with urllib.request.urlopen(req, timeout=10) as resp:
            body = json.loads(resp.read().decode())
            return resp.status, body
    except urllib.error.HTTPError as e:
        body = json.loads(e.read().decode()) if e.fp else {}
        return e.code, body
    except Exception as e:
        raise ConnectionError(f"Cannot reach {url}: {e}")


def api_get(path: str, token: str = None) -> tuple[int, dict]:
    url = f"{BASE_URL}{path}"
    headers = {"Accept": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    try:
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=10) as resp:
            body = json.loads(resp.read().decode())
            return resp.status, body
    except urllib.error.HTTPError as e:
        body = json.loads(e.read().decode()) if e.fp else {}
        return e.code, body
    except Exception as e:
        raise ConnectionError(f"Cannot reach {url}: {e}")


def get_token(username: str, password: str) -> str:
    status, body = api_post("/api/v1/auth/login", {"username": username, "password": password})
    if status == 200:
        return body["data"]["token"]
    return None


class TestAuthLogin(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        _reset_account_lockouts()

    def test_login_admin_success(self):
        status, body = api_post("/api/v1/auth/login", {"username": "admin", "password": "Admin@12345678"})
        self.assertEqual(status, 200)
        self.assertTrue(body["success"])
        self.assertIn("token", body["data"])
        self.assertEqual(body["data"]["user"]["role"], "admin")

    def test_login_author_success(self):
        status, body = api_post("/api/v1/auth/login", {"username": "author", "password": "Author@1234567"})
        self.assertEqual(status, 200)
        self.assertEqual(body["data"]["user"]["role"], "staff_author")

    def test_login_reviewer_success(self):
        status, body = api_post("/api/v1/auth/login", {"username": "reviewer", "password": "Review@1234567"})
        self.assertEqual(status, 200)
        self.assertEqual(body["data"]["user"]["role"], "dept_reviewer")

    def test_login_invalid_password(self):
        status, _ = api_post("/api/v1/auth/login", {"username": "admin", "password": "wrongpassword!"})
        self.assertIn(status, [400, 401])

    def test_login_nonexistent_user(self):
        status, _ = api_post("/api/v1/auth/login", {"username": "nonexistent", "password": "SomePassword1!"})
        self.assertIn(status, [400, 401])


class TestAuthMe(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        _reset_account_lockouts()
        cls.token = get_token("admin", "Admin@12345678")

    def test_me_returns_user_info(self):
        if not self.token:
            self.skipTest("Login failed")
        status, body = api_get("/api/v1/auth/me", self.token)
        self.assertEqual(status, 200)
        self.assertEqual(body["data"]["username"], "admin")
        self.assertEqual(body["data"]["role"], "admin")

    def test_me_without_token_returns_401(self):
        status, _ = api_get("/api/v1/auth/me")
        self.assertEqual(status, 401)

    def test_me_with_invalid_token(self):
        status, _ = api_get("/api/v1/auth/me", "invalid.token.here")
        self.assertEqual(status, 401)


class TestReauth(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        _reset_account_lockouts()
        cls.token = get_token("admin", "Admin@12345678")

    def test_reauth_success(self):
        if not self.token:
            self.skipTest("Login failed")
        status, body = api_post("/api/v1/auth/reauth", {"password": "Admin@12345678"}, self.token)
        self.assertEqual(status, 200)

    def test_reauth_wrong_password(self):
        if not self.token:
            self.skipTest("Login failed")
        status, _ = api_post("/api/v1/auth/reauth", {"password": "WrongPassword1!"}, self.token)
        self.assertIn(status, [400, 401])


class TestReauthEnforcement(unittest.TestCase):
    """Sensitive endpoints must require a recent re-authentication."""

    @classmethod
    def setUpClass(cls):
        # Fresh login token with no prior reauth call
        cls.fresh_token = get_token("author", "Author@1234567")
        # Admin token used for reauth then change-password test
        cls.admin_token = get_token("admin", "Admin@12345678")

    def test_change_password_without_reauth_returns_403(self):
        """change-password must be rejected if reauth has not been performed."""
        if not self.fresh_token:
            self.skipTest("Login failed")
        status, _ = api_post("/api/v1/auth/change-password", {
            "current_password": "Author@1234567",
            "new_password": "NewAuthor@9999999",
        }, self.fresh_token)
        self.assertEqual(status, 403)

    def test_change_password_succeeds_after_reauth(self):
        """change-password must succeed (or fail on wrong credentials, not 403) after reauth."""
        if not self.admin_token:
            self.skipTest("Login failed")
        # Perform reauth first
        s, _ = api_post("/api/v1/auth/reauth", {"password": "Admin@12345678"}, self.admin_token)
        if s != 200:
            self.skipTest("Reauth failed")
        # Now attempt change-password with a wrong current password — expect 400/401, NOT 403
        status, _ = api_post("/api/v1/auth/change-password", {
            "current_password": "WrongCurrent!",
            "new_password": "Admin@12345678X",
        }, self.admin_token)
        self.assertIn(status, [400, 401])  # Auth fails on wrong password, but NOT forbidden


if __name__ == "__main__":
    unittest.main()
