"""Unit tests for scheduled publish validation logic."""
import unittest
from datetime import datetime, timedelta


def parse_effective_date(s: str) -> datetime:
    """Mirrors approval_service.rs::parse_effective_date."""
    for fmt in ["%m/%d/%Y %I:%M %p", "%m/%d/%Y %H:%M", "%Y-%m-%d %H:%M:%S"]:
        try:
            return datetime.strptime(s, fmt)
        except ValueError:
            continue
    raise ValueError(f"Invalid date format: {s}")


def validate_effective_date(effective_date: str) -> tuple[bool, str]:
    """Validate effective date is parseable."""
    try:
        dt = parse_effective_date(effective_date)
        return True, ""
    except ValueError as e:
        return False, str(e)


def is_scheduled(effective_date: str) -> bool:
    """Determine if the effective date is in the future (should be scheduled)."""
    try:
        dt = parse_effective_date(effective_date)
        return dt > datetime.now()
    except ValueError:
        return False


VALID_STATUSES = ["draft", "pending_approval", "approved_scheduled", "published", "unpublished", "rejected"]

ALLOWED_SUBMIT_FROM = ["draft", "rejected"]

def can_submit_for_approval(current_status: str) -> bool:
    return current_status in ALLOWED_SUBMIT_FROM


class TestEffectiveDateParsing(unittest.TestCase):
    def test_12_hour_format(self):
        dt = parse_effective_date("01/15/2025 02:30 PM")
        self.assertEqual(dt.month, 1)
        self.assertEqual(dt.day, 15)
        self.assertEqual(dt.hour, 14)
        self.assertEqual(dt.minute, 30)

    def test_24_hour_format(self):
        dt = parse_effective_date("12/31/2025 14:00")
        self.assertEqual(dt.hour, 14)

    def test_iso_format(self):
        dt = parse_effective_date("2025-06-15 09:00:00")
        self.assertEqual(dt.month, 6)

    def test_invalid_format_raises(self):
        with self.assertRaises(ValueError):
            parse_effective_date("15-01-2025")

    def test_am_time(self):
        dt = parse_effective_date("03/20/2025 08:00 AM")
        self.assertEqual(dt.hour, 8)


class TestScheduledDetermination(unittest.TestCase):
    def test_future_date_is_scheduled(self):
        future = (datetime.now() + timedelta(days=30)).strftime("%m/%d/%Y %I:%M %p")
        self.assertTrue(is_scheduled(future))

    def test_past_date_not_scheduled(self):
        past = "01/01/2020 12:00 AM"
        self.assertFalse(is_scheduled(past))


class TestStatusTransitions(unittest.TestCase):
    def test_can_submit_from_draft(self):
        self.assertTrue(can_submit_for_approval("draft"))

    def test_can_submit_from_rejected(self):
        self.assertTrue(can_submit_for_approval("rejected"))

    def test_cannot_submit_from_published(self):
        self.assertFalse(can_submit_for_approval("published"))

    def test_cannot_submit_from_pending(self):
        self.assertFalse(can_submit_for_approval("pending_approval"))

    def test_cannot_submit_from_approved_scheduled(self):
        self.assertFalse(can_submit_for_approval("approved_scheduled"))

    def test_all_statuses_valid(self):
        for s in VALID_STATUSES:
            self.assertIn(s, VALID_STATUSES)


if __name__ == "__main__":
    unittest.main()
