"""Smart block-level diff engine for SmartMerge.

This engine compares files at the block level, finding matching sections
and preserving their alignment, making identical content appear at the same
vertical level in the two-pane view.
"""

from typing import List, Tuple


def smart_diff(
    left_lines: List[str], right_lines: List[str]
) -> List[Tuple[str, int, int, int, int]]:
    """
    Perform smart block-level diff that preserves alignment of identical lines.

    Returns list of (tag, i1, i2, j1, j2) tuples similar to SequenceMatcher,
    but optimized for block-level comparison to keep identical content aligned.

    Args:
        left_lines: Lines from left file
        right_lines: Lines from right file

    Returns:
        List of opcodes (tag, i1, i2, j1, j2) where:
        - tag is 'equal', 'replace', 'delete', or 'insert'
        - i1:i2 is range in left_lines
        - j1:j2 is range in right_lines
    """
    opcodes = []
    i = j = 0

    while i < len(left_lines) and j < len(right_lines):
        # Try to find matching lines/blocks from current position
        if left_lines[i] == right_lines[j]:
            # Found matching line - extend the match block
            match_start_i, match_start_j = i, j

            # Extend matching sequence
            while (
                i < len(left_lines)
                and j < len(right_lines)
                and left_lines[i] == right_lines[j]
            ):
                i += 1
                j += 1

            # Add equal block
            if match_start_i < i:
                opcodes.append(("equal", match_start_i, i, match_start_j, j))
        else:
            # Lines don't match - look ahead to find next matching line
            left_match_idx = _find_next_match(left_lines, right_lines[j], start=i)
            right_match_idx = _find_next_match(right_lines, left_lines[i], start=j)

            if left_match_idx is None and right_match_idx is None:
                # No matches ahead - treat as replace and move on
                opcodes.append(("replace", i, i + 1, j, j + 1))
                i += 1
                j += 1
            elif left_match_idx is not None and (
                right_match_idx is None or left_match_idx - i <= right_match_idx - j
            ):
                # Next match is in left file (or closer) - delete from left
                opcodes.append(("delete", i, left_match_idx, j, j))
                i = left_match_idx
            else:
                # Next match is in right file - insert from right
                opcodes.append(("insert", i, i, j, right_match_idx))
                j = right_match_idx

    # Handle remaining lines
    if i < len(left_lines):
        opcodes.append(("delete", i, len(left_lines), j, j))
    if j < len(right_lines):
        opcodes.append(("insert", i, i, j, len(right_lines)))

    return opcodes


def _find_next_match(lines: List[str], target: str, start: int) -> int | None:
    """Find the next occurrence of target starting from start index.

    Returns index if found, None otherwise.
    """
    for idx in range(start, len(lines)):
        if lines[idx] == target:
            return idx
    return None


def analyze_similarity(left_lines: List[str], right_lines: List[str]) -> float:
    """Calculate similarity score between two file contents.

    Returns a score between 0.0 and 1.0 where 1.0 is identical.
    """
    if not left_lines and not right_lines:
        return 1.0
    if not left_lines or not right_lines:
        return 0.0

    opcodes = smart_diff(left_lines, right_lines)

    equal_lines = 0
    for tag, i1, i2, j1, j2 in opcodes:
        if tag == "equal":
            equal_lines += i2 - i1

    total_lines = max(len(left_lines), len(right_lines))
    return equal_lines / total_lines if total_lines > 0 else 0.0
