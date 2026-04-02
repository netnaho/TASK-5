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


class TestRiskEngine(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.author_token = get_token("author", "Author@1234567")

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


if __name__ == "__main__":
    unittest.main()
