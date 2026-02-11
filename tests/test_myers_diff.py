from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from smartmerge.core.diff_engine import myers_opcodes, normalize_opcodes  # noqa: E402


def _apply_opcodes(left_lines, right_lines, opcodes):
    left_out = []
    right_out = []
    for tag, i1, i2, j1, j2 in opcodes:
        if tag == "equal":
            for i, j in zip(range(i1, i2), range(j1, j2)):
                left_out.append(left_lines[i])
                right_out.append(right_lines[j])
        elif tag == "replace":
            llen = i2 - i1
            rlen = j2 - j1
            maxlen = max(llen, rlen)
            for k in range(maxlen):
                left_out.append(left_lines[i1 + k] if k < llen else "")
                right_out.append(right_lines[j1 + k] if k < rlen else "")
        elif tag == "insert":
            for j in range(j1, j2):
                left_out.append("")
                right_out.append(right_lines[j])
        elif tag == "delete":
            for i in range(i1, i2):
                left_out.append(left_lines[i])
                right_out.append("")
    return left_out, right_out


def test_myers_opcodes_stay_in_bounds():
    left = ["Alpha\n", "Beta\n", "Gamma\n", "Delta\n", "Epsilon\n"]
    right = ["Alpha\n", "Beta\n", "Gamma\n", "DeltaX\n", "Epsilon\n"]

    opcodes = myers_opcodes(left, right)
    opcodes = normalize_opcodes(opcodes, len(left), len(right))

    for tag, i1, i2, j1, j2 in opcodes:
        assert 0 <= i1 <= i2 <= len(left)
        assert 0 <= j1 <= j2 <= len(right)

    left_out, right_out = _apply_opcodes(left, right, opcodes)
    assert len(left_out) == len(right_out)
    assert "".join(left_out).replace("\n", "")
    assert "".join(right_out).replace("\n", "")


def test_myers_opcodes_with_insert_delete():
    left = ["one\n", "two\n", "three\n"]
    right = ["zero\n", "one\n", "three\n", "four\n"]

    opcodes = myers_opcodes(left, right)
    opcodes = normalize_opcodes(opcodes, len(left), len(right))

    left_out, right_out = _apply_opcodes(left, right, opcodes)
    assert len(left_out) == len(right_out)
    assert "one\n" in left_out
    assert "zero\n" in right_out
