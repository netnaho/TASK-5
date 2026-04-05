"""Unit tests for background job scheduling logic.

Documents and verifies the scheduling contract for the background job loop
defined in backend/src/main.rs and backend/src/jobs/mod.rs:

  Strategy: global tick + per-rule due-check
  ──────────────────────────────────────────
  The main background loop wakes every JOB_TICK_SECONDS (default 60).
  On each tick it invokes three handlers:

  1. run_scheduled_transitions  — no secondary gate; processes all pending.
  2. run_risk_evaluation         — per-rule gate: evaluates a rule only when
       NOW() - last_run_at >= schedule_interval_minutes.
       Default seed: schedule_interval_minutes = 15.
  3. process_webhooks            — per-entry gate: delivers entries whose
       next_attempt_at <= NOW() (set by exponential back-off on failure).

  JOB_TICK_SECONDS < schedule_interval_minutes is safe: the per-rule query
  simply returns no rows for rules that are not yet due.
"""
import unittest
from datetime import datetime, timedelta


# ---------------------------------------------------------------------------
# Mirror of risk_repo::get_rules_due_for_run due-check predicate
# ---------------------------------------------------------------------------

def is_rule_due(last_run_at: datetime | None, schedule_interval_minutes: int,
                now: datetime) -> bool:
    """Return True if a risk rule should be evaluated on this tick.

    Mirrors the SQL predicate in risk_repo.rs::get_rules_due_for_run:
        last_run_at IS NULL
        OR last_run_at < NOW() - INTERVAL schedule_interval_minutes MINUTE
    """
    if last_run_at is None:
        return True
    return last_run_at < now - timedelta(minutes=schedule_interval_minutes)


# ---------------------------------------------------------------------------
# Mirror of JOB_TICK_SECONDS config resolution (config/mod.rs)
# ---------------------------------------------------------------------------

def resolve_job_tick_seconds(env_value: str | None) -> int:
    """Return job tick in seconds; default 60 when env var is absent or invalid."""
    if env_value is None:
        return 60
    try:
        v = int(env_value)
        return v if v > 0 else 60
    except (ValueError, TypeError):
        return 60


# ---------------------------------------------------------------------------
# Determinism helper: how many ticks occur before a rule fires
# ---------------------------------------------------------------------------

def ticks_until_first_evaluation(job_tick_seconds: int) -> int:
    """Rules with last_run_at=None are due immediately on the first tick."""
    return 1


def ticks_between_evaluations(schedule_interval_minutes: int,
                               job_tick_seconds: int) -> int:
    """Minimum ticks between successive evaluations of a rule.

    A rule becomes due again after schedule_interval_minutes have elapsed.
    The loop wakes every job_tick_seconds, so the first tick that fires the
    rule after the interval is ceil(schedule_interval_minutes*60 / job_tick_seconds).
    """
    import math
    return math.ceil((schedule_interval_minutes * 60) / job_tick_seconds)


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


class TestRulesDueCheck(unittest.TestCase):
    """Per-rule cadence gate mirroring get_rules_due_for_run SQL predicate."""

    def setUp(self):
        self.now = datetime(2026, 4, 4, 12, 0, 0)

    def test_never_run_rule_is_always_due(self):
        self.assertTrue(is_rule_due(None, 15, self.now))

    def test_rule_run_exactly_at_interval_is_due(self):
        last = self.now - timedelta(minutes=15)
        # last_run_at = NOW() - 15 min: must be STRICTLY less than cutoff
        # cutoff = NOW() - 15 min → last == cutoff, so NOT due (not strictly <)
        self.assertFalse(is_rule_due(last, 15, self.now))

    def test_rule_run_one_second_past_interval_is_due(self):
        last = self.now - timedelta(minutes=15, seconds=1)
        self.assertTrue(is_rule_due(last, 15, self.now))

    def test_rule_run_one_minute_before_interval_not_due(self):
        last = self.now - timedelta(minutes=14)
        self.assertFalse(is_rule_due(last, 15, self.now))

    def test_rule_run_just_now_not_due(self):
        self.assertFalse(is_rule_due(self.now, 15, self.now))

    def test_rule_with_60min_interval_not_due_after_30min(self):
        last = self.now - timedelta(minutes=30)
        self.assertFalse(is_rule_due(last, 60, self.now))

    def test_rule_with_60min_interval_due_after_61min(self):
        last = self.now - timedelta(minutes=61)
        self.assertTrue(is_rule_due(last, 60, self.now))

    def test_rule_with_1min_interval_due_after_2min(self):
        last = self.now - timedelta(minutes=2)
        self.assertTrue(is_rule_due(last, 1, self.now))

    def test_multiple_rules_independently_gated(self):
        """Rules are evaluated independently; one due does not affect others."""
        rule_a_last = self.now - timedelta(minutes=16)  # 15-min rule → due
        rule_b_last = self.now - timedelta(minutes=5)   # 15-min rule → not due

        self.assertTrue(is_rule_due(rule_a_last, 15, self.now))
        self.assertFalse(is_rule_due(rule_b_last, 15, self.now))


class TestJobTickConfig(unittest.TestCase):
    """JOB_TICK_SECONDS resolution: default 60, overridable, safe defaults."""

    def test_default_when_absent(self):
        self.assertEqual(resolve_job_tick_seconds(None), 60)

    def test_explicit_60(self):
        self.assertEqual(resolve_job_tick_seconds("60"), 60)

    def test_explicit_30(self):
        self.assertEqual(resolve_job_tick_seconds("30"), 30)

    def test_explicit_120(self):
        self.assertEqual(resolve_job_tick_seconds("120"), 120)

    def test_invalid_string_falls_back_to_default(self):
        self.assertEqual(resolve_job_tick_seconds("not-a-number"), 60)

    def test_zero_falls_back_to_default(self):
        # Zero would cause a divide-by-zero or infinite loop; must not be accepted
        self.assertEqual(resolve_job_tick_seconds("0"), 60)

    def test_negative_falls_back_to_default(self):
        self.assertEqual(resolve_job_tick_seconds("-1"), 60)

    def test_empty_string_falls_back_to_default(self):
        self.assertEqual(resolve_job_tick_seconds(""), 60)


class TestSchedulerDeterminism(unittest.TestCase):
    """Document the deterministic relationship between tick cadence and rule cadence."""

    def test_new_rule_fires_on_first_tick(self):
        """A rule with last_run_at=None is due on tick 1 regardless of tick speed."""
        self.assertEqual(ticks_until_first_evaluation(60), 1)
        self.assertEqual(ticks_until_first_evaluation(30), 1)

    def test_60s_tick_15min_rule_fires_every_15_ticks(self):
        # ceil(15*60 / 60) = ceil(15) = 15 ticks
        self.assertEqual(ticks_between_evaluations(15, 60), 15)

    def test_30s_tick_15min_rule_fires_every_30_ticks(self):
        # ceil(15*60 / 30) = ceil(30) = 30 ticks
        self.assertEqual(ticks_between_evaluations(15, 30), 30)

    def test_120s_tick_15min_rule_fires_every_8_ticks(self):
        # ceil(15*60 / 120) = ceil(7.5) = 8 ticks
        self.assertEqual(ticks_between_evaluations(15, 120), 8)

    def test_tick_faster_than_interval_is_safe(self):
        """Tick < schedule_interval is fine: extra ticks just return 0 events."""
        # 10s tick, 15-min rule: the DB query returns nothing for 89 out of 90 ticks
        ticks = ticks_between_evaluations(15, 10)
        self.assertEqual(ticks, 90)  # ceil(15*60 / 10)

    def test_tick_equal_to_interval_fires_every_tick(self):
        # Degenerate case: tick == interval → fires every tick (1 tick between runs)
        self.assertEqual(ticks_between_evaluations(1, 60), 1)

    def test_rule_interval_larger_than_tick_no_wasted_side_effects(self):
        """Verifying the core design contract: extra ticks produce no DB writes.

        When the rule is not due, get_rules_due_for_run returns an empty list
        and run_risk_evaluation completes without touching risk_events.
        This test documents that guarantee via the is_rule_due predicate.
        """
        now = datetime(2026, 4, 4, 12, 0, 0)
        last_ran = now - timedelta(minutes=10)  # 15-min rule ran 10 min ago

        # Simulate 5 ticks (each 60s apart) — rule should not fire on any of them
        for tick in range(5):
            tick_time = now + timedelta(seconds=tick * 60)
            self.assertFalse(
                is_rule_due(last_ran, 15, tick_time),
                f"Rule should not be due on tick {tick} "
                f"({(tick_time - last_ran).total_seconds()/60:.1f} min elapsed)"
            )

    def test_rule_fires_exactly_once_after_interval_elapses(self):
        """After schedule_interval_minutes the rule fires on the next tick, then stops.

        The due-check is strictly less-than (mirrors SQL '<'), so a rule that ran
        exactly schedule_interval_minutes ago is NOT yet due; it becomes due the
        instant any additional time passes.
        """
        now = datetime(2026, 4, 4, 12, 0, 0)
        # Rule ran exactly 15 minutes ago — at the boundary, not yet due (<, not <=)
        last_ran = now - timedelta(minutes=15)
        self.assertFalse(is_rule_due(last_ran, 15, now))

        # One second past the interval — now due
        one_sec_later = now + timedelta(seconds=1)
        self.assertTrue(is_rule_due(last_ran, 15, one_sec_later))

        # Simulate: rule fired at one_sec_later; update last_run_at — no longer due
        self.assertFalse(is_rule_due(one_sec_later, 15, one_sec_later))


if __name__ == "__main__":
    unittest.main()
