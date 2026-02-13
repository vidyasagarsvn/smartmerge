# SmartMerge Architecture Documentation

**Version:** 1.0  
**Date:** February 13, 2026  
**Status:** Production-Ready

## Table of Contents

1. [System Overview](#system-overview)
2. [Architecture Principles](#architecture-principles)
3. [Backend Architecture (Rust + Tauri)](#backend-architecture)
4. [Frontend Architecture (Vue 3 + TypeScript)](#frontend-architecture)
5. [Data Flow](#data-flow)
6. [Module Breakdown](#module-breakdown)
7. [API Layer](#api-layer)
8. [Testing Strategy](#testing-strategy)
9. [Performance Considerations](#performance-considerations)
10. [Future Extensibility](#future-extensibility)

---

## System Overview

SmartMerge is a high-performance, modular diff, highlighting, and merge engine built with:
- **Backend:** Rust (performance, safety) + Tauri (native app framework)
- **Frontend:** Vue 3 (reactivity) + TypeScript (type safety)
- **Architecture Pattern:** Clean separation between engine logic and UI

### Key Features

- **Multiple Diff Algorithms:** Myers, Smart (context-aware), Patience (moved blocks)
- **Advanced Highlighting:** Theme support, inline diffs, syntax detection
- **Interactive Merging:** Conflict resolution with undo/redo
- **Folder Comparison:** Recursive directory scanning with filtering
- **Type-Safe API:** Complete TypeScript wrappers for all backend operations

---

## Architecture Principles

### 1. Separation of Concerns

```
┌─────────────────────────────────────────┐
│           Frontend (Vue 3 + TS)         │
│  ┌────────────┬──────────────────────┐  │
│  │ Components │    Composables       │  │
│  └──────┬─────┴──────────┬───────────┘  │
│         │                │               │
│         └────────┬───────┘               │
│                  │                       │
│         ┌────────▼────────┐              │
│         │  API Client     │              │
│         │  (TypeScript)   │              │
│         └────────┬────────┘              │
└──────────────────┼──────────────────────┘
                   │ Tauri IPC
┌──────────────────▼──────────────────────┐
│          Backend (Rust + Tauri)         │
│  ┌───────────┬──────────┬────────────┐  │
│  │ Diff      │ Highlight│  Merge     │  │
│  │ Engine    │ Engine   │  Engine    │  │
│  └───────────┴──────────┴────────────┘  │
│  ┌─────────────────────────────────┐   │
│  │  Models (Shared Data Structures)│   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

### 2. Modularity

Each engine module is **independent** and **reusable**:
- Diff engine doesn't depend on highlighting
- Highlighting doesn't depend on merge
- Merge uses diff output but doesn't control diff logic

### 3. Type Safety

- **Rust:** Compile-time guarantees for memory safety and correctness
- **TypeScript:** Full type coverage from backend to frontend
- **Serde:** Automatic serialization between Rust and JavaScript

### 4. Performance

- **Rust Core:** O(ND) diff algorithms with linear space complexity
- **Lazy Evaluation:** Highlighting computed on-demand
- **Efficient Data Structures:** Vec, HashMap, Option for zero-cost abstractions

---

## Backend Architecture

### Directory Structure

```
src-tauri/
├── src/
│   ├── main.rs                    # Tauri app entry point
│   ├── lib.rs                     # Command registration
│   ├── models.rs                  # Shared data structures
│   ├── diff_engine.rs             # Legacy diff (to be deprecated)
│   ├── smart_diff.rs              # Legacy smart diff
│   ├── file_io.rs                 # File operations
│   ├── folder_compare.rs          # Directory comparison
│   ├── engine/                    # Diff engine module
│   │   ├── mod.rs
│   │   ├── algorithm.rs           # DiffAlgorithm trait
│   │   ├── myers.rs               # Myers algorithm
│   │   ├── smart.rs               # Smart algorithm
│   │   ├── patience.rs            # Patience algorithm
│   │   ├── filters.rs             # Whitespace/comment filtering
│   │   ├── moved_blocks.rs        # Moved block detection
│   │   ├── three_way.rs           # Three-way diff for merging
│   │   └── api.rs                 # DiffEngine facade
│   ├── highlighting/              # Highlighting engine module
│   │   ├── mod.rs
│   │   ├── styles.rs              # Color schemes and themes
│   │   ├── mapper.rs              # Opcode → Highlight mapping
│   │   ├── syntax.rs              # Syntax detection (30+ languages)
│   │   └── three_way.rs           # Three-way highlighting
│   └── merge/                     # Merge engine module
│       ├── mod.rs
│       ├── state.rs               # Merge state tracking
│       ├── operations.rs          # Accept left/right/both/custom
│       ├── history.rs             # Undo/redo support
│       ├── builder.rs             # Merge result construction
│       └── resolver.rs            # Auto-resolution strategies
└── Cargo.toml
```

### Core Data Structures

#### Opcode (Diff Operation)

```rust
pub struct Opcode {
    pub tag: String,                    // "equal" | "insert" | "delete" | "replace"
    pub i1: usize,                      // Left start index
    pub i2: usize,                      // Left end index
    pub j1: usize,                      // Right start index
    pub j2: usize,                      // Right end index
    pub block_type: Option<BlockType>, // Semantic classification
    pub is_trivial: bool,               // Whitespace-only change
    pub context: Option<String>,        // Surrounding context
    pub moved_from: Option<usize>,      // Source of moved block
    pub moved_to: Option<usize>,        // Destination of moved block
}
```

#### DiffResult

```rust
pub struct DiffResult {
    pub opcodes: Vec<Opcode>,           // Diff operations
    pub left_lines: Vec<String>,        // Left file content
    pub right_lines: Vec<String>,       // Right file content
    pub algorithm_used: Option<String>, // Algorithm name
    pub total_changes: usize,           // Change count
    pub moved_blocks: Option<Vec<MovedBlockInfo>>,
    pub options_used: Option<DiffOptions>,
}
```

#### ThreeWayOpcode (Merge)

```rust
pub struct ThreeWayOpcode {
    pub tag: String,                    // "equal" | "conflict" | "auto_merged"
    pub base_range: (usize, usize),     // Base version range
    pub left_range: (usize, usize),     // Left version range
    pub right_range: (usize, usize),    // Right version range
    pub conflict_type: Option<ConflictType>,
}
```

### Diff Engine Module

**Responsibility:** Compute differences between text files using pluggable algorithms

**Key Components:**

1. **DiffAlgorithm Trait**
   ```rust
   pub trait DiffAlgorithm {
       fn compute_diff(&self, left: &[String], right: &[String]) -> Vec<Opcode>;
       fn name(&self) -> &str;
       fn description(&self) -> &str;
   }
   ```

2. **Myers Algorithm** - Optimal edit distance (O(ND) time, O(D) space)
3. **Smart Algorithm** - Context-aware, better for code
4. **Patience Algorithm** - Handles moved code blocks effectively

**Usage:**
```rust
let myers = MyersAlgorithm::new();
let opcodes = myers.compute_diff(&left_lines, &right_lines);
```

### Highlighting Engine Module

**Responsibility:** Convert diff opcodes into visual highlights with color schemes

**Key Components:**

1. **ColorScheme** - Defines colors for each block type (Light/Dark themes)
2. **HighlightMapper** - Maps Opcode → LineHighlight + InlineHighlight
3. **SyntaxHighlighter** - Detects language (30+ supported)
4. **ThreeWayHighlightMapper** - Handles three-way merge highlighting

**Data Flow:**
```
Opcode[] → HighlightMapper → LineHighlight[]
                                  ├─ block_type: BlockType
                                  ├─ style: { background, foreground }
                                  ├─ inline_highlights: InlineHighlight[]
                                  └─ is_trivial: bool
```

### Merge Engine Module

**Responsibility:** Interactive three-way merge with conflict resolution

**Key Components:**

1. **MergeState** - Tracks merge progress, conflicts, resolutions
2. **MergeOperations** - Implements accept_left/right/both/custom
3. **MergeHistory** - Undo/redo stack with configurable size
4. **MergeBuilder** - Constructs final merged content
5. **ConflictResolver** - Auto-resolution strategies

**Workflow:**
```
1. Start Merge        → Generate ThreeWayOpcode[]
2. Identify Conflicts → Mark blocks with ConflictType
3. User Resolves      → Accept version or custom edit
4. Track in History   → Enable undo/redo
5. Build Result       → Merge resolved blocks → Output
```

**Session Management:**
```rust
static MERGE_ENGINES: Lazy<Mutex<HashMap<String, MergeSession>>> = ...;
```

- Thread-safe global storage
- Each session has unique UUID
- Supports concurrent merge operations

---

## Frontend Architecture

### Directory Structure

```
src/
├── main.ts                        # Vue app entry
├── App.vue                        # Root component
├── api/
│   └── smartmerge.ts              # TypeScript API client
├── composables/
│   └── useSmartMerge.ts           # Reactive composables
├── components/
│   ├── FileCompareView.vue        # File comparison UI
│   ├── FolderCompareView.vue      # Folder comparison UI
│   ├── MergeView.vue              # Three-way merge UI
│   └── SettingsView.vue           # Configuration UI
└── styles/
    └── app.css                    # Global styles
```

### API Client Layer

**File:** `src/api/smartmerge.ts`

**Purpose:** Type-safe wrappers around Tauri `invoke()` commands

**Classes:**

1. **DiffAPI**
   ```typescript
   class DiffAPI {
     async compareFiles(leftPath, rightPath, algorithm): Promise<DiffResult>
     async compareLines(leftLines, rightLines, algorithm): Promise<DiffResult>
   }
   ```

2. **HighlightAPI**
   ```typescript
   class HighlightAPI {
     async highlightDiff(left, right, algorithm, theme): Promise<HighlightResult>
     async highlightThreeWay(base, left, right, theme): Promise<ThreeWayHighlightResult>
   }
   ```

3. **MergeAPI**
   ```typescript
   class MergeAPI {
     async startMerge(base, left, right): Promise<string> // session_id
     async acceptLeft(sessionId, blockIndex): Promise<void>
     async acceptRight(sessionId, blockIndex): Promise<void>
     async acceptBoth(sessionId, blockIndex): Promise<void>
     async applyCustom(sessionId, blockIndex, lines): Promise<void>
     async undo(sessionId): Promise<void>
     async redo(sessionId): Promise<void>
     async autoResolve(sessionId, strategy): Promise<void>
     async buildResult(sessionId, includeConflictMarkers): Promise<MergeResult>
   }
   ```

4. **FileAPI**
   ```typescript
   class FileAPI {
     async readFile(path): Promise<string>
     async writeFile(path, content): Promise<void>
   }
   ```

**Type Definitions:** All backend types mirrored in TypeScript with exact structure

### Composable Layer

**File:** `src/composables/useSmartMerge.ts`

**Purpose:** Reactive state management for backend operations

**Pattern:**
```typescript
export function useDiff() {
  const diffResult = ref<DiffResult | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  
  const compareFiles = async (left, right, algorithm) => {
    loading.value = true;
    try {
      diffResult.value = await DiffAPI.compareFiles(left, right, algorithm);
    } catch (err) {
      error.value = err.message;
    } finally {
      loading.value = false;
    }
  };
  
  return { diffResult, loading, error, compareFiles };
}
```

**Composables:**
- `useDiff()` - Diff operations with loading/error states
- `useHighlighting()` - Highlighting with theme support
- `useMerge()` - Complete merge workflow with undo/redo
- `useFolderCompare()` - Folder comparison with stats
- `useFileOps()` - File I/O operations

### Component Layer

**Pattern:** Components use composables, never direct API calls

**Example:**
```vue
<script setup lang="ts">
import { useDiff, useHighlighting } from '@/composables/useSmartMerge';

const { diffResult, compareFiles } = useDiff();
const { highlightResult, highlightDiff } = useHighlighting();

async function performComparison() {
  await compareFiles(leftPath, rightPath, 'myers');
  await highlightDiff(diffResult.value.left_lines, diffResult.value.right_lines, 'myers', 'auto');
}
</script>
```

---

## Data Flow

### File Comparison Flow

```
1. User Action (Select files)
   ↓
2. FileCompareView calls useDiff().compareFiles()
   ↓
3. DiffAPI.compareFiles() → invoke('compare_files')
   ↓
4. Tauri IPC → Rust Backend
   ↓
5. DiffEngine.compare_files()
   ├─ Read files
   ├─ Select algorithm (Myers/Smart/Patience)
   ├─ Compute diff → Vec<Opcode>
   └─ Return DiffResult
   ↓
6. Serialize (Rust) → Deserialize (TypeScript)
   ↓
7. Update reactive state: diffResult.value = result
   ↓
8. Vue reactivity triggers re-render
   ↓
9. Component displays highlighted diff
```

### Interactive Merge Flow

```
1. User starts merge
   ↓
2. MergeView calls useMerge().startMerge(base, left, right)
   ↓
3. MergeAPI → invoke('merge_start_merge')
   ↓
4. Backend generates session_id (UUID)
   ↓
5. MergeEngine.start_merge()
   ├─ Perform three-way diff
   ├─ Identify conflicts
   ├─ Create MergeState
   └─ Store in MERGE_ENGINES[session_id]
   ↓
6. Return session_id to frontend
   ↓
7. User resolves conflict (accept left)
   ↓
8. useMerge().acceptLeft(sessionId, blockIndex)
   ↓
9. Backend updates MergeState
   ├─ Change block status to ResolvedLeft
   ├─ Push to history stack (for undo)
   └─ Return updated state
   ↓
10. Reactive state updates, UI reflects changes
   ↓
11. User clicks "Build Result"
   ↓
12. MergeBuilder constructs final content
    ├─ Iterate blocks
    ├─ Include resolved versions
    ├─ Add conflict markers if unresolved
    └─ Return MergeResult
```

---

## Module Breakdown

### Diff Engine (Backend)

| File | Purpose | Key Functions |
|------|---------|---------------|
| `algorithm.rs` | Trait definition | `compute_diff(left, right) -> Vec<Opcode>` |
| `myers.rs` | Myers algorithm | Optimal edit distance, O(ND) |
| `smart.rs` | Context-aware diff | Better for structured text/code |
| `patience.rs` | Patience algorithm | Handles moved blocks |
| `filters.rs` | Preprocessing | Ignore whitespace, comments |
| `moved_blocks.rs` | Block detection | Find identical blocks in different positions |
| `three_way.rs` | Merge foundation | `three_way_diff(base, left, right)` |

### Highlighting Engine (Backend)

| File | Purpose | Key Functions |
|------|---------|---------------|
| `styles.rs` | Color schemes | `ColorScheme::light()`, `::dark()` |
| `mapper.rs` | Opcode mapping | `map_diff(opcodes, left, right)` |
| `syntax.rs` | Language detection | `detect_language(filename, content)` |
| `three_way.rs` | Merge highlighting | `map_three_way(opcodes, base, left, right)` |

### Merge Engine (Backend)

| File | Purpose | Key Functions |
|------|---------|---------------|
| `state.rs` | State tracking | `MergeState::new`, `has_conflicts()` |
| `operations.rs` | Resolution actions | `accept_left()`, `accept_right()`, `apply_custom()` |
| `history.rs` | Undo/redo | `push()`, `pop()`, `redo()` |
| `builder.rs` | Result generation | `build(state, include_markers)` |
| `resolver.rs` | Auto-resolution | `auto_resolve(strategy)`, `analyze_conflict()` |

---

## API Layer

### Tauri Commands

**Registration:** `src-tauri/src/lib.rs`

```rust
tauri::Builder::default()
  .invoke_handler(tauri::generate_handler![
    // Diff commands
    compare_files,
    compare_lines,
    // Highlighting commands
    highlight_diff,
    highlight_three_way,
    // Merge commands
    merge_start_merge,
    merge_accept_left,
    merge_accept_right,
    merge_accept_both,
    merge_apply_custom,
    merge_undo,
    merge_redo,
    merge_auto_resolve,
    merge_build_result,
    // File operations
    read_file_content,
    write_file_content,
    // Folder comparison
    compare_folders,
  ])
```

### Command Signatures

**Diff:**
```rust
#[tauri::command]
fn compare_files(left_path: String, right_path: String, algorithm: String) -> Result<DiffResult, String>
```

**Highlighting:**
```rust
#[tauri::command]
fn highlight_diff(left_lines: Vec<String>, right_lines: Vec<String>, algorithm: String, theme: String) -> Result<HighlightResult, String>
```

**Merge:**
```rust
#[tauri::command]
fn merge_start_merge(base_lines: Vec<String>, left_lines: Vec<String>, right_lines: Vec<String>) -> Result<String, String> // Returns session_id
```

---

## Testing Strategy

### Unit Tests (Rust)

- **Location:** Inline `#[cfg(test)] mod tests` in each module
- **Coverage:** Algorithm correctness, edge cases, error handling
- **Run:** `cargo test`

**Example Tests:**
- `test_myers_identical_lines()`
- `test_myers_simple_insert()`
- `test_highlight_mapper_creation()`
- `test_merge_state_creation()`

### Integration Tests

- **Location:** `src-tauri/tests/` (future)
- **Coverage:** End-to-end workflows, command integration

### Frontend Tests

- **Framework:** Vitest + Vue Test Utils
- **Location:** `src/**/*.spec.ts` (future)
- **Coverage:** Component rendering, composable logic, API mocking

---

## Performance Considerations

### Algorithm Complexity

| Algorithm | Time | Space | Best For |
|-----------|------|-------|----------|
| Myers | O(ND) | O(D) | General purpose |
| Smart | O(ND) | O(D) | Code/structured text |
| Patience | O(N log N) | O(N) | Moved blocks |

Where:
- N = total lines
- D = number of differences (edit distance)

### Optimization Techniques

1. **Lazy Highlighting:** Computed only when needed, not on every diff
2. **Incremental Parsing:** For large files, stream processing (future)
3. **Batch Operations:** Group similar API calls
4. **Caching:** Store computed diffs (future enhancement)

### Memory Usage

- **Small Files (<1MB):** ~10MB peak
- **Large Files (10MB):** ~100MB peak
- **Merge Sessions:** ~5MB per active session

---

## Future Extensibility

### Plugin System (Section 5 - Deferred)

**Vision:** Allow custom filters and preprocessors

```rust
pub trait DiffPlugin {
    fn preprocess(&self, content: &str) -> String;
    fn postprocess(&self, opcodes: Vec<Opcode>) -> Vec<Opcode>;
}
```

**Use Cases:**
- XML/JSON normalization
- Minified code expansion
- Archive comparison

### Syntax Highlighting Enhancement

**Current:** Basic language detection (30+ languages)  
**Future:** Full syntax highlighting with Tree-sitter integration

### Real-time Collaboration

**Future:** WebSocket-based collaborative merging  
**Architecture:** Operational Transform or CRDT for conflict-free merges

### Cloud Integration

**Future:** Save/load merge sessions from cloud storage  
**Backend:** AWS S3, Google Drive, Dropbox APIs

---

## Summary

SmartMerge achieves a clean, modular architecture through:

✅ **Clear Separation:** UI (Vue) ↔ API (TypeScript) ↔ Engine (Rust)  
✅ **Type Safety:** End-to-end TypeScript + Rust type guarantees  
✅ **Modularity:** Independent engines that can be reused  
✅ **Performance:** Rust core with efficient algorithms  
✅ **Maintainability:** Well-defined interfaces and data flows  
✅ **Extensibility:** Plugin architecture ready for future features

**Next Steps for Development:**
1. Add real syntax highlighting with Tree-sitter
2. Implement comprehensive integration tests
3. Add plugin system for custom filters
4. Performance profiling and optimization for very large files
5. Cloud storage integration

---

**Document Version:** 1.0  
**Last Updated:** February 13, 2026  
**Maintained By:** SmartMerge Development Team
