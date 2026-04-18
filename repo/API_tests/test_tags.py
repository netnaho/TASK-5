"""API integration tests for the tags endpoints (POST/GET /api/v1/tags)."""
import os
import subprocess
import unittest
import urllib.request
import urllib.error
import json
import uuid as uuid_mod

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8000")

TAG_PATHS = ("/api/v1/tags/", "/api/v1/tags")


def _reset_account_lockouts():
    subprocess.run(
        ["docker", "exec", "campuslearn-mysql", "mysql", "-ucampus", "-pcampus_pass",
         "campus_learn", "-e",
         "UPDATE users SET failed_login_count=0, locked_until=NULL; "
         "DELETE FROM ip_rate_limits; "
         "DELETE FROM rate_limit_entries;"],
        capture_output=True,
    )


_reset_account_lockouts()


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
        try:
            body = json.loads(e.read().decode()) if e.fp else {}
        except Exception:
            body = {}
        return e.code, body
    except Exception as e:
        raise ConnectionError(f"Cannot reach {url}: {e}")


def _try_paths(method, data=None, token=None):
    """Try tag endpoints with and without trailing slash and return first non-404 result."""
    last = (404, {})
    for p in TAG_PATHS:
        s, b = api_request(method, p, data, token)
        if s != 404:
            return s, b
        last = (s, b)
    return last


def get_token(username, password):
    s, b = api_request("POST", "/api/v1/auth/login", {"username": username, "password": password})
    return b["data"]["token"] if s == 200 else None


class TestTagsEndpoints(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        _reset_account_lockouts()
        cls.admin_token = get_token("admin", "Admin@12345678")
        cls.author_token = get_token("author", "Author@1234567")
        cls.student_token = get_token("student", "Student@12345")
        cls.faculty_token = get_token("faculty", "Faculty@123456")
        cls.created_tag_name = f"api-tag-{uuid_mod.uuid4().hex[:8]}"
        cls.created_tag_uuid = None

    def test_01_list_tags_requires_auth(self):
        """GET /api/v1/tags without a token returns 401 (AuthenticatedUser)."""
        s, _ = _try_paths("GET")
        self.assertEqual(s, 401)

    def test_02_create_tag_requires_auth(self):
        """POST /api/v1/tags without a token returns 401."""
        s, _ = _try_paths("POST", {"name": "unauth-tag"})
        self.assertEqual(s, 401)

    def test_03_student_cannot_create_tag(self):
        """Students are not CourseAuthorGuard — must get 403."""
        if not self.student_token:
            self.skipTest("Student login failed")
        s, b = _try_paths("POST", {"name": f"stu-{uuid_mod.uuid4().hex[:6]}"}, self.student_token)
        self.assertEqual(s, 403)
        self.assertEqual(b.get("status"), 403)

    def test_04_faculty_cannot_create_tag(self):
        if not self.faculty_token:
            self.skipTest("Faculty login failed")
        s, _ = _try_paths("POST", {"name": f"fac-{uuid_mod.uuid4().hex[:6]}"}, self.faculty_token)
        self.assertEqual(s, 403)

    def test_05_author_creates_tag_success(self):
        """Author (staff_author) is CourseAuthorGuard-authorized to create tags."""
        if not self.author_token:
            self.skipTest("Author login failed")
        s, b = _try_paths("POST", {"name": self.created_tag_name}, self.author_token)
        self.assertEqual(s, 200, msg=f"body={b}")
        self.assertTrue(b.get("success"))
        data = b.get("data") or {}
        self.assertEqual(data.get("name"), self.created_tag_name)
        self.assertEqual(data.get("slug"), self.created_tag_name.lower())
        self.assertIn("uuid", data)
        self.assertIn("id", data)
        self.assertIsInstance(data["id"], int)
        self.__class__.created_tag_uuid = data["uuid"]

    def test_06_create_tag_validation_empty_name(self):
        """Empty tag name must be rejected with 400."""
        if not self.author_token:
            self.skipTest("Author login failed")
        s, b = _try_paths("POST", {"name": ""}, self.author_token)
        self.assertEqual(s, 400)
        self.assertEqual(b.get("status"), 400)

    def test_07_create_tag_idempotent_by_slug(self):
        """Creating the same name again returns the existing tag (same slug)."""
        if not self.author_token or not self.created_tag_uuid:
            self.skipTest("Depends on test_05")
        s, b = _try_paths("POST", {"name": self.created_tag_name}, self.author_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["uuid"], self.created_tag_uuid)
        self.assertEqual(b["data"]["slug"], self.created_tag_name.lower())

    def test_08_admin_can_create_tag(self):
        """Admin role satisfies CourseAuthorGuard."""
        if not self.admin_token:
            self.skipTest("Admin login failed")
        name = f"admin-tag-{uuid_mod.uuid4().hex[:6]}"
        s, b = _try_paths("POST", {"name": name}, self.admin_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["name"], name)

    def test_09_list_tags_returns_created(self):
        """Any authenticated user can list tags — must contain previously created tag."""
        if not self.student_token or not self.created_tag_uuid:
            self.skipTest("Depends on test_05")
        s, b = _try_paths("GET", token=self.student_token)
        self.assertEqual(s, 200)
        self.assertTrue(b.get("success"))
        self.assertIsInstance(b.get("data"), list)
        uuids = {t["uuid"] for t in b["data"]}
        slugs = {t["slug"] for t in b["data"]}
        self.assertIn(self.created_tag_uuid, uuids)
        self.assertIn(self.created_tag_name.lower(), slugs)
        # Shape check on each entry
        for t in b["data"]:
            self.assertIn("id", t)
            self.assertIn("uuid", t)
            self.assertIn("name", t)
            self.assertIn("slug", t)

    def test_10_create_tag_slug_normalization(self):
        """Slug must lowercase and strip non-alphanumeric chars (keeping dashes)."""
        if not self.author_token:
            self.skipTest("Author login failed")
        unique = uuid_mod.uuid4().hex[:6]
        name = f"Data Science {unique}!"
        s, b = _try_paths("POST", {"name": name}, self.author_token)
        self.assertEqual(s, 200)
        # Expected slug: lowercase, spaces -> dashes, "!" stripped
        expected_slug = f"data-science-{unique}"
        self.assertEqual(b["data"]["slug"], expected_slug)


if __name__ == "__main__":
    unittest.main()
