"""API integration tests for login throttling and account lockout."""
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


class TestSuccessfulLoginNotThrottled(unittest.TestCase):
    """Normal login should succeed without throttling."""

    def test_successful_login_not_throttled(self):
        s, b = api_request("POST", "/api/v1/auth/login",
            {"username": "faculty", "password": "Faculty@123456"})
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertIn("token", b["data"])


class TestAccountLockoutAfterFailures(unittest.TestCase):
    """Account should be locked after 5 consecutive failed login attempts.

    NOTE: This test uses the 'student' account to avoid interfering with
    other tests that depend on admin/author/reviewer accounts. The lockout
    lasts 15 minutes in production.
    """

    @classmethod
    def setUpClass(cls):
        # First, ensure the student account is not already locked by logging in successfully
        s, _ = api_request("POST", "/api/v1/auth/login",
            {"username": "student", "password": "Student@12345"})
        # If already locked, tests will reflect that

    @classmethod
    def tearDownClass(cls):
        """Reset the student account lockout to avoid polluting other test suites."""
        import subprocess
        subprocess.run([
            "docker", "exec", "campuslearn-mysql",
            "mysql", "-ucampus", "-pcampus_pass", "campus_learn",
            "-e", "UPDATE users SET failed_login_count = 0, locked_until = NULL WHERE username = 'student';"
        ], capture_output=True)

    def test_account_lockout_after_failures(self):
        """Repeated wrong password attempts should eventually lock the account.

        The lockout threshold is configurable (LOGIN_LOCKOUT_THRESHOLD, default 10
        in production, often 20 in dev). We try up to 25 times to trigger it.
        """
        wrong_password = "TotallyWrong!999"
        threshold = int(os.environ.get("LOGIN_LOCKOUT_THRESHOLD", "20"))
        locked = False
        for i in range(threshold + 5):
            s, b = api_request("POST", "/api/v1/auth/login",
                {"username": "student", "password": wrong_password})
            msg = b.get("message", "").lower()
            if "locked" in msg or "too many" in msg:
                locked = True
                break

        self.assertTrue(locked, f"Expected lockout after {threshold} attempts but it didn't trigger")

        # Even the correct password should fail while locked
        s, b = api_request("POST", "/api/v1/auth/login",
            {"username": "student", "password": "Student@12345"})
        self.assertIn(s, [400, 429])


class TestSuccessfulLoginResetsCounter(unittest.TestCase):
    """A successful login should reset the failed attempt counter.

    NOTE: This test uses the 'faculty' account. We do fewer than 5 failures
    so the account does not lock, then verify a successful login, then
    confirm the counter was reset by doing another round of fewer than 5
    failures without lockout.
    """

    def test_successful_login_resets_counter(self):
        """3 failures, then success, then 3 more failures should not lock."""
        wrong_password = "WrongPass!1234"

        # Round 1: 3 failed attempts (below lockout threshold of 5)
        for i in range(3):
            s, _ = api_request("POST", "/api/v1/auth/login",
                {"username": "faculty", "password": wrong_password})
            self.assertIn(s, [400, 401, 429], f"Round 1 attempt {i+1}: unexpected status {s}")

        # Successful login should reset counter
        s, b = api_request("POST", "/api/v1/auth/login",
            {"username": "faculty", "password": "Faculty@123456"})
        self.assertEqual(s, 200, "Faculty login should succeed after 3 failed attempts")
        self.assertIn("token", b["data"])

        # Round 2: 3 more failed attempts (counter was reset, so total is 3 not 6)
        for i in range(3):
            s, _ = api_request("POST", "/api/v1/auth/login",
                {"username": "faculty", "password": wrong_password})
            self.assertIn(s, [400, 401, 429], f"Round 2 attempt {i+1}: unexpected status {s}")

        # Account should NOT be locked (only 3 since reset, not 5)
        s, b = api_request("POST", "/api/v1/auth/login",
            {"username": "faculty", "password": "Faculty@123456"})
        self.assertEqual(s, 200, "Faculty login should still succeed; counter was reset")


if __name__ == "__main__":
    unittest.main()
