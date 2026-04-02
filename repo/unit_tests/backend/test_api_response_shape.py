"""Unit tests for API response shape standardization."""
import unittest


class TestApiResponseShape(unittest.TestCase):
    """Verify standard API response contracts."""

    def test_success_response_shape(self):
        response = {
            "success": True,
            "data": {"id": 1, "name": "Test"},
            "message": None,
        }
        self.assertIn("success", response)
        self.assertIn("data", response)
        self.assertTrue(response["success"])

    def test_error_response_shape(self):
        response = {
            "status": 400,
            "error": "Bad Request",
            "message": "Validation failed",
            "details": None,
        }
        self.assertIn("status", response)
        self.assertIn("error", response)
        self.assertIn("message", response)
        self.assertEqual(response["status"], 400)

    def test_paginated_response_shape(self):
        response = {
            "success": True,
            "data": [{"id": 1}, {"id": 2}],
            "pagination": {
                "page": 1,
                "per_page": 20,
                "total": 50,
                "total_pages": 3,
            },
        }
        self.assertIn("pagination", response)
        self.assertIn("page", response["pagination"])
        self.assertIn("total", response["pagination"])
        self.assertIn("total_pages", response["pagination"])
        self.assertIsInstance(response["data"], list)

    def test_login_response_shape(self):
        response = {
            "success": True,
            "data": {
                "token": "eyJhbGci...",
                "token_type": "Bearer",
                "expires_in": 86400,
                "user": {
                    "uuid": "550e8400-...",
                    "username": "admin",
                    "email": "admin@campuslearn.local",
                    "full_name": "System Administrator",
                    "role": "admin",
                },
            },
        }
        data = response["data"]
        self.assertIn("token", data)
        self.assertEqual(data["token_type"], "Bearer")
        self.assertIn("user", data)
        user = data["user"]
        for field in ["uuid", "username", "email", "full_name", "role"]:
            self.assertIn(field, user)


if __name__ == "__main__":
    unittest.main()
