"""API integration tests for booking engine."""
import os
import unittest
import urllib.request
import json
from datetime import datetime, timedelta

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


class TestBookingHappyPath(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.token = get_token("faculty", "Faculty@123456")
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.resource_uuid = None
        cls.booking_uuid = None

    def test_01_list_resources(self):
        if not self.token:
            self.skipTest("Login failed")
        s, b = api_request("GET", "/api/v1/bookings/resources", token=self.token)
        self.assertEqual(s, 200)
        self.assertGreater(len(b["data"]), 0)
        # Save first room resource UUID
        rooms = [r for r in b["data"] if r["resource_type"] == "room"]
        if rooms:
            self.__class__.resource_uuid = rooms[0]["uuid"]

    def test_02_check_availability(self):
        if not self.resource_uuid:
            self.skipTest("No resource")
        date = (datetime.now() + timedelta(days=1)).strftime("%Y-%m-%d")
        s, b = api_request("GET", f"/api/v1/bookings/resources/{self.resource_uuid}/availability?date={date}", token=self.token)
        self.assertEqual(s, 200)
        self.assertIsInstance(b["data"], list)

    def test_03_create_booking(self):
        if not self.resource_uuid:
            self.skipTest("No resource")
        tomorrow = datetime.now() + timedelta(days=1)
        start = tomorrow.replace(hour=10, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S")
        end = tomorrow.replace(hour=12, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S")
        s, b = api_request("POST", "/api/v1/bookings", {
            "resource_uuid": self.resource_uuid,
            "title": "Test Meeting",
            "start_time": start,
            "end_time": end,
        }, self.token)
        self.assertEqual(s, 200)
        self.__class__.booking_uuid = b["data"]["uuid"]

    def test_04_conflict_detection(self):
        """Booking same slot should fail."""
        if not self.resource_uuid or not self.booking_uuid:
            self.skipTest("No prior booking")
        tomorrow = datetime.now() + timedelta(days=1)
        start = tomorrow.replace(hour=10, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S")
        end = tomorrow.replace(hour=12, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S")
        s, b = api_request("POST", "/api/v1/bookings", {
            "resource_uuid": self.resource_uuid,
            "title": "Conflicting Meeting",
            "start_time": start,
            "end_time": end,
        }, self.token)
        self.assertIn(s, [400, 409])

    def test_05_list_my_bookings(self):
        s, b = api_request("GET", "/api/v1/bookings/my", token=self.token)
        self.assertEqual(s, 200)
        self.assertIsInstance(b["data"], list)

    def test_06_reschedule_booking(self):
        if not self.booking_uuid:
            self.skipTest("No booking")
        tomorrow = datetime.now() + timedelta(days=2)
        s, b = api_request("POST", f"/api/v1/bookings/{self.booking_uuid}/reschedule", {
            "new_start_time": tomorrow.replace(hour=14, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S"),
            "new_end_time": tomorrow.replace(hour=16, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S"),
            "reason": "Schedule change",
        }, self.token)
        self.assertEqual(s, 200)

    def test_07_cancel_booking(self):
        """Cancel the rescheduled booking (not late, so no breach)."""
        if not self.booking_uuid:
            self.skipTest("No booking")
        s, b = api_request("POST", f"/api/v1/bookings/{self.booking_uuid}/cancel", token=self.token)
        self.assertEqual(s, 200)

    def test_08_view_breaches(self):
        s, b = api_request("GET", "/api/v1/bookings/breaches", token=self.token)
        self.assertEqual(s, 200)
        self.assertIsInstance(b["data"], list)

    def test_09_view_restrictions(self):
        s, b = api_request("GET", "/api/v1/bookings/restrictions", token=self.token)
        self.assertEqual(s, 200)
        self.assertIsInstance(b["data"], list)


class TestBookingValidation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.token = get_token("student", "Student@12345")
        # Get a room resource
        s, b = api_request("GET", "/api/v1/bookings/resources", token=cls.token)
        if s == 200:
            rooms = [r for r in b["data"] if r["resource_type"] == "room"]
            cls.resource_uuid = rooms[0]["uuid"] if rooms else None
        else:
            cls.resource_uuid = None

    def test_invalid_hours(self):
        """Booking outside resource hours should fail."""
        if not self.resource_uuid:
            self.skipTest("No resource")
        tomorrow = datetime.now() + timedelta(days=1)
        s, _ = api_request("POST", "/api/v1/bookings", {
            "resource_uuid": self.resource_uuid,
            "title": "Late Night",
            "start_time": tomorrow.replace(hour=23, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S"),
            "end_time": tomorrow.replace(hour=23, minute=59, second=0).strftime("%Y-%m-%d %H:%M:%S"),
        }, self.token)
        self.assertIn(s, [400])

    def test_exceeds_max_hours(self):
        """5-hour room booking should fail (max 4)."""
        if not self.resource_uuid:
            self.skipTest("No resource")
        tomorrow = datetime.now() + timedelta(days=3)
        s, _ = api_request("POST", "/api/v1/bookings", {
            "resource_uuid": self.resource_uuid,
            "title": "Long Meeting",
            "start_time": tomorrow.replace(hour=8, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S"),
            "end_time": tomorrow.replace(hour=13, minute=0, second=0).strftime("%Y-%m-%d %H:%M:%S"),
        }, self.token)
        self.assertIn(s, [400])


if __name__ == "__main__":
    unittest.main()
