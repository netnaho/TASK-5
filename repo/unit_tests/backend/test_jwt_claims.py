"""Unit tests for JWT claim structure validation."""
import unittest
import json
import base64
import time


def decode_jwt_payload(token: str) -> dict:
    """Decode JWT payload without verification (for structure testing)."""
    parts = token.split(".")
    if len(parts) != 3:
        raise ValueError("Invalid JWT format")
    payload = parts[1]
    # Add padding
    padding = 4 - len(payload) % 4
    if padding != 4:
        payload += "=" * padding
    decoded = base64.urlsafe_b64decode(payload)
    return json.loads(decoded)


class TestJWTClaimStructure(unittest.TestCase):
    # Sample token for structure testing (not cryptographically valid)
    SAMPLE_PAYLOAD = {
        "sub": "550e8400-e29b-41d4-a716-446655440000",
        "username": "admin",
        "role": "admin",
        "exp": int(time.time()) + 86400,
        "iat": int(time.time()),
    }

    def test_required_claims_present(self):
        required = {"sub", "username", "role", "exp", "iat"}
        self.assertTrue(required.issubset(self.SAMPLE_PAYLOAD.keys()))

    def test_sub_is_uuid_format(self):
        sub = self.SAMPLE_PAYLOAD["sub"]
        parts = sub.split("-")
        self.assertEqual(len(parts), 5)

    def test_role_is_valid(self):
        valid_roles = {"admin", "staff", "instructor", "reviewer", "viewer"}
        self.assertIn(self.SAMPLE_PAYLOAD["role"], valid_roles)

    def test_exp_is_future(self):
        self.assertGreater(self.SAMPLE_PAYLOAD["exp"], time.time())

    def test_iat_is_past_or_now(self):
        self.assertLessEqual(self.SAMPLE_PAYLOAD["iat"], time.time() + 1)

    def test_decode_jwt_payload_valid(self):
        # Create a minimal valid-structure JWT
        header = base64.urlsafe_b64encode(json.dumps({"alg": "HS256", "typ": "JWT"}).encode()).rstrip(b"=").decode()
        payload = base64.urlsafe_b64encode(json.dumps(self.SAMPLE_PAYLOAD).encode()).rstrip(b"=").decode()
        token = f"{header}.{payload}.fakesignature"
        decoded = decode_jwt_payload(token)
        self.assertEqual(decoded["sub"], self.SAMPLE_PAYLOAD["sub"])
        self.assertEqual(decoded["role"], "admin")

    def test_decode_jwt_payload_invalid_format(self):
        with self.assertRaises(ValueError):
            decode_jwt_payload("not.a.valid.jwt.token")


if __name__ == "__main__":
    unittest.main()
