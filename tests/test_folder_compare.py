"""Tests for folder comparison functionality."""

import pytest
import tempfile
from pathlib import Path
from smartmerge.core.folder_compare import (
    compare_folders,
    FolderItem,
    ItemType,
    ItemStatus,
    is_folder_navigable,
)


@pytest.fixture
def test_folders():
    """Create temporary test folder structure."""
    with tempfile.TemporaryDirectory() as tmpdir:
        root = Path(tmpdir)

        # Create left folder
        left = root / "left"
        left.mkdir()
        (left / "file1.txt").write_text("content1")
        (left / "file2.txt").write_text("modified")
        (left / "only_left.txt").write_text("left only")
        (left / "folder1").mkdir()
        (left / "folder_only_left").mkdir()

        # Create right folder
        right = root / "right"
        right.mkdir()
        (right / "file1.txt").write_text("content1")  # identical
        (right / "file2.txt").write_text("different")  # modified
        (right / "only_right.txt").write_text("right only")
        (right / "folder1").mkdir()
        (right / "folder_only_right").mkdir()

        yield left, right


def test_compare_folders_basic(test_folders):
    """Test basic folder comparison."""
    left, right = test_folders
    items = compare_folders(left, right)

    assert len(items) > 0
    assert all(isinstance(item, FolderItem) for item in items)


def test_identical_files_detected(test_folders):
    """Test that identical files are detected correctly."""
    left, right = test_folders
    items = compare_folders(left, right)

    file1_items = [i for i in items if i.name == "file1.txt"]
    assert len(file1_items) == 1
    assert file1_items[0].status == ItemStatus.IDENTICAL
    assert file1_items[0].type == ItemType.FILE


def test_modified_files_detected(test_folders):
    """Test that modified files are detected correctly."""
    left, right = test_folders
    items = compare_folders(left, right)

    file2_items = [i for i in items if i.name == "file2.txt"]
    assert len(file2_items) == 1
    assert file2_items[0].status == ItemStatus.MODIFIED
    assert file2_items[0].type == ItemType.FILE


def test_added_files_detected(test_folders):
    """Test that added files are detected correctly."""
    left, right = test_folders
    items = compare_folders(left, right)

    # File only in left
    left_only = [i for i in items if i.name == "only_left.txt"]
    assert len(left_only) == 1
    assert left_only[0].status == ItemStatus.ADDED_LEFT
    assert left_only[0].right_path is None

    # File only in right
    right_only = [i for i in items if i.name == "only_right.txt"]
    assert len(right_only) == 1
    assert right_only[0].status == ItemStatus.ADDED_RIGHT
    assert right_only[0].left_path is None


def test_folders_with_both_sides(test_folders):
    """Test that folders present on both sides are marked correctly."""
    left, right = test_folders
    items = compare_folders(left, right)

    folder1 = [i for i in items if i.name == "folder1"]
    assert len(folder1) == 1
    assert folder1[0].type == ItemType.FOLDER
    assert folder1[0].status == ItemStatus.IDENTICAL
    assert folder1[0].left_path is not None
    assert folder1[0].right_path is not None


def test_folders_with_only_left(test_folders):
    """Test that folders only on left are detected."""
    left, right = test_folders
    items = compare_folders(left, right)

    folder_left = [i for i in items if i.name == "folder_only_left"]
    assert len(folder_left) == 1
    assert folder_left[0].type == ItemType.FOLDER
    assert folder_left[0].status == ItemStatus.FOLDER_LEFT_ONLY
    assert folder_left[0].right_path is None


def test_folders_with_only_right(test_folders):
    """Test that folders only on right are detected."""
    left, right = test_folders
    items = compare_folders(left, right)

    folder_right = [i for i in items if i.name == "folder_only_right"]
    assert len(folder_right) == 1
    assert folder_right[0].type == ItemType.FOLDER
    assert folder_right[0].status == ItemStatus.FOLDER_RIGHT_ONLY
    assert folder_right[0].left_path is None


def test_is_folder_navigable(test_folders):
    """Test that only bi-present folders are navigable."""
    left, right = test_folders
    items = compare_folders(left, right)

    # Folder on both sides should be navigable
    folder1 = [i for i in items if i.name == "folder1"][0]
    assert is_folder_navigable(folder1) is True

    # Folders on only one side should not be navigable
    folder_left = [i for i in items if i.name == "folder_only_left"][0]
    assert is_folder_navigable(folder_left) is False

    folder_right = [i for i in items if i.name == "folder_only_right"][0]
    assert is_folder_navigable(folder_right) is False

    # Files should not be navigable even on both sides
    file1 = [i for i in items if i.name == "file1.txt"][0]
    assert is_folder_navigable(file1) is False


def test_items_are_sorted(test_folders):
    """Test that items are returned in sorted order."""
    left, right = test_folders
    items = compare_folders(left, right)
    names = [item.name for item in items]

    assert names == sorted(names)
