def diff_files(file1, file2):
    # TODO: Implement actual diff logic
    return []


def myers_opcodes(a, b):
    n = len(a)
    m = len(b)
    maxd = n + m
    v = {0: 0}
    trace = []

    for d in range(maxd + 1):
        v_next = {}
        for k in range(-d, d + 1, 2):
            if k == -d or (k != d and v.get(k - 1, -1) < v.get(k + 1, -1)):
                x = v.get(k + 1, 0)
            else:
                x = v.get(k - 1, 0) + 1
            y = x - k
            while x < n and y < m and a[x] == b[y]:
                x += 1
                y += 1
            v_next[k] = x
            if x >= n and y >= m:
                trace.append(v_next)
                return _build_opcodes_from_trace(trace, n, m)
        trace.append(v_next)
        v = v_next

    return [("equal", 0, n, 0, m)]


def _build_opcodes_from_trace(trace, n, m):
    x = n
    y = m
    edits = []

    for d in range(len(trace) - 1, -1, -1):
        v = trace[d]
        v_prev = trace[d - 1] if d > 0 else {0: 0}
        k = x - y
        if k == -d or (k != d and v.get(k - 1, -1) < v.get(k + 1, -1)):
            k_prev = k + 1
            op = "insert"
        else:
            k_prev = k - 1
            op = "delete"

        x_prev = v_prev.get(k_prev, 0)
        y_prev = x_prev - k_prev

        while x > x_prev and y > y_prev:
            edits.append(("equal", x - 1, x, y - 1, y))
            x -= 1
            y -= 1

        if d == 0:
            break

        if op == "delete":
            edits.append(("delete", x_prev, x_prev + 1, y_prev, y_prev))
        else:
            edits.append(("insert", x_prev, x_prev, y_prev, y_prev + 1))

        x = x_prev
        y = y_prev

    edits.reverse()
    return _merge_opcodes(edits)


def _merge_opcodes(edits):
    if not edits:
        return []

    merged = []
    cur_tag, i1, i2, j1, j2 = edits[0]
    for tag, ni1, ni2, nj1, nj2 in edits[1:]:
        if tag == cur_tag and i2 == ni1 and j2 == nj1:
            i2, j2 = ni2, nj2
        else:
            merged.append((cur_tag, i1, i2, j1, j2))
            cur_tag, i1, i2, j1, j2 = tag, ni1, ni2, nj1, nj2
    merged.append((cur_tag, i1, i2, j1, j2))

    normalized = []
    idx = 0
    while idx < len(merged):
        tag, i1, i2, j1, j2 = merged[idx]
        if tag == "delete" and idx + 1 < len(merged) and merged[idx + 1][0] == "insert":
            _, ni1, ni2, nj1, nj2 = merged[idx + 1]
            if i2 == ni1 and j1 == nj1:
                normalized.append(("replace", i1, i2, j1, nj2))
                idx += 2
                continue
        normalized.append((tag, i1, i2, j1, j2))
        idx += 1

    return normalized


def normalize_opcodes(opcodes, n, m):
    normalized = []
    for tag, i1, i2, j1, j2 in opcodes:
        i1 = max(0, min(i1, n))
        i2 = max(0, min(i2, n))
        j1 = max(0, min(j1, m))
        j2 = max(0, min(j2, m))
        if i2 < i1 or j2 < j1:
            continue
        if tag == "equal" and (i2 - i1) == 0 and (j2 - j1) == 0:
            continue
        normalized.append((tag, i1, i2, j1, j2))
    return normalized
