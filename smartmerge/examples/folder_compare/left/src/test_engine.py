"""Tests for the diff engine."""

import unittest
from src.engine import compute_diff, merge_diffs, highlight_changes


class TestDiffEngine(unittest.TestCase):
    def test_compute_diff_identical(self):
        text1 = ["line 1", "line 2", "line 3"]
        text2 = ["line 1", "line 2", "line 3"]
        result = compute_diff(text1, text2)
        self.assertTrue(len(result) > 0)

    def test_compute_diff_different(self):
        text1 = ["line 1", "line 2"]
        text2 = ["line 1", "modified line 2"]
        result = compute_diff(text1, text2)
        self.assertTrue(len(result) > 0)


if __name__ == "__main__":
    unittest.main()
