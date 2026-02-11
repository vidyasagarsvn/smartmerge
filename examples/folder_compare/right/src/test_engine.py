"""Tests for the diff engine."""

import unittest
from src.engine import (
    compute_diff,
    merge_diffs,
    highlight_changes,
    get_similarity_ratio,
)


class TestDiffEngine(unittest.TestCase):
    def test_compute_diff_identical(self):
        """Test diff of identical texts."""
        text1 = ["line 1", "line 2", "line 3"]
        text2 = ["line 1", "line 2", "line 3"]
        result = compute_diff(text1, text2)
        self.assertTrue(len(result) > 0)

    def test_compute_diff_different(self):
        """Test diff of different texts."""
        text1 = ["line 1", "line 2"]
        text2 = ["line 1", "modified line 2"]
        result = compute_diff(text1, text2)
        self.assertTrue(len(result) > 0)

    def test_get_similarity_ratio(self):
        """Test similarity calculation."""
        text1 = "hello world"
        text2 = "hello world"
        ratio = get_similarity_ratio(text1, text2)
        self.assertEqual(ratio, 1.0)

    def test_merge_diffs(self):
        """Test merging diff opcodes."""
        opcodes1 = [("equal", 0, 1, 0, 1)]
        opcodes2 = [("insert", 1, 1, 1, 2)]
        result = merge_diffs(opcodes1, opcodes2)
        self.assertEqual(len(result), 2)


if __name__ == "__main__":
    unittest.main()
