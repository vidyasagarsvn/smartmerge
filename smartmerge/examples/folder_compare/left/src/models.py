"""Data models for SmartMerge."""

from dataclasses import dataclass
from typing import List, Optional


@dataclass
class DiffLine:
    """Represents a line in a diff."""

    line_number: int
    content: str
    change_type: str  # 'equal', 'delete', 'insert', 'replace'


@dataclass
class DiffBlock:
    """Represents a block of changes."""

    start_left: int
    start_right: int
    lines: List[DiffLine]


@dataclass
class FileComparison:
    """Result of comparing two files."""

    left_path: str
    right_path: str
    diff_blocks: List[DiffBlock]
    identical: bool


class Repository:
    """Base class for version control repositories."""

    def __init__(self, path: str):
        self.path = path

    def get_diff(self, file1: str, file2: str) -> Optional[DiffBlock]:
        """Get diff between two files."""
        pass
