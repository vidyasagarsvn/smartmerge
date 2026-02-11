"""Tests for smart diff engine."""

import pytest
from smartmerge.core.smart_diff_engine import smart_diff, analyze_similarity


def test_smart_diff_identical_files():
    """Test that identical files produce only equal opcodes."""
    left = ["line1", "line2", "line3"]
    right = ["line1", "line2", "line3"]
    opcodes = smart_diff(left, right)

    assert len(opcodes) == 1
    assert opcodes[0][0] == "equal"
    assert opcodes[0] == ("equal", 0, 3, 0, 3)


def test_smart_diff_simple_insertion():
    """Test insertion detection."""
    left = ["line1", "line2"]
    right = ["line1", "new_line", "line2"]
    opcodes = smart_diff(left, right)

    # Should have: equal, insert, equal
    assert any(op[0] == "equal" for op in opcodes)
    assert any(op[0] == "insert" for op in opcodes)


def test_smart_diff_simple_deletion():
    """Test deletion detection."""
    left = ["line1", "extra_line", "line2"]
    right = ["line1", "line2"]
    opcodes = smart_diff(left, right)

    # Should have: equal, delete, equal
    assert any(op[0] == "equal" for op in opcodes)
    assert any(op[0] == "delete" for op in opcodes)


def test_smart_diff_replacement():
    """Test replacement detection."""
    left = ["line1", "old_line", "line3"]
    right = ["line1", "new_line", "line3"]
    opcodes = smart_diff(left, right)

    assert any(op[0] == "equal" for op in opcodes)
    assert any(op[0] == "replace" for op in opcodes)


def test_smart_diff_empty_files():
    """Test empty file handling."""
    left = []
    right = []
    opcodes = smart_diff(left, right)

    assert len(opcodes) == 0


def test_smart_diff_empty_left():
    """Test empty left file."""
    left = []
    right = ["line1", "line2"]
    opcodes = smart_diff(left, right)

    assert any(op[0] == "insert" for op in opcodes)


def test_smart_diff_empty_right():
    """Test empty right file."""
    left = ["line1", "line2"]
    right = []
    opcodes = smart_diff(left, right)

    assert any(op[0] == "delete" for op in opcodes)


def test_analyze_similarity_identical():
    """Test similarity score for identical files."""
    left = ["line1", "line2", "line3"]
    right = ["line1", "line2", "line3"]
    score = analyze_similarity(left, right)

    assert score == 1.0


def test_analyze_similarity_different():
    """Test similarity score for different files."""
    left = ["line1", "line2"]
    right = ["new1", "new2"]
    score = analyze_similarity(left, right)

    assert score < 1.0
    assert score >= 0.0


def test_analyze_similarity_empty():
    """Test similarity score for empty files."""
    left = []
    right = []
    score = analyze_similarity(left, right)

    assert score == 1.0


def test_smart_diff_preserves_alignment():
    """Test that matching content stays aligned."""
    left = ["a", "b", "c", "d"]
    right = ["a", "x", "c", "d"]
    opcodes = smart_diff(left, right)

    # First line should be equal
    assert opcodes[0][0] == "equal"
    assert opcodes[0] == ("equal", 0, 1, 0, 1)

    # Last lines should be equal
    last_equal = [op for op in opcodes if op[0] == "equal"][-1]
    assert last_equal == ("equal", 2, 4, 2, 4)
