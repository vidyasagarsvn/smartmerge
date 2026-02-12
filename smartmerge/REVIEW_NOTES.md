# Codebase Review Notes (2026-02-12)

## Findings
- High: Folder entries are marked Identical without recursive comparison, so a directory with modified contents can appear identical until users drill down.
  - Location: src-tauri/src/folder_compare.rs (FolderItem status for folders)
- Medium: smart_diff does linear scans on each mismatch, which can degrade to O(n^2) on large files.
  - Location: src-tauri/src/smart_diff.rs (find_next_match usage)
- Medium: updateMenuState is called via watchEffect and after many actions, which likely sends redundant IPC updates.
  - Location: src/App.vue (watchEffect + repeated await updateMenuState)
- Low: Last-path persistence is written but no longer read, leaving dead state.
  - Location: src/App.vue (smartmerge.lastLeftPath / smartmerge.lastRightPath)
- Low: The engine action and prop remain but there is no UI entry; this is effectively dead code.
  - Location: src/components/TopToolbar.vue, src/components/FileCompareView.vue, src/App.vue
- Low: DeletedLeft/DeletedRight statuses are defined but never emitted.
  - Location: src-tauri/src/models.rs
- Low: compare_files and compare_lines duplicate engine selection logic.
  - Location: src-tauri/src/lib.rs

## Question / Assumption
- Should folder comparison be shallow (current behavior) or should it surface a modified status if any child differs?

## Suggested Follow-ups
- Make folder compare recursive or summary-based so folder status reflects descendant changes.
- Consolidate menu-state updates into a single watch or a debounced update.
- Either re-enable engine toggle UI or remove engine-related dead code and props.
- Remove last-path persistence or restore the read-on-start behavior.
