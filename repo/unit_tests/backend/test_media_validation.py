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


EXTENSION_MAP = {
    "application/pdf": [".pdf"],
    "video/mp4": [".mp4"],
    "image/png": [".png"],
}


def validate_media_content(file_name, mime_type, checksum, file_path):
    """Deterministic content validation matching Rust validate_media logic.

    Returns (status, validated, errors) tuple.
    """
    errors = []
    expected = EXTENSION_MAP.get(mime_type, [])
    if not any(file_name.lower().endswith(ext) for ext in expected):
        errors.append(f"File extension does not match MIME type {mime_type}")
    if not checksum:
        errors.append("Checksum is missing")
    if not file_path.startswith("/"):
        errors.append("File path must be absolute")
    if errors:
        return "failed", False, errors
    return "ready", True, []


def determine_initial_media_status(validated, validation_error):
    """Mirror of course_repo::create_media status derivation."""
    if validation_error:
        return "failed"
    elif validated:
        return "ready"
    else:
        return "pending_scan"


class TestMediaRegistrationDefaults(unittest.TestCase):
    def test_registration_defaults_to_pending_scan(self):
        self.assertEqual(determine_initial_media_status(False, None), "pending_scan")

    def test_validated_true_means_ready(self):
        self.assertEqual(determine_initial_media_status(True, None), "ready")

    def test_validation_error_means_failed(self):
        self.assertEqual(determine_initial_media_status(False, "some error"), "failed")


class TestMediaContentValidation(unittest.TestCase):
    def test_valid_pdf_passes(self):
        status, validated, errors = validate_media_content(
            "document.pdf", "application/pdf", "abc123", "/uploads/document.pdf")
        self.assertEqual(status, "ready")
        self.assertTrue(validated)
        self.assertEqual(errors, [])

    def test_extension_mismatch_fails(self):
        status, validated, errors = validate_media_content(
            "photo.jpg", "application/pdf", "abc123", "/uploads/photo.jpg")
        self.assertEqual(status, "failed")
        self.assertFalse(validated)
        self.assertTrue(any("extension" in e.lower() for e in errors))

    def test_missing_checksum_fails(self):
        status, validated, errors = validate_media_content(
            "document.pdf", "application/pdf", None, "/uploads/document.pdf")
        self.assertEqual(status, "failed")
        self.assertTrue(any("checksum" in e.lower() for e in errors))

    def test_empty_checksum_fails(self):
        status, validated, errors = validate_media_content(
            "document.pdf", "application/pdf", "", "/uploads/document.pdf")
        self.assertEqual(status, "failed")

    def test_relative_path_fails(self):
        status, validated, errors = validate_media_content(
            "document.pdf", "application/pdf", "abc123", "uploads/document.pdf")
        self.assertEqual(status, "failed")
        self.assertTrue(any("absolute" in e.lower() for e in errors))

    def test_multiple_errors(self):
        status, validated, errors = validate_media_content(
            "photo.jpg", "application/pdf", None, "relative/path.jpg")
        self.assertEqual(status, "failed")
        self.assertEqual(len(errors), 3)

    def test_case_insensitive_extension(self):
        status, validated, errors = validate_media_content(
            "DOCUMENT.PDF", "application/pdf", "abc123", "/uploads/DOCUMENT.PDF")
        self.assertEqual(status, "ready")
        self.assertTrue(validated)


if __name__ == "__main__":
    unittest.main()
