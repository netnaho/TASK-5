"""API integration tests for risk engine."""
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


def _reset_reauth_timestamps(*usernames):
    import subprocess
    names = "', '".join(usernames)
    subprocess.run(
        ["docker", "exec", "campuslearn-mysql", "mysql", "-ucampus", "-pcampus_pass",
         "campus_learn", "-e",
         f"UPDATE users SET last_reauth_at=NULL WHERE username IN ('{names}');"],
        capture_output=True,
    )


class TestRiskEngine(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.author_token = get_token("author", "Author@1234567")
        # Reauth admin so reauth-guarded endpoints (blacklist, evaluate, update_event) pass
        if cls.admin_token:
            api_request("POST", "/api/v1/auth/reauth",
                {"password": "Admin@12345678"}, cls.admin_token)

    def test_01_list_risk_rules(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("GET", "/api/v1/risk/rules", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertGreater(len(b["data"]), 0)

    def test_02_create_posting(self):
        if not self.author_token:
            self.skipTest("Author login failed")
        s, b = api_request("POST", "/api/v1/risk/postings", {
            "employer_name": "TechCorp",
            "posting_type": "internship",
            "title": "Summer Intern 2025",
            "compensation": 5000.00,
        }, self.author_token)
        self.assertEqual(s, 200)

    def test_03_run_risk_evaluation(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("POST", "/api/v1/risk/evaluate", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertIn("events_created", b["data"])

    def test_04_list_risk_events(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("GET", "/api/v1/risk/events", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertIsInstance(b["data"], list)

    def test_05_non_admin_blocked_from_rules(self):
        if not self.author_token:
            self.skipTest("Author login failed")
        s, _ = api_request("GET", "/api/v1/risk/rules", token=self.author_token)
        self.assertEqual(s, 403)

    def test_06_add_blacklist(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("POST", "/api/v1/risk/blacklist", {
            "employer_name": "ScamCorp",
            "reason": "Known fraudulent employer",
        }, self.admin_token)
        self.assertEqual(s, 200)

    def test_07_blacklisted_employer_posting_blocked(self):
        if not self.author_token:
            self.skipTest("Author login failed")
        s, b = api_request("POST", "/api/v1/risk/postings", {
            "employer_name": "ScamCorp",
            "posting_type": "job",
            "title": "Fake Job",
        }, self.author_token)
        self.assertEqual(s, 403)

    def test_08_create_subscription(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("POST", "/api/v1/risk/subscriptions", {
            "event_type": "risk_high",
            "channel": "in_app",
        }, self.admin_token)
        self.assertEqual(s, 200)

    def test_09_list_subscriptions(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("GET", "/api/v1/risk/subscriptions", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertIsInstance(b["data"], list)

    def test_10_webhook_subscription_requires_target_url(self):
        """channel=webhook without target_url must be rejected with 400."""
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("POST", "/api/v1/risk/subscriptions", {
            "event_type": "risk_high",
            "channel": "webhook",
        }, self.admin_token)
        self.assertEqual(s, 400)
        self.assertIn("target_url", b.get("message", "").lower())

    def test_11_webhook_subscription_with_valid_onprem_url(self):
        """channel=webhook with a localhost URL must be accepted."""
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("POST", "/api/v1/risk/subscriptions", {
            "event_type": "risk_webhook_test",
            "channel": "webhook",
            "target_url": "http://localhost:9191/hook",
        }, self.admin_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["channel"], "webhook")
        self.assertEqual(b["data"]["target_url"], "http://localhost:9191/hook")
        # signing_secret must never appear in the response
        self.assertNotIn("signing_secret", b["data"])

    def test_12_webhook_subscription_with_public_url_rejected(self):
        """channel=webhook with an external/public URL must be rejected with 400."""
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("POST", "/api/v1/risk/subscriptions", {
            "event_type": "risk_high",
            "channel": "webhook",
            "target_url": "http://example.com/hook",
        }, self.admin_token)
        self.assertEqual(s, 400)
        self.assertIn("on-prem", b.get("message", "").lower())

    def test_13_in_app_subscription_without_target_url_succeeds(self):
        """Existing in_app subscription flow is unaffected."""
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("POST", "/api/v1/risk/subscriptions", {
            "event_type": "risk_medium",
            "channel": "in_app",
        }, self.admin_token)
        self.assertEqual(s, 200)
        self.assertIsNone(b["data"].get("target_url"))


class TestReauthRiskOperations(unittest.TestCase):
    """Verify that update_event and run_evaluation require recent reauth."""

    @classmethod
    def setUpClass(cls):
        # Reset so last_reauth_at is NULL — guard must then return 403
        _reset_reauth_timestamps("admin")
        # Fresh login only — no reauth
        cls.admin_token = get_token("admin", "Admin@12345678")
        s, b = api_request("GET", "/api/v1/risk/events", token=cls.admin_token)
        cls.event_uuid = b["data"][0]["uuid"] if s == 200 and b.get("data") else None

    def test_01_update_event_without_reauth_returns_403(self):
        if not self.event_uuid:
            self.skipTest("No risk events available")
        s, b = api_request("PUT", f"/api/v1/risk/events/{self.event_uuid}",
            {"status": "acknowledged"}, self.admin_token)
        self.assertEqual(s, 403)

    def test_02_run_evaluation_without_reauth_returns_403(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("POST", "/api/v1/risk/evaluate", token=self.admin_token)
        self.assertEqual(s, 403)

    def test_03_update_event_after_reauth_succeeds(self):
        if not self.event_uuid:
            self.skipTest("No risk events available")
        s, _ = api_request("POST", "/api/v1/auth/reauth",
            {"password": "Admin@12345678"}, self.admin_token)
        if s != 200:
            self.skipTest("Reauth failed")
        s, _ = api_request("PUT", f"/api/v1/risk/events/{self.event_uuid}",
            {"status": "acknowledged"}, self.admin_token)
        self.assertEqual(s, 200)

    def test_04_run_evaluation_after_reauth_succeeds(self):
        if not self.admin_token:
            self.skipTest("Admin login failed")
        # Perform reauth explicitly (test_03 may have been skipped if no event_uuid)
        s, _ = api_request("POST", "/api/v1/auth/reauth",
            {"password": "Admin@12345678"}, self.admin_token)
        if s != 200:
            self.skipTest("Reauth failed")
        s, b = api_request("POST", "/api/v1/risk/evaluate", token=self.admin_token)
        self.assertEqual(s, 200)
        self.assertIn("events_created", b.get("data", {}))


if __name__ == "__main__":
    unittest.main()
