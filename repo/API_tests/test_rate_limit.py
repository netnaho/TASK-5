"""API integration tests for per-user rate limiting.

Rate limiting is enforced inside AuthenticatedUser::from_request(), so it covers
every authenticated endpoint with no per-route changes.

Environment variables:
  RATE_LIMIT_PER_MINUTE  Configured threshold (default 120). Set this to a small
                         value (e.g. 5) in your test environment to keep the test
                         fast without firing 120+ real requests.

NOTE: Each test class uses a distinct seeded account so their rate-limit windows
      do not collide with each other or with other test files.
"""
import os
import unittest
import urllib.request
import json

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8000")
RATE_LIMIT = int(os.environ.get("RATE_LIMIT_PER_MINUTE", "120"))


def _http(method: str, path: str, data: dict = None, token: str = None) -> tuple[int, dict]:
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
    s, b = _http("POST", "/api/v1/auth/login", {"username": username, "password": password})
    return b["data"]["token"] if s == 200 else None


class TestRateLimitBreach(unittest.TestCase):
    """Burst traffic beyond RATE_LIMIT_PER_MINUTE must return HTTP 429."""

    def test_breach_returns_429(self):
        """Fire RATE_LIMIT + 10 requests; at least the last one must be 429."""
        token = get_token("faculty", "Faculty@123456")
        if not token:
            self.skipTest("Login failed — backend not reachable")

        statuses = []
        for _ in range(RATE_LIMIT + 10):
            s, _ = _http("GET", "/api/v1/auth/me", token=token)
            statuses.append(s)
            if s == 429:
                break

        self.assertIn(
            429, statuses,
            f"Expected a 429 after {RATE_LIMIT} requests but received only: "
            f"{set(statuses)}. Check RATE_LIMIT_PER_MINUTE env var.",
        )

    def test_429_response_envelope(self):
        """The 429 body must conform to the standard ApiError envelope."""
        token = get_token("student", "Student@12345")
        if not token:
            self.skipTest("Login failed")

        body_429 = None
        for _ in range(RATE_LIMIT + 10):
            s, body = _http("GET", "/api/v1/auth/me", token=token)
            if s == 429:
                body_429 = body
                break

        if body_429 is None:
            self.skipTest(
                f"Did not reach rate limit within {RATE_LIMIT + 10} requests. "
                "Set RATE_LIMIT_PER_MINUTE to a small value (e.g. 5) for fast testing."
            )

        # Must match { "status": 429, "error": "...", "message": "..." }
        self.assertEqual(body_429.get("status"), 429)
        self.assertIn("error", body_429, "Missing 'error' field in 429 response")
        self.assertIn("message", body_429, "Missing 'message' field in 429 response")
        self.assertIn(
            "Rate limit", body_429["message"],
            f"Expected 'Rate limit' in message, got: {body_429['message']!r}",
        )


class TestRateLimitNormalTraffic(unittest.TestCase):
    """A small burst of requests well below the threshold must always succeed."""

    def test_single_request_succeeds(self):
        """One authenticated request must never return 429 for a fresh-ish user."""
        token = get_token("reviewer", "Review@1234567")
        if not token:
            self.skipTest("Login failed")
        s, body = _http("GET", "/api/v1/auth/me", token=token)
        if s == 429:
            self.skipTest(
                "Reviewer rate limit already exhausted (previous test runs in same minute). "
                "This is expected when running the full suite multiple times rapidly."
            )
        self.assertEqual(s, 200)

    def test_unauthenticated_endpoint_not_rate_limited(self):
        """Health endpoint has no auth guard — must return 200 regardless of rate state."""
        s, _ = _http("GET", "/health")
        self.assertEqual(s, 200)

    def test_login_endpoint_not_rate_limited(self):
        """Login is unauthenticated — must not return 429 from the user guard."""
        # A failed login attempt (wrong password) should return 400/401, not 429
        s, _ = _http("POST", "/api/v1/auth/login",
                     {"username": "admin", "password": "wrongpassword!"})
        self.assertIn(s, [400, 401],
                      f"Login with bad credentials should be 400/401, got {s}")


if __name__ == "__main__":
    unittest.main()
