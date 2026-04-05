"""Unit tests for webhook endpoint validation logic.

Mirrors the rules implemented in
backend/src/services/webhook_service.rs::validate_webhook_endpoint:

  - Scheme must be http or https.
  - Host must be localhost, 127.0.0.1, a private IPv4 range, or a bare
    intranet hostname (alphanumeric + hyphens, no dots).
  - Everything else (public hostnames, external IPs, empty URLs) is rejected.

Also covers the subscription-level policy enforced in
backend/src/services/risk_service.rs::create_subscription:

  - channel = "webhook" requires target_url.
  - channel = "in_app" / "email" does not require target_url.
  - signing_secret is always optional.

And mirrors the retry/backoff constants in
backend/src/repositories/webhook_repo.rs::mark_failed:

  - Backoff = 2^attempts * 30 seconds.
  - status -> dead_letter when attempts + 1 >= max_attempts.
"""
import unittest


# ---------------------------------------------------------------------------
# Pure-Python mirror of validate_webhook_endpoint (webhook_service.rs)
# ---------------------------------------------------------------------------

def _parse_ipv4(host: str):
    """Return (a, b, c, d) tuple or None if host is not a valid IPv4 address."""
    parts = host.split(".")
    if len(parts) != 4:
        return None
    try:
        octets = tuple(int(p) for p in parts)
    except ValueError:
        return None
    if any(o < 0 or o > 255 for o in octets):
        return None
    return octets


def _is_approved_onprem_host(host: str) -> bool:
    if host in ("localhost", "127.0.0.1"):
        return True
    ipv4 = _parse_ipv4(host)
    if ipv4 is not None:
        a, b, c, d = ipv4
        if a == 10:
            return True
        if a == 172 and 16 <= b <= 31:
            return True
        if a == 192 and b == 168:
            return True
        return False
    # Bare hostname: alphanumeric + hyphens, no dots
    return bool(host) and "." not in host and all(
        c.isalnum() or c == "-" for c in host
    )


def validate_webhook_endpoint(url: str) -> list[str]:
    """Return a list of validation errors; empty list means valid."""
    errors = []
    if url.startswith("https://"):
        rest = url[len("https://"):]
    elif url.startswith("http://"):
        rest = url[len("http://"):]
    else:
        scheme = url.split("://")[0] if "://" in url else url
        errors.append(
            f"webhook endpoint must use http or https scheme; got '{scheme}'"
        )
        return errors

    if not rest:
        errors.append("webhook endpoint has an empty host")
        return errors

    authority = rest.split("/")[0]
    if not authority:
        errors.append("webhook endpoint has an empty authority")
        return errors

    # Strip port
    host = authority.split(":")[0]
    if not host:
        errors.append("webhook endpoint has an empty host")
        return errors

    if not _is_approved_onprem_host(host):
        errors.append(
            f"'{host}' is not an approved on-prem host; "
            "allowed: localhost, 127.0.0.1, private IP ranges "
            "(10.x, 172.16-31.x, 192.168.x), "
            "or a bare intranet hostname (no dots)"
        )
    return errors


# ---------------------------------------------------------------------------
# Mirror of subscription policy (risk_service.rs::create_subscription)
# ---------------------------------------------------------------------------

def validate_subscription(
    event_type: str,
    channel: str | None,
    target_url: str | None,
    signing_secret: str | None,
) -> list[str]:
    """Return validation errors for a CreateSubscriptionRequest."""
    errors = []
    ch = channel or "in_app"
    if not event_type:
        errors.append("event_type must not be empty")
    if ch == "webhook":
        if target_url is None:
            errors.append("target_url is required for webhook subscriptions")
        else:
            errors.extend(validate_webhook_endpoint(target_url))
    # signing_secret is always optional — no errors possible from its absence
    return errors


# ---------------------------------------------------------------------------
# Mirror of backoff / dead-letter logic (webhook_repo.rs::mark_failed)
# ---------------------------------------------------------------------------

def compute_next_attempt_delay_seconds(current_attempts: int) -> int:
    """2^current_attempts * 30 seconds."""
    return (2 ** current_attempts) * 30


def would_dead_letter(current_attempts: int, max_attempts: int) -> bool:
    """True if incrementing attempts would reach or exceed max_attempts."""
    return current_attempts + 1 >= max_attempts


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


class TestOnPremHostValidation(unittest.TestCase):
    """Validate on-prem endpoint URL forms."""

    def test_localhost_http_valid(self):
        self.assertEqual(validate_webhook_endpoint("http://localhost/hook"), [])

    def test_localhost_https_valid(self):
        self.assertEqual(validate_webhook_endpoint("https://localhost:9090/hook"), [])

    def test_127_0_0_1_valid(self):
        self.assertEqual(validate_webhook_endpoint("http://127.0.0.1:9191/webhooks/risk"), [])

    def test_private_10_block_valid(self):
        self.assertEqual(validate_webhook_endpoint("http://10.0.0.5:8080/hook"), [])

    def test_private_10_block_upper_valid(self):
        self.assertEqual(validate_webhook_endpoint("http://10.255.255.254/hook"), [])

    def test_private_172_16_valid(self):
        self.assertEqual(validate_webhook_endpoint("http://172.16.0.1/hook"), [])

    def test_private_172_31_valid(self):
        self.assertEqual(validate_webhook_endpoint("http://172.31.255.255/hook"), [])

    def test_private_192_168_valid(self):
        self.assertEqual(validate_webhook_endpoint("https://192.168.1.100:3000/webhook"), [])

    def test_bare_hostname_no_dot_valid(self):
        self.assertEqual(validate_webhook_endpoint("http://campuslearn-receiver:9090/hook"), [])

    def test_bare_hostname_single_word_valid(self):
        self.assertEqual(validate_webhook_endpoint("http://internal/hook"), [])

    def test_public_hostname_rejected(self):
        errors = validate_webhook_endpoint("http://example.com/hook")
        self.assertTrue(any("not an approved on-prem host" in e for e in errors))

    def test_external_ip_rejected(self):
        errors = validate_webhook_endpoint("http://8.8.8.8/hook")
        self.assertTrue(any("not an approved on-prem host" in e for e in errors))

    def test_172_15_rejected(self):
        # 172.15.x.x is just outside the 172.16–31 private range
        errors = validate_webhook_endpoint("http://172.15.0.1/hook")
        self.assertTrue(any("not an approved on-prem host" in e for e in errors))

    def test_172_32_rejected(self):
        # 172.32.x.x is just outside the 172.16–31 private range
        errors = validate_webhook_endpoint("http://172.32.0.1/hook")
        self.assertTrue(any("not an approved on-prem host" in e for e in errors))

    def test_ftp_scheme_rejected(self):
        errors = validate_webhook_endpoint("ftp://localhost/hook")
        self.assertTrue(any("http or https" in e for e in errors))

    def test_empty_url_rejected(self):
        errors = validate_webhook_endpoint("")
        self.assertGreater(len(errors), 0)

    def test_url_with_path_and_query_valid(self):
        self.assertEqual(
            validate_webhook_endpoint("http://10.1.2.3:8080/api/webhook?token=abc"),
            [],
        )

    def test_multi_segment_hostname_rejected(self):
        # Internal hostnames with dots look like external FQDNs — reject them
        errors = validate_webhook_endpoint("http://campus.internal/hook")
        self.assertTrue(any("not an approved on-prem host" in e for e in errors))


class TestWebhookSubscriptionPolicy(unittest.TestCase):
    """Subscription-level policy: channel=webhook requires a valid target_url."""

    def test_webhook_channel_requires_target_url(self):
        errors = validate_subscription("risk_high", "webhook", None, None)
        self.assertTrue(any("target_url is required" in e for e in errors))

    def test_webhook_channel_with_valid_url_accepted(self):
        errors = validate_subscription(
            "risk_high", "webhook", "http://localhost:9191/hook", None
        )
        self.assertEqual(errors, [])

    def test_webhook_channel_with_invalid_public_url_rejected(self):
        errors = validate_subscription(
            "risk_high", "webhook", "http://example.com/hook", None
        )
        self.assertTrue(any("not an approved on-prem host" in e for e in errors))

    def test_in_app_channel_no_url_required(self):
        errors = validate_subscription("risk_high", "in_app", None, None)
        self.assertEqual(errors, [])

    def test_email_channel_no_url_required(self):
        errors = validate_subscription("risk_high", "email", None, None)
        self.assertEqual(errors, [])

    def test_default_channel_no_url_required(self):
        # channel=None defaults to in_app
        errors = validate_subscription("risk_high", None, None, None)
        self.assertEqual(errors, [])

    def test_signing_secret_optional_for_webhook(self):
        # Valid webhook subscription without a signing secret is accepted
        errors = validate_subscription(
            "risk_high", "webhook", "http://192.168.1.10:9090/hook", None
        )
        self.assertEqual(errors, [])

    def test_signing_secret_present_for_webhook_accepted(self):
        errors = validate_subscription(
            "risk_high", "webhook", "http://10.0.0.1/hook", "my-secret-key"
        )
        self.assertEqual(errors, [])


class TestWebhookQueueBehavior(unittest.TestCase):
    """Retry backoff and dead-letter promotion logic."""

    def test_backoff_attempt_0_is_30s(self):
        self.assertEqual(compute_next_attempt_delay_seconds(0), 30)

    def test_backoff_attempt_1_is_60s(self):
        self.assertEqual(compute_next_attempt_delay_seconds(1), 60)

    def test_backoff_attempt_2_is_120s(self):
        self.assertEqual(compute_next_attempt_delay_seconds(2), 120)

    def test_backoff_attempt_3_is_240s(self):
        self.assertEqual(compute_next_attempt_delay_seconds(3), 240)

    def test_max_attempts_triggers_dead_letter(self):
        # With max_attempts=3, after the 3rd failure (attempts becomes 3) → dead_letter
        self.assertTrue(would_dead_letter(2, 3))

    def test_below_max_attempts_stays_pending(self):
        self.assertFalse(would_dead_letter(1, 3))

    def test_at_max_attempts_triggers_dead_letter(self):
        self.assertTrue(would_dead_letter(3, 3))


if __name__ == "__main__":
    unittest.main()
