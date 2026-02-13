# SmartMerge Backend API Documentation

Complete reference for all Tauri backend commands available in SmartMerge.

## Table of Contents

- [Diff Operations](#diff-operations)
- [Highlighting Operations](#highlighting-operations)
- [Merge Operations](#merge-operations)
- [File & Utility Operations](#file--utility-operations)
- [Type Definitions](#type-definitions)

---

## Diff Operations

### `compare_files`

Compare two files and return complete diff result.

**Command:** `compare_files`

**Parameters:**
- `leftPath` (string): Path to left file
- `rightPath` (string): Path to right file
- `engine` (string): Algorithm to use - `"myers"`, `"smart"`, or `"patience"` (default: `"myers"`)

**Returns:** `DiffResult`

**Example:**
```typescript
const result = await SmartMergeAPI.diff.compareFiles(
  '/path/to/left.txt',
  '/path/to/right.txt',
  'myers'
);
```

---

### `compare_lines`

Compare two arrays of lines directly.

**Command:** `compare_lines`

**Parameters:**
- `leftLines` (string[]): Left lines array
- `rightLines` (string[]): Right lines array
- `engine` (string): Algorithm to use

**Returns:** `Opcode[]`

**Example:**
```typescript
const opcodes = await SmartMergeAPI.diff.compareLines(
  ['line 1', 'line 2'],
  ['line 1 modified', 'line 2'],
  'patience'
);
```

---

### `compare_folders_command`

Compare two folder structures recursively.

**Command:** `compare_folders_command`

**Parameters:**
- `leftPath` (string): Path to left folder
- `rightPath` (string): Path to right folder

**Returns:** `FolderItem[]`

**Example:**
```typescript
const items = await SmartMergeAPI.diff.compareFolders(
  '/path/to/folder1',
  '/path/to/folder2'
);
```

---

## Highlighting Operations

### `highlight_diff`

Generate visual highlights for a two-way diff with theme support.

**Command:** `highlight_diff`

**Parameters:**
- `leftLines` (string[]): Left lines
- `rightLines` (string[]): Right lines
- `algorithm` (string): Diff algorithm
- `theme` (string): Color theme - `"light"`, `"dark"`, or `"auto"`

**Returns:** `HighlightResult`

**Example:**
```typescript
const highlights = await SmartMergeAPI.highlight.highlightDiff(
  leftLines,
  rightLines,
  'myers',
  'dark'
);
```

**Features:**
- Block-level highlighting (added, deleted, changed, moved)
- Inline/word-level diff for single-line changes
- GitHub-inspired color schemes
- Theme-aware colors

---

### `highlight_three_way`

Generate highlights for three-way merge with conflict visualization.

**Command:** `highlight_three_way`

**Parameters:**
- `baseLines` (string[]): Base/ancestor lines
- `leftLines` (string[]): Left/ours lines
- `rightLines` (string[]): Right/theirs lines
- `theme` (string): Color theme

**Returns:** `ThreeWayHighlightResult`

**Example:**
```typescript
const highlights = await SmartMergeAPI.highlight.highlightThreeWay(
  baseLines,
  leftLines,
  rightLines,
  'light'
);
```

---

## Merge Operations

All merge operations require a `mergeId` to identify the merge session. Multiple concurrent sessions are supported.

### `start_merge`

Initialize a new merge session with three-way diff.

**Command:** `start_merge`

**Parameters:**
- `mergeId` (string): Unique identifier for this merge session
- `baseLines` (string[]): Base version
- `leftLines` (string[]): Left version (ours)
- `rightLines` (string[]): Right version (theirs)

**Returns:** `MergeState`

**Example:**
```typescript
const state = await SmartMergeAPI.merge.startMerge(
  'merge-session-1',
  baseLines,
  leftLines,
  rightLines
);
```

---

### `merge_accept_left`

Accept the left (ours) version of a conflict block.

**Command:** `merge_accept_left`

**Parameters:**
- `mergeId` (string): Merge session ID
- `blockIndex` (number): Index of the block to resolve

**Returns:** `MergeState` (updated)

**Example:**
```typescript
const state = await SmartMergeAPI.merge.acceptLeft('merge-session-1', 0);
```

---

### `merge_accept_right`

Accept the right (theirs) version of a conflict block.

**Command:** `merge_accept_right`

**Parameters:**
- `mergeId` (string): Merge session ID
- `blockIndex` (number): Block index

**Returns:** `MergeState`

**Example:**
```typescript
const state = await SmartMergeAPI.merge.acceptRight('merge-session-1', 0);
```

---

### `merge_accept_both`

Accept both versions (combine left and right).

**Command:** `merge_accept_both`

**Parameters:**
- `mergeId` (string): Merge session ID
- `blockIndex` (number): Block index
- `leftFirst` (boolean): If true, left lines come first; otherwise right first

**Returns:** `MergeState`

**Example:**
```typescript
const state = await SmartMergeAPI.merge.acceptBoth('merge-session-1', 0, true);
```

---

### `merge_apply_custom`

Apply a custom resolution with user-provided lines.

**Command:** `merge_apply_custom`

**Parameters:**
- `mergeId` (string): Merge session ID
- `blockIndex` (number): Block index
- `customLines` (string[]): Custom resolution lines

**Returns:** `MergeState`

**Example:**
```typescript
const state = await SmartMergeAPI.merge.applyCustom(
  'merge-session-1',
  0,
  ['custom line 1', 'custom line 2']
);
```

---

### `merge_undo`

Undo the last merge action.

**Command:** `merge_undo`

**Parameters:**
- `mergeId` (string): Merge session ID

**Returns:** `MergeState`

**Example:**
```typescript
const state = await SmartMergeAPI.merge.undo('merge-session-1');
```

---

### `merge_redo`

Redo the next merge action (after undo).

**Command:** `merge_redo`

**Parameters:**
- `mergeId` (string): Merge session ID

**Returns:** `MergeState`

**Example:**
```typescript
const state = await SmartMergeAPI.merge.redo('merge-session-1');
```

---

### `merge_auto_resolve`

Automatically resolve all trivial conflicts.

**Command:** `merge_auto_resolve`

**Parameters:**
- `mergeId` (string): Merge session ID

**Returns:** `{ count: number, state: MergeState }`

**Example:**
```typescript
const { count, state } = await SmartMergeAPI.merge.autoResolve('merge-session-1');
console.log(`Auto-resolved ${count} conflicts`);
```

---

### `merge_build_result`

Build the final merged result.

**Command:** `merge_build_result`

**Parameters:**
- `mergeId` (string): Merge session ID
- `includeUnresolved` (boolean): If true, includes conflict markers for unresolved blocks

**Returns:** `MergeResult`

**Example:**
```typescript
const result = await SmartMergeAPI.merge.buildResult('merge-session-1', true);
console.log(`Merged successfully: ${result.success}`);
console.log(`Conflicts remaining: ${result.conflicts_remaining}`);
```

---

## File & Utility Operations

### `save_file`

Save lines to a file.

**Command:** `save_file`

**Parameters:**
- `path` (string): File path
- `lines` (string[]): Lines to save

**Returns:** `void`

**Example:**
```typescript
await SmartMergeAPI.file.saveFile('/path/to/output.txt', lines);
```

---

### `get_path_kind`

Check if a path is a file, directory, or missing.

**Command:** `get_path_kind`

**Parameters:**
- `path` (string): Path to check

**Returns:** `"file" | "dir" | "other" | "missing"`

**Example:**
```typescript
const kind = await SmartMergeAPI.file.getPathKind('/some/path');
if (kind === 'file') {
  // It's a file
}
```

---

## Type Definitions

### DiffResult

```typescript
interface DiffResult {
  left_lines: string[];
  right_lines: string[];
  opcodes: Opcode[];
  algorithm_used?: string;
  total_changes?: number;
  moved_blocks?: MovedBlockInfo[];
  options_used?: DiffOptionsInfo;
}
```

### Opcode

```typescript
interface Opcode {
  tag: string;               // "equal", "insert", "delete", "replace"
  i1: number;                // Left start
  i2: number;                // Left end (exclusive)
  j1: number;                // Right start
  j2: number;                // Right end (exclusive)
  block_type?: BlockType;
  is_trivial?: boolean;
  context_before?: number;
  context_after?: number;
  moved_from?: [number, number];
  moved_to?: [number, number];
}
```

### BlockType

```typescript
type BlockType = 'Equal' | 'Insert' | 'Delete' | 'Replace' | 'Moved';
```

### HighlightResult

```typescript
interface HighlightResult {
  left_highlights: LineHighlight[];
  right_highlights: LineHighlight[];
  color_scheme: ColorScheme;
  theme: Theme;
}
```

### LineHighlight

```typescript
interface LineHighlight {
  line_number: number;
  style: HighlightStyle;
  block_type: BlockType;
  inline_highlights: InlineHighlight[];
  is_conflict: boolean;
}
```

### MergeState

```typescript
interface MergeState {
  blocks: BlockStatus[];
  unresolved_conflicts: number[];
  resolved_conflicts: number[];
  stats: MergeStats;
}
```

### MergeStats

```typescript
interface MergeStats {
  total_blocks: number;
  conflict_blocks: number;
  resolved_blocks: number;
  auto_merged_blocks: number;
  pending_blocks: number;
}
```

### MergeResult

```typescript
interface MergeResult {
  merged_lines: string[];
  success: boolean;
  conflicts_resolved: number;
  conflicts_remaining: number;
  merged_blocks: MergedBlockInfo[];
}
```

---

## Usage Patterns

### Basic File Comparison

```typescript
// Compare files with highlighting
const result = await SmartMergeAPI.diff.compareFiles(
  leftPath,
  rightPath,
  'myers'
);

const highlights = await SmartMergeAPI.highlight.highlightDiff(
  result.left_lines,
  result.right_lines,
  'myers',
  'dark'
);

// Render highlights in UI
renderDiff(highlights);
```

### Three-Way Merge Workflow

```typescript
// 1. Start merge session
const mergeId = `merge-${Date.now()}`;
const state = await SmartMergeAPI.merge.startMerge(
  mergeId,
  baseLines,
  leftLines,
  rightLines
);

// 2. Auto-resolve trivial conflicts
const { count } = await SmartMergeAPI.merge.autoResolve(mergeId);
console.log(`Auto-resolved ${count} conflicts`);

// 3. Manually resolve remaining conflicts
for (const blockIndex of state.unresolved_conflicts) {
  await SmartMergeAPI.merge.acceptLeft(mergeId, blockIndex);
}

// 4. Build final result
const result = await SmartMergeAPI.merge.buildResult(mergeId, false);

// 5. Save merged file
await SmartMergeAPI.file.saveFile(outputPath, result.merged_lines);
```

### Folder Comparison

```typescript
const items = await SmartMergeAPI.diff.compareFolders(
  leftFolder,
  rightFolder
);

const modifiedFiles = items.filter(item => item.status === 'modified');
const addedFiles = items.filter(item => item.status === 'addedright');
```

---

## Error Handling

All API functions return Promises that may reject with error messages:

```typescript
try {
  const result = await SmartMergeAPI.diff.compareFiles(leftPath, rightPath);
} catch (error) {
  console.error('Failed to compare files:', error);
}
```

---

## Performance Considerations

- **Algorithm Selection**: 
  - `myers` - Most accurate, good for most cases
  - `smart` - Fastest, good for large files
  - `patience` - Best for code with unique lines
  
- **Merge Sessions**: Sessions are stored in memory - clean up when done

- **Large Files**: Consider chunking or streaming for very large files

---

## Integration with Vue Components

Use the composable pattern for reactive state management:

```typescript
import { ref } from 'vue';
import SmartMergeAPI from '@/api/smartmerge';

export function useDiff() {
  const diffResult = ref<DiffResult | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function compare(left: string, right: string) {
    loading.value = true;
    error.value = null;
    try {
      diffResult.value = await SmartMergeAPI.diff.compareFiles(left, right);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  return { diffResult, loading, error, compare };
}
```
