"""Core diff engine for SmartMerge."""

from typing import List, Tuple, Optional
import difflib


def compute_diff(
    text1: List[str], text2: List[str]
) -> List[Tuple[str, int, int, int, int]]:
    """
    Compute diff between two lists of strings.

    Returns list of (tag, i1, i2, j1, j2) tuples where:
    - tag is 'equal', 'replace', 'delete', or 'insert'
    - i1, i2 are indices in text1
    - j1, j2 are indices in text2
    """
    matcher = difflib.SequenceMatcher(None, text1, text2)
    opcodes = []
    for tag, i1, i2, j1, j2 in matcher.get_opcodes():
        opcodes.append((tag, i1, i2, j1, j2))
    return opcodes


def merge_diffs(opcodes1: List, opcodes2: List) -> List:
    """Merge two sets of diff opcodes."""
    return sorted(opcodes1 + opcodes2, key=lambda x: (x[1], x[3]))


def highlight_changes(text: str, changes: List) -> str:
    """Apply highlighting to text based on changes."""
    highlighted = []
    for change in changes:
        highlighted.append(f">> {change}")
    return "\n".join(highlighted) if highlighted else text


def get_similarity_ratio(text1: str, text2: str) -> float:
    """Calculate similarity ratio between two texts."""
    matcher = difflib.SequenceMatcher(None, text1, text2)
    return matcher.ratio()
