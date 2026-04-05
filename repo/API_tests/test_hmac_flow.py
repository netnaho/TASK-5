"""API integration tests for HMAC key management flow."""
import os
import unittest
import urllib.request
import json
import uuid as uuid_mod

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


def _reset_reauth_timestamps(*usernames):
    import subprocess
    names = "', '".join(usernames)
    subprocess.run(
        ["docker", "exec", "campuslearn-mysql", "mysql", "-ucampus", "-pcampus_pass",
         "campus_learn", "-e",
         f"UPDATE users SET last_reauth_at=NULL WHERE username IN ('{names}');"],
        capture_output=True,
    )


class TestHmacKeyCreation(unittest.TestCase):
    """HMAC key creation requires ReauthAdminGuard (admin + recent reauth)."""

    @classmethod
    def setUpClass(cls):
        # Reset so last_reauth_at is NULL — guard must then return 403
        _reset_reauth_timestamps("admin")
        # Fresh admin token -- no reauth performed
        cls.admin_token = get_token("admin", "Admin@12345678")
        # Non-admin token for role check
        cls.author_token = get_token("author", "Author@1234567")

    def test_01_create_hmac_key_requires_reauth(self):
        """POST /api/v1/auth/hmac-keys without reauth returns 403."""
        if not self.admin_token:
            self.skipTest("Admin login failed")
        key_id = f"test-key-{uuid_mod.uuid4().hex[:8]}"
        s, b = api_request("POST", "/api/v1/auth/hmac-keys", {
            "key_id": key_id,
            "secret": "supersecret123",
            "description": "Test HMAC key",
        }, self.admin_token)
        self.assertEqual(s, 403)

    def test_02_create_hmac_key_after_reauth(self):
        """Admin who has reauthenticated can create an HMAC key."""
        if not self.admin_token:
            self.skipTest("Admin login failed")
        # Perform reauth
        s, _ = api_request("POST", "/api/v1/auth/reauth",
            {"password": "Admin@12345678"}, self.admin_token)
        if s != 200:
            self.skipTest("Reauth failed")
        key_id = f"test-key-{uuid_mod.uuid4().hex[:8]}"
        s, b = api_request("POST", "/api/v1/auth/hmac-keys", {
            "key_id": key_id,
            "secret": "supersecret456",
            "description": "Test HMAC key after reauth",
        }, self.admin_token)
        self.assertEqual(s, 200)
        self.assertTrue(b["success"])
        self.assertEqual(b["data"]["key_id"], key_id)
        self.assertIn("uuid", b["data"])

    def test_03_non_admin_cannot_create_hmac_key(self):
        """Non-admin user cannot create HMAC keys even with reauth."""
        if not self.author_token:
            self.skipTest("Author login failed")
        # Perform reauth as author
        s, _ = api_request("POST", "/api/v1/auth/reauth",
            {"password": "Author@1234567"}, self.author_token)
        if s != 200:
            self.skipTest("Author reauth failed")
        key_id = f"test-key-{uuid_mod.uuid4().hex[:8]}"
        s, b = api_request("POST", "/api/v1/auth/hmac-keys", {
            "key_id": key_id,
            "secret": "nosecret",
        }, self.author_token)
        self.assertEqual(s, 403)


if __name__ == "__main__":
    unittest.main()
