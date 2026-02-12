using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Windows.Media;
using SmartMerge.Services;
using DiffEngineService = SmartMerge.Services.DiffEngine;

namespace SmartMerge.ViewModels
{
    public enum DiffEngineType
    {
        Myers,
        Smart
    }

    public enum DiffLineKind
    {
        Unchanged,
        Added,
        Removed,
        Modified
    }

    public class DiffRowViewModel : INotifyPropertyChanged
    {
        private bool _isInCurrentRegion;
        private bool _isSearchMatch;

        public int RenderIndex { get; init; }
        public int LeftIndex { get; init; }
        public int RightIndex { get; init; }
        public string LeftText { get; init; } = string.Empty;
        public string RightText { get; init; } = string.Empty;
        public DiffLineKind LeftKind { get; init; }
        public DiffLineKind RightKind { get; init; }

        public string LeftLineNumber => (RenderIndex + 1).ToString().PadLeft(4) + " |";
        public string RightLineNumber => (RenderIndex + 1).ToString().PadLeft(4) + " |";

        public bool IsInCurrentRegion
        {
            get => _isInCurrentRegion;
            set
            {
                if (_isInCurrentRegion == value)
                {
                    return;
                }

                _isInCurrentRegion = value;
                OnPropertyChanged();
            }
        }

        public bool IsSearchMatch
        {
            get => _isSearchMatch;
            set
            {
                if (_isSearchMatch == value)
                {
                    return;
                }

                _isSearchMatch = value;
                OnPropertyChanged();
            }
        }

        public event PropertyChangedEventHandler? PropertyChanged;

        protected void OnPropertyChanged([CallerMemberName] string? name = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
        }
    }

    public class FileCompareViewModel : INotifyPropertyChanged
    {
        private string? _leftFilePath;
        private string? _rightFilePath;
        private bool _isLeftModified;
        private bool _isRightModified;
        private DiffEngineType _selectedDiffEngine = DiffEngineType.Smart;
        private int _currentRegionIndex = -1;
        private string _theme = "light";
        private FontFamily _diffFontFamily = new("Consolas");
        private double _diffFontSize = 13;
        private string? _searchText;
        private bool _searchCaseSensitive;
        private bool _searchInLeft = true;
        private int _searchRowIndex = -1;

        private readonly List<(int Start, int End)> _changeRegions = new();
        private readonly List<((int Start, int End), (int Start, int End))> _regionFileRanges = new();
        private readonly List<string> _leftLines = new();
        private readonly List<string> _rightLines = new();
        private readonly List<(List<string> Left, List<string> Right, int RegionIndex)> _undoStack = new();
        private readonly List<(List<string> Left, List<string> Right, int RegionIndex)> _redoStack = new();

        public ObservableCollection<DiffRowViewModel> Rows { get; } = new();

        public event EventHandler<int>? ScrollToRowRequested;

        public string? LeftFilePath
        {
            get => _leftFilePath;
            set
            {
                if (_leftFilePath == value)
                {
                    return;
                }

                _leftFilePath = value;
                OnPropertyChanged();
                OnPropertyChanged(nameof(LeftFileLabel));
            }
        }

        public string? RightFilePath
        {
            get => _rightFilePath;
            set
            {
                if (_rightFilePath == value)
                {
                    return;
                }

                _rightFilePath = value;
                OnPropertyChanged();
                OnPropertyChanged(nameof(RightFileLabel));
            }
        }

        public string LeftFileLabel => string.IsNullOrWhiteSpace(LeftFilePath)
            ? "No file selected"
            : $"{LeftFilePath}{(IsLeftModified ? " *" : string.Empty)}";

        public string RightFileLabel => string.IsNullOrWhiteSpace(RightFilePath)
            ? "No file selected"
            : $"{RightFilePath}{(IsRightModified ? " *" : string.Empty)}";

        public bool IsLeftModified
        {
            get => _isLeftModified;
            set
            {
                if (_isLeftModified == value)
                {
                    return;
                }

                _isLeftModified = value;
                OnPropertyChanged();
                OnPropertyChanged(nameof(LeftFileLabel));
            }
        }

        public bool IsRightModified
        {
            get => _isRightModified;
            set
            {
                if (_isRightModified == value)
                {
                    return;
                }

                _isRightModified = value;
                OnPropertyChanged();
                OnPropertyChanged(nameof(RightFileLabel));
            }
        }

        public DiffEngineType SelectedDiffEngine
        {
            get => _selectedDiffEngine;
            set
            {
                if (_selectedDiffEngine == value)
                {
                    return;
                }

                _selectedDiffEngine = value;
                OnPropertyChanged();
                RefreshDiff();
            }
        }

        public string Theme
        {
            get => _theme;
            set
            {
                if (_theme == value)
                {
                    return;
                }

                _theme = value;
                OnPropertyChanged();
            }
        }

        public FontFamily DiffFontFamily
        {
            get => _diffFontFamily;
            set
            {
                if (Equals(_diffFontFamily, value))
                {
                    return;
                }

                _diffFontFamily = value;
                OnPropertyChanged();
            }
        }

        public double DiffFontSize
        {
            get => _diffFontSize;
            set
            {
                if (Math.Abs(_diffFontSize - value) < 0.01)
                {
                    return;
                }

                _diffFontSize = value;
                OnPropertyChanged();
            }
        }

        public void LoadLeftFile(string path)
        {
            if (!File.Exists(path))
            {
                return;
            }

            _leftLines.Clear();
            _leftLines.AddRange(File.ReadAllLines(path));
            LeftFilePath = path;
            IsLeftModified = false;
            _undoStack.Clear();
            _redoStack.Clear();
            _currentRegionIndex = -1;
            RefreshDiff();
        }

        public void LoadRightFile(string path)
        {
            if (!File.Exists(path))
            {
                return;
            }

            _rightLines.Clear();
            _rightLines.AddRange(File.ReadAllLines(path));
            RightFilePath = path;
            IsRightModified = false;
            _undoStack.Clear();
            _redoStack.Clear();
            _currentRegionIndex = -1;
            RefreshDiff();
        }

        public void RefreshDiff()
        {
            Rows.Clear();
            _changeRegions.Clear();
            _regionFileRanges.Clear();

            var opcodes = SelectedDiffEngine == DiffEngineType.Smart
                ? DiffEngineService.SmartDiff(_leftLines, _rightLines)
                : DiffEngineService.NormalizeOpcodes(
                    DiffEngineService.MyersOpcodes(_leftLines, _rightLines),
                    _leftLines.Count,
                    _rightLines.Count);

            var renderIndex = 0;
            var leftMap = new List<int>();
            var rightMap = new List<int>();
            var rowData = new List<DiffRowViewModel>();

            foreach (var op in opcodes)
            {
                switch (op.Tag)
                {
                    case DiffTag.Equal:
                        for (var i = 0; i < op.I2 - op.I1; i++)
                        {
                            var leftLine = _leftLines[op.I1 + i];
                            var rightLine = _rightLines[op.J1 + i];
                            rowData.Add(BuildRow(renderIndex, op.I1 + i, op.J1 + i, leftLine, rightLine, DiffLineKind.Unchanged, DiffLineKind.Unchanged));
                            leftMap.Add(op.I1 + i);
                            rightMap.Add(op.J1 + i);
                            renderIndex++;
                        }
                        break;
                    case DiffTag.Replace:
                        var leftBlock = _leftLines.Skip(op.I1).Take(op.I2 - op.I1).ToList();
                        var rightBlock = _rightLines.Skip(op.J1).Take(op.J2 - op.J1).ToList();
                        var subOps = DiffEngineService.NormalizeOpcodes(
                            DiffEngineService.MyersOpcodes(leftBlock, rightBlock),
                            leftBlock.Count,
                            rightBlock.Count);
                        foreach (var sub in subOps)
                        {
                            if (sub.Tag == DiffTag.Equal)
                            {
                                for (var i = 0; i < sub.I2 - sub.I1; i++)
                                {
                                    var leftLine = leftBlock[sub.I1 + i];
                                    var rightLine = rightBlock[sub.J1 + i];
                                    rowData.Add(BuildRow(renderIndex, op.I1 + sub.I1 + i, op.J1 + sub.J1 + i, leftLine, rightLine, DiffLineKind.Unchanged, DiffLineKind.Unchanged));
                                    leftMap.Add(op.I1 + sub.I1 + i);
                                    rightMap.Add(op.J1 + sub.J1 + i);
                                    renderIndex++;
                                }
                            }
                            else if (sub.Tag == DiffTag.Replace)
                            {
                                var leftLen = sub.I2 - sub.I1;
                                var rightLen = sub.J2 - sub.J1;
                                var maxLen = Math.Max(leftLen, rightLen);
                                for (var k = 0; k < maxLen; k++)
                                {
                                    var leftLine = k < leftLen ? leftBlock[sub.I1 + k] : string.Empty;
                                    var rightLine = k < rightLen ? rightBlock[sub.J1 + k] : string.Empty;
                                    var leftIndex = k < leftLen ? op.I1 + sub.I1 + k : -1;
                                    var rightIndex = k < rightLen ? op.J1 + sub.J1 + k : -1;

                                    var leftKind = leftLine == string.Empty && rightLine != string.Empty
                                        ? DiffLineKind.Unchanged
                                        : DiffLineKind.Modified;
                                    var rightKind = rightLine == string.Empty && leftLine != string.Empty
                                        ? DiffLineKind.Unchanged
                                        : DiffLineKind.Modified;

                                    if (leftLine == string.Empty && rightLine != string.Empty)
                                    {
                                        rightKind = DiffLineKind.Added;
                                    }
                                    else if (rightLine == string.Empty && leftLine != string.Empty)
                                    {
                                        leftKind = DiffLineKind.Removed;
                                    }

                                    rowData.Add(BuildRow(renderIndex, leftIndex, rightIndex, leftLine, rightLine, leftKind, rightKind));
                                    leftMap.Add(leftIndex);
                                    rightMap.Add(rightIndex);
                                    renderIndex++;
                                }
                            }
                            else if (sub.Tag == DiffTag.Insert)
                            {
                                for (var k = sub.J1; k < sub.J2; k++)
                                {
                                    var rightLine = rightBlock[k];
                                    rowData.Add(BuildRow(renderIndex, -1, op.J1 + k, string.Empty, rightLine, DiffLineKind.Unchanged, DiffLineKind.Added));
                                    leftMap.Add(-1);
                                    rightMap.Add(op.J1 + k);
                                    renderIndex++;
                                }
                            }
                            else if (sub.Tag == DiffTag.Delete)
                            {
                                for (var k = sub.I1; k < sub.I2; k++)
                                {
                                    var leftLine = leftBlock[k];
                                    rowData.Add(BuildRow(renderIndex, op.I1 + k, -1, leftLine, string.Empty, DiffLineKind.Removed, DiffLineKind.Unchanged));
                                    leftMap.Add(op.I1 + k);
                                    rightMap.Add(-1);
                                    renderIndex++;
                                }
                            }
                        }
                        break;
                    case DiffTag.Insert:
                        for (var k = op.J1; k < op.J2; k++)
                        {
                            var rightLine = _rightLines[k];
                            rowData.Add(BuildRow(renderIndex, -1, k, string.Empty, rightLine, DiffLineKind.Unchanged, DiffLineKind.Added));
                            leftMap.Add(-1);
                            rightMap.Add(k);
                            renderIndex++;
                        }
                        break;
                    case DiffTag.Delete:
                        for (var k = op.I1; k < op.I2; k++)
                        {
                            var leftLine = _leftLines[k];
                            rowData.Add(BuildRow(renderIndex, k, -1, leftLine, string.Empty, DiffLineKind.Removed, DiffLineKind.Unchanged));
                            leftMap.Add(k);
                            rightMap.Add(-1);
                            renderIndex++;
                        }
                        break;
                }
            }

            for (var i = 0; i < rowData.Count; i++)
            {
                if (rowData[i].LeftKind != DiffLineKind.Unchanged || rowData[i].RightKind != DiffLineKind.Unchanged)
                {
                    _changeRegions.Add((i, i));
                }
            }

            _changeRegions.Clear();
            var changeIndices = rowData
                .Select((row, idx) => new { row, idx })
                .Where(item => item.row.LeftKind != DiffLineKind.Unchanged || item.row.RightKind != DiffLineKind.Unchanged)
                .Select(item => item.idx)
                .ToList();

            if (changeIndices.Count > 0)
            {
                var start = changeIndices[0];
                var end = changeIndices[0];
                for (var i = 1; i < changeIndices.Count; i++)
                {
                    if (changeIndices[i] == end + 1)
                    {
                        end = changeIndices[i];
                    }
                    else
                    {
                        _changeRegions.Add((start, end));
                        start = changeIndices[i];
                        end = changeIndices[i];
                    }
                }

                _changeRegions.Add((start, end));
            }

            foreach (var region in _changeRegions)
            {
                var leftIndices = new List<int>();
                var rightIndices = new List<int>();
                for (var i = region.Start; i <= region.End; i++)
                {
                    if (i < leftMap.Count && leftMap[i] >= 0)
                    {
                        leftIndices.Add(leftMap[i]);
                    }
                    if (i < rightMap.Count && rightMap[i] >= 0)
                    {
                        rightIndices.Add(rightMap[i]);
                    }
                }

                var leftRange = leftIndices.Count > 0 ? (leftIndices.Min(), leftIndices.Max()) : (-1, -1);
                var rightRange = rightIndices.Count > 0 ? (rightIndices.Min(), rightIndices.Max()) : (-1, -1);
                _regionFileRanges.Add((leftRange, rightRange));
            }

            foreach (var row in rowData)
            {
                Rows.Add(row);
            }

            if (_changeRegions.Count == 0)
            {
                _currentRegionIndex = -1;
                ClearSelections();
            }
            else if (_currentRegionIndex < 0)
            {
                _currentRegionIndex = 0;
                SelectRegion(_currentRegionIndex);
            }
            else
            {
                SelectRegion(_currentRegionIndex);
            }

            OnPropertyChanged(nameof(StatusInfo));
            OnPropertyChanged(nameof(HasUndo));
            OnPropertyChanged(nameof(HasRedo));
        }

        public void NextChange()
        {
            if (_changeRegions.Count == 0)
            {
                return;
            }

            if (_currentRegionIndex < 0)
            {
                _currentRegionIndex = 0;
            }
            else if (_currentRegionIndex < _changeRegions.Count - 1)
            {
                _currentRegionIndex++;
            }

            SelectRegion(_currentRegionIndex);
        }

        public void PreviousChange()
        {
            if (_changeRegions.Count == 0)
            {
                return;
            }

            if (_currentRegionIndex > 0)
            {
                _currentRegionIndex--;
            }
            else if (_currentRegionIndex < 0)
            {
                _currentRegionIndex = 0;
            }

            SelectRegion(_currentRegionIndex);
        }

        public void CopyToRight()
        {
            if (!EnsureRegion())
            {
                return;
            }

            PushUndo();
            _redoStack.Clear();

            var (leftRange, rightRange) = _regionFileRanges[_currentRegionIndex];
            var region = _changeRegions[_currentRegionIndex];

            if (leftRange.Start >= 0 && leftRange.End >= 0)
            {
                if (rightRange.Start >= 0 && rightRange.End >= 0)
                {
                    var leftBlock = _leftLines.GetRange(leftRange.Start, leftRange.End - leftRange.Start + 1);
                    _rightLines.RemoveRange(rightRange.Start, rightRange.End - rightRange.Start + 1);
                    _rightLines.InsertRange(rightRange.Start, leftBlock);
                }
                else
                {
                    var leftBlock = _leftLines.GetRange(leftRange.Start, leftRange.End - leftRange.Start + 1);
                    var insertPos = _rightLines.Count;
                    for (var i = region.End + 1; i < Rows.Count; i++)
                    {
                        if (Rows[i].RightIndex >= 0)
                        {
                            insertPos = Rows[i].RightIndex;
                            break;
                        }
                    }
                    _rightLines.InsertRange(insertPos, leftBlock);
                }
            }
            else if (rightRange.Start >= 0 && rightRange.End >= 0)
            {
                _rightLines.RemoveRange(rightRange.Start, rightRange.End - rightRange.Start + 1);
            }

            IsRightModified = true;
            RefreshDiff();
        }

        public void CopyToLeft()
        {
            if (!EnsureRegion())
            {
                return;
            }

            PushUndo();
            _redoStack.Clear();

            var (leftRange, rightRange) = _regionFileRanges[_currentRegionIndex];
            var region = _changeRegions[_currentRegionIndex];

            if (rightRange.Start >= 0 && rightRange.End >= 0)
            {
                if (leftRange.Start >= 0 && leftRange.End >= 0)
                {
                    var rightBlock = _rightLines.GetRange(rightRange.Start, rightRange.End - rightRange.Start + 1);
                    _leftLines.RemoveRange(leftRange.Start, leftRange.End - leftRange.Start + 1);
                    _leftLines.InsertRange(leftRange.Start, rightBlock);
                }
                else
                {
                    var rightBlock = _rightLines.GetRange(rightRange.Start, rightRange.End - rightRange.Start + 1);
                    var insertPos = _leftLines.Count;
                    for (var i = region.End + 1; i < Rows.Count; i++)
                    {
                        if (Rows[i].LeftIndex >= 0)
                        {
                            insertPos = Rows[i].LeftIndex;
                            break;
                        }
                    }
                    _leftLines.InsertRange(insertPos, rightBlock);
                }
            }
            else if (leftRange.Start >= 0 && leftRange.End >= 0)
            {
                _leftLines.RemoveRange(leftRange.Start, leftRange.End - leftRange.Start + 1);
            }

            IsLeftModified = true;
            RefreshDiff();
        }

        public void CopyAllToRight()
        {
            if (_leftLines.Count == 0)
            {
                return;
            }

            PushUndo();
            _redoStack.Clear();
            _rightLines.Clear();
            _rightLines.AddRange(_leftLines);
            IsRightModified = true;
            RefreshDiff();
        }

        public void CopyAllToLeft()
        {
            if (_rightLines.Count == 0)
            {
                return;
            }

            PushUndo();
            _redoStack.Clear();
            _leftLines.Clear();
            _leftLines.AddRange(_rightLines);
            IsLeftModified = true;
            RefreshDiff();
        }

        public void Undo()
        {
            if (_undoStack.Count == 0)
            {
                return;
            }

            var current = (new List<string>(_leftLines), new List<string>(_rightLines), _currentRegionIndex);
            _redoStack.Add(current);

            var previous = _undoStack[^1];
            _undoStack.RemoveAt(_undoStack.Count - 1);

            _leftLines.Clear();
            _leftLines.AddRange(previous.Left);
            _rightLines.Clear();
            _rightLines.AddRange(previous.Right);

            _currentRegionIndex = previous.RegionIndex;
            UpdateModifiedFlags();
            RefreshDiff();
        }

        public void Redo()
        {
            if (_redoStack.Count == 0)
            {
                return;
            }

            var current = (new List<string>(_leftLines), new List<string>(_rightLines), _currentRegionIndex);
            _undoStack.Add(current);

            var next = _redoStack[^1];
            _redoStack.RemoveAt(_redoStack.Count - 1);

            _leftLines.Clear();
            _leftLines.AddRange(next.Left);
            _rightLines.Clear();
            _rightLines.AddRange(next.Right);

            _currentRegionIndex = next.RegionIndex;
            UpdateModifiedFlags();
            RefreshDiff();
        }

        public bool SaveLeftFile()
        {
            if (string.IsNullOrWhiteSpace(LeftFilePath))
            {
                return false;
            }

            try
            {
                File.WriteAllLines(LeftFilePath, _leftLines);
                IsLeftModified = false;
                return true;
            }
            catch
            {
                return false;
            }
        }

        public bool SaveRightFile()
        {
            if (string.IsNullOrWhiteSpace(RightFilePath))
            {
                return false;
            }

            try
            {
                File.WriteAllLines(RightFilePath, _rightLines);
                IsRightModified = false;
                return true;
            }
            catch
            {
                return false;
            }
        }

        public string StatusInfo
        {
            get
            {
                if (_changeRegions.Count == 0)
                {
                    return "No differences found";
                }

                var regionInfo = $"Region {_currentRegionIndex + 1}/{_changeRegions.Count}";
                var leftSize = !string.IsNullOrWhiteSpace(LeftFilePath) && File.Exists(LeftFilePath)
                    ? $"Left: {FormatSize(new FileInfo(LeftFilePath).Length)}"
                    : "";
                var rightSize = !string.IsNullOrWhiteSpace(RightFilePath) && File.Exists(RightFilePath)
                    ? $"Right: {FormatSize(new FileInfo(RightFilePath).Length)}"
                    : "";
                var lineInfo = $"Lines: {_leftLines.Count} vs {_rightLines.Count}";

                var parts = new List<string> { regionInfo, lineInfo };
                if (!string.IsNullOrWhiteSpace(leftSize))
                {
                    parts.Add(leftSize);
                }
                if (!string.IsNullOrWhiteSpace(rightSize))
                {
                    parts.Add(rightSize);
                }

                return string.Join(" | ", parts);
            }
        }

        public bool HasUndo => _undoStack.Count > 0;
        public bool HasRedo => _redoStack.Count > 0;

        public void SetSearch(string text, bool caseSensitive)
        {
            _searchText = text;
            _searchCaseSensitive = caseSensitive;
            _searchRowIndex = -1;
            ClearSearchMatches();
        }

        public void FindNext()
        {
            if (string.IsNullOrWhiteSpace(_searchText))
            {
                return;
            }

            var comparer = _searchCaseSensitive ? StringComparison.Ordinal : StringComparison.OrdinalIgnoreCase;
            var startIndex = _searchRowIndex < 0 ? 0 : _searchRowIndex + 1;

            if (TryFindInPane(_searchInLeft, startIndex, comparer, out var foundIndex))
            {
                ApplySearchMatch(foundIndex);
                return;
            }

            if (TryFindInPane(!_searchInLeft, 0, comparer, out foundIndex))
            {
                _searchInLeft = !_searchInLeft;
                ApplySearchMatch(foundIndex);
                return;
            }

            if (TryFindInPane(_searchInLeft, 0, comparer, out foundIndex))
            {
                ApplySearchMatch(foundIndex);
                return;
            }

            if (TryFindInPane(!_searchInLeft, 0, comparer, out foundIndex))
            {
                _searchInLeft = !_searchInLeft;
                ApplySearchMatch(foundIndex);
            }
        }

        public void FindPrevious()
        {
            if (string.IsNullOrWhiteSpace(_searchText))
            {
                return;
            }

            var comparer = _searchCaseSensitive ? StringComparison.Ordinal : StringComparison.OrdinalIgnoreCase;
            var startIndex = _searchRowIndex < 0 ? Rows.Count - 1 : _searchRowIndex - 1;

            if (TryFindInPaneBackward(_searchInLeft, startIndex, comparer, out var foundIndex))
            {
                ApplySearchMatch(foundIndex);
                return;
            }

            if (TryFindInPaneBackward(!_searchInLeft, Rows.Count - 1, comparer, out foundIndex))
            {
                _searchInLeft = !_searchInLeft;
                ApplySearchMatch(foundIndex);
                return;
            }

            if (TryFindInPaneBackward(_searchInLeft, Rows.Count - 1, comparer, out foundIndex))
            {
                ApplySearchMatch(foundIndex);
                return;
            }

            if (TryFindInPaneBackward(!_searchInLeft, Rows.Count - 1, comparer, out foundIndex))
            {
                _searchInLeft = !_searchInLeft;
                ApplySearchMatch(foundIndex);
            }
        }

        public void GoToLine(int lineIndex)
        {
            if (lineIndex < 0 || lineIndex >= Rows.Count)
            {
                return;
            }

            ScrollToRowRequested?.Invoke(this, lineIndex);
        }

        private bool TryFindInPane(bool leftPane, int startIndex, StringComparison comparer, out int foundIndex)
        {
            for (var i = startIndex; i < Rows.Count; i++)
            {
                var text = leftPane ? Rows[i].LeftText : Rows[i].RightText;
                if (!string.IsNullOrEmpty(text) && text.IndexOf(_searchText!, comparer) >= 0)
                {
                    foundIndex = i;
                    return true;
                }
            }

            foundIndex = -1;
            return false;
        }

        private bool TryFindInPaneBackward(bool leftPane, int startIndex, StringComparison comparer, out int foundIndex)
        {
            for (var i = startIndex; i >= 0; i--)
            {
                var text = leftPane ? Rows[i].LeftText : Rows[i].RightText;
                if (!string.IsNullOrEmpty(text) && text.IndexOf(_searchText!, comparer) >= 0)
                {
                    foundIndex = i;
                    return true;
                }
            }

            foundIndex = -1;
            return false;
        }

        private void ApplySearchMatch(int rowIndex)
        {
            ClearSearchMatches();
            _searchRowIndex = rowIndex;
            Rows[rowIndex].IsSearchMatch = true;
            ScrollToRowRequested?.Invoke(this, rowIndex);
        }

        private void ClearSearchMatches()
        {
            foreach (var row in Rows)
            {
                row.IsSearchMatch = false;
            }
        }

        private bool EnsureRegion()
        {
            return _changeRegions.Count > 0 && _currentRegionIndex >= 0 && _currentRegionIndex < _regionFileRanges.Count;
        }

        private void PushUndo()
        {
            _undoStack.Add((new List<string>(_leftLines), new List<string>(_rightLines), _currentRegionIndex));
            if (_undoStack.Count > 50)
            {
                _undoStack.RemoveAt(0);
            }
        }

        private void UpdateModifiedFlags()
        {
            if (!string.IsNullOrWhiteSpace(LeftFilePath) && File.Exists(LeftFilePath))
            {
                var saved = File.ReadAllLines(LeftFilePath);
                IsLeftModified = !_leftLines.SequenceEqual(saved);
            }
            else
            {
                IsLeftModified = _leftLines.Count > 0;
            }

            if (!string.IsNullOrWhiteSpace(RightFilePath) && File.Exists(RightFilePath))
            {
                var saved = File.ReadAllLines(RightFilePath);
                IsRightModified = !_rightLines.SequenceEqual(saved);
            }
            else
            {
                IsRightModified = _rightLines.Count > 0;
            }
        }

        private void SelectRegion(int index)
        {
            ClearSelections();
            if (index < 0 || index >= _changeRegions.Count)
            {
                return;
            }

            var region = _changeRegions[index];
            for (var i = region.Start; i <= region.End && i < Rows.Count; i++)
            {
                Rows[i].IsInCurrentRegion = true;
            }

            ScrollToRowRequested?.Invoke(this, region.Start);
            OnPropertyChanged(nameof(StatusInfo));
        }

        private void ClearSelections()
        {
            foreach (var row in Rows)
            {
                row.IsInCurrentRegion = false;
            }
        }

        private DiffRowViewModel BuildRow(int renderIndex, int leftIndex, int rightIndex, string leftLine, string rightLine, DiffLineKind leftKind, DiffLineKind rightKind)
        {
            if (!string.Equals(leftLine, rightLine, StringComparison.Ordinal) && leftLine.TrimEnd() == rightLine.TrimEnd())
            {
                leftKind = DiffLineKind.Unchanged;
                rightKind = DiffLineKind.Unchanged;
            }

            return new DiffRowViewModel
            {
                RenderIndex = renderIndex,
                LeftIndex = leftIndex,
                RightIndex = rightIndex,
                LeftText = leftLine,
                RightText = rightLine,
                LeftKind = leftKind,
                RightKind = rightKind
            };
        }

        private static string FormatSize(long size)
        {
            var units = new[] { "B", "KB", "MB", "GB", "TB" };
            var value = (double)size;
            var unitIndex = 0;
            while (value >= 1024 && unitIndex < units.Length - 1)
            {
                value /= 1024;
                unitIndex++;
            }

            return $"{value:0.0}{units[unitIndex]}";
        }

        public event PropertyChangedEventHandler? PropertyChanged;

        protected void OnPropertyChanged([CallerMemberName] string? name = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
        }
    }
}
