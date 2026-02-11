# Folder Comparison Implementation

## Overview
Successfully implemented folder/directory comparison feature for SmartMerge with non-recursive navigation, as specified.

## Features Implemented

### 1. Core Logic (`smartmerge/core/folder_compare.py`)
- **`compare_folders(left_path, right_path)`**: Compares two folders at root level
  - Returns list of `FolderItem` entries with status for each item
  - Detects: identical files, modified files, added files (left/right only)
  - Handles folders: marks bi-present folders as navigable, single-side folders as read-only
  
- **Item Status Classification**:
  - `IDENTICAL`: File exists on both sides with same content
  - `MODIFIED`: File exists on both sides with different content
  - `ADDED_LEFT`: File only on left side
  - `ADDED_RIGHT`: File only on right side
  - `FOLDER_LEFT_ONLY`: Folder only on left (non-navigable)
  - `FOLDER_RIGHT_ONLY`: Folder only on right (non-navigable)

- **Navigation Control**:
  - `is_folder_navigable(item)`: Returns True only if folder exists on both sides
  - Non-navigable folders are grayed out in UI

### 2. UI Widget (`smartmerge/ui/folder_compare_widget.py`)
- **FolderCompareWidget**: QTableWidget-based display
  - Columns: Name, Type, Left Status, Right Status
  - Displays emojis (📁 for folders, 📄 for files)
  - Color-coded status indicators:
    - Light green (✓): Identical items
    - Light yellow (≠): Modified items
    - Light green/red: Added/deleted items
  
- **Interaction**:
  - Double-click folders present on both sides → navigate deeper
  - Double-click files on both sides → compare in file view
  - Right-click context menu for actions
  - Non-navigable folders appear grayed out (visual feedback)

- **Signals**:
  - `folder_selected(left_path, right_path)`: Emitted when navigating into folder
  - `file_selected(left_path, right_path)`: Emitted when comparing files

### 3. Main Window Integration (`smartmerge/main.py`)
- **Stacked Widget**: Switches between file and folder comparison views
  - Index 0: File comparison (FileCompareWidget)
  - Index 1: Folder comparison (FolderCompareWidget)

- **New Menu Items**:
  - `File → Open Folders` (Ctrl+Shift+O): Opens folder selection dialogs
  - Maintains backward compatibility with `File → Open Files` (Ctrl+O)

- **Navigation Support**:
  - `navigation_stack`: Tracks folder hierarchy for future "back" navigation
  - Folder selection triggers view switch and updates status bar
  - File selection from folder view loads into file comparison view

### 4. Unit Tests (`tests/test_folder_compare.py`)
- **9 comprehensive tests** covering:
  - Basic folder comparison structure
  - Identical file detection
  - Modified file detection
  - Added files (left/right only)
  - Bi-side folder detection
  - Single-side folder detection
  - Navigability checks
  - Alphabetical sorting of results
  
- **All tests passing**: 12/12 tests (9 folder tests + 3 existing tests)

## Keyboard Shortcuts
- `Ctrl+O`: Open Files (existing)
- `Ctrl+Shift+O`: Open Folders (new)
- `Ctrl+Q`: Exit (existing)
- `Ctrl+Shift+F`: Font (existing)

## Design Decisions

### Non-Recursive by Default
- Only shows root-level items
- User must navigate into folders to see their contents
- Prevents performance issues with large directory trees
- Keeps UI focused and manageable

### Bi-Presence Requirement for Navigation
- Folders can only be navigated if they exist on **both** sides
- Single-side folders are visually disabled (grayed out)
- Prevents confusion about what "merging" means
- Makes the merge intent clear

### File Content Comparison
- Uses existing Myers diff algorithm to detect content changes
- Treats files with same path but different content as "MODIFIED"
- Supports round-trip to file comparison view

## Technical Improvements
- Extracted folder comparison into separate module (clean separation of concerns)
- Reuses Myers diff algorithm for file content detection
- QSettings integration preserves last folder paths
- Font selector applies to both file and folder views
- All linting checks pass (ruff)

## Future Enhancement Opportunities
1. Add "Back" button/breadcrumb for folder navigation
2. Implement recursive folder comparison (optional mode)
3. Add folder filtering (by name, extension)
4. Support for 3-way folder comparison
5. Sync/copy operations between folders
6. Diff summary statistics (total files, modified count, etc.)

## File Structure
```
smartmerge/
  core/
    folder_compare.py      ← New: Folder comparison logic
    diff_engine.py         ← Existing: Reused for file comparison
  ui/
    folder_compare_widget.py  ← New: UI for folder view
    file_compare_widget.py    ← Existing: File comparison UI
    open_files_dialog.py      ← Existing: File selection
  main.py                   ← Updated: Stacked widget, folder menu
tests/
  test_folder_compare.py    ← New: 9 unit tests
  test_myers_diff.py        ← Existing: Diff tests
  test_import.py            ← Existing: Import tests
```

## Status
✅ **Complete**: All requirements implemented, tested, and linted.
