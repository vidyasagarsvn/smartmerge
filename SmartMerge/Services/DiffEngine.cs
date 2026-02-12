using System;
using System.Collections.Generic;

namespace SmartMerge.Services
{
    public enum DiffTag
    {
        Equal,
        Replace,
        Delete,
        Insert
    }

    public readonly struct DiffOpcode
    {
        public DiffTag Tag { get; }
        public int I1 { get; }
        public int I2 { get; }
        public int J1 { get; }
        public int J2 { get; }

        public DiffOpcode(DiffTag tag, int i1, int i2, int j1, int j2)
        {
            Tag = tag;
            I1 = i1;
            I2 = i2;
            J1 = j1;
            J2 = j2;
        }
    }

    public static class DiffEngine
    {
        public static List<DiffOpcode> MyersOpcodes(IReadOnlyList<string> a, IReadOnlyList<string> b)
        {
            var n = a.Count;
            var m = b.Count;
            var max = n + m;
            var v = new Dictionary<int, int> { { 0, 0 } };
            var trace = new List<Dictionary<int, int>>();

            for (var d = 0; d <= max; d++)
            {
                var vNext = new Dictionary<int, int>();
                for (var k = -d; k <= d; k += 2)
                {
                    int x;
                    if (k == -d || (k != d && GetValue(v, k - 1, -1) < GetValue(v, k + 1, -1)))
                    {
                        x = GetValue(v, k + 1, 0);
                    }
                    else
                    {
                        x = GetValue(v, k - 1, 0) + 1;
                    }

                    var y = x - k;
                    while (x >= 0 && y >= 0 && x < n && y < m && a[x] == b[y])
                    {
                        x++;
                        y++;
                    }

                    vNext[k] = x;
                    if (x >= n && y >= m)
                    {
                        trace.Add(vNext);
                        return BuildOpcodesFromTrace(trace, n, m);
                    }
                }

                trace.Add(vNext);
                v = vNext;
            }

            return new List<DiffOpcode> { new DiffOpcode(DiffTag.Equal, 0, n, 0, m) };
        }

        public static List<DiffOpcode> NormalizeOpcodes(List<DiffOpcode> opcodes, int n, int m)
        {
            var normalized = new List<DiffOpcode>();
            foreach (var op in opcodes)
            {
                var i1 = Math.Max(0, Math.Min(op.I1, n));
                var i2 = Math.Max(0, Math.Min(op.I2, n));
                var j1 = Math.Max(0, Math.Min(op.J1, m));
                var j2 = Math.Max(0, Math.Min(op.J2, m));
                if (i2 < i1 || j2 < j1)
                {
                    continue;
                }

                if (op.Tag == DiffTag.Equal && (i2 - i1) == 0 && (j2 - j1) == 0)
                {
                    continue;
                }

                normalized.Add(new DiffOpcode(op.Tag, i1, i2, j1, j2));
            }

            return normalized;
        }

        public static List<DiffOpcode> SmartDiff(IReadOnlyList<string> leftLines, IReadOnlyList<string> rightLines)
        {
            var opcodes = new List<DiffOpcode>();
            var i = 0;
            var j = 0;

            while (i < leftLines.Count && j < rightLines.Count)
            {
                if (leftLines[i] == rightLines[j])
                {
                    var matchStartI = i;
                    var matchStartJ = j;
                    while (i < leftLines.Count && j < rightLines.Count && leftLines[i] == rightLines[j])
                    {
                        i++;
                        j++;
                    }

                    if (matchStartI < i)
                    {
                        opcodes.Add(new DiffOpcode(DiffTag.Equal, matchStartI, i, matchStartJ, j));
                    }
                }
                else
                {
                    var leftMatch = FindNextMatch(leftLines, rightLines[j], i);
                    var rightMatch = FindNextMatch(rightLines, leftLines[i], j);

                    if (leftMatch == null && rightMatch == null)
                    {
                        opcodes.Add(new DiffOpcode(DiffTag.Replace, i, i + 1, j, j + 1));
                        i++;
                        j++;
                    }
                    else if (leftMatch != null && (rightMatch == null || leftMatch.Value - i <= rightMatch.Value - j))
                    {
                        opcodes.Add(new DiffOpcode(DiffTag.Delete, i, leftMatch.Value, j, j));
                        i = leftMatch.Value;
                    }
                    else
                    {
                        opcodes.Add(new DiffOpcode(DiffTag.Insert, i, i, j, rightMatch!.Value));
                        j = rightMatch!.Value;
                    }
                }
            }

            if (i < leftLines.Count)
            {
                opcodes.Add(new DiffOpcode(DiffTag.Delete, i, leftLines.Count, j, j));
            }

            if (j < rightLines.Count)
            {
                opcodes.Add(new DiffOpcode(DiffTag.Insert, i, i, j, rightLines.Count));
            }

            return opcodes;
        }

        private static int? FindNextMatch(IReadOnlyList<string> lines, string target, int start)
        {
            for (var idx = start; idx < lines.Count; idx++)
            {
                if (lines[idx] == target)
                {
                    return idx;
                }
            }

            return null;
        }

        private static int GetValue(Dictionary<int, int> dict, int key, int defaultValue)
        {
            return dict.TryGetValue(key, out var value) ? value : defaultValue;
        }

        private static List<DiffOpcode> BuildOpcodesFromTrace(List<Dictionary<int, int>> trace, int n, int m)
        {
            var x = n;
            var y = m;
            var edits = new List<DiffOpcode>();

            for (var d = trace.Count - 1; d >= 0; d--)
            {
                var v = trace[d];
                var vPrev = d > 0 ? trace[d - 1] : new Dictionary<int, int> { { 0, 0 } };
                var k = x - y;

                int kPrev;
                DiffTag op;
                if (k == -d || (k != d && GetValue(v, k - 1, -1) < GetValue(v, k + 1, -1)))
                {
                    kPrev = k + 1;
                    op = DiffTag.Insert;
                }
                else
                {
                    kPrev = k - 1;
                    op = DiffTag.Delete;
                }

                var xPrev = GetValue(vPrev, kPrev, 0);
                var yPrev = xPrev - kPrev;

                while (x > xPrev && y > yPrev)
                {
                    edits.Add(new DiffOpcode(DiffTag.Equal, x - 1, x, y - 1, y));
                    x--;
                    y--;
                }

                if (d == 0)
                {
                    break;
                }

                if (op == DiffTag.Delete)
                {
                    edits.Add(new DiffOpcode(DiffTag.Delete, xPrev, xPrev + 1, yPrev, yPrev));
                }
                else
                {
                    edits.Add(new DiffOpcode(DiffTag.Insert, xPrev, xPrev, yPrev, yPrev + 1));
                }

                x = xPrev;
                y = yPrev;
            }

            edits.Reverse();
            return MergeOpcodes(edits);
        }

        private static List<DiffOpcode> MergeOpcodes(List<DiffOpcode> edits)
        {
            if (edits.Count == 0)
            {
                return new List<DiffOpcode>();
            }

            var merged = new List<DiffOpcode>();
            var current = edits[0];

            for (var i = 1; i < edits.Count; i++)
            {
                var next = edits[i];
                if (next.Tag == current.Tag && current.I2 == next.I1 && current.J2 == next.J1)
                {
                    current = new DiffOpcode(current.Tag, current.I1, next.I2, current.J1, next.J2);
                }
                else
                {
                    merged.Add(current);
                    current = next;
                }
            }

            merged.Add(current);

            var normalized = new List<DiffOpcode>();
            var idx = 0;
            while (idx < merged.Count)
            {
                var op = merged[idx];
                if (op.Tag == DiffTag.Delete && idx + 1 < merged.Count && merged[idx + 1].Tag == DiffTag.Insert)
                {
                    var next = merged[idx + 1];
                    if (op.I2 == next.I1 && op.J1 == next.J1)
                    {
                        normalized.Add(new DiffOpcode(DiffTag.Replace, op.I1, op.I2, op.J1, next.J2));
                        idx += 2;
                        continue;
                    }
                }

                normalized.Add(op);
                idx++;
            }

            return normalized;
        }
    }
}
