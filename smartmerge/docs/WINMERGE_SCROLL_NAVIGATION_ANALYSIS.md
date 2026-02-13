# WinMerge Synchronized Scrolling & Navigation Analysis

## Executive Summary

This document details how WinMerge implements synchronized scrolling and navigation between diff regions. The implementation is sophisticated and handles edge cases like files with different line counts using "ghost lines" and careful coordinate mapping.

## 1. Next/Previous Diff Navigation

### Core Navigation Methods

**Location**: `MergeEditView.cpp` (lines 1233-1400)

#### OnNextdiff() Implementation

```cpp
void CMergeEditView::OnNextdiff()
{
    CMergeDoc *pd = GetDocument();
    int cnt = pd->m_ptBuf[0]->GetLineCount();
    if (cnt <= 0)
        return;

    // Returns -1 if no diff selected
    int nextDiff = -1;
    int curDiff = pd->GetCurrentDiff();
    
    if (curDiff != -1)
    {
        // We're on a diff
        if (!IsDiffVisible(curDiff))
        {
            // Selected difference not visible, select next from cursor
            int line = GetCursorPos().y;
            ++line;
            if (!IsValidTextPosY(CPoint(0, line)))
                line = m_nTopLine;
            nextDiff = pd->m_diffList.NextSignificantDiffFromLine(line);
        }
        else
        {
            // Find out if there is a following significant diff
            if (curDiff < pd->m_diffList.GetSize() - 1)
            {
                nextDiff = pd->m_diffList.NextSignificantDiff(curDiff);
            }
        }
    }
    else
    {
        // We don't have a selected difference,
        // but cursor can be inside inactive diff
        int line = GetCursorPos().y;
        if (!IsValidTextPosY(CPoint(0, line)))
            line = m_nTopLine;
        nextDiff = pd->m_diffList.NextSignificantDiffFromLine(line);
    }

    int lastDiff = pd->m_diffList.LastSignificantDiff();
    if (nextDiff >= 0 && nextDiff <= lastDiff)
        SelectDiff(nextDiff, true, false);
}
```

**Key Algorithm Steps**:
1. Get current diff index (returns -1 if none selected)
2. If current diff is not visible, find next diff from cursor position
3. If current diff exists and is visible, get the next significant diff using linked list
4. If no current diff, find next diff from current cursor line
5. Call `SelectDiff()` to navigate to the found diff

#### OnPrevdiff() Implementation

Similar logic but in reverse:
- Uses `PrevSignificantDiff()` and `PrevSignificantDiffFromLine()`
- Decrements line by 1 to ensure we don't stay in the same diff

### Diff List Navigation API

**Location**: `DiffList.h/cpp`

WinMerge maintains a **doubly-linked list** of significant diffs for efficient navigation:

```cpp
struct DiffRangeInfo: public DIFFRANGE
{
    ptrdiff_t next; /**< link (array index) for doubly-linked chain */
    ptrdiff_t prev; /**< link (array index) for doubly-linked chain */
};
```

**Key Methods**:

1. **LineToDiff(int nLine)** - Binary search to find diff containing a line
   ```cpp
   int DiffList::LineToDiff(int nLine) const
   {
       // Binary search through diff ranges
       int left = 0;
       int right = nDiffCount - 1;
       
       while (left <= right)
       {
           int middle = (left + right) / 2;
           int result = LineRelDiff(nLine, middle);
           switch (result)
           {
           case -1: right = middle - 1; break;  // Line before diff
           case 0:  return middle;               // Line in diff
           case 1:  left = middle + 1; break;   // Line after diff
           }
       }
       return -1;
   }
   ```

2. **NextSignificantDiffFromLine(int nLine)** - Linear search forward
   ```cpp
   int DiffList::NextSignificantDiffFromLine(int nLine) const
   {
       for (int i = 0; i < nDiffCount; i++)
       {
           const DIFFRANGE * dfi = DiffRangeAt(i);
           if (dfi->op != OP_TRIVIAL && dfi->dbegin >= nLine)
           {
               return i;
           }
       }
       return -1;
   }
   ```

3. **NextSignificantDiff(int nDiff)** - O(1) using linked list
   ```cpp
   inline int DiffList::NextSignificantDiff(int nDiff) const
   {
       return (int)m_diffs[nDiff].next;
   }
   ```

### SelectDiff() - The Navigation Orchestrator

**Location**: `MergeEditView.cpp` (line 925)

```cpp
void CMergeEditView::SelectDiff(int nDiff, bool bScroll, bool bSelectText)
{
    CMergeDoc *pd = GetDocument();
    
    SelectNone();
    pd->SetCurrentDiff(nDiff);
    ShowDiff(bScroll, bSelectText);
    pd->UpdateAllViews(this);
    UpdateSiblingScrollPos(false);  // Synchronize scroll position
    
    // Notify detail views
    pd->ForEachView(0, [&](auto& pView) { 
        if (pView->m_bDetailView) 
            pView->OnDisplayDiff(nDiff); 
    });
}
```

**What it does**:
1. Clears current selection
2. Sets the diff as current in the document
3. Calls ShowDiff() to scroll and highlight
4. Updates all views
5. **Synchronizes scroll position across panes** via UpdateSiblingScrollPos()
6. Notifies detail views

## 2. Synchronized Scrolling

### The Master-Slave Synchronization Pattern

WinMerge uses a "broadcast" pattern where one view updates and notifies all sibling views.

#### UpdateSiblingScrollPos() - The Synchronization Broadcaster

**Location**: `MergeEditView.cpp` (line 779)

```cpp
void CMergeEditView::UpdateSiblingScrollPos(bool bHorz)
{
    CSplitterWnd *pSplitterWnd = GetParentSplitter(this, false);
    if (pSplitterWnd != nullptr)
    {
        int nCurrentRow = (GetDlgCtrlID() - AFX_IDW_PANE_FIRST) / 16;
        int nCurrentCol = (GetDlgCtrlID() - AFX_IDW_PANE_FIRST) % 16;
        
        int nRows = pSplitterWnd->GetRowCount();
        int nCols = pSplitterWnd->GetColumnCount();
        
        for (int nRow = 0; nRow < nRows; nRow++)
        {
            for (int nCol = 0; nCol < nCols; nCol++)
            {
                if (!(nRow == nCurrentRow && nCol == nCurrentCol))
                {
                    CMergeEditView *pSiblingView = 
                        static_cast<CMergeEditView*>(GetSiblingView(nRow, nCol));
                    if (pSiblingView != nullptr && 
                        pSiblingView->m_nThisGroup == m_nThisGroup)
                        pSiblingView->OnUpdateSibling(this, bHorz);
                }
            }
        }
    }
}
```

**Algorithm**:
1. Get the splitter window containing all panes
2. Calculate which pane is the current one (using control ID arithmetic)
3. Loop through all panes in the splitter
4. Skip the current pane (don't update self)
5. For each sibling pane in the same group, call `OnUpdateSibling()`

#### OnUpdateSibling() - The Synchronization Receiver

**Location**: `MergeEditView.cpp` (line 826)

```cpp
void CMergeEditView::OnUpdateSibling(CCrystalTextView * pUpdateSource, bool bHorz)
{
    if (pUpdateSource != this)
    {
        CMergeEditView *pSrcView = static_cast<CMergeEditView*>(pUpdateSource);
        if (!bHorz)  // Vertical scrolling
        {
            if (pSrcView->m_nTopSubLine != m_nTopSubLine)
            {
                ScrollToSubLine(pSrcView->m_nTopSubLine, true, false);
                UpdateCaret();
                RecalcVertScrollBar(true);
                RecalcHorzScrollBar();
            }
        }
        else  // Horizontal scrolling
        {
            if (pSrcView->m_nOffsetChar != m_nOffsetChar)
            {
                ScrollToChar(pSrcView->m_nOffsetChar, true, false);
                UpdateCaret();
                RecalcHorzScrollBar(true);
            }
        }
    }
}
```

**Key Insight**: Uses **SubLine** (not just Line) for synchronization

- `m_nTopSubLine` - Current top "sub-line" visible in the view
- Sub-lines account for word-wrapping and ghost lines
- This ensures perfect alignment even when files have different actual line counts

### ShowDiff() - Smart Scrolling with Context

**Location**: `MergeEditView.cpp` (line 2189)

```cpp
void CMergeEditView::ShowDiff(bool bScroll, bool bSelectText)
{
    CMergeDoc *pd = GetDocument();
    const int nDiff = pd->GetCurrentDiff();
    
    if (nDiff >= 0 && nDiff < pd->m_diffList.GetSize())
    {
        CPoint ptStart, ptEnd;
        DIFFRANGE curDiff;
        pd->m_diffList.GetDiff(nDiff, curDiff);
        
        ptStart.x = 0;
        ptStart.y = curDiff.dbegin;  // Use synchronized line numbers
        ptEnd.x = 0;
        ptEnd.y = curDiff.dend;
        
        if (bScroll)
        {
            if (!IsDiffVisible(curDiff, CONTEXT_LINES_BELOW))
            {
                // Scroll diff into view with context
                int nLine = GetSubLineIndex(ptStart.y);
                if (nLine > CONTEXT_LINES_ABOVE)
                {
                    nLine -= CONTEXT_LINES_ABOVE;  // Show 5 lines above
                }
                
                // Synchronize all panes
                GetGroupView(m_nThisPane)->ScrollToSubLine(nLine);
                for (int nPane = 0; nPane < pd->m_nBuffers; nPane++)
                {
                    if (nPane != m_nThisPane)
                        GetGroupView(nPane)->ScrollToSubLine(nLine);
                }
            }
            
            // Set cursor and selection in all panes
            GetGroupView(m_nThisPane)->SetCursorPos(ptStart);
            GetGroupView(m_nThisPane)->SetAnchor(ptStart);
            GetGroupView(m_nThisPane)->SetSelection(ptStart, ptStart);
            
            for (int nPane = 0; nPane < pd->m_nBuffers; nPane++)
            {
                if (nPane != m_nThisPane)
                {
                    GetGroupView(nPane)->SetCursorPos(ptStart);
                    GetGroupView(nPane)->SetAnchor(ptStart);
                    GetGroupView(nPane)->SetSelection(ptStart, ptStart);
                }
            }
        }
        
        if (bSelectText)
        {
            ptEnd.x = GetLineLength(ptEnd.y);
            SetSelection(ptStart, ptEnd);
            UpdateCaret();
        }
    }
}
```

**Smart Scrolling Features**:
1. Shows **CONTEXT_LINES_ABOVE (5)** and **CONTEXT_LINES_BELOW (3)** around the diff
2. Uses `dbegin` and `dend` (synchronized line numbers with ghost lines)
3. **Explicitly synchronizes all panes** by calling ScrollToSubLine on each
4. Sets cursor position consistently across all panes

## 3. Current Diff Region Highlighting

### GetLineColors2() - The Highlighting Engine

**Location**: `MergeEditView.cpp` (line 627)

```cpp
void CMergeEditView::GetLineColors2(int nLineIndex, DWORD ignoreFlags, 
                                   COLORREF & crBkgnd, COLORREF & crText, 
                                   bool & bDrawWhitespace)
{
    DWORD dwLineFlags = GetLineFlags(nLineIndex);
    
    if (dwLineFlags & LF_WINMERGE_FLAGS)
    {
        crText = m_cachedColors.clrDiffText;
        bDrawWhitespace = true;
        
        bool lineInCurrentDiff = IsLineInCurrentDiff(nLineIndex);
        
        if (dwLineFlags & LF_DIFF)
        {
            if (lineInCurrentDiff)
            {
                // CURRENT DIFF HIGHLIGHTING
                if (dwLineFlags & LF_MOVED)
                {
                    if (dwLineFlags & LF_GHOST)
                        crBkgnd = m_cachedColors.clrSelMovedDeleted;
                    else
                        crBkgnd = m_cachedColors.clrSelMoved;
                    crText = m_cachedColors.clrSelMovedText;
                }
                else
                {
                    crBkgnd = m_cachedColors.clrSelDiff;
                    crText = m_cachedColors.clrSelDiffText;
                }
            }
            else
            {
                // NON-CURRENT DIFF (regular diff colors)
                if (dwLineFlags & LF_MOVED)
                {
                    if (dwLineFlags & LF_GHOST)
                        crBkgnd = m_cachedColors.clrMovedDeleted;
                    else
                        crBkgnd = m_cachedColors.clrMoved;
                    crText = m_cachedColors.clrMovedText;
                }
                else
                {
                    crBkgnd = m_cachedColors.clrDiff;
                    crText = m_cachedColors.clrDiffText;
                }
            }
        }
    }
}
```

**Highlighting Strategy**:
1. Each line has flags (LF_DIFF, LF_MOVED, LF_GHOST, etc.)
2. `IsLineInCurrentDiff(nLineIndex)` determines if line is in the selected diff
3. Uses **different color sets** for:
   - Selected diff (`clrSelDiff`, `clrSelDiffText`)
   - Regular diff (`clrDiff`, `clrDiffText`)
   - Moved blocks (special colors)
   - Ghost lines (deleted areas)

### IsLineInCurrentDiff() Implementation

```cpp
bool CMergeEditView::IsLineInCurrentDiff(int nLine) const
{
    const CMergeDoc *pd = GetDocument();
    int nDiff = pd->GetCurrentDiff();
    if (nDiff >= 0)
    {
        DIFFRANGE dr;
        pd->m_diffList.GetDiff(nDiff, dr);
        return (nLine >= dr.dbegin && nLine <= dr.dend);
    }
    return false;
}
```

**Simple and efficient**: Just checks if line is within the synchronized diff range (dbegin to dend).

## 4. Line Number Mapping Between Panes

### The Ghost Lines System

WinMerge uses "ghost lines" to handle files with different line counts. This is the key to synchronized scrolling.

#### DIFFRANGE Structure

**Location**: `DiffList.h`

```cpp
struct DIFFRANGE
{
    int begin[3];      // First diff line in ORIGINAL file1,2,3
    int end[3];        // Last diff line in ORIGINAL file1,2,3
    int dbegin;        // Synchronized (ghost lines added) first diff line
    int dend;          // Synchronized (ghost lines added) last diff line
    int blank[3];      // Number of blank (ghost) lines in file1,2,3
    OP_TYPE op;        // Operation type (OP_DIFF, OP_1STONLY, etc.)
};
```

**Key Concept**:
- `begin[]` and `end[]` are the ORIGINAL line numbers in each file
- `dbegin` and `dend` are the SYNCHRONIZED line numbers (after adding ghost lines)
- `blank[]` tells how many ghost lines were added to each pane

Example:
```
File 1: Lines 10-15 (6 lines)
File 2: Lines 10-12 (3 lines)

After synchronization:
dbegin = 10
dend = 15
blank[0] = 0   (File 1: no ghost lines needed)
blank[1] = 3   (File 2: 3 ghost lines added to match File 1's length)
```

### AdjustDiffBlock() - The Line Mapping Algorithm

**Location**: `MergeDocDiffSync.cpp` (line 169)

This is a **recursive algorithm** that maps lines between files within a diff block:

```cpp
void CMergeDoc::AdjustDiffBlock(DiffMap & diffMap, const DIFFRANGE & diffrange, 
                                int lo0, int hi0, int lo1, int hi1)
{
    int offset0 = diffrange.begin[0];
    int offset1 = diffrange.begin[1];
    int lines0 = hi0 - lo0 + 1;
    int lines1 = hi1 - lo1 + 1;
    
    // Shortcut for equal lines
    if (lines0 == 1 && lines1 == 1)
    {
        diffMap.m_map[lo0] = hi1;
        return;
    }
    
    // Bail out for large ranges - use simple 1:1 mapping
    if (lines0 > 15 || lines1 > 15)
    {
        for (int w=0; w<lines0; ++w)
        {
            if (w < lines1)
                diffMap.m_map[w] = w + tlo;
            else
                diffMap.m_map[w] = DiffMap::GHOST_MAP_ENTRY;
        }
        return;
    }
    
    // Find best fit using Levenshtein distance
    int ibest=-1, isavings=0x7fffffff, itarget=-1;
    for (int i=lo0; i<=hi0; ++i)
    {
        m_ptBuf[0]->GetLine(offset0 + i, sLine0);
        for (int j=lo1; j<=hi1; ++j)
        {
            m_ptBuf[1]->GetLine(offset1 + j, sLine1);
            int savings = GetMatchCost(sLine0, sLine1);
            if (savings < isavings)
            {
                ibest = i;
                itarget = j;
                isavings = savings;
            }
        }
    }
    
    diffMap.m_map[ibest] = itarget;
    
    // Recursively solve subproblems above and below the match
    if (lo0 < ibest)
    {
        if (lo1 < itarget)
            AdjustDiffBlock(diffMap, diffrange, lo0, ibest-1, lo1, itarget-1);
        else
            // Mark as ghost lines
            for (int x = lo0; x < ibest; ++x)
                diffMap.m_map[x] = DiffMap::GHOST_MAP_ENTRY;
    }
    
    if (hi0 > ibest)
    {
        if (itarget < hi1)
            AdjustDiffBlock(diffMap, diffrange, ibest + 1, hi0, 
                          itarget + 1, hi1);
        else
            // Mark as ghost lines
            for (int x = ibest + 1; x <= hi0; ++x)
                diffMap.m_map[x] = DiffMap::GHOST_MAP_ENTRY;
    }
}
```

**Algorithm Strategy**:
1. For small ranges (≤15 lines), use detailed matching
2. Find the best matching line pair using **Levenshtein distance** (edit distance)
3. Use that match as a "pivot" to split the problem
4. **Recursively solve** above and below the pivot
5. Lines that can't be matched become `GHOST_MAP_ENTRY`

### Sub-Line System for Word Wrapping

WinMerge supports word-wrapping, so one logical line can span multiple visible lines.

**Key Methods**:
- `GetSubLineIndex(int nLine)` - Convert line to sub-line index
- `ScrollToSubLine(int nSubLine)` - Scroll to a sub-line position
- `GetSubLineCount()` - Total sub-lines in view

This ensures synchronization works even with word wrapping enabled.

## 5. LocationView - The Overview Bar

### UpdateVisiblePos() - Indicating Current View Position

**Location**: `LocationView.cpp` (line 973)

```cpp
void CLocationView::UpdateVisiblePos(int nTopLine, int nBottomLine)
{
    if (m_bDrawn)
    {
        CMergeDoc *pDoc = GetDocument();
        int nGroup = pDoc->GetActiveMergeView()->m_nThisGroup;
        
        // Convert line numbers to pixel coordinates
        int nTopCoord = static_cast<int>(Y_OFFSET +
                (static_cast<double>(nTopLine * m_lineInPix)));
        int nBottomCoord = static_cast<int>(Y_OFFSET +
                (static_cast<double>(nBottomLine * m_lineInPix)));
        
        if (m_visibleTop != nTopCoord || m_visibleBottom != nBottomCoord)
        {
            // Redraw the visible area indicator
            if (m_pSavedBackgroundBitmap != nullptr)
            {
                CClientDC dc(this);
                CMyMemDC dcMem(&dc);
                DrawBitmap(&dcMem, 0, 0, m_pSavedBackgroundBitmap.get());
                DrawVisibleAreaRect(&dcMem, nTopLine, nBottomLine);
            }
        }
    }
}
```

**Visualization**:
- Converts line numbers to pixel coordinates using `m_lineInPix` scaling factor
- Draws a rectangle showing the currently visible area
- Updates only when the visible range changes

### GotoLocation() - Click to Navigate

**Location**: `LocationView.cpp` (line 690)

```cpp
bool CLocationView::GotoLocation(const CPoint& point, bool bRealLine)
{
    CRect rc;
    GetClientRect(rc);
    CMergeDoc* pDoc = GetDocument();
    
    int line = -1;
    int bar = IsInsideBar(rc, point);
    
    if (bar == BAR_0 || bar == BAR_1 || bar == BAR_2)
    {
        line = GetLineFromYPos(point.y, bar, bRealLine);
    }
    else if (bar == BAR_YAREA)
    {
        bar = BAR_0;
        line = GetLineFromYPos(point.y, bar, false);
    }
    
    if (line >= 0)
    {
        pDoc->GetActiveMergeGroupView(0)->GotoLine(line, bRealLine, bar);
        if (bar == BAR_0 || bar == BAR_1 || bar == BAR_2)
            pDoc->GetActiveMergeGroupView(bar)->SetFocus();
        return true;
    }
    
    return false;
}
```

**Click Navigation**:
1. Determine which bar was clicked (left, middle, right)
2. Convert Y-coordinate to line number
3. Navigate all views to that line
4. Set focus to the clicked pane

## Key Implementation Principles for Vue.js/TypeScript

### 1. Data Structures Needed

```typescript
interface DiffRange {
    begin: number[];      // Original line numbers per pane
    end: number[];        // Original line numbers per pane
    dbegin: number;       // Synchronized line number (start)
    dend: number;         // Synchronized line number (end)
    blank: number[];      // Ghost lines added per pane
    op: DiffOpType;       // Type of difference
}

interface DiffList {
    diffs: DiffRange[];
    currentDiff: number;  // Index of selected diff (-1 if none)
    firstSignificant: number;
    lastSignificant: number;
}

interface ViewState {
    topLine: number;      // Current scroll position
    topSubLine: number;   // With word-wrap support
    cursorLine: number;
    cursorCol: number;
}
```

### 2. Navigation Implementation Pattern

```typescript
class DiffNavigator {
    private diffList: DiffList;
    
    nextDiff(): number {
        const curDiff = this.diffList.currentDiff;
        
        if (curDiff === -1) {
            // Find next diff from cursor position
            return this.nextDiffFromLine(this.cursorLine);
        }
        
        if (!this.isDiffVisible(curDiff)) {
            // Diff scrolled out of view, find next from cursor
            return this.nextDiffFromLine(this.cursorLine + 1);
        }
        
        // Get next in linked list
        return this.diffList.diffs[curDiff].next;
    }
    
    lineToDiff(line: number): number {
        // Binary search
        let left = 0;
        let right = this.diffList.diffs.length - 1;
        
        while (left <= right) {
            const mid = Math.floor((left + right) / 2);
            const diff = this.diffList.diffs[mid];
            
            if (line < diff.dbegin) {
                right = mid - 1;
            } else if (line > diff.dend) {
                left = mid + 1;
            } else {
                return mid;  // Found it
            }
        }
        
        return -1;  // Not in any diff
    }
}
```

### 3. Synchronized Scrolling Pattern

```typescript
class ScrollSynchronizer {
    private panes: EditorPane[];
    private ignoreNextScroll = false;
    
    handleScroll(sourcePane: EditorPane, topLine: number) {
        if (this.ignoreNextScroll) return;
        
        this.ignoreNextScroll = true;
        
        // Update all other panes
        for (const pane of this.panes) {
            if (pane !== sourcePane) {
                pane.scrollToLine(topLine, {
                    smooth: false,
                    updateSiblings: false
                });
            }
        }
        
        this.ignoreNextScroll = false;
        
        // Update location bar
        this.locationView.updateVisiblePos(topLine, 
            topLine + sourcePane.visibleLines);
    }
}
```

### 4. Highlighting Implementation

```typescript
class LineRenderer {
    getLineColor(lineIndex: number): {bg: string, fg: string} {
        const lineFlags = this.getLineFlags(lineIndex);
        const isInCurrentDiff = this.isLineInCurrentDiff(lineIndex);
        
        if (lineFlags.isDiff) {
            if (isInCurrentDiff) {
                // Highlighted colors for current diff
                return {
                    bg: this.colors.selectedDiffBg,
                    fg: this.colors.selectedDiffText
                };
            } else {
                // Regular diff colors
                return {
                    bg: this.colors.diffBg,
                    fg: this.colors.diffText
                };
            }
        }
        
        return {
            bg: this.colors.normalBg,
            fg: this.colors.normalText
        };
    }
    
    isLineInCurrentDiff(line: number): boolean {
        const curDiff = this.diffList.currentDiff;
        if (curDiff < 0) return false;
        
        const diff = this.diffList.diffs[curDiff];
        return line >= diff.dbegin && line <= diff.dend;
    }
}
```

### 5. Ghost Lines for Line Count Differences

```typescript
class GhostLineManager {
    // Map from original line to display line
    private lineMap: Map<number, number> = new Map();
    private ghostLines: Set<number> = new Set();
    
    getDisplayLine(originalLine: number): number {
        return this.lineMap.get(originalLine) ?? originalLine;
    }
    
    isGhostLine(displayLine: number): boolean {
        return this.ghostLines.has(displayLine);
    }
    
    // Build maps when diffs are calculated
    buildMaps(diffs: DiffRange[], pane: number) {
        let displayLine = 0;
        let originalLine = 0;
        
        for (const diff of diffs) {
            // Lines before diff
            while (originalLine < diff.begin[pane]) {
                this.lineMap.set(originalLine, displayLine);
                displayLine++;
                originalLine++;
            }
            
            // Lines in diff
            const lineCount = diff.end[pane] - diff.begin[pane] + 1;
            const ghostCount = diff.blank[pane];
            
            for (let i = 0; i < lineCount; i++) {
                this.lineMap.set(originalLine, displayLine);
                displayLine++;
                originalLine++;
            }
            
            // Add ghost lines
            for (let i = 0; i < ghostCount; i++) {
                this.ghostLines.add(displayLine);
                displayLine++;
            }
        }
    }
}
```

## Performance Considerations

1. **Binary Search**: WinMerge uses binary search for LineToDiff() - O(log n)
2. **Linked List**: Significant diffs are in a doubly-linked list - O(1) navigation
3. **Cached Colors**: Color settings are cached to avoid repeated lookups
4. **Smart Redraw**: Only redraws visible area indicator when it actually moves
5. **Sub-line Limits**: Large diff blocks (>15 lines) use simplified 1:1 mapping

## Testing Edge Cases

Based on WinMerge's implementation, test these scenarios:

1. **No diffs selected**: Should find next/prev from cursor position
2. **Diff not visible**: Should find next visible diff
3. **At first/last diff**: Should handle gracefully (possibly move to next file)
4. **Empty files**: Check for zero line count
5. **Very long diffs**: Ensure performance with >15 line blocks
6. **Ghost lines**: Ensure scrolling works when panes have different line counts
7. **Word wrapping**: Sub-lines should stay synchronized

## Conclusion

WinMerge's implementation is sophisticated but follows clear principles:

1. **Centralized diff list** with efficient navigation (linked list + binary search)
2. **Broadcast synchronization** from master to all siblings
3. **Ghost lines** to handle different line counts
4. **Context-aware scrolling** (show lines above/below)
5. **Differential highlighting** (current diff vs. other diffs)
6. **Sub-line system** for word-wrapping support

The key to successful implementation in Vue.js is maintaining these same data structures and communication patterns, adapted to Vue's reactive system.
