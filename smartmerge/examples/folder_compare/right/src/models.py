"""Data models for SmartMerge."""

from dataclasses import dataclass, field
from typing import List, Optional
from enum import Enum


class ChangeType(Enum):
    """Enumeration of change types."""

    EQUAL = "equal"
    DELETE = "delete"
    INSERT = "insert"
    REPLACE = "replace"


@dataclass
class DiffLine:
    """Represents a line in a diff."""

    line_number: int
    content: str
    change_type: ChangeType


@dataclass
class DiffBlock:
    """Represents a block of changes."""

    start_left: int
    start_right: int
    lines: List[DiffLine] = field(default_factory=list)


@dataclass
class FileComparison:
    """Result of comparing two files."""

    left_path: str
    right_path: str
    diff_blocks: List[DiffBlock] = field(default_factory=list)
    identical: bool = False
    similarity_score: float = 0.0


class Repository:
    """Base class for version control repositories."""

    def __init__(self, path: str):
        self.path = path
        self.name: str = ""

    def get_diff(self, file1: str, file2: str) -> Optional[DiffBlock]:
        """Get diff between two files."""
        pass

    def get_status(self) -> dict:
        """Get repository status."""
        return {}
