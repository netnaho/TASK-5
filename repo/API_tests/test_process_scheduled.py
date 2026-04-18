"""API integration tests for POST /api/v1/approvals/process-scheduled (HMAC-authenticated)."""
import os
import hmac
import hashlib
import time
import subprocess
import unittest
import urllib.request
import urllib.error
import json
import uuid as uuid_mod

BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8000")

HMAC_KEY_ID = "dev-scheduler-key"
HMAC_SECRET = "campus-learn-hmac-dev-secret-2024"
PATH = "/api/v1/approvals/process-scheduled"


def _cleanup_nonces():
    """Ensure used_nonces does not contain our test nonces; harmless if table empty."""
    subprocess.run(
        ["docker", "exec", "campuslearn-mysql", "mysql", "-ucampus", "-pcampus_pass",
         "campus_learn", "-e",
         "DELETE FROM used_nonces WHERE key_id='dev-scheduler-key' AND nonce LIKE 'api-test-%';"],
        capture_output=True,
    )


def api_request(method, path, data=None, headers=None):
    url = f"{BASE_URL}{path}"
    final_headers = {"Content-Type": "application/json", "Accept": "application/json"}
    if headers:
        final_headers.update(headers)
    body = json.dumps(data).encode() if data else None
    try:
        req = urllib.request.Request(url, data=body, headers=final_headers, method=method)
        with urllib.request.urlopen(req, timeout=15) as resp:
            return resp.status, json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        try:
            body = json.loads(e.read().decode()) if e.fp else {}
        except Exception:
            body = {}
        return e.code, body
    except Exception as e:
        raise ConnectionError(f"Cannot reach {url}: {e}")


def _signing_message(key_id, nonce, timestamp, method, path):
    # Matches backend/src/auth/hmac.rs :: build_signing_message
    return f"{key_id}:{nonce}:{timestamp}:{method}:{path}"


def _sign(secret, message):
    return hmac.new(secret.encode(), message.encode(), hashlib.sha256).hexdigest()


def _hmac_headers(key_id=HMAC_KEY_ID, secret=HMAC_SECRET, nonce=None, timestamp=None, method="POST", path=PATH):
    nonce = nonce or f"api-test-{uuid_mod.uuid4()}"
    timestamp = str(timestamp or int(time.time()))
    message = _signing_message(key_id, nonce, int(timestamp), method, path)
    sig = _sign(secret, message)
    return {
        "X-HMAC-Key-Id": key_id,
        "X-HMAC-Nonce": nonce,
        "X-HMAC-Timestamp": timestamp,
        "X-HMAC-Signature": sig,
    }


class TestProcessScheduledHmac(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        _cleanup_nonces()

    def test_01_missing_headers_unauthorized(self):
        """No HMAC headers → 401."""
        s, b = api_request("POST", "/api/v1/approvals/process-scheduled")
        self.assertEqual(s, 401)
        self.assertEqual(b.get("status"), 401)
        self.assertIn("error", b)
        self.assertIn("message", b)

    def test_02_invalid_signature_unauthorized(self):
        """Valid key/nonce/timestamp but wrong signature → 401."""
        h = _hmac_headers()
        h["X-HMAC-Signature"] = "deadbeef" * 8
        s, b = api_request("POST", PATH, headers=h)
        self.assertEqual(s, 401)
        self.assertEqual(b.get("status"), 401)

    def test_03_expired_timestamp_unauthorized(self):
        """Timestamp older than 5 min window → 401."""
        old_ts = int(time.time()) - 3600  # 1h old
        h = _hmac_headers(timestamp=old_ts)
        s, b = api_request("POST", PATH, headers=h)
        self.assertEqual(s, 401)
        self.assertEqual(b.get("status"), 401)

    def test_04_unknown_key_id_unauthorized(self):
        """Unknown HMAC key id → 401."""
        h = _hmac_headers(key_id="unknown-key", secret="whatever")
        s, b = api_request("POST", PATH, headers=h)
        self.assertEqual(s, 401)
        self.assertEqual(b.get("status"), 401)

    def test_05_valid_hmac_returns_transition_count(self):
        """Valid HMAC headers → 200 with transitions_processed integer."""
        h = _hmac_headers()
        s, b = api_request("POST", PATH, headers=h)
        self.assertEqual(s, 200, msg=f"body={b}")
        self.assertTrue(b.get("success"))
        data = b.get("data") or {}
        self.assertIn("transitions_processed", data)
        self.assertIsInstance(data["transitions_processed"], int)
        self.assertGreaterEqual(data["transitions_processed"], 0)

    def test_06_replay_attack_blocked(self):
        """Reusing a nonce that has already been accepted → 401 (replay prevention)."""
        nonce = f"api-test-{uuid_mod.uuid4()}"
        ts = int(time.time())
        h = _hmac_headers(nonce=nonce, timestamp=ts)
        s1, _ = api_request("POST", PATH, headers=h)
        self.assertEqual(s1, 200)

        # Second request with the same nonce — replay detection kicks in.
        h2 = _hmac_headers(nonce=nonce, timestamp=ts)
        s2, b2 = api_request("POST", PATH, headers=h2)
        self.assertEqual(s2, 401)
        self.assertEqual(b2.get("status"), 401)

    def test_07_jwt_bearer_not_sufficient(self):
        """The endpoint accepts only HMAC; sending a JWT Bearer must still 401 (missing HMAC headers)."""
        login_status, login_body = api_request(
            "POST", "/api/v1/auth/login",
            data={"username": "admin", "password": "Admin@12345678"},
        )
        if login_status != 200:
            self.skipTest("Admin login failed")
        token = login_body["data"]["token"]
        s, b = api_request("POST", PATH, headers={"Authorization": f"Bearer {token}"})
        self.assertEqual(s, 401)


if __name__ == "__main__":
    unittest.main()
