# SmartMerge UI Improvement Analysis
## Comparing with WinMerge Architecture and Design Patterns

**Date:** February 13, 2026  
**Comparison Focus:** UI/UX design, feature organization, user workflow patterns

---

## Executive Summary

WinMerge is a mature (25+ years), feature-rich desktop diff/merge tool built with C++/MFC. While it uses legacy technology, it has evolved excellent UI patterns for professional merge workflows. SmartMerge has a modern Vue 3 foundation but can adopt proven WinMerge design patterns to enhance usability and feature coverage.

**Key Findings:**
- ✅ SmartMerge has cleaner, modern UI foundation (Vue 3 > MFC)
- ⚠️ Missing several professional workflow features WinMerge provides
- 🎯 Opportunity to combine modern tech with proven UX patterns
- 🔧 Several architectural improvements possible

---

## 1. Location Pane / Minimap (MISSING)

### WinMerge Implementation
- **LocationView.cpp** (1,100+ lines) - Comprehensive minimap visualization
- Shows entire file at 1:1 mapping
- Visual indicators for:
  - Moved blocks (connecting lines between sections)
  - Different change types (colors for additions, deletions, modifications)
  - Trivial changes (whitespace-only diffs)
  - Word-level diffs within blocks
  - Current viewport indicator
- Clickable navigation - click on minimap to jump to that location

### SmartMerge Status
- ❌ **Not implemented**
- Current implementation jumps between regions but no visual overview
- Users have to scroll through large files without spatial understanding

### Recommendation: Implement Location Pane

```vue
<!-- Add to FileCompareView.vue -->
<template>
  <div class="merger-container">
    <div class="main-diff-area">
      <!-- existing diff view -->
    </div>
    <div class="location-pane">
      <!-- Minimap showing: -->
      <!-- - Entire file overview -->
      <!-- - Color-coded diff regions -->
      <!-- - Viewport indicator (current visible area) -->
      <!-- - Clickable for navigation -->
    </div>
  </div>
</template>
```

**Complexity:** Medium (4-6 weeks)  
**User Value:** High - essential for >1000-line files

---

## 2. Multi-Level Color Customization (PARTIAL)

### WinMerge Implementation
- **PropColors.cpp** (468+ lines) - Extensive color customization
- Separate colors for:
  - **Regular differences** (background color + text color + selected state)
  - **Trivial diffs** (whitespace-only changes) 
  - **Moved blocks** (code relocated to different position)
  - **Inline word diffs** (character-level changes within lines)
  - **Syntax-aware highlighting** (selected-non-selected states)
- Pre-built color schemes (GitHub, Bitbucket, custom)
- Per-category: background, text, deleted text colors

### SmartMerge Status
- ✅ Basic highlighting exists (SettingsView.vue)
- ❌ Limited customization - only theme selection (light/dark)
- ❌ No per-category color customization
- ❌ No moved-block-specific colors
- ❌ No preset color schemes

### Recommendation: Expand Color System

```typescript
// Enhanced color model in SettingsView
interface ColorScheme {
  name: string;
  baseBackground: string;
  
  // Diff categories
  additions: { background: string; text: string; selected: string };
  deletions: { background: string; text: string; selected: string };
  modifications: { background: string; text: string; selected: string };
  trivialChanges: { background: string; text: string };
  movedBlocks: { background: string; text: string; lineColor: string };
  
  // Within-line word diffs
  inlineChangeBackground: string;
  inlineChangeText: string;
}

// Presets
const SCHEMES = {
  github: { /* colors matching GitHub diff view */ },
  bitbucket: { /* colors matching Bitbucket */ },
  vscode: { /* matching VS Code theme */ },
  solarized: { /* solarized theme */ },
};
```

**Complexity:** Medium (2-3 weeks)  
**User Value:** High - professional customization expected

---

## 3. Moved Blocks Detection & Visualization (PARTIAL)

### WinMerge Implementation
- **MovedBlocks.cpp** / **MovedLines.cpp**
- Detects when large code blocks are relocated
- Visual indicators:
  - Specific "moved block" color (distinct from regular changes)
  - Lines connecting related blocks (in LocationView)
  - Separate row status indication
- User can filter out moved blocks from view
- Statistics on moved blocks vs. actual changes

### SmartMerge Status
- ✅ Backend detection exists (engine/moved_blocks.rs)
- ⚠️ Frontend highlighting minimal
- ❌ No connecting lines between moved blocks
- ❌ No filter to hide moved blocks
- ❌ No statistics

### Recommendation: Enhanced Moved Block UI

```vue
<!-- In FileCompareView.vue -->
<script setup>
  // Add moved block visualization
  const showMovedBlockConnectors = ref(true);
  const hideMovedBlocksOnly = ref(false);
  
  function renderMovedBlockConnectors() {
    // Draw SVG lines connecting matching moved blocks
    // Show in LocationPane and main view
  }
  
  function highlightMovedBlocks(line, highlight) {
    // Apply special "moved-block" color from color scheme
    // Show connection indicators in margins
  }
</script>

<template>
  <div class="settings-bar">
    <label>
      <input v-model="showMovedBlockConnectors" type="checkbox">
      Show Moved Block Connections
    </label>
    <label>
      <input v-model="hideMovedBlocksOnly" type="checkbox">
      Hide Moved Blocks (show only real changes)
    </label>
    <span class="stats">
      Real differences: {{ realChanges }}, Moved: {{ movedBlocks }}
    </span>
  </div>
  
  <svg class="moved-block-connectors">
    <!-- Lines connecting moved blocks -->
  </svg>
</template>
```

**Complexity:** Medium (2-3 weeks)  
**User Value:** High - critical for refactoring reviews

---

## 4. Trivial Diffs Filtering (MISSING)

### WinMerge Implementation
- **OptionsDiffOptions.cpp** - Extensive trivial diff filtering
- Built-in filters:
  - Whitespace (all vs. leading vs. trailing)
  - Case differences
  - EOL differences (CR/LF variations)
  - Empty lines
  - Comment-only changes
- Option to hide trivial diffs from view
- Option to exclude from comparison entirely
- Shows count of trivial vs. real changes

### SmartMerge Status
- ❌ **Not implemented**
- All changes treated equally
- Users can't focus on meaningful differences

### Recommendation: Trivial Diff Filtering

```vue
<!-- In SettingsView.vue or new DiffFiltersPanel.vue -->
<script setup>
  interface IgnoreOptions {
    ignoreAllWhitespace: boolean;
    ignoreLeadingWhitespace: boolean;
    ignoreTrailingWhitespace: boolean;
    ignoreBlankLines: boolean;
    ignoreCaseChanges: boolean;
    ignoreEOLDifferences: boolean;
    ignoreCommentOnlyChanges: boolean;
  }
  
  const ignoreOptions = ref<IgnoreOptions>({});
</script>

<template>
  <div class="trivial-diff-filters">
    <h3>Ignore Options</h3>
    <div class="filter-group">
      <label>
        <input v-model="ignoreOptions.ignoreAllWhitespace" type="checkbox">
        Ignore all whitespace
      </label>
      <label>
        <input v-model="ignoreOptions.ignoreLeadingWhitespace" type="checkbox">
        Ignore leading whitespace
      </label>
      <label>
        <input v-model="ignoreOptions.ignoreTrailingWhitespace" type="checkbox">
        Ignore trailing whitespace
      </label>
      <label>
        <input v-model="ignoreOptions.ignoreBlankLines" type="checkbox">
        Ignore blank lines
      </label>
      <label>
        <input v-model="ignoreOptions.ignoreCaseChanges" type="checkbox">
        Ignore case changes
      </label>
    </div>
    <div class="stats">
      <span>Real changes: {{ realDiffCount }} / {{ totalDiffCount }}</span>
      <span>Hidden: {{ trivialDiffCount }}</span>
    </div>
  </div>
</template>
```

**Complexity:** High (3-4 weeks) - requires backend implementation  
**User Value:** Very High - essential for real-world use

---

## 5. Three-Way Merge UI (GOOD, but can improve)

### WinMerge Implementation
- **MergeEditFrm.cpp** / **MergeEditView.cpp**
- Vertical stack of 3 panes (left/base/right or left/middle/right)
- Conflict highlighting with resolution buttons
- Merge operation history tracking
- Statistical summary of conflicts resolved
- Context menu for quick resolution

### SmartMerge Status
- ✅ MergeView.vue exists (756 lines) - comprehensive
- ✅ Three-way layout implemented
- ✅ Conflict resolution UI present
- ✅ Undo/redo working
- ⚠️ Could add:
  - Conflict statistics
  - Pre-built resolution templates
  - Batch operations

### Recommendation: Enhance Merge Workflow

```vue
<!-- Add to MergeView.vue -->
<script setup>
  const mergeStats = computed(() => ({
    totalConflicts: conflicts.value.length,
    resolved: resolvedCount.value,
    remaining: conflicts.value.length - resolvedCount.value,
    percentComplete: (resolvedCount.value / conflicts.value.length) * 100,
  }));
  
  // Quick resolution buttons
  const resolutionTemplates = [
    { label: 'Keep Ours', strategy: 'ours' },
    { label: 'Take Theirs', strategy: 'theirs' },
    { label: 'Take Both', strategy: 'both' },
    { label: 'Keep Base', strategy: 'base' },
  ];
</script>

<template>
  <div class="merge-view">
    <!-- Progress tracker -->
    <div class="merge-progress">
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: mergeStats.percentComplete + '%' }"></div>
      </div>
      <span>{{ mergeStats.resolved }} / {{ mergeStats.totalConflicts }} conflicts resolved</span>
    </div>
    
    <!-- Quick resolution buttons -->
    <div class="resolution-templates">
      <button 
        v-for="template in resolutionTemplates"
        :key="template.strategy"
        @click="applyTemplate(template.strategy)"
      >
        {{ template.label }}
      </button>
    </div>
  </div>
</template>
```

**Complexity:** Low (1-2 weeks)  
**User Value:** Medium - quality-of-life improvements

---

## 6. Context Menu Integration (PARTIAL)

### WinMerge Implementation
- Extensive context menus on right-click:
  - Copy/merge operations
  - Line filtering options
  - Navigation commands
  - View toggling
  - Diff block operations
- Shell integration (Windows Explorer)
- Customizable context menu items

### SmartMerge Status
- ❌ No context menus
- All operations require toolbar/keyboard shortcuts
- Less discoverable for casual users

### Recommendation: Add Context Menus

```vue
<script setup>
  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    
    const contextMenu = [
      { label: 'Copy to Left', action: 'copy-left', shortcut: 'Alt+←' },
      { label: 'Copy to Right', action: 'copy-right', shortcut: 'Alt+→' },
      { separator: true },
      { label: 'Copy All to Left', action: 'copy-all-left', shortcut: 'Ctrl+Alt+←' },
      { label: 'Copy All to Right', action: 'copy-all-right', shortcut: 'Ctrl+Alt+→' },
      { separator: true },
      { label: 'Select Block', action: 'select-block' },
      { label: 'Mark as Resolved', action: 'mark-resolved' },
    ];
    
    showContextMenu(contextMenu, event);
  }
</script>

<template>
  <div class="diff-region" @contextmenu="handleContextMenu">
    <!-- diff content -->
  </div>
</template>
```

**Complexity:** Low (1 week)  
**User Value:** Medium - UX improvement

---

## 7. File Encoding Detection & Handling (MISSING)

### WinMerge Implementation
- **FileTextEncoding.cpp** - automatic encoding detection
- **LoadSaveCodepageDlg.cpp** - encoding selection UI
- Supports UTF-8, UTF-16, ASCII, various code pages
- Automatic detection with manual override
- Encoding mismatch warnings
- Re-encode option before merge

### SmartMerge Status
- ❌ **Not implemented** (backend may support, frontend doesn't show)
- No encoding detection UI
- No encoding mismatch warnings
- Potential data corruption risk

### Recommendation: Add Encoding Support

```vue
<!-- In FileCompareView.vue / settings -->
<script setup>
  interface FileMetadata {
    path: string;
    encoding: string;
    lineEnding: 'LF' | 'CRLF' | 'CR';
    size: number;
    lastModified: Date;
  }
  
  const leftMetadata = ref<FileMetadata>();
  const rightMetadata = ref<FileMetadata>();
  
  async function detectEncoding(path: string) {
    return await invoke('detect_encoding', { path });
  }
</script>

<template>
  <div class="file-metadata">
    <div class="metadata-panel">
      <h3>Left File</h3>
      <span>Encoding: {{ leftMetadata?.encoding }}</span>
      <span>Line Ending: {{ leftMetadata?.lineEnding }}</span>
      <button @click="changeEncoding('left')">Change...</button>
    </div>
    
    <div v-if="encodingMismatch" class="warning">
      ⚠️ Encoding mismatch detected
    </div>
  </div>
</template>
```

**Complexity:** Medium (2-3 weeks with backend work)  
**User Value:** High - essential for real-world files

---

## 8. Inline Diff / Word-Level Highlighting (GOOD)

### WinMerge Implementation
- Highlights character-level changes within lines
- Different styling for word diffs vs. block diffs
- Optional inline diff display

### SmartMerge Status
- ✅ Inline highlighting implemented (engine/smart.rs)
- ✅ Visual differentiation working
- Could enhance with:
  - Toggle for inline diffs on/off
  - Adjustable sensitivity

**Current Quality:** GOOD - No action needed immediately

---

## 9. Folder Comparison UI (GOOD, but could enhance)

### WinMerge Implementation
- **DirView.cpp** (4,100+ lines) - comprehensive folder view
- Tree-based navigation
- Multiple sort/filter options
- Column customization
- Statistics summaries
- Batch operations on selected items

### SmartMerge Status
- ✅ FolderCompareView.vue exists (532 lines)
- ✅ Comparison and navigation working
- ✅ Status indicators functioning
- Could add:
  - Tree view instead of flat list
  - Column customization dialog
  - Batch copy/delete operations
  - More detailed statistics

### Recommendation: Enhance Folder View

```vue
<!-- In FolderCompareView.vue -->
<script setup>
  interface FolderViewSettings {
    viewMode: 'tree' | 'flat' | 'side-by-side';
    visibleColumns: Array<'name' | 'status' | 'size' | 'modified' | 'type'>;
    sortBy: 'name' | 'status' | 'size' | 'modified';
    sortOrder: 'asc' | 'desc';
    showHiddenFiles: boolean;
  }
  
  const viewSettings = ref<FolderViewSettings>({
    viewMode: 'tree',
    visibleColumns: ['name', 'status', 'size', 'modified'],
    sortBy: 'name',
    sortOrder: 'asc',
    showHiddenFiles: false,
  });
</script>
```

**Complexity:** Medium (2-3 weeks)  
**User Value:** Medium-High

---

## 10. Keyboard Shortcuts & Discoverability (PARTIAL)

### WinMerge Implementation
- Extensive keyboard shortcuts (50+)
- Documented in Help menu
- Context-sensitive (different shortcuts for different views)
- Customizable through Options

### SmartMerge Status
- ✅ Keyboard shortcuts implemented (App.vue - 12+ shortcuts)
- ⚠️ Limited documentation
- ❌ No keyboard shortcut editor/customization
- ❌ No discoverability UI (help panel)

### Recommendation: Improvements

```vue
<!-- Add keyboard shortcuts panel -->
<script setup>
  const shortcuts = [
    { key: 'Ctrl+O', description: 'Open files for comparison', category: 'File' },
    { key: 'Ctrl+Shift+O', description: 'Open folders', category: 'File' },
    { key: 'Ctrl+S', description: 'Save changes', category: 'File' },
    { key: 'Alt+↓', description: 'Next difference', category: 'Navigation' },
    { key: 'Alt+↑', description: 'Previous difference', category: 'Navigation' },
    { key: 'Alt+→', description: 'Copy to right', category: 'Merge' },
    { key: 'Alt+←', description: 'Copy to left', category: 'Merge' },
    { key: 'Ctrl+Alt+→', description: 'Copy all to right', category: 'Merge' },
    { key: 'Ctrl+Alt+←', description: 'Copy all to left', category: 'Merge' },
  ];
</script>

<template>
  <div class="keyboard-shortcuts-help">
    <div v-for="category in groupedShortcuts" :key="category.name">
      <h3>{{ category.name }}</h3>
      <table>
        <tr v-for="shortcut in category.shortcuts" :key="shortcut.key">
          <td class="key-binding">{{ shortcut.key }}</td>
          <td class="description">{{ shortcut.description }}</td>
        </tr>
      </table>
    </div>
  </div>
</template>
```

**Complexity:** Low (1 week)  
**User Value:** Medium

---

## 11. Status Bar Information (PARTIAL)

### WinMerge Implementation
- **MergeStatusBar.cpp** - comprehensive status display
- Shows:
  - Current file encoding
  - EOL style of file
  - Cursor position (line/column)
  - Diff count summary
  - Filter status
  - Read-only indicators
  - Current algorithm
  - Progress indicators

### SmartMerge Status
- ✅ StatusBar.vue exists but minimal content
- ❌ Limited information displayed
- ❌ Most data not shown

### Recommendation: Enhance Status Bar

```vue
<!-- Update StatusBar.vue -->
<script setup>
  const statusInfo = computed(() => ({
    fileEncoding: 'UTF-8',
    lineEnding: 'LF',
    cursorPosition: `${line}:${col}`,
    totalLines: allLines.length,
    diffCount: { added: 5, deleted: 3, modified: 8 },
    readOnly: false,
    algorithm: diffEngine.value,
    filterStatus: `${activeFilters} filters applied`,
  }));
</script>

<template>
  <div class="status-bar">
    <div class="status-item">{{ statusInfo.fileEncoding }}</div>
    <div class="status-item">{{ statusInfo.lineEnding }}</div>
    <div class="status-item">Line {{ statusInfo.cursorPosition }} / {{ statusInfo.totalLines }}</div>
    <div class="status-item">
      <span class="added">+{{ statusInfo.diffCount.added }}</span>
      <span class="deleted">-{{ statusInfo.diffCount.deleted }}</span>
      <span class="modified">~{{ statusInfo.diffCount.modified }}</span>
    </div>
    <div class="status-item" v-if="statusInfo.readOnly">🔒 Read-only</div>
    <div class="status-item">{{ statusInfo.algorithm }}</div>
    <div v-if="statusInfo.filterStatus" class="status-item warning">{{ statusInfo.filterStatus }}</div>
  </div>
</template>
```

**Complexity:** Low (1 week)  
**User Value:** Medium

---

## 12. Docking Panes / Layout Customization (MISSING)

### WinMerge Implementation
- Dockable bars (LocationView, DiffViewBar)
- Splitter customization
- Window positioning saved between sessions
- View hiding/showing toggled in menus

### SmartMerge Status
- ❌ **Not implemented** - fixed layout
- Could benefit from flexible pane arrangement

### Recommendation: Dockable UI (Future)

**Complexity:** Very High (4-6 weeks)  
**User Value:** Medium-High - advanced feature

---

## Priority Implementation Roadmap

### Phase 1: Critical (High Impact, Medium Effort)
*Weeks 1-6*

1. **Location Pane/Minimap** (4 weeks)
   - Essential for large file navigation
   - Major UX improvement
   
2. **Trivial Diff Filtering** (3 weeks)
   - Backend support + UI
   - Critical for real-world use

3. **Color Scheme Expansion** (2 weeks)
   - Moved blocks, syntax categories
   - Professional customization

### Phase 2: Important (Medium Impact, Low Effort)
*Weeks 7-10*

4. **Moved Blocks Visualization** (2 weeks)
   - Connecting lines, statistics
   - Refactoring workflow support

5. **Context Menus** (1 week)
   - UX discoverability
   
6. **Enhanced Status Bar** (1 week)
   - Information density

7. **Keyboard Shortcut Help** (1 week)
   - Better discoverability

### Phase 3: Supporting (Medium Impact, Medium Effort)
*Weeks 11-14*

8. **File Encoding Detection** (2 weeks)
   - Data integrity critical
   
9. **Merge Progress Tracking** (1 week)
   - Quality-of-life enhancement

### Phase 4: Advanced (Lower Priority)
*Future*

10. **Dockable UI Framework** (Major refactor)
11. **Folder View Enhancements** (Tree view, batch ops)
12. **Advanced Filtering UI**

---

## Technical Implementation Notes

### Architecture Impact
- Most features can be added to existing Vue components
- Location pane requires new SVG rendering module
- Trivial diff filtering needs backend integration
- No major architectural changes necessary

### Backend Integration Needs
```rust
// New backend commands needed:
pub async fn filter_diffs(
    opcodes: Vec<Opcode>,
    options: IgnoreOptions,
) -> Result<Vec<Opcode>>

pub async fn detect_encoding(path: &str) -> Result<String>

pub async fn get_moved_blocks(
    opcodes: Vec<Opcode>,
) -> Result<Vec<MovedBlockInfo>>

pub async fn get_diff_statistics(
    opcodes: Vec<Opcode>,
) -> Result<DiffStatistics>
```

### Frontend Components to Create/Enhance
- `LocationPane.vue` (new) - minimap visualization
- `ColorSchemeEditor.vue` (new) - advanced color settings
- `TrivialDiffFilters.vue` (new) - filter UI
- `FileMetadataPanel.vue` (new) - encoding, line endings
- `MergeProgressTracker.vue` - progress visualization
- `KeyboardShortcutsHelp.vue` (new) - shortcuts reference

---

## Competitive Analysis

| Feature | WinMerge | SmartMerge | Notes |
|---------|----------|-----------|-------|
| Modern UI | ❌ MFC | ✅ Vue 3 | SmartMerge advantage |
| Location Pane | ✅ Full | ❌ Missing | WinMerge advantage |
| Color Customization | ✅ Extensive | ⚠️ Basic | WinMerge advantage |
| Trivial Diff Filtering | ✅ Full | ❌ Missing | WinMerge advantage |
| Moved Blocks | ✅ Full | ⚠️ Partial | WinMerge advantage |
| Three-Way Merge | ✅ Solid | ✅ Solid | Tie |
| Keyboard Shortcuts | ✅ 50+ | ⚠️ 12 | WinMerge advantage |
| Code Quality | ⚠️ Legacy | ✅ Modern | SmartMerge advantage |
| Performance | ✅ Native | ✅ Tauri | Tie |
| Encoding Support | ✅ Full | ❌ Missing | WinMerge advantage |
| Cross-Platform | ⚠️ Windows | ✅ Mac/Win | SmartMerge advantage |

---

## Conclusion

**SmartMerge Strengths:**
- Modern, clean technology stack (Vue 3, TypeScript, Rust)
- Better cross-platform support
- Cleaner codebase for maintenance/extension
- Responsive, non-blocking operations

**Opportunities for SmartMerge:**
- Location pane/minimap (foundational UX enhancement)
- Trivial diff filtering (real-world essential)
- Expanded color customization (professional feature)
- Moved block visualization (refactoring workflows)

**Strategic Direction:**
Match WinMerge's feature breadth while maintaining SmartMerge's superior modern architecture and UX. The location pane and trivial diff filtering are the highest-ROI improvements.

---

**Document Version:** 1.0  
**Analysis Date:** February 13, 2026  
**Status:** Ready for implementation planning
