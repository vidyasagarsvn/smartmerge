# WinMerge Implementation Quick Reference

## Critical Implementation Patterns

### 1. Synchronized Line Numbers (Ghost Lines)

**Problem**: Files have different line counts  
**Solution**: Add "ghost" (blank) lines to align differences

```typescript
interface DIFFRANGE {
    begin: number[];    // [line10, line5]  - original positions
    end: number[];      // [line15, line8]  - original positions
    dbegin: number;     // 10 - synchronized start (same for both)
    dend: number;       // 15 - synchronized end (same for both)
    blank: number[];    // [0, 3] - ghost lines added to each pane
}
```

**Key Insight**: Always use `dbegin`/`dend` for scrolling and display, not `begin`/`end`.

### 2. Navigation Algorithm

```typescript
function nextDiff() {
    if (currentDiff === -1) {
        // No diff selected - find from cursor
        return diffList.nextDiffFromLine(cursorLine);
    }
    
    if (!isDiffVisible(currentDiff)) {
        // Current diff scrolled away - find from cursor
        return diffList.nextDiffFromLine(cursorLine + 1);
    }
    
    // Use linked list for O(1) navigation
    return diffList.diffs[currentDiff].next;
}
```

**Key Insight**: Check visibility before navigating to avoid confusion.

### 3. Scroll Synchronization

```typescript
function updateSiblingScrollPos(sourcePane: Pane, topLine: number) {
    for (const pane of allPanes) {
        if (pane !== sourcePane && pane.group === sourcePane.group) {
            pane.scrollToLine(topLine, { updateSiblings: false });
        }
    }
}
```

**Key Insight**: Prevent infinite loops with a flag to skip recursive updates.

### 4. Smart Scrolling with Context

```typescript
function showDiff(diffIndex: number) {
    const diff = diffList[diffIndex];
    let scrollLine = diff.dbegin;
    
    // Show context lines above
    if (scrollLine > CONTEXT_LINES_ABOVE) {
        scrollLine -= CONTEXT_LINES_ABOVE;
    }
    
    // Synchronize ALL panes explicitly
    for (const pane of allPanes) {
        pane.scrollToLine(scrollLine);
        pane.setCursor(diff.dbegin, 0);
    }
}
```

**Constants**: 
- `CONTEXT_LINES_ABOVE = 5`
- `CONTEXT_LINES_BELOW = 3`

### 5. Current Diff Highlighting

```typescript
function getLineColor(lineIndex: number) {
    const isCurrentDiff = isLineInCurrentDiff(lineIndex);
    
    if (lineFlags & DIFF) {
        if (isCurrentDiff) {
            return SELECTED_DIFF_COLOR;  // Bright highlight
        } else {
            return DIFF_COLOR;            // Muted highlight
        }
    }
    
    return NORMAL_COLOR;
}

function isLineInCurrentDiff(line: number): boolean {
    if (currentDiff < 0) return false;
    const diff = diffList[currentDiff];
    return line >= diff.dbegin && line <= diff.dend;
}
```

**Key Insight**: Two-tier highlighting - selected diff stands out from other diffs.

### 6. Line-to-Diff Lookup (Binary Search)

```typescript
function lineToDiff(line: number): number {
    let left = 0;
    let right = diffs.length - 1;
    
    while (left <= right) {
        const mid = Math.floor((left + right) / 2);
        const diff = diffs[mid];
        
        if (line < diff.dbegin) {
            right = mid - 1;
        } else if (line > diff.dend) {
            left = mid + 1;
        } else {
            return mid;
        }
    }
    
    return -1;
}
```

**Complexity**: O(log n) - essential for large diff lists

### 7. Diff List with Doubly-Linked Significant Chain

```typescript
interface DiffRangeInfo extends DIFFRANGE {
    next: number;   // Index of next significant diff
    prev: number;   // Index of previous significant diff
}

class DiffList {
    diffs: DiffRangeInfo[] = [];
    firstSignificant: number = -1;
    lastSignificant: number = -1;
    
    constructSignificantChain() {
        let prev = -1;
        for (let i = 0; i < this.diffs.length; i++) {
            if (this.diffs[i].op !== OP_TRIVIAL) {
                if (prev !== -1) {
                    this.diffs[prev].next = i;
                    this.diffs[i].prev = prev;
                } else {
                    this.firstSignificant = i;
                }
                prev = i;
            }
        }
        this.lastSignificant = prev;
    }
}
```

**Key Insight**: Maintain separate linked list for non-trivial diffs for fast traversal.

### 8. Location Bar Visualization

```typescript
function updateVisiblePos(topLine: number, bottomLine: number) {
    const topCoord = Y_OFFSET + (topLine * linePixelRatio);
    const bottomCoord = Y_OFFSET + (bottomLine * linePixelRatio);
    
    // Draw semi-transparent rectangle showing visible area
    ctx.fillStyle = 'rgba(0, 0, 0, 0.1)';
    ctx.fillRect(barX, topCoord, barWidth, bottomCoord - topCoord);
}

function gotoLocation(clickY: number) {
    const line = Math.floor((clickY - Y_OFFSET) / linePixelRatio);
    
    // Synchronize all panes to that line
    for (const pane of allPanes) {
        pane.scrollToLine(line);
    }
}
```

**Key Insight**: Click-to-navigate is just coordinate-to-line conversion + synchronized scroll.

## Data Flow

```
User Action (Next Diff)
    ↓
OnNextdiff()
    ↓
Find next diff index (using linked list or line search)
    ↓
SelectDiff(diffIndex)
    ↓
├→ ClearSelection()
├→ SetCurrentDiff(diffIndex)
├→ ShowDiff(scroll=true)
│      ↓
│      ├→ ScrollToLine(diff.dbegin - CONTEXT)
│      ├→ SetCursor(diff.dbegin)
│      └→ Synchronize ALL panes
│
├→ UpdateAllViews()
└→ UpdateSiblingScrollPos()
        ↓
    OnUpdateSibling() for each pane
        ↓
    ScrollToLine(sourcePane.topLine)
```

## Common Pitfalls to Avoid

1. **Don't use original line numbers for scrolling** - Always use synchronized (`dbegin`/`dend`)
2. **Prevent scroll loops** - Use a flag to ignore scroll events during synchronization
3. **Handle -1 current diff** - Many methods assume a diff is selected
4. **Check visibility** - Don't assume current diff is on screen
5. **Binary search bounds** - Check if line is before first or after last diff
6. **Ghost line rendering** - Don't try to get text for ghost lines
7. **Update location bar** - Call after every scroll operation

## Vue.js Specific Adaptations

### Reactive State

```typescript
const diffState = reactive({
    diffs: [] as DiffRange[],
    currentDiff: -1,
    firstSignificant: -1,
    lastSignificant: -1
});

const viewState = reactive({
    topLine: 0,
    cursorLine: 0,
    visibleLines: 50
});

// Watch for current diff changes to trigger highlighting
watch(() => diffState.currentDiff, (newDiff) => {
    // Re-render affected lines
    invalidateLineRange(oldDiff.dbegin, oldDiff.dend);
    invalidateLineRange(newDiff.dbegin, newDiff.dend);
});
```

### Composable for Synchronization

```typescript
export function useSyncScroll(panes: Ref<EditorPane[]>) {
    const isUpdating = ref(false);
    
    function syncScroll(sourcePane: EditorPane, topLine: number) {
        if (isUpdating.value) return;
        
        isUpdating.value = true;
        
        for (const pane of panes.value) {
            if (pane !== sourcePane) {
                pane.scrollToLine(topLine);
            }
        }
        
        nextTick(() => {
            isUpdating.value = false;
        });
    }
    
    return { syncScroll };
}
```

### Virtual Scrolling Compatibility

```typescript
function getVirtualLineData(displayLine: number) {
    const isGhost = ghostLines.has(displayLine);
    
    if (isGhost) {
        return {
            text: '',
            color: GHOST_LINE_COLOR,
            height: LINE_HEIGHT
        };
    }
    
    const originalLine = displayToOriginalLine(displayLine);
    return {
        text: getLineText(originalLine),
        color: getLineColor(displayLine),
        height: LINE_HEIGHT
    };
}
```

## Testing Checklist

- [ ] Navigate through diffs with Next/Prev
- [ ] Navigate when no diff is selected
- [ ] Navigate when current diff is off-screen
- [ ] Scroll one pane, verify others follow
- [ ] Click location bar, verify all panes jump
- [ ] Verify current diff is highlighted differently
- [ ] Test with files of equal and different line counts
- [ ] Test with trivial diffs (should be skipped)
- [ ] Test first/last diff edge cases
- [ ] Test with word wrapping enabled
- [ ] Verify ghost lines appear in shorter file
- [ ] Test performance with 1000+ diffs

## Key Files for Reference

WinMerge source locations:
- **Navigation**: `MergeEditView.cpp` lines 1233-1400
- **Scroll Sync**: `MergeEditView.cpp` lines 779-850
- **Highlighting**: `MergeEditView.cpp` lines 610-770
- **Line Mapping**: `MergeDocDiffSync.cpp` lines 169-286
- **Diff List**: `DiffList.h` and `DiffList.cpp`
- **Location Bar**: `LocationView.cpp` lines 350-1020

## Performance Targets

- **Navigation**: < 16ms (60 FPS)
- **Scroll sync**: < 16ms (60 FPS, all panes update together)
- **Line lookup**: < 1ms (binary search)
- **Highlighting update**: < 16ms (only affected lines)
- **Location bar**: < 33ms (30 FPS is acceptable for overview)
