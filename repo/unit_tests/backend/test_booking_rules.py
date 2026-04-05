"""Specification tests: Python mirrors of Rust booking business rules.
These validate rule logic independently but do not execute the production Rust code."""
import unittest
from datetime import datetime, timedelta

MAX_RESCHEDULES = 2
MAX_ACTIVE_PER_RESOURCE = 2
MAX_ADVANCE_DAYS = 90
BREACH_WINDOW_DAYS = 60
BREACH_THRESHOLD = 3
LATE_CANCEL_HOURS = 2


def validate_booking(start, end, resource_open="07:00", resource_close="22:00", max_hours=4, active_count=0, advance_days=None):
    """Validate booking request rules."""
    errors = []
    if end <= start:
        errors.append("End time must be after start time")
    duration_minutes = (end - start).total_seconds() / 60
    if duration_minutes > max_hours * 60:
        errors.append(f"Maximum booking duration is {max_hours} hours")
    if active_count >= MAX_ACTIVE_PER_RESOURCE:
        errors.append(f"Maximum {MAX_ACTIVE_PER_RESOURCE} active reservations per resource")

    open_t = datetime.strptime(resource_open, "%H:%M").time()
    close_t = datetime.strptime(resource_close, "%H:%M").time()
    if start.time() < open_t or end.time() > close_t:
        errors.append(f"Booking must be within resource hours: {resource_open} - {resource_close}")

    if advance_days is not None and advance_days > MAX_ADVANCE_DAYS:
        errors.append(f"Cannot book more than {MAX_ADVANCE_DAYS} days in advance")

    return errors


def has_time_conflict(new_start, new_end, existing_bookings):
    for (s, e) in existing_bookings:
        if s < new_end and e > new_start:
            return True
    return False


def is_late_cancellation(booking_start, cancel_time):
    hours_until = (booking_start - cancel_time).total_seconds() / 3600
    return 0 < hours_until < LATE_CANCEL_HOURS


def should_auto_restrict(breach_count):
    return breach_count >= BREACH_THRESHOLD


class TestBookingConflict(unittest.TestCase):
    def test_no_conflict(self):
        existing = [(datetime(2025, 6, 1, 10, 0), datetime(2025, 6, 1, 11, 0))]
        self.assertFalse(has_time_conflict(datetime(2025, 6, 1, 12, 0), datetime(2025, 6, 1, 13, 0), existing))

    def test_overlap_start(self):
        existing = [(datetime(2025, 6, 1, 10, 0), datetime(2025, 6, 1, 12, 0))]
        self.assertTrue(has_time_conflict(datetime(2025, 6, 1, 11, 0), datetime(2025, 6, 1, 13, 0), existing))

    def test_overlap_end(self):
        existing = [(datetime(2025, 6, 1, 12, 0), datetime(2025, 6, 1, 14, 0))]
        self.assertTrue(has_time_conflict(datetime(2025, 6, 1, 11, 0), datetime(2025, 6, 1, 13, 0), existing))

    def test_contained(self):
        existing = [(datetime(2025, 6, 1, 9, 0), datetime(2025, 6, 1, 17, 0))]
        self.assertTrue(has_time_conflict(datetime(2025, 6, 1, 10, 0), datetime(2025, 6, 1, 12, 0), existing))

    def test_exact_match(self):
        existing = [(datetime(2025, 6, 1, 10, 0), datetime(2025, 6, 1, 12, 0))]
        self.assertTrue(has_time_conflict(datetime(2025, 6, 1, 10, 0), datetime(2025, 6, 1, 12, 0), existing))

    def test_adjacent_no_conflict(self):
        existing = [(datetime(2025, 6, 1, 10, 0), datetime(2025, 6, 1, 12, 0))]
        self.assertFalse(has_time_conflict(datetime(2025, 6, 1, 12, 0), datetime(2025, 6, 1, 14, 0), existing))


class TestBookingValidation(unittest.TestCase):
    def test_valid_booking(self):
        start = datetime(2025, 6, 1, 10, 0)
        end = datetime(2025, 6, 1, 12, 0)
        self.assertEqual(validate_booking(start, end), [])

    def test_end_before_start(self):
        errors = validate_booking(datetime(2025, 6, 1, 14, 0), datetime(2025, 6, 1, 10, 0))
        self.assertTrue(any("End time" in e for e in errors))

    def test_exceeds_4_hour_cap(self):
        start = datetime(2025, 6, 1, 8, 0)
        end = datetime(2025, 6, 1, 13, 0)
        errors = validate_booking(start, end, max_hours=4)
        self.assertTrue(any("Maximum booking duration" in e for e in errors))

    def test_exactly_4_hours_ok(self):
        start = datetime(2025, 6, 1, 8, 0)
        end = datetime(2025, 6, 1, 12, 0)
        self.assertEqual(validate_booking(start, end, max_hours=4), [])

    def test_4h01m_fails(self):
        start = datetime(2025, 6, 1, 8, 0)
        end = datetime(2025, 6, 1, 12, 1)
        errors = validate_booking(start, end, max_hours=4)
        self.assertTrue(any("Maximum booking duration" in e for e in errors))

    def test_3h59m_passes(self):
        start = datetime(2025, 6, 1, 8, 0)
        end = datetime(2025, 6, 1, 11, 59)
        self.assertEqual(validate_booking(start, end, max_hours=4), [])

    def test_outside_operating_hours_early(self):
        start = datetime(2025, 6, 1, 5, 0)
        end = datetime(2025, 6, 1, 7, 0)
        errors = validate_booking(start, end, resource_open="07:00")
        self.assertTrue(any("resource hours" in e for e in errors))

    def test_outside_operating_hours_late(self):
        start = datetime(2025, 6, 1, 21, 0)
        end = datetime(2025, 6, 1, 23, 0)
        errors = validate_booking(start, end, resource_close="22:00")
        self.assertTrue(any("resource hours" in e for e in errors))

    def test_max_active_exceeded(self):
        start = datetime(2025, 6, 1, 10, 0)
        end = datetime(2025, 6, 1, 11, 0)
        errors = validate_booking(start, end, active_count=2)
        self.assertTrue(any("active reservations" in e for e in errors))

    def test_90_day_advance_limit(self):
        start = datetime(2025, 6, 1, 10, 0)
        end = datetime(2025, 6, 1, 11, 0)
        errors = validate_booking(start, end, advance_days=91)
        self.assertTrue(any("90 days" in e for e in errors))

    def test_89_days_ok(self):
        start = datetime(2025, 6, 1, 10, 0)
        end = datetime(2025, 6, 1, 11, 0)
        self.assertEqual(validate_booking(start, end, advance_days=89), [])


class TestRescheduleRules(unittest.TestCase):
    def test_reschedule_allowed(self):
        self.assertTrue(0 < MAX_RESCHEDULES)
        self.assertTrue(1 < MAX_RESCHEDULES)

    def test_reschedule_exceeded(self):
        self.assertFalse(2 < MAX_RESCHEDULES)
        self.assertFalse(3 < MAX_RESCHEDULES)


class TestBreachGeneration(unittest.TestCase):
    def test_late_cancel_within_2_hours(self):
        booking_start = datetime(2025, 6, 1, 10, 0)
        cancel_time = datetime(2025, 6, 1, 9, 0)  # 1 hour before
        self.assertTrue(is_late_cancellation(booking_start, cancel_time))

    def test_cancel_3_hours_before_ok(self):
        booking_start = datetime(2025, 6, 1, 10, 0)
        cancel_time = datetime(2025, 6, 1, 7, 0)
        self.assertFalse(is_late_cancellation(booking_start, cancel_time))

    def test_cancel_exactly_2_hours_not_late(self):
        booking_start = datetime(2025, 6, 1, 10, 0)
        cancel_time = datetime(2025, 6, 1, 8, 0)
        self.assertFalse(is_late_cancellation(booking_start, cancel_time))

    def test_cancel_after_start_not_late(self):
        booking_start = datetime(2025, 6, 1, 10, 0)
        cancel_time = datetime(2025, 6, 1, 11, 0)
        self.assertFalse(is_late_cancellation(booking_start, cancel_time))


class TestAutoRestriction(unittest.TestCase):
    def test_3_breaches_triggers(self):
        self.assertTrue(should_auto_restrict(3))

    def test_4_breaches_triggers(self):
        self.assertTrue(should_auto_restrict(4))

    def test_2_breaches_no_trigger(self):
        self.assertFalse(should_auto_restrict(2))

    def test_0_breaches_no_trigger(self):
        self.assertFalse(should_auto_restrict(0))


def reschedule_conflict_check(booking_id, resource_id, new_start, new_end, existing_bookings):
    """Mirror of reschedule_booking_atomic conflict predicate.

    existing_bookings: list of (id, start, end) tuples with status in
    ('confirmed', 'pending').  Excludes the booking being rescheduled (id ==
    booking_id) before checking overlap — mirrors the SQL:
        WHERE resource_id = ? AND id != ? AND status IN (...) AND start_time < ? AND end_time > ?
    """
    for (bid, s, e) in existing_bookings:
        if bid == booking_id:
            continue  # exclude self
        if s < new_end and e > new_start:
            return True
    return False


def can_reschedule(status, reschedule_count):
    """Pre-conditions checked in booking_service before hitting the DB."""
    if status != "confirmed":
        return False, "Only confirmed bookings can be rescheduled"
    if reschedule_count >= MAX_RESCHEDULES:
        return False, f"Maximum {MAX_RESCHEDULES} reschedules allowed"
    return True, None


class TestRescheduleAtomicity(unittest.TestCase):
    """Documents the conflict-detection logic used in reschedule_booking_atomic."""

    def setUp(self):
        self.t = lambda h, m: datetime(2026, 4, 10, h, m, 0)

    def test_reschedule_conflict_detection_excludes_self(self):
        """The booking being rescheduled must not count as a conflict with itself."""
        booking_id = 42
        new_start = self.t(10, 0)
        new_end = self.t(11, 0)
        # Only entry is the booking itself — should be no conflict
        existing = [(42, self.t(9, 0), self.t(11, 30))]
        self.assertFalse(reschedule_conflict_check(booking_id, 1, new_start, new_end, existing))

    def test_reschedule_conflict_with_other_booking_detected(self):
        """An overlapping booking with a different ID must block the reschedule."""
        booking_id = 42
        new_start = self.t(10, 0)
        new_end = self.t(11, 0)
        existing = [
            (42, self.t(9, 0), self.t(10, 30)),   # self — excluded
            (99, self.t(10, 30), self.t(11, 30)),  # other, overlaps new slot
        ]
        self.assertTrue(reschedule_conflict_check(booking_id, 1, new_start, new_end, existing))

    def test_max_reschedules_boundary(self):
        """At MAX_RESCHEDULES the request is rejected before touching the DB."""
        ok, _ = can_reschedule("confirmed", MAX_RESCHEDULES - 1)
        self.assertTrue(ok)
        ok, msg = can_reschedule("confirmed", MAX_RESCHEDULES)
        self.assertFalse(ok)
        self.assertIn("Maximum", msg)

    def test_reschedule_requires_confirmed_status(self):
        """Only bookings in 'confirmed' status can be rescheduled."""
        for bad_status in ("pending", "cancelled", "completed"):
            ok, msg = can_reschedule(bad_status, 0)
            self.assertFalse(ok, f"Expected rejection for status={bad_status}")
            self.assertIn("confirmed", msg)
        ok, _ = can_reschedule("confirmed", 0)
        self.assertTrue(ok)


def determine_initial_status(requires_approval):
    """Mirror of booking_service logic for determining initial booking status."""
    return "pending" if requires_approval else "confirmed"


def should_apply_late_cancel_breach(booking_status):
    """Late cancellation breach only applies to confirmed bookings."""
    return booking_status == "confirmed"


class TestBookingApprovalRules(unittest.TestCase):
    def test_approval_required_returns_pending(self):
        self.assertEqual(determine_initial_status(True), "pending")

    def test_no_approval_returns_confirmed(self):
        self.assertEqual(determine_initial_status(False), "confirmed")

    def test_late_cancel_applies_to_confirmed(self):
        self.assertTrue(should_apply_late_cancel_breach("confirmed"))

    def test_late_cancel_not_applied_to_pending(self):
        self.assertFalse(should_apply_late_cancel_breach("pending"))

    def test_late_cancel_not_applied_to_cancelled(self):
        self.assertFalse(should_apply_late_cancel_breach("cancelled"))


if __name__ == "__main__":
    unittest.main()
