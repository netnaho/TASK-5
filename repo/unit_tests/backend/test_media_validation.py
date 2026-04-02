"""Unit tests for media validation rules."""
import unittest

ALLOWED_MEDIA_TYPES = ["application/pdf", "video/mp4", "image/png"]
MAX_MEDIA_SIZE_BYTES = 500 * 1024 * 1024  # 500 MB


def validate_media(mime_type: str, file_size_bytes: int) -> list[str]:
    errors = []
    if mime_type not in ALLOWED_MEDIA_TYPES:
        errors.append(f"Invalid media type '{mime_type}'. Allowed: PDF, MP4, PNG")
    if file_size_bytes > MAX_MEDIA_SIZE_BYTES:
        errors.append(f"File size {file_size_bytes} bytes exceeds maximum of 500 MB")
    return errors


class TestMediaValidation(unittest.TestCase):
    def test_valid_pdf(self):
        self.assertEqual(validate_media("application/pdf", 1024), [])

    def test_valid_mp4(self):
        self.assertEqual(validate_media("video/mp4", 100 * 1024 * 1024), [])

    def test_valid_png(self):
        self.assertEqual(validate_media("image/png", 5 * 1024 * 1024), [])

    def test_invalid_type_jpg(self):
        errors = validate_media("image/jpeg", 1024)
        self.assertEqual(len(errors), 1)
        self.assertIn("Invalid media type", errors[0])

    def test_invalid_type_docx(self):
        errors = validate_media("application/vnd.openxmlformats-officedocument.wordprocessingml.document", 1024)
        self.assertEqual(len(errors), 1)

    def test_exceeds_size(self):
        errors = validate_media("application/pdf", 600 * 1024 * 1024)
        self.assertEqual(len(errors), 1)
        self.assertIn("exceeds maximum", errors[0])

    def test_exact_max_size(self):
        self.assertEqual(validate_media("application/pdf", MAX_MEDIA_SIZE_BYTES), [])

    def test_over_max_size_by_one(self):
        errors = validate_media("application/pdf", MAX_MEDIA_SIZE_BYTES + 1)
        self.assertEqual(len(errors), 1)

    def test_both_invalid(self):
        errors = validate_media("image/gif", 600 * 1024 * 1024)
        self.assertEqual(len(errors), 2)


if __name__ == "__main__":
    unittest.main()
