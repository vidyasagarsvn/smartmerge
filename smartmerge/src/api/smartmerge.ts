/**
 * SmartMerge Backend API
 * 
 * TypeScript client for interacting with the Tauri Rust backend.
 * Provides type-safe wrappers around all available commands.
 */

import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types
// ============================================================================

export type DiffAlgorithm = 'myers' | 'smart' | 'patience';
export type Theme = 'light' | 'dark' | 'auto';

export interface Opcode {
  tag: string;
  i1: number;
  i2: number;
  j1: number;
  j2: number;
  block_type?: BlockType;
  is_trivial?: boolean;
  context_before?: number;
  context_after?: number;
  moved_from?: [number, number];
  moved_to?: [number, number];
}

export type BlockType = 'Equal' | 'Insert' | 'Delete' | 'Replace' | 'Moved';

export interface DiffResult {
  left_lines: string[];
  right_lines: string[];
  opcodes: Opcode[];
  algorithm_used?: string;
  total_changes?: number;
  moved_blocks?: MovedBlockInfo[];
  options_used?: DiffOptionsInfo;
}

export interface MovedBlockInfo {
  left_start: number;
  left_end: number;
  right_start: number;
  right_end: number;
  size: number;
}

export interface DiffOptionsInfo {
  ignore_case: boolean;
  ignore_whitespace: string;
  ignore_blank_lines: boolean;
  ignore_comments: boolean;
}

export interface HighlightStyle {
  background: string;
  foreground: string;
  highlighted: boolean;
  show_gutter: boolean;
  gutter_color?: string;
  css_classes: string[];
  border?: string;
}

export interface InlineHighlight {
  start: number;
  end: number;
  background: string;
}

export interface LineHighlight {
  line_number: number;
  style: HighlightStyle;
  block_type: BlockType;
  inline_highlights: InlineHighlight[];
  is_conflict: boolean;
}

export interface ColorScheme {
  added_bg: string;
  added_fg: string;
  deleted_bg: string;
  deleted_fg: string;
  changed_bg: string;
  changed_fg: string;
  moved_bg: string;
  moved_fg: string;
  trivial_bg: string;
  trivial_fg: string;
  equal_bg: string;
  equal_fg: string;
  conflict_bg: string;
  conflict_fg: string;
  inline_bg: string;
  border_color: string;
}

export interface HighlightResult {
  left_highlights: LineHighlight[];
  right_highlights: LineHighlight[];
  color_scheme: ColorScheme;
  theme: Theme;
}

export type ConflictType = 'none' | 'bothmodified' | 'deletemodify' | 'modifydelete' | 'bothadded';

export interface ThreeWayOpcode {
  base_start: number;
  base_end: number;
  left_start: number;
  left_end: number;
  right_start: number;
  right_end: number;
  conflict_type: ConflictType;
  auto_mergeable: boolean;
  resolved_lines?: string[];
}

export interface ConflictRegion {
  start_line: number;
  end_line: number;
  conflict_type: ConflictType;
  auto_mergeable: boolean;
  description: string;
}

export interface ThreeWayHighlightResult {
  base_highlights: LineHighlight[];
  left_highlights: LineHighlight[];
  right_highlights: LineHighlight[];
  color_scheme: ColorScheme;
  theme: Theme;
  conflict_regions: ConflictRegion[];
}

export type MergeAction = 'pending' | 'acceptleft' | 'acceptright' | 'acceptbase' | 'custom' | 'skip' | 'automerged';

export interface BlockStatus {
  block_index: number;
  action: MergeAction;
  has_conflict: boolean;
  conflict_type: ConflictType;
  custom_lines?: string[];
  timestamp?: number;
}

export interface MergeStats {
  total_blocks: number;
  conflict_blocks: number;
  resolved_blocks: number;
  auto_merged_blocks: number;
  pending_blocks: number;
}

export interface MergeState {
  blocks: BlockStatus[];
  unresolved_conflicts: number[];
  resolved_conflicts: number[];
  stats: MergeStats;
}

export interface MergedBlockInfo {
  block_index: number;
  action: MergeAction;
  line_start: number;
  line_end: number;
}

export interface MergeResult {
  merged_lines: string[];
  success: boolean;
  conflicts_resolved: number;
  conflicts_remaining: number;
  merged_blocks: MergedBlockInfo[];
}

export type ItemType = 'file' | 'folder';
export type ItemStatus = 'identical' | 'modified' | 'addedleft' | 'addedright' | 'deletedleft' | 'deletedright' | 'folderleftonly' | 'folderrightonly';

export interface FolderItem {
  name: string;
  item_type: ItemType;
  status: ItemStatus;
  left_path?: string;
  right_path?: string;
}

export interface ChangeTypeBreakdown {
  whitespace_only: number;
  case_only: number;
  line_endings: number;
  tab_space: number;
  other_trivial: number;
}

export interface TrivialChangeStats {
  total_changes: number;
  trivial_count: number;
  non_trivial_count: number;
  trivial_percentage: number;
  breakdown: ChangeTypeBreakdown;
}

// ============================================================================
// Diff Operations
// ============================================================================

export class DiffAPI {
  /**
   * Compare two files and return diff result
   */
  static async compareFiles(
    leftPath: string,
    rightPath: string,
    algorithm: DiffAlgorithm = 'myers'
  ): Promise<DiffResult> {
    return invoke<DiffResult>('compare_files', {
      leftPath,
      rightPath,
      engine: algorithm,
    });
  }

  /**
   * Compare two arrays of lines and return opcodes
   */
  static async compareLines(
    leftLines: string[],
    rightLines: string[],
    algorithm: DiffAlgorithm = 'myers'
  ): Promise<Opcode[]> {
    return invoke<Opcode[]>('compare_lines', {
      leftLines,
      rightLines,
      engine: algorithm,
    });
  }

  /**
   * Compare folder structures
   */
  static async compareFolders(
    leftPath: string,
    rightPath: string
  ): Promise<FolderItem[]> {
    return invoke<FolderItem[]>('compare_folders_command', {
      leftPath,
      rightPath,
    });
  }
}

// ============================================================================
// Highlighting Operations
// ============================================================================

export class HighlightAPI {
  /**
   * Generate highlights for a two-way diff
   */
  static async highlightDiff(
    leftLines: string[],
    rightLines: string[],
    algorithm: DiffAlgorithm = 'myers',
    theme: Theme = 'auto'
  ): Promise<HighlightResult> {
    return invoke<HighlightResult>('highlight_diff', {
      leftLines,
      rightLines,
      algorithm,
      theme,
    });
  }

  /**
   * Generate highlights for a three-way merge
   */
  static async highlightThreeWay(
    baseLines: string[],
    leftLines: string[],
    rightLines: string[],
    theme: Theme = 'auto'
  ): Promise<ThreeWayHighlightResult> {
    return invoke<ThreeWayHighlightResult>('highlight_three_way', {
      baseLines,
      leftLines,
      rightLines,
      theme,
    });
  }
}

// ============================================================================
// Trivial Diff Filtering
// ============================================================================

export interface LineMapping {
  index: number;
  maps_to: number | null;
}

export interface GhostLineLayout {
  left_mappings: LineMapping[];
  right_mappings: LineMapping[];
}

export class TrivialDiffAPI {
  /**
   * Analyze a diff result and mark trivial changes
   */
  static async analyzeDiffForTrivial(
    diffResult: DiffResult,
    treatCaseAsTrivial: boolean = false
  ): Promise<[DiffResult, TrivialChangeStats]> {
    return invoke<[DiffResult, TrivialChangeStats]>('analyze_diff_for_trivial', {
      diffResult,
      treatCaseAsTrivial,
    });
  }

  /**
   * Get trivial change statistics for a diff result
   */
  static async getTrivialStats(
    diffResult: DiffResult,
    treatCaseAsTrivial: boolean = false
  ): Promise<TrivialChangeStats> {
    return invoke<TrivialChangeStats>('get_trivial_stats', {
      diffResult,
      treatCaseAsTrivial,
    });
  }

  /**
   * Filter out trivial changes from opcodes (keep non-trivial and equal blocks)
   */
  static async filterOutTrivialChanges(opcodes: Opcode[]): Promise<Opcode[]> {
    return invoke<Opcode[]>('filter_out_trivial_changes', {
      opcodes,
    });
  }

  /**
   * Get only trivial changes from opcodes
   */
  static async getOnlyTrivialChanges(opcodes: Opcode[]): Promise<Opcode[]> {
    return invoke<Opcode[]>('get_only_trivial_changes', {
      opcodes,
    });
  }
}

// ============================================================================
// Ghost Line Mapping (WinMerge-style intelligent alignment)
// ============================================================================

export class GhostLineAPI {
  /**
   * Compute intelligent line mappings using Levenshtein distance
   * This creates optimal ghost line placements for visual alignment
   */
  static async computeGhostLineMappings(
    leftLines: string[],
    rightLines: string[],
    opcodes: Opcode[]
  ): Promise<GhostLineLayout[]> {
    return invoke<GhostLineLayout[]>('compute_ghost_line_mappings', {
      leftLines,
      rightLines,
      opcodes,
    });
  }
}

// ============================================================================
// Merge Operations
// ============================================================================

export class MergeAPI {
  /**
   * Start a new merge session
   */
  static async startMerge(
    mergeId: string,
    baseLines: string[],
    leftLines: string[],
    rightLines: string[]
  ): Promise<MergeState> {
    return invoke<MergeState>('start_merge', {
      mergeId,
      baseLines,
      leftLines,
      rightLines,
    });
  }

  /**
   * Accept left version of a block
   */
  static async acceptLeft(
    mergeId: string,
    blockIndex: number
  ): Promise<MergeState> {
    return invoke<MergeState>('merge_accept_left', {
      mergeId,
      blockIndex,
    });
  }

  /**
   * Accept right version of a block
   */
  static async acceptRight(
    mergeId: string,
    blockIndex: number
  ): Promise<MergeState> {
    return invoke<MergeState>('merge_accept_right', {
      mergeId,
      blockIndex,
    });
  }

  /**
   * Accept both versions (left then right, or right then left)
   */
  static async acceptBoth(
    mergeId: string,
    blockIndex: number,
    leftFirst: boolean = true
  ): Promise<MergeState> {
    return invoke<MergeState>('merge_accept_both', {
      mergeId,
      blockIndex,
      leftFirst,
    });
  }

  /**
   * Apply custom resolution
   */
  static async applyCustom(
    mergeId: string,
    blockIndex: number,
    customLines: string[]
  ): Promise<MergeState> {
    return invoke<MergeState>('merge_apply_custom', {
      mergeId,
      blockIndex,
      customLines,
    });
  }

  /**
   * Undo last merge action
   */
  static async undo(mergeId: string): Promise<MergeState> {
    return invoke<MergeState>('merge_undo', { mergeId });
  }

  /**
   * Redo next merge action
   */
  static async redo(mergeId: string): Promise<MergeState> {
    return invoke<MergeState>('merge_redo', { mergeId });
  }

  /**
   * Auto-resolve all trivial conflicts
   */
  static async autoResolve(
    mergeId: string
  ): Promise<{ count: number; state: MergeState }> {
    const result = await invoke<[number, MergeState]>('merge_auto_resolve', {
      mergeId,
    });
    return { count: result[0], state: result[1] };
  }

  /**
   * Build final merge result
   */
  static async buildResult(
    mergeId: string,
    includeUnresolved: boolean = true
  ): Promise<MergeResult> {
    return invoke<MergeResult>('merge_build_result', {
      mergeId,
      includeUnresolved,
    });
  }
}

// ============================================================================
// File & Utility Operations
// ============================================================================

export class FileAPI {
  /**
   * Save lines to a file
   */
  static async saveFile(path: string, lines: string[]): Promise<void> {
    return invoke<void>('save_file', { path, lines });
  }

  /**
   * Get path kind (file, directory, or missing)
   */
  static async getPathKind(path: string): Promise<'file' | 'dir' | 'other' | 'missing'> {
    return invoke<'file' | 'dir' | 'other' | 'missing'>('get_path_kind', { path });
  }
}

// ============================================================================
// Unified API Export
// ============================================================================

export const SmartMergeAPI = {
  diff: DiffAPI,
  highlight: HighlightAPI,
  merge: MergeAPI,
  file: FileAPI,
};

export default SmartMergeAPI;
