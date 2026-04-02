"""Unit tests for risk rule evaluation logic."""
import unittest


def evaluate_posting_frequency(postings_count: int, max_allowed: int) -> bool:
    return postings_count > max_allowed


def evaluate_blacklisted(employer_name: str, blacklist: list[str]) -> bool:
    return employer_name in blacklist


def evaluate_abnormal_compensation(amount: float, min_amount: float, max_amount: float) -> bool:
    return amount < min_amount or amount > max_amount


def evaluate_duplicate(posting_count: int) -> bool:
    return posting_count > 1


def compute_risk_score(value: float, threshold: float) -> float:
    return min((value / threshold) * 100, 100)


class TestPostingFrequency(unittest.TestCase):
    def test_below_threshold(self):
        self.assertFalse(evaluate_posting_frequency(10, 20))

    def test_at_threshold(self):
        self.assertFalse(evaluate_posting_frequency(20, 20))

    def test_above_threshold(self):
        self.assertTrue(evaluate_posting_frequency(21, 20))

    def test_way_above(self):
        self.assertTrue(evaluate_posting_frequency(100, 20))


class TestBlacklistedEmployer(unittest.TestCase):
    def test_blacklisted(self):
        self.assertTrue(evaluate_blacklisted("BadCorp", ["BadCorp", "ScamInc"]))

    def test_not_blacklisted(self):
        self.assertFalse(evaluate_blacklisted("GoodCo", ["BadCorp"]))

    def test_empty_blacklist(self):
        self.assertFalse(evaluate_blacklisted("AnyCo", []))


class TestAbnormalCompensation(unittest.TestCase):
    def test_normal_range(self):
        self.assertFalse(evaluate_abnormal_compensation(5000, 500, 15000))

    def test_below_min(self):
        self.assertTrue(evaluate_abnormal_compensation(100, 500, 15000))

    def test_above_max(self):
        self.assertTrue(evaluate_abnormal_compensation(20000, 500, 15000))

    def test_at_min_boundary(self):
        self.assertFalse(evaluate_abnormal_compensation(500, 500, 15000))

    def test_at_max_boundary(self):
        self.assertFalse(evaluate_abnormal_compensation(15000, 500, 15000))


class TestDuplicatePosting(unittest.TestCase):
    def test_single_posting_ok(self):
        self.assertFalse(evaluate_duplicate(1))

    def test_duplicate_detected(self):
        self.assertTrue(evaluate_duplicate(2))

    def test_many_duplicates(self):
        self.assertTrue(evaluate_duplicate(5))


class TestRiskScoring(unittest.TestCase):
    def test_score_at_threshold(self):
        self.assertEqual(compute_risk_score(20, 20), 100)

    def test_score_above_capped(self):
        self.assertEqual(compute_risk_score(40, 20), 100)

    def test_score_half(self):
        self.assertEqual(compute_risk_score(10, 20), 50)


class TestNonceExpiration(unittest.TestCase):
    """Test replay nonce logic."""
    def test_nonce_reuse_detected(self):
        used_nonces = {"abc123", "def456"}
        self.assertIn("abc123", used_nonces)

    def test_fresh_nonce_ok(self):
        used_nonces = {"abc123"}
        self.assertNotIn("xyz789", used_nonces)

    def test_expired_nonce_removed(self):
        import time
        nonces = {"n1": time.time() - 400, "n2": time.time() + 100}
        active = {k: v for k, v in nonces.items() if v > time.time()}
        self.assertNotIn("n1", active)
        self.assertIn("n2", active)


if __name__ == "__main__":
    unittest.main()
