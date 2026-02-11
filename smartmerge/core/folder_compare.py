"""Folder/directory comparison logic."""

from pathlib import Path
from enum import Enum
from typing import NamedTuple, Optional
from .diff_engine import myers_opcodes


class ItemType(Enum):
    """Type of item in folder."""

    FILE = "file"
    FOLDER = "folder"


class ItemStatus(Enum):
    """Status of item relative to both sides."""

    IDENTICAL = "identical"
    MODIFIED = "modified"
    ADDED_LEFT = "added_left"
    ADDED_RIGHT = "added_right"
    DELETED_LEFT = "deleted_left"
    DELETED_RIGHT = "deleted_right"
    FOLDER_LEFT_ONLY = "folder_left_only"
    FOLDER_RIGHT_ONLY = "folder_right_only"


class FolderItem(NamedTuple):
    """Represents a file or folder in comparison."""

    name: str
    type: ItemType
    status: ItemStatus
    left_path: Optional[Path]
    right_path: Optional[Path]


def compare_folders(left_path: Path, right_path: Path) -> list[FolderItem]:
    """
    Compare two folders at root level (non-recursive).

    Returns list of FolderItem entries with status for each item.
    Items present only on one side will have None for the missing path.
    """
    if not left_path.is_dir() or not right_path.is_dir():
        return []

    # Get items in each folder
    left_items = {p.name: p for p in left_path.iterdir()}
    right_items = {p.name: p for p in right_path.iterdir()}

    all_names = set(left_items.keys()) | set(right_items.keys())
    results = []

    for name in sorted(all_names):
        left_p = left_items.get(name)
        right_p = right_items.get(name)

        # Both sides exist
        if left_p and right_p:
            if left_p.is_dir() and right_p.is_dir():
                # Both are folders - always identical (no recursive compare)
                status = ItemStatus.IDENTICAL
                item_type = ItemType.FOLDER
            elif left_p.is_file() and right_p.is_file():
                # Both are files - compare contents
                item_type = ItemType.FILE
                status = _compare_files(left_p, right_p)
            else:
                # One is file, one is folder - treat as modified
                status = ItemStatus.MODIFIED
                item_type = ItemType.FILE

            results.append(
                FolderItem(
                    name=name,
                    type=item_type,
                    status=status,
                    left_path=left_p,
                    right_path=right_p,
                )
            )

        # Only on left
        elif left_p:
            item_type = ItemType.FOLDER if left_p.is_dir() else ItemType.FILE
            status = (
                ItemStatus.FOLDER_LEFT_ONLY
                if left_p.is_dir()
                else ItemStatus.ADDED_LEFT
            )
            results.append(
                FolderItem(
                    name=name,
                    type=item_type,
                    status=status,
                    left_path=left_p,
                    right_path=None,
                )
            )

        # Only on right
        else:
            item_type = ItemType.FOLDER if right_p.is_dir() else ItemType.FILE
            status = (
                ItemStatus.FOLDER_RIGHT_ONLY
                if right_p.is_dir()
                else ItemStatus.ADDED_RIGHT
            )
            results.append(
                FolderItem(
                    name=name,
                    type=item_type,
                    status=status,
                    left_path=None,
                    right_path=right_p,
                )
            )

    return results


def _compare_files(left_path: Path, right_path: Path) -> ItemStatus:
    """Compare two files and return status."""
    try:
        with open(left_path, "r", encoding="utf-8", errors="ignore") as f:
            left_content = f.read()
    except (OSError, IOError):
        return ItemStatus.MODIFIED

    try:
        with open(right_path, "r", encoding="utf-8", errors="ignore") as f:
            right_content = f.read()
    except (OSError, IOError):
        return ItemStatus.MODIFIED

    if left_content == right_content:
        return ItemStatus.IDENTICAL

    # Use Myers diff to check if files differ
    opcodes = myers_opcodes(
        left_content.splitlines(keepends=False),
        right_content.splitlines(keepends=False),
    )

    return ItemStatus.IDENTICAL if not opcodes else ItemStatus.MODIFIED


def is_folder_navigable(item: FolderItem) -> bool:
    """
    Check if a folder item can be navigated into.

    Only folders that exist on both sides can be navigated.
    """
    return (
        item.type == ItemType.FOLDER
        and item.left_path is not None
        and item.right_path is not None
    )
