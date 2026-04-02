"""API integration tests for course CRUD."""
import os
import unittest
import urllib.request
import json
import uuid

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8000")


def api_request(method: str, path: str, data: dict = None, token: str = None) -> tuple[int, dict]:
    url = f"{BASE_URL}{path}"
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    body_bytes = json.dumps(data).encode() if data else None
    try:
        req = urllib.request.Request(url, data=body_bytes, headers=headers, method=method)
        with urllib.request.urlopen(req, timeout=10) as resp:
            body = json.loads(resp.read().decode())
            return resp.status, body
    except urllib.error.HTTPError as e:
        body = json.loads(e.read().decode()) if e.fp else {}
        return e.code, body
    except Exception as e:
        raise ConnectionError(f"Cannot reach {url}: {e}")


def get_token(username: str, password: str) -> str:
    status, body = api_request("POST", "/api/v1/auth/login", {"username": username, "password": password})
    return body["data"]["token"] if status == 200 else None


class TestCourseCRUD(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.author_token = get_token("author", "Author@1234567")
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.student_token = get_token("student", "Student@12345")
        cls.course_uuid = None

    def test_01_create_course(self):
        if not self.author_token:
            self.skipTest("Author login failed")
        code = f"TST-{uuid.uuid4().hex[:6].upper()}"
        status, body = api_request("POST", "/api/v1/courses", {
            "title": "Test Course",
            "code": code,
            "description": "A test course",
        }, self.author_token)
        self.assertEqual(status, 200)
        self.assertTrue(body["success"])
        self.__class__.course_uuid = body["data"]["uuid"]

    def test_02_list_courses_as_author(self):
        if not self.author_token:
            self.skipTest("Author login failed")
        status, body = api_request("GET", "/api/v1/courses", token=self.author_token)
        self.assertEqual(status, 200)
        self.assertIsInstance(body["data"], list)

    def test_03_get_course(self):
        if not self.course_uuid or not self.author_token:
            self.skipTest("No course created")
        status, body = api_request("GET", f"/api/v1/courses/{self.course_uuid}", token=self.author_token)
        self.assertEqual(status, 200)
        self.assertEqual(body["data"]["title"], "Test Course")
        self.assertEqual(body["data"]["status"], "draft")

    def test_04_update_course(self):
        if not self.course_uuid or not self.author_token:
            self.skipTest("No course created")
        status, body = api_request("PUT", f"/api/v1/courses/{self.course_uuid}", {
            "title": "Updated Test Course",
        }, self.author_token)
        self.assertEqual(status, 200)

    def test_05_student_cannot_create_course(self):
        if not self.student_token:
            self.skipTest("Student login failed")
        status, _ = api_request("POST", "/api/v1/courses", {
            "title": "Forbidden Course",
            "code": "FAIL-001",
        }, self.student_token)
        self.assertEqual(status, 403)

    def test_06_create_section(self):
        if not self.course_uuid or not self.author_token:
            self.skipTest("No course")
        status, body = api_request("POST", f"/api/v1/courses/{self.course_uuid}/sections", {
            "title": "Section 1",
            "sort_order": 1,
        }, self.author_token)
        self.assertEqual(status, 200)
        self.__class__.section_uuid = body["data"]["uuid"]

    def test_07_create_lesson(self):
        if not hasattr(self, 'section_uuid') or not self.section_uuid:
            self.skipTest("No section")
        status, body = api_request("POST", f"/api/v1/courses/sections/{self.section_uuid}/lessons", {
            "title": "Lesson 1",
            "content_type": "text",
            "content_body": "Hello world",
        }, self.author_token)
        self.assertEqual(status, 200)

    def test_08_list_sections(self):
        if not self.course_uuid or not self.author_token:
            self.skipTest("No course")
        status, body = api_request("GET", f"/api/v1/courses/{self.course_uuid}/sections", token=self.author_token)
        self.assertEqual(status, 200)
        self.assertGreaterEqual(len(body["data"]), 1)

    def test_09_delete_draft_course(self):
        """Create and delete a draft course."""
        if not self.author_token:
            self.skipTest("Author login failed")
        code = f"DEL-{uuid.uuid4().hex[:6].upper()}"
        s, b = api_request("POST", "/api/v1/courses", {"title": "Delete Me", "code": code}, self.author_token)
        if s == 200:
            uuid_val = b["data"]["uuid"]
            status, _ = api_request("DELETE", f"/api/v1/courses/{uuid_val}", token=self.author_token)
            self.assertEqual(status, 200)


if __name__ == "__main__":
    unittest.main()
