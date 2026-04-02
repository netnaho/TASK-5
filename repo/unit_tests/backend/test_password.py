"""Unit tests for password validation logic (mirrors backend rules)."""
import re
import unittest


def validate_password(password: str) -> list[str]:
    """Validate password complexity - mirrors backend/src/auth/password.rs."""
    errors = []
    if len(password) < 12:
        errors.append("Password must be at least 12 characters long")
    if not re.search(r"[A-Z]", password):
        errors.append("Password must contain at least one uppercase letter")
    if not re.search(r"[a-z]", password):
        errors.append("Password must contain at least one lowercase letter")
    if not re.search(r"\d", password):
        errors.append("Password must contain at least one digit")
    if not re.search(r"[^a-zA-Z0-9]", password):
        errors.append("Password must contain at least one special character")
    return errors


class TestPasswordValidation(unittest.TestCase):
    def test_valid_password(self):
        self.assertEqual(validate_password("Admin@12345678"), [])

    def test_too_short(self):
        errors = validate_password("Ab1!")
        self.assertIn("Password must be at least 12 characters long", errors)

    def test_no_uppercase(self):
        errors = validate_password("admin@12345678")
        self.assertIn("Password must contain at least one uppercase letter", errors)

    def test_no_lowercase(self):
        errors = validate_password("ADMIN@12345678")
        self.assertIn("Password must contain at least one lowercase letter", errors)

    def test_no_digit(self):
        errors = validate_password("Admin@abcdefgh")
        self.assertIn("Password must contain at least one digit", errors)

    def test_no_special(self):
        errors = validate_password("Admin12345678a")
        self.assertIn("Password must contain at least one special character", errors)

    def test_all_rules_fail(self):
        errors = validate_password("abc")
        self.assertEqual(len(errors), 4)

    def test_exact_min_length(self):
        self.assertEqual(validate_password("Abcdefg1234!"), [])

    def test_seeded_admin_password(self):
        self.assertEqual(validate_password("Admin@12345678"), [])

    def test_seeded_author_password(self):
        self.assertEqual(validate_password("Author@1234567"), [])

    def test_seeded_reviewer_password(self):
        self.assertEqual(validate_password("Review@1234567"), [])

    def test_seeded_faculty_password(self):
        self.assertEqual(validate_password("Faculty@123456"), [])

    def test_seeded_student_password(self):
        self.assertEqual(validate_password("Student@12345"), [])


if __name__ == "__main__":
    unittest.main()
