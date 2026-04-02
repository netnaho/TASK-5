"""API integration tests for health and info endpoints."""
import os
import unittest
import urllib.request
import json

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8000")


def api_get(path: str) -> tuple[int, dict]:
    url = f"{BASE_URL}{path}"
    try:
        req = urllib.request.Request(url, headers={"Accept": "application/json"})
        with urllib.request.urlopen(req, timeout=10) as resp:
            body = json.loads(resp.read().decode())
            return resp.status, body
    except urllib.error.HTTPError as e:
        body = json.loads(e.read().decode()) if e.fp else {}
        return e.code, body
    except Exception as e:
        raise ConnectionError(f"Cannot reach {url}: {e}")


class TestHealthEndpoint(unittest.TestCase):
    def test_health_returns_200(self):
        status, body = api_get("/health")
        self.assertEqual(status, 200)

    def test_health_returns_ok_status(self):
        _, body = api_get("/health")
        self.assertEqual(body["status"], "ok")

    def test_health_returns_service_name(self):
        _, body = api_get("/health")
        self.assertEqual(body["service"], "campus-learn-backend")


class TestInfoEndpoint(unittest.TestCase):
    def test_info_returns_200(self):
        status, _ = api_get("/api/v1/info")
        self.assertEqual(status, 200)

    def test_info_returns_name(self):
        _, body = api_get("/api/v1/info")
        self.assertEqual(body["name"], "CampusLearn Operations Suite")

    def test_info_returns_api_version(self):
        _, body = api_get("/api/v1/info")
        self.assertEqual(body["api_version"], "v1")


if __name__ == "__main__":
    unittest.main()
