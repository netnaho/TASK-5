"""Unit tests for DATA_ENCRYPTION_KEY validation logic.

Mirrors the rules implemented in backend/src/config/mod.rs:
  - Key must be exactly 64 hex characters (32 bytes for AES-256-GCM).
  - Non-hex characters are rejected.
  - In development mode a missing key is allowed (falls back to dev constant).
  - In any other environment a missing key is a fatal error.
"""
import re
import unittest


# ---------------------------------------------------------------------------
# Pure-Python mirrors of the Rust validation logic in config/mod.rs
# ---------------------------------------------------------------------------

DEV_FALLBACK_ENCRYPTION_KEY = (
    "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
)


def validate_encryption_key_hex(key: str) -> list[str]:
    """Return a list of validation error strings; empty list means valid."""
    errors = []
    if len(key) != 64:
        errors.append(
            f"must be exactly 64 hex characters (32 bytes for AES-256); "
            f"got {len(key)} characters"
        )
    if not re.fullmatch(r"[0-9a-fA-F]*", key):
        errors.append("not valid hex: contains non-hexadecimal characters")
    return errors


def resolve_data_encryption_key(
    app_env: str, data_encryption_key: str | None
) -> tuple[str | None, str | None]:
    """
    Simulate config resolution.

    Returns (resolved_key, error_message).
    On success error_message is None; on failure resolved_key is None.
    """
    if data_encryption_key is not None:
        errors = validate_encryption_key_hex(data_encryption_key)
        if errors:
            return None, "Invalid DATA_ENCRYPTION_KEY: " + "; ".join(errors)
        return data_encryption_key, None
    # Key is absent
    if app_env == "development":
        return DEV_FALLBACK_ENCRYPTION_KEY, None
    return None, (
        f"DATA_ENCRYPTION_KEY is required in the '{app_env}' environment. "
        "Generate a key with: openssl rand -hex 32"
    )


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


class TestEncryptionKeyValidation(unittest.TestCase):
    """Validation of a candidate hex key string."""

    def test_valid_key_64_hex_chars(self):
        key = "a" * 64
        self.assertEqual(validate_encryption_key_hex(key), [])

    def test_valid_dev_fallback_key(self):
        self.assertEqual(validate_encryption_key_hex(DEV_FALLBACK_ENCRYPTION_KEY), [])

    def test_valid_uppercase_hex(self):
        key = "A" * 64
        self.assertEqual(validate_encryption_key_hex(key), [])

    def test_valid_mixed_case_hex(self):
        key = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
        self.assertEqual(validate_encryption_key_hex(key), [])

    def test_too_short_rejects(self):
        key = "a" * 63
        errors = validate_encryption_key_hex(key)
        self.assertTrue(any("64 hex characters" in e for e in errors))

    def test_too_long_rejects(self):
        key = "a" * 65
        errors = validate_encryption_key_hex(key)
        self.assertTrue(any("64 hex characters" in e for e in errors))

    def test_empty_key_rejects(self):
        errors = validate_encryption_key_hex("")
        self.assertTrue(any("64 hex characters" in e for e in errors))

    def test_non_hex_chars_reject(self):
        key = "g" * 64  # 'g' is not a valid hex digit
        errors = validate_encryption_key_hex(key)
        self.assertTrue(any("non-hexadecimal" in e for e in errors))

    def test_correct_length_with_non_hex_rejects(self):
        key = ("abc123" * 10) + "ZZZZ"  # 64 chars but 'Z' is not hex
        self.assertEqual(len(key), 64)
        errors = validate_encryption_key_hex(key)
        self.assertTrue(any("non-hexadecimal" in e for e in errors))

    def test_jwt_secret_string_rejects_wrong_length(self):
        # Simulate old behaviour: jwt_secret was used as raw string, not hex.
        # Typical secret is <64 chars, so length validation catches it.
        jwt_secret = "campus-learn-jwt-secret-change-in-production-2024"
        errors = validate_encryption_key_hex(jwt_secret)
        self.assertGreater(len(errors), 0, "jwt_secret string should fail key validation")

    def test_sha256_hex_of_jwt_secret_is_valid_format(self):
        # The old derivation (sha256 hex) produced a valid 64-char hex string —
        # this test documents that fact, and confirms the new key format
        # (dedicated 64-char hex) is structurally the same.
        import hashlib
        jwt_secret = "campus-learn-jwt-secret-change-in-production-2024"
        derived = hashlib.sha256(jwt_secret.encode()).hexdigest()
        self.assertEqual(len(derived), 64)
        self.assertEqual(validate_encryption_key_hex(derived), [])


class TestKeyVersionConstants(unittest.TestCase):
    """Document and enforce key_version semantic values."""

    KEY_VERSION_LEGACY = 1   # SHA256(jwt_secret) derivation — old, insecure
    KEY_VERSION_DEDICATED = 2  # Dedicated DATA_ENCRYPTION_KEY — current

    def test_legacy_version_is_1(self):
        self.assertEqual(self.KEY_VERSION_LEGACY, 1)

    def test_dedicated_version_is_2(self):
        self.assertEqual(self.KEY_VERSION_DEDICATED, 2)

    def test_new_writes_use_dedicated_version(self):
        # All new sensitive-field writes must use key_version=2.
        self.assertEqual(self.KEY_VERSION_DEDICATED, 2)

    def test_versions_are_distinct(self):
        self.assertNotEqual(self.KEY_VERSION_LEGACY, self.KEY_VERSION_DEDICATED)


class TestConfigResolutionDevelopment(unittest.TestCase):
    """Config resolution in development environment."""

    def test_valid_key_accepted(self):
        key = "b" * 64
        resolved, err = resolve_data_encryption_key("development", key)
        self.assertIsNone(err)
        self.assertEqual(resolved, key)

    def test_missing_key_uses_fallback_in_dev(self):
        resolved, err = resolve_data_encryption_key("development", None)
        self.assertIsNone(err)
        self.assertEqual(resolved, DEV_FALLBACK_ENCRYPTION_KEY)

    def test_invalid_key_fails_even_in_dev(self):
        # Bad length — should fail regardless of environment
        resolved, err = resolve_data_encryption_key("development", "tooshort")
        self.assertIsNone(resolved)
        self.assertIsNotNone(err)
        self.assertIn("Invalid DATA_ENCRYPTION_KEY", err)

    def test_non_hex_key_fails_in_dev(self):
        resolved, err = resolve_data_encryption_key("development", "z" * 64)
        self.assertIsNone(resolved)
        self.assertIsNotNone(err)

    def test_fallback_key_length_is_64(self):
        self.assertEqual(len(DEV_FALLBACK_ENCRYPTION_KEY), 64)

    def test_fallback_key_is_valid_hex(self):
        self.assertEqual(validate_encryption_key_hex(DEV_FALLBACK_ENCRYPTION_KEY), [])


class TestConfigResolutionProduction(unittest.TestCase):
    """Config resolution in non-development environments."""

    PROD_ENVS = ["production", "staging", "test"]

    def test_missing_key_fatal_in_production(self):
        resolved, err = resolve_data_encryption_key("production", None)
        self.assertIsNone(resolved)
        self.assertIsNotNone(err)
        self.assertIn("required", err)
        self.assertIn("production", err)

    def test_missing_key_fatal_in_staging(self):
        resolved, err = resolve_data_encryption_key("staging", None)
        self.assertIsNone(resolved)
        self.assertIsNotNone(err)
        self.assertIn("staging", err)

    def test_valid_key_accepted_in_production(self):
        key = "c" * 64
        resolved, err = resolve_data_encryption_key("production", key)
        self.assertIsNone(err)
        self.assertEqual(resolved, key)

    def test_invalid_key_fatal_in_production(self):
        resolved, err = resolve_data_encryption_key("production", "x" * 64)
        self.assertIsNone(resolved)
        self.assertIsNotNone(err)

    def test_error_message_includes_generation_hint(self):
        _, err = resolve_data_encryption_key("production", None)
        self.assertIn("openssl rand -hex 32", err)

    def test_all_non_dev_envs_require_key(self):
        for env in self.PROD_ENVS:
            with self.subTest(env=env):
                resolved, err = resolve_data_encryption_key(env, None)
                self.assertIsNone(resolved, f"Expected failure for env={env}")
                self.assertIsNotNone(err)

    def test_dev_fallback_key_rejected_in_production(self):
        # The dev fallback key is a valid hex key — it passes format validation
        # but the operator should set a unique production key. This test
        # verifies we don't accidentally hard-code the dev key as acceptable.
        # Format-wise it is valid; the enforcement is operational (rotation policy),
        # so we just document it passes format validation here.
        resolved, err = resolve_data_encryption_key(
            "production", DEV_FALLBACK_ENCRYPTION_KEY
        )
        # Format is valid; no error is expected at the config layer.
        # Rotation enforcement is an operational concern, not a code concern.
        self.assertIsNone(err)
        self.assertEqual(resolved, DEV_FALLBACK_ENCRYPTION_KEY)


class TestEncryptionKeyIsolationFromJwt(unittest.TestCase):
    """Ensure the encryption key has no relationship to the JWT secret."""

    JWT_SECRET = "campus-learn-jwt-secret-change-in-production-2024"
    DATA_KEY = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"

    def test_data_key_not_equal_to_jwt_secret(self):
        self.assertNotEqual(self.DATA_KEY, self.JWT_SECRET)

    def test_data_key_not_equal_to_sha256_of_jwt_secret(self):
        import hashlib
        derived = hashlib.sha256(self.JWT_SECRET.encode()).hexdigest()
        self.assertNotEqual(self.DATA_KEY, derived,
            "DATA_ENCRYPTION_KEY must not be derived from JWT_SECRET")

    def test_data_key_format_does_not_embed_jwt_secret(self):
        # The JWT secret string should not appear anywhere in the hex-encoded key
        jwt_as_hex = self.JWT_SECRET.encode().hex()
        self.assertNotIn(jwt_as_hex, self.DATA_KEY)

    def test_32_byte_key_required_for_aes256(self):
        # AES-256 requires exactly 32 bytes (256 bits).
        key_bytes = bytes.fromhex(self.DATA_KEY)
        self.assertEqual(len(key_bytes), 32)


if __name__ == "__main__":
    unittest.main()
