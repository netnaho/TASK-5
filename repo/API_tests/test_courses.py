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


def _accept_active_term(token: str) -> None:
    s, b = api_request("GET", "/api/v1/terms/active", token=token)
    if s == 200 and b.get("data"):
        api_request("POST", f"/api/v1/terms/{b['data']['uuid']}/accept", token=token)


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


class TestCourseDraftVisibility(unittest.TestCase):
    """Students must not be able to read draft courses by UUID."""

    @classmethod
    def setUpClass(cls):
        cls.author_token = get_token("author", "Author@1234567")
        cls.student_token = get_token("student", "Student@12345")
        cls.draft_uuid = None

        if not cls.author_token:
            return
        code = f"VIS-{uuid.uuid4().hex[:6].upper()}"
        s, b = api_request("POST", "/api/v1/courses", {
            "title": "Visibility Test Course",
            "code": code,
        }, cls.author_token)
        if s == 200:
            cls.draft_uuid = b["data"]["uuid"]

    def test_student_cannot_view_draft_by_uuid(self):
        if not self.draft_uuid or not self.student_token:
            self.skipTest("Setup failed")
        s, _ = api_request("GET", f"/api/v1/courses/{self.draft_uuid}", token=self.student_token)
        self.assertIn(s, [403, 404])

    def test_author_can_view_own_draft(self):
        if not self.draft_uuid or not self.author_token:
            self.skipTest("Setup failed")
        s, b = api_request("GET", f"/api/v1/courses/{self.draft_uuid}", token=self.author_token)
        self.assertEqual(s, 200)
        self.assertEqual(b["data"]["status"], "draft")

    def test_admin_can_view_any_draft(self):
        if not self.draft_uuid:
            self.skipTest("Setup failed")
        admin_token = get_token("admin", "Admin@12345678")
        if not admin_token:
            self.skipTest("Admin login failed")
        s, b = api_request("GET", f"/api/v1/courses/{self.draft_uuid}", token=admin_token)
        self.assertEqual(s, 200)


class TestObjectLevelAuth(unittest.TestCase):
    """Cross-author mutations must be rejected; draft sections/versions hidden from unauthorized users."""

    @classmethod
    def setUpClass(cls):
        cls.author_token  = get_token("author",  "Author@1234567")
        cls.admin_token   = get_token("admin",   "Admin@12345678")
        cls.student_token = get_token("student", "Student@12345")
        # Author must accept active term before submitting for approval
        if cls.author_token:
            _accept_active_term(cls.author_token)

        # Author creates course_A with a section and lesson
        cls.course_a_uuid = None
        cls.section_a_uuid = None
        cls.lesson_a_uuid  = None
        if cls.author_token:
            code_a = f"OLA-{uuid.uuid4().hex[:5].upper()}"
            s, b = api_request("POST", "/api/v1/courses",
                {"title": "OLA Course A", "code": code_a}, cls.author_token)
            if s == 200:
                cls.course_a_uuid = b["data"]["uuid"]
            if cls.course_a_uuid:
                s, b = api_request("POST", f"/api/v1/courses/{cls.course_a_uuid}/sections",
                    {"title": "Section A", "sort_order": 1}, cls.author_token)
                if s == 200:
                    cls.section_a_uuid = b["data"]["uuid"]
            if cls.section_a_uuid:
                s, b = api_request("POST", f"/api/v1/courses/sections/{cls.section_a_uuid}/lessons",
                    {"title": "Lesson A", "content_type": "text", "content_body": "hello"},
                    cls.author_token)
                if s == 200:
                    cls.lesson_a_uuid = b["data"]["uuid"]

        # Admin creates course_B with a section
        cls.course_b_uuid  = None
        cls.section_b_uuid = None
        cls.lesson_b_uuid  = None
        if cls.admin_token:
            code_b = f"OLB-{uuid.uuid4().hex[:5].upper()}"
            s, b = api_request("POST", "/api/v1/courses",
                {"title": "OLA Course B", "code": code_b}, cls.admin_token)
            if s == 200:
                cls.course_b_uuid = b["data"]["uuid"]
            if cls.course_b_uuid:
                s, b = api_request("POST", f"/api/v1/courses/{cls.course_b_uuid}/sections",
                    {"title": "Section B", "sort_order": 1}, cls.admin_token)
                if s == 200:
                    cls.section_b_uuid = b["data"]["uuid"]
            if cls.section_b_uuid:
                s, b = api_request("POST", f"/api/v1/courses/sections/{cls.section_b_uuid}/lessons",
                    {"title": "Admin Lesson", "content_type": "text"}, cls.admin_token)
                if s == 200:
                    cls.lesson_b_uuid = b["data"]["uuid"]

    # --- Cross-author section mutations ---

    def test_author_cannot_update_foreign_section(self):
        if not self.section_b_uuid or not self.author_token:
            self.skipTest("Setup failed")
        s, _ = api_request("PUT", f"/api/v1/courses/sections/{self.section_b_uuid}",
            {"title": "Hacked"}, self.author_token)
        self.assertEqual(s, 403)

    def test_author_cannot_delete_foreign_section(self):
        if not self.section_b_uuid or not self.author_token:
            self.skipTest("Setup failed")
        s, _ = api_request("DELETE", f"/api/v1/courses/sections/{self.section_b_uuid}",
            token=self.author_token)
        self.assertEqual(s, 403)

    def test_author_cannot_add_lesson_to_foreign_section(self):
        if not self.section_b_uuid or not self.author_token:
            self.skipTest("Setup failed")
        s, _ = api_request("POST", f"/api/v1/courses/sections/{self.section_b_uuid}/lessons",
            {"title": "Injected", "content_type": "text"}, self.author_token)
        self.assertEqual(s, 403)

    def test_author_cannot_update_foreign_lesson(self):
        if not self.lesson_b_uuid or not self.author_token:
            self.skipTest("Setup failed")
        s, _ = api_request("PUT", f"/api/v1/courses/lessons/{self.lesson_b_uuid}",
            {"title": "Hacked"}, self.author_token)
        self.assertEqual(s, 403)

    def test_author_cannot_delete_foreign_lesson(self):
        if not self.lesson_b_uuid or not self.author_token:
            self.skipTest("Setup failed")
        s, _ = api_request("DELETE", f"/api/v1/courses/lessons/{self.lesson_b_uuid}",
            token=self.author_token)
        self.assertEqual(s, 403)

    # --- Admin can manage any course's assets ---

    def test_admin_can_update_any_section(self):
        if not self.section_a_uuid or not self.admin_token:
            self.skipTest("Setup failed")
        s, _ = api_request("PUT", f"/api/v1/courses/sections/{self.section_a_uuid}",
            {"title": "Admin Updated"}, self.admin_token)
        self.assertEqual(s, 200)

    def test_admin_can_update_any_lesson(self):
        if not self.lesson_a_uuid or not self.admin_token:
            self.skipTest("Setup failed")
        s, _ = api_request("PUT", f"/api/v1/courses/lessons/{self.lesson_a_uuid}",
            {"title": "Admin Updated Lesson"}, self.admin_token)
        self.assertEqual(s, 200)

    # --- Authors can manage their own assets ---

    def test_author_can_update_own_section(self):
        if not self.section_a_uuid or not self.author_token:
            self.skipTest("Setup failed")
        s, _ = api_request("PUT", f"/api/v1/courses/sections/{self.section_a_uuid}",
            {"title": "Author Updated"}, self.author_token)
        self.assertEqual(s, 200)

    def test_author_can_update_own_lesson(self):
        if not self.lesson_a_uuid or not self.author_token:
            self.skipTest("Setup failed")
        s, _ = api_request("PUT", f"/api/v1/courses/lessons/{self.lesson_a_uuid}",
            {"title": "Author Updated Lesson"}, self.author_token)
        self.assertEqual(s, 200)

    # --- Section/version listing visibility for draft courses ---

    def test_student_cannot_list_sections_of_draft(self):
        if not self.course_a_uuid or not self.student_token:
            self.skipTest("Setup failed")
        s, _ = api_request("GET", f"/api/v1/courses/{self.course_a_uuid}/sections",
            token=self.student_token)
        self.assertIn(s, [403, 404])

    def test_student_cannot_list_versions_of_draft(self):
        if not self.course_a_uuid or not self.student_token:
            self.skipTest("Setup failed")
        s, _ = api_request("GET", f"/api/v1/courses/{self.course_a_uuid}/versions",
            token=self.student_token)
        self.assertIn(s, [403, 404])

    def test_author_can_list_own_draft_sections(self):
        if not self.course_a_uuid or not self.author_token:
            self.skipTest("Setup failed")
        s, _ = api_request("GET", f"/api/v1/courses/{self.course_a_uuid}/sections",
            token=self.author_token)
        self.assertEqual(s, 200)

    # --- Submit-for-approval ownership ---

    def test_author_cannot_submit_foreign_course(self):
        if not self.course_b_uuid or not self.author_token:
            self.skipTest("Setup failed")
        from datetime import datetime, timedelta
        future = (datetime.now() + timedelta(days=30)).strftime("%m/%d/%Y 09:00 AM")
        s, _ = api_request("POST", f"/api/v1/approvals/{self.course_b_uuid}/submit",
            {"release_notes": "Stolen submit", "effective_date": future}, self.author_token)
        self.assertEqual(s, 403)

    def test_author_can_submit_own_course(self):
        if not self.course_a_uuid or not self.author_token:
            self.skipTest("Setup failed")
        from datetime import datetime, timedelta
        future = (datetime.now() + timedelta(days=30)).strftime("%m/%d/%Y 09:00 AM")
        s, _ = api_request("POST", f"/api/v1/approvals/{self.course_a_uuid}/submit",
            {"release_notes": "Own submit", "effective_date": future}, self.author_token)
        # 200 on first submission; 400 if already submitted
        self.assertIn(s, [200, 400])


class TestMediaUploadAndValidation(unittest.TestCase):
    """Tests for media upload pipeline and validation state model."""

    @classmethod
    def setUpClass(cls):
        cls.author_token = get_token("author", "Author@1234567")

    def _multipart_upload(self, file_name, file_content, content_type, token, alt_text=None, lesson_id=None):
        """Upload a file using multipart/form-data."""
        import io
        boundary = "----TestBoundary12345"
        body = io.BytesIO()

        # file field
        body.write(f"--{boundary}\r\n".encode())
        body.write(f'Content-Disposition: form-data; name="file"; filename="{file_name}"\r\n'.encode())
        body.write(f"Content-Type: {content_type}\r\n\r\n".encode())
        body.write(file_content)
        body.write(b"\r\n")

        # alt_text field
        if alt_text:
            body.write(f"--{boundary}\r\n".encode())
            body.write(b'Content-Disposition: form-data; name="alt_text"\r\n\r\n')
            body.write(alt_text.encode())
            body.write(b"\r\n")

        # lesson_id field
        if lesson_id is not None:
            body.write(f"--{boundary}\r\n".encode())
            body.write(b'Content-Disposition: form-data; name="lesson_id"\r\n\r\n')
            body.write(str(lesson_id).encode())
            body.write(b"\r\n")

        body.write(f"--{boundary}--\r\n".encode())

        url = f"{BASE_URL}/api/v1/courses/media/upload"
        headers = {
            "Content-Type": f"multipart/form-data; boundary={boundary}",
            "Authorization": f"Bearer {token}",
        }
        try:
            req = urllib.request.Request(url, data=body.getvalue(), headers=headers, method="POST")
            with urllib.request.urlopen(req, timeout=30) as resp:
                return resp.status, json.loads(resp.read().decode())
        except urllib.error.HTTPError as e:
            resp_body = json.loads(e.read().decode()) if e.fp else {}
            return e.code, resp_body
        except Exception as e:
            raise ConnectionError(f"Cannot reach {url}: {e}")

    def test_01_upload_pdf_returns_pending_scan(self):
        """Uploading a PDF via multipart should create a pending_scan media asset."""
        if not self.author_token:
            self.skipTest("Author login failed")
        content = b"%PDF-1.4 test content"
        s, b = self._multipart_upload("test_doc.pdf", content, "application/pdf", self.author_token)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")
        self.assertEqual(b["data"]["status"], "pending_scan")
        self.assertFalse(b["data"]["validated"])
        self.__class__.uploaded_media_uuid = b["data"]["uuid"]

    def test_02_validate_uploaded_media(self):
        """Validating an uploaded media asset should transition to ready."""
        uuid = getattr(self.__class__, "uploaded_media_uuid", None)
        if not uuid:
            self.skipTest("No uploaded media")
        s, b = api_request("POST", f"/api/v1/courses/media/{uuid}/validate", token=self.author_token)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")
        self.assertEqual(b["data"]["status"], "ready")
        self.assertTrue(b["data"]["validated"])

    def test_03_register_media_returns_pending_scan(self):
        """Metadata-only media registration should return pending_scan."""
        if not self.author_token:
            self.skipTest("Author login failed")
        s, b = api_request("POST", "/api/v1/courses/media", {
            "file_name": "lecture.pdf",
            "file_path": "/uploads/lecture.pdf",
            "mime_type": "application/pdf",
            "file_size_bytes": 1024,
            "checksum": "abc123hash",
        }, self.author_token)
        self.assertEqual(s, 200, f"Expected 200 but got {s}: {b}")
        self.assertEqual(b["data"]["status"], "pending_scan")
        self.assertFalse(b["data"]["validated"])

    def test_04_upload_invalid_type_rejected(self):
        """Uploading an unsupported MIME type should be rejected."""
        if not self.author_token:
            self.skipTest("Author login failed")
        content = b"GIF89a test"
        s, b = self._multipart_upload("image.gif", content, "image/gif", self.author_token)
        self.assertIn(s, [400, 422])

    def test_05_re_validate_ready_media_fails(self):
        """Cannot re-validate a media asset that is already in 'ready' status."""
        uuid = getattr(self.__class__, "uploaded_media_uuid", None)
        if not uuid:
            self.skipTest("No uploaded media")
        s, b = api_request("POST", f"/api/v1/courses/media/{uuid}/validate", token=self.author_token)
        self.assertEqual(s, 400)


if __name__ == "__main__":
    unittest.main()
