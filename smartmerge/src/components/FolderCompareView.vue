<!--
  Updated FolderCompareView using SmartMerge Backend API
  
  This version integrates with the backend folder comparison API instead of
  receiving pre-computed items as props.
-->

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useFolderCompare } from '@/composables/useSmartMerge';
import type { FolderComparisonItem } from '@/api/smartmerge';

// Props
const props = defineProps<{
  leftRoot: string;
  rightRoot: string;
  baseLeft?: string;
  baseRight?: string;
  recursive?: boolean;
}>();

// Emits
const emit = defineEmits<{
  navigate: [string, string];
  openFile: [string, string];
  loaded: [{ itemCount: number; modifiedCount: number }];
}>();

// Composable
const {
  items,
  loading,
  error,
  stats,
  compareFolders,
  navigateToSubfolder,
} = useFolderCompare();

// Initialize comparison
watch(
  () => [props.leftRoot, props.rightRoot],
  async () => {
    if (props.leftRoot && props.rightRoot) {
      await compareFolders(
        props.leftRoot,
        props.rightRoot,
        props.recursive ?? true
      );

      // Notify parent
      if (items.value) {
        emit('loaded', {
          itemCount: items.value.length,
          modifiedCount: stats.value.modifiedFiles,
        });
      }
    }
  },
  { immediate: true }
);

// Status label helper
const statusLabel = (status: FolderComparisonItem['status']) => {
  switch (status) {
    case 'identical':
      return 'Identical';
    case 'modified':
      return 'Modified';
    case 'addedleft':
      return 'Added (left)';
    case 'addedright':
      return 'Added (right)';
    case 'deletedleft':
      return 'Deleted (left)';
    case 'deletedright':
      return 'Deleted (right)';
    case 'folderleftonly':
      return 'Folder (left only)';
    case 'folderrightonly':
      return 'Folder (right only)';
    default:
      return status;
  }
};

// Navigation helpers
const isNavigableFolder = (item: FolderComparisonItem) =>
  item.item_type === 'folder' && item.left_path && item.right_path;

const isComparableFile = (item: FolderComparisonItem) =>
  item.item_type === 'file' && item.left_path && item.right_path;

// File type class helper
const fileTypeClass = (name: string) => {
  const ext = name.split('.').pop()?.toLowerCase() ?? '';
  switch (ext) {
    case 'py':
      return 'filetype--python';
    case 'rs':
      return 'filetype--rust';
    case 'ts':
    case 'js':
      return 'filetype--javascript';
    case 'vue':
      return 'filetype--vue';
    case 'md':
      return 'filetype--markdown';
    case 'json':
      return 'filetype--json';
    case 'txt':
      return 'filetype--text';
    default:
      return 'filetype--generic';
  }
};

// Filter state - handle all status values
const filterState = ref({
  identical: true,
  modified: true,
  addedleft: true,
  addedright: true,
  deletedleft: true,
  deletedright: true,
  folderleftonly: true,
  folderrightonly: true,
});

// Map status to filter key
const statusKey = (status: FolderComparisonItem['status']): keyof typeof filterState.value => {
  switch (status) {
    case 'identical':
      return 'identical';
    case 'modified':
      return 'modified';
    case 'addedleft':
      return 'addedleft';
    case 'addedright':
      return 'addedright';
    case 'deletedleft':
      return 'deletedleft';
    case 'deletedright':
      return 'deletedright';
    case 'folderleftonly':
      return 'folderleftonly';
    case 'folderrightonly':
      return 'folderrightonly';
    default:
      return 'modified';
  }
};

// Filtered items based on user selections
const filteredItems = computed(() => {
  if (!items.value) return [];
  
  return items.value.filter((item) => {
    // Always show folders
    if (item.item_type === 'folder') return true;
    
    // Filter files by status
    return filterState.value[statusKey(item.status)];
  });
});

// Sorted items (folders first, then alphabetical)
const sortedItems = computed(() => {
  const sorted = [...filteredItems.value];
  return sorted.sort((a, b) => {
    // Folders before files
    if (a.item_type !== b.item_type) {
      return a.item_type === 'folder' ? -1 : 1;
    }
    // Alphabetical
    return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
  });
});

// Path manipulation helpers
const normalizePath = (value: string) => value.replace(/[\\/]+$/, '');

const parentPath = (value: string) => {
  const normalized = normalizePath(value);
  if (!normalized) return '';
  
  const separator = normalized.includes('\\') ? '\\' : '/';
  const parts = normalized.split(separator).filter(Boolean);
  
  if (parts.length <= 1) return '';
  
  const parentParts = parts.slice(0, -1);
  const prefix = normalized.startsWith(separator) ? separator : '';
  return prefix + parentParts.join(separator);
};

// Parent navigation
const parentLeft = computed(() => parentPath(props.leftRoot));
const parentRight = computed(() => parentPath(props.rightRoot));

const canNavigateParent = computed(() => {
  if (!parentLeft.value || !parentRight.value) return false;
  
  // Can navigate up if we have base paths and we're not already at base
  if (props.baseLeft && props.baseRight) {
    const normalizedLeft = normalizePath(props.leftRoot);
    const normalizedRight = normalizePath(props.rightRoot);
    const normalizedBaseLeft = normalizePath(props.baseLeft);
    const normalizedBaseRight = normalizePath(props.baseRight);
    
    return (
      normalizedLeft !== normalizedBaseLeft ||
      normalizedRight !== normalizedBaseRight
    );
  }
  
  return true;
});

// Event handlers
const onParentNavigate = () => {
  if (canNavigateParent.value) {
    emit('navigate', parentLeft.value, parentRight.value);
  }
};

const onRowActivate = async (item: FolderComparisonItem) => {
  if (isNavigableFolder(item)) {
    // Navigate into subfolder
    const newLeft = item.left_path as string;
    const newRight = item.right_path as string;
    
    // Use backend to navigate
    await navigateToSubfolder(newLeft, newRight);
    
    // Emit event for parent to update paths if needed
    emit('navigate', newLeft, newRight);
  } else if (isComparableFile(item)) {
    // Open file comparison
    emit('openFile', item.left_path as string, item.right_path as string);
  }
};

// Refresh comparison
const refresh = async () => {
  if (props.leftRoot && props.rightRoot) {
    await compareFolders(
      props.leftRoot,
      props.rightRoot,
      props.recursive ?? true
    );
  }
};

// Expose for parent
defineExpose({
  refresh,
  items,
  stats,
});
</script>

<template>
  <section class="folder-compare">
    <!-- Loading indicator -->
    <div v-if="loading" class="loading-banner">
      <span class="spinner">⏳</span>
      Comparing folders...
    </div>

    <!-- Error banner -->
    <div v-if="error" class="error-banner">
      <span class="error-icon">⚠️</span>
      {{ error }}
      <button @click="refresh" class="retry-button">Retry</button>
    </div>

    <!-- Stats bar -->
    <div v-if="stats && !loading" class="stats-bar">
      <span class="stat">Total: {{ stats.totalFiles }}</span>
      <span class="stat stat--modified">Modified: {{ stats.modifiedFiles }}</span>
      <span class="stat stat--identical">Identical: {{ stats.identical }}</span>
      <span class="stat stat--left">Left only: {{ stats.leftOnly }}</span>
      <span class="stat stat--right">Right only: {{ stats.rightOnly }}</span>
    </div>

    <!-- Filters -->
    <header class="folder-filters">
      <span class="filter-label">Filter:</span>
      <label class="filter-chip">
        <input v-model="filterState.identical" type="checkbox" />
        Identical
      </label>
      <label class="filter-chip">
        <input v-model="filterState.modified" type="checkbox" />
        Modified
      </label>
      <label class="filter-chip">
        <input v-model="filterState.addedleft" type="checkbox" />
        Added (L)
      </label>
      <label class="filter-chip">
        <input v-model="filterState.addedright" type="checkbox" />
        Added (R)
      </label>
      <label class="filter-chip">
        <input v-model="filterState.deletedleft" type="checkbox" />
        Deleted (L)
      </label>
      <label class="filter-chip">
        <input v-model="filterState.deletedright" type="checkbox" />
        Deleted (R)
      </label>
    </header>

    <!-- Folder table -->
    <div class="folder-table">
      <div class="folder-row folder-row--head">
        <span>Name</span>
        <span>Status</span>
      </div>

      <!-- Parent navigation row -->
      <div
        v-if="canNavigateParent"
        class="folder-row folder-row--parent"
        role="button"
        tabindex="0"
        @dblclick="onParentNavigate"
        @keydown.enter="onParentNavigate"
      >
        <span class="folder-name">
          <span class="folder-icon filetype--folder" aria-hidden="true">
            <svg viewBox="0 0 20 20">
              <path
                d="M2 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z"
              />
            </svg>
          </span>
          ..
        </span>
        <span></span>
      </div>

      <!-- Empty state -->
      <div v-if="!items || items.length === 0" class="folder-row">
        <span>No items to display.</span>
        <span>---</span>
      </div>

      <!-- Filtered empty state -->
      <div v-else-if="filteredItems.length === 0" class="folder-row">
        <span>No items match the current filters.</span>
        <span>---</span>
      </div>

      <!-- Items -->
      <div
        v-for="item in sortedItems"
        :key="item.name"
        class="folder-row"
        :class="{
          'folder-row--disabled': !isNavigableFolder(item) && !isComparableFile(item),
          'folder-row--folder': item.item_type === 'folder',
          'folder-row--file': item.item_type === 'file',
          [`folder-row--${item.status}`]: true,
        }"
        :role="isNavigableFolder(item) || isComparableFile(item) ? 'button' : undefined"
        :tabindex="isNavigableFolder(item) || isComparableFile(item) ? 0 : -1"
        @dblclick="onRowActivate(item)"
        @keydown.enter="onRowActivate(item)"
      >
        <span class="folder-name">
          <span
            class="folder-icon"
            :class="
              item.item_type === 'folder' ? 'filetype--folder' : fileTypeClass(item.name)
            "
            aria-hidden="true"
          >
            <!-- Folder icon -->
            <svg v-if="item.item_type === 'folder'" viewBox="0 0 20 20">
              <path
                d="M2 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z"
              />
            </svg>
            <!-- File icon -->
            <svg v-else viewBox="0 0 20 20">
              <path d="M4 2h7l5 5v9a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zm7 1v4h4" />
            </svg>
          </span>
          {{ item.name }}
        </span>
        <span class="status-label">
          {{ item.item_type === 'folder' ? '' : statusLabel(item.status) }}
        </span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.folder-compare {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* Loading & Error banners */
.loading-banner,
.error-banner {
  padding: 0.75rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 500;
}

.loading-banner {
  background: var(--color-background-soft);
  color: var(--color-text-muted);
}

.error-banner {
  background: var(--color-danger-bg, #fdd);
  color: var(--color-danger-fg, #c00);
}

.retry-button {
  margin-left: auto;
  padding: 0.25rem 0.75rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  cursor: pointer;
}

/* Stats bar */
.stats-bar {
  display: flex;
  gap: 1.5rem;
  padding: 0.5rem 1rem;
  background: var(--color-background-mute);
  border-bottom: 1px solid var(--color-border);
  font-size: 0.875rem;
}

.stat {
  font-weight: 500;
}

.stat--modified {
  color: var(--color-warning, #d97706);
}

.stat--identical {
  color: var(--color-success, #059669);
}

.stat--left,
.stat--right {
  color: var(--color-info, #0ea5e9);
}

/* Filters */
.folder-filters {
  display: flex;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  background: var(--color-background-soft);
  border-bottom: 1px solid var(--color-border);
  align-items: center;
}

.filter-label {
  font-weight: 500;
  margin-right: 0.5rem;
}

.filter-chip {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.25rem 0.5rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  cursor: pointer;
  user-select: none;
}

.filter-chip input[type='checkbox'] {
  cursor: pointer;
}

/* Table */
.folder-table {
  flex: 1;
  overflow: auto;
}


.folder-row {
  display: grid;
  grid-template-columns: 1fr auto;
  padding: 0.5rem 1rem;
  gap: 1rem;
  border-bottom: 1px solid var(--color-border-soft, #e5e7eb);
  align-items: center;
  overflow-x: hidden;
  min-height: 2.25rem; /* Fixed minimum height for row consistency */
}

.folder-row--head {
  position: sticky;
  top: 0;
  background: var(--color-background-mute);
  font-weight: 600;
  border-bottom: 2px solid var(--color-border);
  z-index: 1;
  min-height: 2.25rem; /* Match data row height for consistency */
}

.folder-row--parent {
  background: var(--color-background-soft);
  cursor: pointer;
  font-weight: 500;
}

.folder-row--parent:hover {
  background: var(--color-background-hover, #f3f4f6);
}

.folder-row[role='button'] {
  cursor: pointer;
}

.folder-row[role='button']:hover {
  background: var(--color-background-hover, #f9fafb);
}

.folder-row--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Status-based row styling */
.folder-row--modified {
  background: var(--color-diff-modified-bg-light, #fffbeb);
}

.folder-row--only_left {
  background: var(--color-diff-added-bg-light, #f0fdf4);
}

.folder-row--only_right {
  background: var(--color-diff-deleted-bg-light, #fef2f2);
}

/* Folder name with icon */
.folder-name {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.folder-icon {
  width: 1.25rem;
  height: 1.25rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.folder-icon svg {
  width: 100%;
  height: 100%;
  fill: currentColor;
}

/* File type colors */
.filetype--folder {
  color: var(--color-folder, #facc15);
}

.filetype--python {
  color: var(--color-python, #3776ab);
}

.filetype--rust {
  color: var(--color-rust, #ce422b);
}

.filetype--javascript,
.filetype--vue {
  color: var(--color-js, #f7df1e);
}

.filetype--markdown {
  color: var(--color-markdown, #083fa1);
}

.filetype--json {
  color: var(--color-json, #5e5c6c);
}

.filetype--text,
.filetype--generic {
  color: var(--color-text-muted);
}

/* Status label */
.status-label {
  font-size: 0.875rem;
  font-weight: 500;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  white-space: nowrap;
}
</style>
