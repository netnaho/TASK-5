"""Unit tests for version diff generation logic."""
import unittest
import json


def generate_diff(old: dict, new: dict) -> dict:
    """Mirrors backend/src/services/version_service.rs::generate_diff."""
    changes = []

    # Compare top-level fields
    for key in new:
        if key in ("sections", "tags"):
            continue
        if old.get(key) != new.get(key):
            changes.append({
                "field": key,
                "type": "field_changed",
                "old": old.get(key),
                "new": new.get(key),
            })

    # Compare tags
    if old.get("tags") != new.get("tags"):
        changes.append({
            "field": "tags",
            "type": "tags_changed",
            "old": old.get("tags"),
            "new": new.get("tags"),
        })

    # Compare sections
    old_sections = old.get("sections", [])
    new_sections = new.get("sections", [])
    old_uuids = [s["uuid"] for s in old_sections if "uuid" in s]
    new_uuids = [s["uuid"] for s in new_sections if "uuid" in s]

    for sec in new_sections:
        if sec.get("uuid") not in old_uuids:
            changes.append({
                "type": "section_added",
                "section_uuid": sec["uuid"],
                "title": sec.get("title"),
            })

    for sec in old_sections:
        if sec.get("uuid") not in new_uuids:
            changes.append({
                "type": "section_removed",
                "section_uuid": sec["uuid"],
                "title": sec.get("title"),
            })

    for new_sec in new_sections:
        uuid = new_sec.get("uuid")
        old_sec = next((s for s in old_sections if s.get("uuid") == uuid), None)
        if old_sec:
            if old_sec.get("title") != new_sec.get("title"):
                changes.append({
                    "type": "section_title_changed",
                    "section_uuid": uuid,
                    "old_title": old_sec.get("title"),
                    "new_title": new_sec.get("title"),
                })
            old_count = len(old_sec.get("lessons", []))
            new_count = len(new_sec.get("lessons", []))
            if old_count != new_count:
                changes.append({
                    "type": "section_lessons_changed",
                    "section_uuid": uuid,
                    "old_lesson_count": old_count,
                    "new_lesson_count": new_count,
                })

    return {"changes": changes, "total_changes": len(changes)}


class TestVersionDiff(unittest.TestCase):
    def test_no_changes(self):
        v = {"title": "A", "code": "C1", "sections": [], "tags": []}
        diff = generate_diff(v, v)
        self.assertEqual(diff["total_changes"], 0)

    def test_title_change(self):
        old = {"title": "Old", "code": "C1", "sections": [], "tags": []}
        new = {"title": "New", "code": "C1", "sections": [], "tags": []}
        diff = generate_diff(old, new)
        self.assertEqual(diff["total_changes"], 1)
        self.assertEqual(diff["changes"][0]["type"], "field_changed")
        self.assertEqual(diff["changes"][0]["field"], "title")

    def test_section_added(self):
        old = {"title": "A", "sections": [], "tags": []}
        new = {"title": "A", "sections": [{"uuid": "s1", "title": "New Section", "lessons": []}], "tags": []}
        diff = generate_diff(old, new)
        added = [c for c in diff["changes"] if c["type"] == "section_added"]
        self.assertEqual(len(added), 1)
        self.assertEqual(added[0]["section_uuid"], "s1")

    def test_section_removed(self):
        old = {"title": "A", "sections": [{"uuid": "s1", "title": "Old Section", "lessons": []}], "tags": []}
        new = {"title": "A", "sections": [], "tags": []}
        diff = generate_diff(old, new)
        removed = [c for c in diff["changes"] if c["type"] == "section_removed"]
        self.assertEqual(len(removed), 1)

    def test_section_title_changed(self):
        old = {"title": "A", "sections": [{"uuid": "s1", "title": "Old", "lessons": []}], "tags": []}
        new = {"title": "A", "sections": [{"uuid": "s1", "title": "New", "lessons": []}], "tags": []}
        diff = generate_diff(old, new)
        changed = [c for c in diff["changes"] if c["type"] == "section_title_changed"]
        self.assertEqual(len(changed), 1)

    def test_lesson_count_changed(self):
        old = {"title": "A", "sections": [{"uuid": "s1", "title": "S", "lessons": [{"uuid": "l1"}]}], "tags": []}
        new = {"title": "A", "sections": [{"uuid": "s1", "title": "S", "lessons": [{"uuid": "l1"}, {"uuid": "l2"}]}], "tags": []}
        diff = generate_diff(old, new)
        changed = [c for c in diff["changes"] if c["type"] == "section_lessons_changed"]
        self.assertEqual(len(changed), 1)
        self.assertEqual(changed[0]["old_lesson_count"], 1)
        self.assertEqual(changed[0]["new_lesson_count"], 2)

    def test_tags_changed(self):
        old = {"title": "A", "sections": [], "tags": ["math"]}
        new = {"title": "A", "sections": [], "tags": ["math", "cs"]}
        diff = generate_diff(old, new)
        tag_changes = [c for c in diff["changes"] if c["type"] == "tags_changed"]
        self.assertEqual(len(tag_changes), 1)

    def test_multiple_changes(self):
        old = {"title": "Old", "code": "C1", "description": None, "sections": [{"uuid": "s1", "title": "S1", "lessons": []}], "tags": ["tag1"]}
        new = {"title": "New", "code": "C1", "description": "Desc", "sections": [{"uuid": "s1", "title": "S1 Updated", "lessons": [{"uuid": "l1"}]}, {"uuid": "s2", "title": "S2", "lessons": []}], "tags": ["tag1", "tag2"]}
        diff = generate_diff(old, new)
        self.assertGreater(diff["total_changes"], 3)


if __name__ == "__main__":
    unittest.main()
