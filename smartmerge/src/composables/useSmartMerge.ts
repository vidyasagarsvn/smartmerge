/**
 * Vue Composables for SmartMerge Backend API
 * 
 * Reactive wrappers around backend operations for use in Vue components.
 */

import { ref, computed, type Ref } from 'vue';
import { DiffAPI, HighlightAPI, MergeAPI, TrivialDiffAPI, FileAPI } from '@/api/smartmerge';
import type {
  DiffResult,
  HighlightResult,
  ThreeWayHighlightResult,
  MergeState,
  MergeResult,
  FolderItem,
  DiffAlgorithm,
  Theme,
  Opcode,
  TrivialChangeStats,
} from '@/api/smartmerge';

// ============================================================================
// Diff Composable
// ============================================================================

export function useDiff() {
  const diffResult = ref<DiffResult | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function compareFiles(
    leftPath: string,
    rightPath: string,
    algorithm: DiffAlgorithm = 'myers'
  ) {
    loading.value = true;
    error.value = null;
    try {
      diffResult.value = await DiffAPI.compareFiles(
        leftPath,
        rightPath,
        algorithm
      );
    } catch (e) {
      error.value = String(e);
      diffResult.value = null;
    } finally {
      loading.value = false;
    }
  }

  async function compareLines(
    leftLines: string[],
    rightLines: string[],
    algorithm: DiffAlgorithm = 'myers'
  ) {
    loading.value = true;
    error.value = null;
    try {
      const opcodes = await DiffAPI.compareLines(
        leftLines,
        rightLines,
        algorithm
      );
      diffResult.value = {
        left_lines: leftLines,
        right_lines: rightLines,
        opcodes,
      };
    } catch (e) {
      error.value = String(e);
      diffResult.value = null;
    } finally {
      loading.value = false;
    }
  }

  const hasChanges = computed(() => {
    return diffResult.value ? diffResult.value.total_changes ?? 0 > 0 : false;
  });

  const changesCount = computed(() => {
    return diffResult.value?.total_changes ?? 0;
  });

  return {
    diffResult,
    loading,
    error,
    hasChanges,
    changesCount,
    compareFiles,
    compareLines,
  };
}

// ============================================================================
// Highlighting Composable
// ============================================================================

export function useHighlighting() {
  const highlightResult = ref<HighlightResult | null>(null);
  const threeWayHighlightResult = ref<ThreeWayHighlightResult | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function highlightDiff(
    leftLines: string[],
    rightLines: string[],
    algorithm: DiffAlgorithm = 'myers',
    theme: Theme = 'auto'
  ) {
    loading.value = true;
    error.value = null;
    try {
      highlightResult.value = await HighlightAPI.highlightDiff(
        leftLines,
        rightLines,
        algorithm,
        theme
      );
    } catch (e) {
      error.value = String(e);
      highlightResult.value = null;
    } finally {
      loading.value = false;
    }
  }

  async function highlightThreeWay(
    baseLines: string[],
    leftLines: string[],
    rightLines: string[],
    theme: Theme = 'auto'
  ) {
    loading.value = true;
    error.value = null;
    try {
      threeWayHighlightResult.value = await HighlightAPI.highlightThreeWay(
        baseLines,
        leftLines,
        rightLines,
        theme
      );
    } catch (e) {
      error.value = String(e);
      threeWayHighlightResult.value = null;
    } finally {
      loading.value = false;
    }
  }

  const conflictCount = computed(() => {
    return threeWayHighlightResult.value?.conflict_regions.length ?? 0;
  });

  return {
    highlightResult,
    threeWayHighlightResult,
    loading,
    error,
    conflictCount,
    highlightDiff,
    highlightThreeWay,
  };
}

// ============================================================================
// Merge Composable
// ============================================================================

export function useMerge(mergeId?: string) {
  const id = ref(mergeId ?? `merge-${Date.now()}`);
  const state = ref<MergeState | null>(null);
  const result = ref<MergeResult | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function startMerge(
    baseLines: string[],
    leftLines: string[],
    rightLines: string[]
  ) {
    loading.value = true;
    error.value = null;
    try {
      state.value = await MergeAPI.startMerge(
        id.value,
        baseLines,
        leftLines,
        rightLines
      );
    } catch (e) {
      error.value = String(e);
      state.value = null;
    } finally {
      loading.value = false;
    }
  }

  async function acceptLeft(blockIndex: number) {
    loading.value = true;
    error.value = null;
    try {
      state.value = await MergeAPI.acceptLeft(id.value, blockIndex);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function acceptRight(blockIndex: number) {
    loading.value = true;
    error.value = null;
    try {
      state.value = await MergeAPI.acceptRight(id.value, blockIndex);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function acceptBoth(blockIndex: number, leftFirst: boolean = true) {
    loading.value = true;
    error.value = null;
    try {
      state.value = await MergeAPI.acceptBoth(
        id.value,
        blockIndex,
        leftFirst
      );
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function applyCustom(blockIndex: number, customLines: string[]) {
    loading.value = true;
    error.value = null;
    try {
      state.value = await MergeAPI.applyCustom(
        id.value,
        blockIndex,
        customLines
      );
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function undo() {
    loading.value = true;
    error.value = null;
    try {
      state.value = await MergeAPI.undo(id.value);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function redo() {
    loading.value = true;
    error.value = null;
    try {
      state.value = await MergeAPI.redo(id.value);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function autoResolve() {
    loading.value = true;
    error.value = null;
    try {
      const { count, state: newState } = await MergeAPI.autoResolve(
        id.value
      );
      state.value = newState;
      return count;
    } catch (e) {
      error.value = String(e);
      return 0;
    } finally {
      loading.value = false;
    }
  }

  async function buildResult(includeUnresolved: boolean = true) {
    loading.value = true;
    error.value = null;
    try {
      result.value = await MergeAPI.buildResult(
        id.value,
        includeUnresolved
      );
    } catch (e) {
      error.value = String(e);
      result.value = null;
    } finally {
      loading.value = false;
    }
  }

  const isComplete = computed(() => {
    return state.value?.stats.pending_blocks === 0;
  });

  const completionPercentage = computed(() => {
    if (!state.value) return 0;
    const { resolved_blocks, total_blocks } = state.value.stats;
    return total_blocks > 0 ? (resolved_blocks / total_blocks) * 100 : 0;
  });

  const unresolvedConflicts = computed(() => {
    return state.value?.unresolved_conflicts ?? [];
  });

  const stats = computed(() => state.value?.stats ?? null);

  return {
    id,
    state,
    result,
    loading,
    error,
    isComplete,
    completionPercentage,
    unresolvedConflicts,
    stats,
    startMerge,
    acceptLeft,
    acceptRight,
    acceptBoth,
    applyCustom,
    undo,
    redo,
    autoResolve,
    buildResult,
  };
}

// ============================================================================
// Folder Comparison Composable
// ============================================================================

export function useFolderCompare() {
  const items = ref<FolderItem[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);


  // Normalize status from backend (snake_case) to filter key (lowercase, no underscore)
  function normalizeStatus(status: string): string {
    return status.replace(/_/g, '').toLowerCase();
  }

  async function compareFolders(leftPath: string, rightPath: string, recursive: boolean = true) {
    loading.value = true;
    error.value = null;
    try {
      const rawItems = await DiffAPI.compareFolders(leftPath, rightPath);
      // Normalize status for all items
      items.value = rawItems.map(item => ({
        ...item,
        status: normalizeStatus(item.status),
      }));
    } catch (e) {
      error.value = String(e);
      items.value = [];
    } finally {
      loading.value = false;
    }
  }

  async function navigateToSubfolder(leftPath: string, rightPath: string) {
    await compareFolders(leftPath, rightPath);
  }

  const modifiedItems = computed(() => {
    return items.value.filter((item) => item.status === 'modified');
  });

  const addedToLeft = computed(() => {
    return items.value.filter((item) => item.status === 'addedleft');
  });

  const addedToRight = computed(() => {
    return items.value.filter((item) => item.status === 'addedright');
  });

  const deletedFromLeft = computed(() => {
    return items.value.filter((item) => item.status === 'deletedleft');
  });

  const deletedFromRight = computed(() => {
    return items.value.filter((item) => item.status === 'deletedright');
  });

  const identical = computed(() => {
    return items.value.filter((item) => item.status === 'identical');
  });

  const stats = computed(() => ({
    totalFiles: items.value.length,
    modifiedFiles: modifiedItems.value.length,
    identical: identical.value.length,
    leftOnly: addedToLeft.value.length,
    rightOnly: addedToRight.value.length,
  }));

  return {
    items,
    loading,
    error,
    stats,
    modifiedItems,
    addedToLeft,
    addedToRight,
    deletedFromLeft,
    deletedFromRight,
    identical,
    compareFolders,
    navigateToSubfolder,
  };
}

// ============================================================================
// File Operations Composable
// ============================================================================

export function useFileOps() {
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function saveFile(path: string, lines: string[]) {
    loading.value = true;
    error.value = null;
    try {
      await FileAPI.saveFile(path, lines);
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      loading.value = false;
    }
  }

  async function getPathKind(path: string) {
    loading.value = true;
    error.value = null;
    try {
      return await FileAPI.getPathKind(path);
    } catch (e) {
      error.value = String(e);
      return 'missing';
    } finally {
      loading.value = false;
    }
  }

  return {
    loading,
    error,
    saveFile,
    getPathKind,
  };
}
