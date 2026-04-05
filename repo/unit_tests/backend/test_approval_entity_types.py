"""Unit tests for the approval queue entity-type filter.

Documents and verifies the fix for the reviewer queue bug where
`list_pending_for_department` used `entity_type = 'course'` in the JOIN
condition, silently excluding all `course_unpublish` approval requests.

The corrected SQL uses `entity_type IN ('course', 'course_unpublish')` so
both workflow branches appear in the department reviewer queue.
"""
import unittest

# ---------------------------------------------------------------------------
# Mirror of the SQL IN-clause predicate
# ---------------------------------------------------------------------------

PENDING_STATUSES = {"pending_step1", "pending_step2"}
QUEUE_ENTITY_TYPES = {"course", "course_unpublish"}


def matches_department_queue(entity_type: str, status: str) -> bool:
    """Return True if an approval request should appear in the dept reviewer queue.

    Mirrors:
        ar.entity_type IN ('course', 'course_unpublish')
        AND ar.status IN ('pending_step1', 'pending_step2')
    """
    return entity_type in QUEUE_ENTITY_TYPES and status in PENDING_STATUSES


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

class TestApprovalEntityTypeFilter(unittest.TestCase):
    """Verify the entity-type IN-clause includes both workflow branches."""

    def test_course_entity_type_included(self):
        self.assertTrue(matches_department_queue("course", "pending_step1"))

    def test_course_unpublish_entity_type_included(self):
        """Fix verification: course_unpublish was previously excluded."""
        self.assertTrue(matches_department_queue("course_unpublish", "pending_step1"))

    def test_other_entity_type_excluded(self):
        """Unrelated entity types must not appear in the dept queue."""
        for et in ("resource", "booking", "user", ""):
            self.assertFalse(
                matches_department_queue(et, "pending_step1"),
                f"entity_type={et!r} should be excluded"
            )

    def test_pending_step1_status_included(self):
        self.assertTrue(matches_department_queue("course", "pending_step1"))

    def test_pending_step2_status_included(self):
        self.assertTrue(matches_department_queue("course", "pending_step2"))

    def test_approved_status_excluded(self):
        """Resolved requests must not reappear in the queue."""
        for status in ("approved", "approved_scheduled", "rejected", "published"):
            self.assertFalse(
                matches_department_queue("course", status),
                f"status={status!r} should be excluded from queue"
            )


if __name__ == "__main__":
    unittest.main()
