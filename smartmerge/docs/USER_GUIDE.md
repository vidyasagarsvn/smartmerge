# SmartMerge User Guide

**Version:** 1.0  
**Last Updated:** February 13, 2026

Welcome to SmartMerge! This guide will help you get started with comparing files, resolving merge conflicts, and using advanced features.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [File Comparison](#file-comparison)
3. [Folder Comparison](#folder-comparison)
4. [Three-Way Merge](#three-way-merge)
5. [Settings & Configuration](#settings--configuration)
6. [Advanced Features](#advanced-features)
7. [Keyboard Shortcuts](#keyboard-shortcuts)
8. [Tips & Tricks](#tips--tricks)
9. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Installation

1. **Download** SmartMerge from releases
2. **Install** by opening the `.dmg` (macOS) or `.exe` (Windows)
3. **Launch** SmartMerge from your applications

### First Comparison

1. Click **"Compare Files"** from the welcome screen
2. Select your **left file** (e.g., `old_version.py`)
3. Select your **right file** (e.g., `new_version.py`)
4. SmartMerge automatically highlights the differences!

---

## File Comparison

### Basic File Comparison

**Use Case:** Compare two versions of a file to see what changed

**Steps:**
1. **File → Compare Files** (or `Cmd+O` / `Ctrl+O`)
2. Choose left file (original/base version)
3. Choose right file (modified version)
4. View side-by-side diff with highlighting

### Understanding the Diff View

```
┌──────────────────────────┬──────────────────────────┐
│      Left (Original)     │      Right (Modified)    │
├──────────────────────────┼──────────────────────────┤
│ 1  def hello():          │ 1  def hello():          │  ← Equal (white)
│ 2    print("Hello")      │ 2    print("Hi")         │  ← Changed (yellow)
│ 3                        │ 3    print("World")      │  ← Added (green)
│ 4    return True         │ 4    return True         │  ← Equal (white)
└──────────────────────────┴──────────────────────────┘
```

**Color Legend:**
- 🟢 **Green** - Added lines (only in right)
- 🔴 **Red** - Deleted lines (only in left)
- 🟡 **Yellow** - Modified lines (changed between left/right)
- 🔵 **Blue** - Moved lines (same content, different position)
- ⚪ **White** - Equal lines (no changes)

### Navigating Differences

**Next Difference:** `F8` or click **"Next"** button
**Previous Difference:** `Shift+F8` or click **"Previous"** button

The diff view automatically scrolls to show the selected difference.

### Choosing Diff Algorithms

**Algorithm Selector:** Top toolbar dropdown

1. **Myers (Default)**
   - Best for: General text files
   - Speed: Fast (O(ND))
   - Use when: Standard diff needs

2. **Smart**
   - Best for: Code files (Python, JavaScript, etc.)
   - Speed: Fast (O(ND))
   - Use when: Need context-aware diffing
   - Example: Better at understanding function boundaries

3. **Patience**
   - Best for: Files with moved/rearranged blocks
   - Speed: Slower (O(N log N))
   - Use when: Code has been reorganized
   - Example: Functions moved between files

**Example Scenario:**

```python
# Left file (old_version.py)
def function_a():
    pass

def function_b():
    pass

# Right file (new_version.py)
def function_b():    # Moved up!
    pass

def function_a():    # Moved down!
    pass
```

- **Myers:** Shows as delete + insert
- **Patience:** Detects as moved blocks (🔵 blue highlighting)

---

## Folder Comparison

### Comparing Directories

**Use Case:** Compare entire project folders to find all changed files

**Steps:**
1. **File → Compare Folders** (or `Cmd+Shift+O`)
2. Select left folder (e.g., `project_v1/`)
3. Select right folder (e.g., `project_v2/`)
4. View file comparison table

### Folder Comparison View

```
┌──────────────────┬────────────┐
│ File/Folder      │ Status     │
├──────────────────┼────────────┤
│ 📁 ..            │            │  ← Parent folder
│ 📄 README.md     │ Modified   │  ← File changed
│ 📄 config.json   │ Identical  │  ← No changes
│ 📄 newfile.txt   │ Right only │  ← Added file
│ 📄 oldfile.txt   │ Left only  │  ← Deleted file
│ 📁 src/          │            │  ← Subfolder
└──────────────────┴────────────┘
```

**Actions:**
- **Double-click file:** Open file comparison
- **Double-click folder:** Navigate into subfolder
- **Double-click "..":** Go to parent folder

### Filtering Results

Use the filter checkboxes at the top:

- ☑ **Identical** - Files with no changes
- ☑ **Modified** - Files that changed
- ☑ **Left only** - Files deleted in right
- ☑ **Right only** - Files added in right

**Tip:** Uncheck "Identical" to focus only on changes!

### Statistics Bar

The statistics bar shows:
- **Total:** Total files compared
- **Modified:** Files with differences
- **Identical:** Files with no changes
- **Left only:** Files only in left folder
- **Right only:** Files only in right folder

---

## Three-Way Merge

### What is Three-Way Merge?

**Scenario:** You have:
- **Base:** Original version
- **Left ("Yours"):** Your changes
- **Right ("Theirs"):** Someone else's changes

**Goal:** Merge both sets of changes into one file

### Starting a Merge

1. **File → Three-Way Merge** (or `Cmd+M`)
2. Select **base file** (original)
3. Select **left file** (your version)
4. Select **right file** (their version)
5. Merge view opens with conflict list

### Merge View Layout

```
┌────────────────────────────────────────────────────────┐
│          Block #1 - Both Modified (Conflict)           │
├──────────────┬──────────────┬──────────────────────────┤
│     Base     │     Left      │         Right           │
│  (Original)  │   (Yours)     │       (Theirs)          │
├──────────────┼──────────────┼──────────────────────────┤
│ def hello(): │ def hello():  │ def hello():             │
│   print("Hi")│   print("Hey")│   print("Hello World")  │
└──────────────┴──────────────┴──────────────────────────┘
                [Accept Left] [Accept Right] [Accept Both] [Custom]
```

### Resolving Conflicts

#### Option 1: Accept Left (Your Version)

Click **"Accept Left"** - Uses your changes, discards theirs

**Result:**
```python
def hello():
    print("Hey")
```

#### Option 2: Accept Right (Their Version)

Click **"Accept Right"** - Uses their changes, discards yours

**Result:**
```python
def hello():
    print("Hello World")
```

#### Option 3: Accept Both

Click **"Accept Both"** - Combines both changes (left first, then right)

**Result:**
```python
def hello():
    print("Hey")
    print("Hello World")
```

#### Option 4: Custom Edit

Click **"Custom Edit"** - Write your own resolution

1. Text area opens with left content pre-filled
2. Edit as needed (e.g., combine and modify)
3. Click **"Apply Custom"**

**Example Custom Resolution:**
```python
def hello():
    print("Hey, Hello World!")  # Combined both
```

### Conflict Types

SmartMerge identifies different conflict types:

1. **Both Modified**
   - Both changed the same lines differently
   - Most common conflict type
   - Requires manual resolution

2. **Both Added**
   - Both added new content at same location
   - Usually safe to merge both

3. **Deleted vs Modified**
   - One deleted, other modified
   - Decide: keep modification or honor deletion?

### Auto-Resolve

**Use Case:** Automatically resolve simple, non-overlapping conflicts

**Steps:**
1. Select **resolution strategy** from dropdown:
   - **Prefer Left** - Choose your changes when conflicted
   - **Prefer Right** - Choose their changes when conflicted
   - **Prefer Both** - Combine both when possible
   - **Prefer Base** - Keep original when conflicted

2. Click **"Auto-resolve"**

3. Review auto-resolved blocks

**Note:** Complex conflicts still require manual resolution

### Undo/Redo

Made a mistake? Use undo/redo:
- **Undo:** `Cmd+Z` / `Ctrl+Z` or click **"↶ Undo"**
- **Redo:** `Cmd+Shift+Z` / `Ctrl+Y` or click **"↷ Redo"**

Undo history keeps last 100 actions (configurable in settings).

### Preview Merge Result

Before finalizing:
1. Click **"Preview"**
2. Modal shows complete merged content
3. Review for correctness
4. Close preview

### Complete Merge

When all conflicts are resolved:
1. **"Complete Merge"** button becomes enabled (green)
2. Click to finalize
3. Choose where to save merged file
4. Done!

**Status Indicators:**
- 🔴 **Conflicts remaining:** Button disabled, shows count
- 🟢 **All resolved:** Button enabled, ready to save

---

## Settings & Configuration

Access settings via **File → Settings** or `Cmd+,` / `Ctrl+,`

### Diff Settings

**Algorithm:** Choose default algorithm (Myers/Smart/Patience)

**Context Lines:** Number of unchanged lines around changes (0-10)
- Lower = more compact diff
- Higher = more context for understanding

**Ignore Whitespace:** ☑ Compare ignoring spaces/tabs  
**Ignore Case:** ☑ Treat uppercase and lowercase as same  
**Detect Moved Blocks:** ☑ Identify moved code (requires Patience algorithm)

### Highlighting Settings

**Color Theme:**
- **Auto:** Follow system light/dark mode
- **Light:** Use light color scheme
- **Dark:** Use dark color scheme

**Inline Highlights:** ☑ Show character-level differences within lines

**Show Line Numbers:** ☑ Display line numbers in diff view

**Syntax Highlighting:** ☑ Apply syntax colors (future feature)

### Merge Settings

**Default Resolution Strategy:** Prefer Left/Right/Both/Base

**Auto-resolve Simple Conflicts:** ☑ Automatically resolve non-overlapping

**Include Conflict Markers:** ☑ Use `<<<<` `====` `>>>>` in unresolved output

**Max Undo History Size:** 10-500 (default: 100)

### UI Preferences

**Split View Ratio:** 30%-70% (adjust left/right pane sizes)

**Font Size:** 10-24px (default: 13px)

**Tab Size:** 2-8 spaces (default: 4)

**Wrap Long Lines:** ☑ Wrap lines at viewport edge

### Saving Settings

Settings auto-save to local storage.

**Export Settings:**
1. Click **"Export"**
2. Save `.json` file

**Import Settings:**
1. Click **"Import"**
2. Select `.json` file
3. Settings apply immediately

---

## Advanced Features

### Inline Character Highlighting

When a line is modified, SmartMerge highlights the exact characters that changed:

```
Left:  "The quick brown fox"
Right: "The quick red fox"
                  ^^^^^ (red highlighted)
```

### Trivial Change Detection

SmartMerge identifies trivial changes (whitespace only):

- Shown with reduced opacity
- Can be filtered if needed
- Helps focus on meaningful changes

### Moved Block Detection (Patience Algorithm)

Enable **Detect Moved Blocks** in settings:

```python
# Left
def a():
    pass
def b():
    pass

# Right  
def b():  # ← Detected as moved, not deleted+added
    pass
def a():
    pass
```

Moved blocks highlighted in blue instead of red+green.

### Syntax-Aware Detection

SmartMerge detects 30+ languages automatically:

Python, JavaScript, TypeScript, Rust, Go, Java, C++, C#, Ruby, PHP, Swift, Kotlin, Scala, Bash, PowerShell, JSON, XML, HTML, CSS, YAML, Markdown, SQL, Dockerfile, and more!

Detection based on:
1. File extension (`.py`, `.js`, `.rs`)
2. Shebang line (`#!/usr/bin/env python3`)
3. Content analysis

---

## Keyboard Shortcuts

### Global

| Shortcut | Action |
|----------|--------|
| `Cmd+O` / `Ctrl+O` | Compare Files |
| `Cmd+Shift+O` / `Ctrl+Shift+O` | Compare Folders |
| `Cmd+M` / `Ctrl+M` | Three-Way Merge |
| `Cmd+,` / `Ctrl+,` | Open Settings |
| `Cmd+Q` / `Ctrl+Q` | Quit Application |

### Diff View

| Shortcut | Action |
|----------|--------|
| `F8` | Next Difference |
| `Shift+F8` | Previous Difference |
| `Cmd+F` / `Ctrl+F` | Search in Diff |
| `Cmd+G` / `Ctrl+G` | Go to Line |

### Merge View

| Shortcut | Action |
|----------|--------|
| `Cmd+Z` / `Ctrl+Z` | Undo |
| `Cmd+Shift+Z` / `Ctrl+Y` | Redo |
| `Cmd+Enter` / `Ctrl+Enter` | Complete Merge |
| `Esc` | Cancel Merge |
| `←` `→` | Navigate Blocks |

---

## Tips & Tricks

### Tip 1: Compare Git Versions

Compare committed versions using Git:

```bash
# Save versions to temp files
git show main:file.py > /tmp/main_version.py
git show HEAD:file.py > /tmp/head_version.py

# Compare in SmartMerge
# Left: /tmp/main_version.py
# Right: /tmp/head_version.py
```

### Tip 2: Merge Multiple Files

Create a batch script:

```bash
#!/bin/bash
for file in src/*.py; do
  smartmerge merge base/$file yours/$file theirs/$file -o merged/$file
done
```

### Tip 3: Ignore Whitespace for Code Reviews

When reviewing code with formatting changes:
1. Enable **"Ignore Whitespace"** in settings
2. Focus only on meaningful logic changes
3. Whitespace-only lines appear grayed out

### Tip 4: Use Smart Algorithm for Code

When comparing source code:
- Use **Smart algorithm** (not Myers)
- Better at understanding code structure
- Handles function/class boundaries intelligently

### Tip 5: Preview Before Committing

After merging:
1. Use **"Preview"** to review entire result
2. Search for unresolved conflict markers (`<<<<`)
3. Test merged code before committing

### Tip 6: Export Merge Sessions

Planning to merge later?
1. Start merge, resolve some conflicts
2. **File → Save Merge Session**
3. Resume later with **File → Load Merge Session**

(Note: This is a future feature)

---

## Troubleshooting

### Problem: Diff view is empty

**Solution:**
- Check if files are truly identical
- Try different algorithm (Myers → Smart → Patience)
- Ensure files are text (not binary)

### Problem: Merge shows too many conflicts

**Solution:**
- Use **Auto-resolve** with appropriate strategy
- Check if files diverged too much from base
- Consider manual merge of smaller sections

### Problem: Slow performance with large files

**Solution:**
- Split large files into smaller chunks
- Use command-line tools for files >10MB
- Increase system RAM allocation

### Problem: Unicode characters not displaying

**Solution:**
- Ensure files are UTF-8 encoded
- Check system font supports the characters
- Try re-opening with explicit encoding

### Problem: Undo button disabled

**Solution:**
- No actions to undo yet
- Check if undo history is full (increase in settings)
- Clear history and start fresh if needed

### Problem: Can't save merged result

**Solution:**
- Check file permissions
- Ensure destination directory exists
- Try saving to different location

---

## Example Workflows

### Workflow 1: Code Review

**Scenario:** Review colleague's pull request

1. Export PR diff to files:
   ```bash
   git diff main...feature-branch > /tmp/pr-diff.patch
   ```

2. Or compare branches directly:
   ```bash
   git show main:src/app.py > /tmp/main.py
   git show feature-branch:src/app.py > /tmp/feature.py
   ```

3. Compare in SmartMerge (Smart algorithm)

4. Navigate changes with F8

5. Verify logic correctness

### Workflow 2: Resolving Git Merge Conflict

**Scenario:** Git merge failed with conflicts

1. Git creates conflict markers:
   ```python
   <<<<<<< HEAD
   your_code()
   =======
   their_code()
   >>>>>>> feature-branch
   ```

2. Extract versions:
   ```bash
   git show :1:file.py > base.py  # Common ancestor
   git show :2:file.py > yours.py  # Your version
   git show :3:file.py > theirs.py  # Their version
   ```

3. Open SmartMerge three-way merge

4. Resolve each conflict

5. Save merged result

6. Complete Git merge:
   ```bash
   cp merged.py file.py
   git add file.py
   git commit
   ```

### Workflow 3: Synchronizing Directories

**Scenario:** Sync local project with remote backup

1. Compare folders (Recursive enabled)

2. Filter to show only "Modified" and "Right only"

3. For each file:
   - Modified: Review changes, decide to keep/overwrite
   - Right only: New files to copy over

4. Use file manager to sync based on comparison

---

## Getting Help

- **Documentation:** [docs/](../docs)
- **GitHub Issues:** Report bugs or request features
- **Community:** Discord/Forum (links in README)
- **Email Support:** support@smartmerge.dev

---

**Document Version:** 1.0  
**Last Updated:** February 13, 2026

Happy merging! 🚀
