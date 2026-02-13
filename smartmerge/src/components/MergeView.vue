<!--
  MergeView Component - Three-way merge interface
  
  Provides an interactive UI for resolving merge conflicts between base, left, and right versions.
  Uses the backend merge engine with undo/redo, auto-resolution, and custom edits.
-->

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { useMerge } from '@/composables/useSmartMerge';
import type { ConflictType, ResolutionStrategy, MergeAction } from '@/api/smartmerge';

// Props
const props = defineProps<{
  basePath?: string;
  leftPath?: string;
  rightPath?: string;
  baseContent?: string;
  leftContent?: string;
  rightContent?: string;
  baseLabel?: string;
  leftLabel?: string;
  rightLabel?: string;
}>();

// Emits
const emit = defineEmits<{
  'merge-complete': [result: string];
  'merge-cancelled': [];
}>();

// Composables
const {
  mergeState,
  mergeResult,
  loading,
  error,
  startMerge,
  acceptLeft,
  acceptRight,
  acceptBoth,
  applyCustom,
  undo,
  redo,
  autoResolve,
  buildResult,
  canUndo,
  canRedo,
  hasConflicts,
  stats,
} = useMerge();

// Local state
const sessionId = ref<string>('');
const showPreview = ref(false);
const previewContent = ref<string>('');
const selectedBlockIndex = ref<number | null>(null);
const customEditContent = ref<string>('');
const showCustomEdit = ref(false);
const autoResolveStrategy = ref<ResolutionStrategy>('prefer_left');

// Initialize merge session
onMounted(async () => {
  await initializeMerge();
});

// Watch for prop changes
watch(
  () => [props.basePath, props.leftPath, props.rightPath, props.baseContent, props.leftContent, props.rightContent],
  () => {
    initializeMerge();
  }
);

// Initialize merge
async function initializeMerge() {
  try {
    let id: string;
    
    if (props.basePath && props.leftPath && props.rightPath) {
      // Load from files
      id = await startMerge(props.basePath, props.leftPath, props.rightPath);
    } else if (props.baseContent && props.leftContent && props.rightContent) {
      // Use provided content
      const baseLines = props.baseContent.split('\n');
      const leftLines = props.leftContent.split('\n');
      const rightLines = props.rightContent.split('\n');
      
      id = await startMerge(baseLines, leftLines, rightLines);
    } else {
      throw new Error('Must provide either file paths or content');
    }
    
    sessionId.value = id;
  } catch (err) {
    console.error('Failed to initialize merge:', err);
  }
}

// Computed properties
const blocks = computed(() => mergeState.value?.blocks || []);

const conflictBlocks = computed(() => 
  blocks.value.filter(block => block.status === 'Conflict')
);

const resolvedBlocks = computed(() => 
  blocks.value.filter(block => block.status !== 'Conflict')
);

const currentBlock = computed(() => {
  if (selectedBlockIndex.value === null) return null;
  return blocks.value[selectedBlockIndex.value];
});

const labels = computed(() => ({
  base: props.baseLabel || 'Base',
  left: props.leftLabel || 'Left',
  right: props.rightLabel || 'Right',
}));

// Block selection
function selectBlock(index: number) {
  selectedBlockIndex.value = index;
  customEditContent.value = '';
  showCustomEdit.value = false;
}

// Accept actions
async function acceptLeftForBlock(blockIndex: number) {
  if (!sessionId.value) return;
  await acceptLeft(sessionId.value, blockIndex);
  selectNextConflict();
}

async function acceptRightForBlock(blockIndex: number) {
  if (!sessionId.value) return;
  await acceptRight(sessionId.value, blockIndex);
  selectNextConflict();
}

async function acceptBothForBlock(blockIndex: number) {
  if (!sessionId.value) return;
  await acceptBoth(sessionId.value, blockIndex);
  selectNextConflict();
}

async function acceptCustomForBlock(blockIndex: number) {
  if (!sessionId.value || !customEditContent.value) return;
  const lines = customEditContent.value.split('\n');
  await applyCustom(sessionId.value, blockIndex, lines);
  showCustomEdit.value = false;
  customEditContent.value = '';
  selectNextConflict();
}

// Select next unresolved conflict
function selectNextConflict() {
  const unresolvedIndex = blocks.value.findIndex(
    (block, idx) => idx > (selectedBlockIndex.value || -1) && block.status === 'Conflict'
  );
  
  if (unresolvedIndex !== -1) {
    selectedBlockIndex.value = unresolvedIndex;
  } else {
    // Wrap around to first conflict
    const firstConflict = blocks.value.findIndex(block => block.status === 'Conflict');
    selectedBlockIndex.value = firstConflict !== -1 ? firstConflict : null;
  }
}

// Undo/Redo
async function handleUndo() {
  if (!sessionId.value) return;
  await undo(sessionId.value);
}

async function handleRedo() {
  if (!sessionId.value) return;
  await redo(sessionId.value);
}

// Auto-resolve all conflicts
async function handleAutoResolve() {
  if (!sessionId.value) return;
  await autoResolve(sessionId.value, autoResolveStrategy.value);
}

// Preview merge result
async function showMergePreview() {
  if (!sessionId.value) return;
  
  const result = await buildResult(sessionId.value, true);
  if (result) {
    previewContent.value = result.content;
    showPreview.value = true;
  }
}

// Complete merge
async function completeMerge() {
  if (!sessionId.value) return;
  
  const result = await buildResult(sessionId.value, false);
  if (result) {
    emit('merge-complete', result.content);
  }
}

// Cancel merge
function cancelMerge() {
  emit('merge-cancelled');
}

// Helper to get block type label
function getBlockTypeLabel(conflictType?: ConflictType): string {
  if (!conflictType) return 'No conflict';
  
  switch (conflictType) {
    case 'BothModified':
      return 'Both modified';
    case 'BothAdded':
      return 'Both added';
    case 'DeletedVsModified':
      return 'Deleted vs modified';
    default:
      return conflictType;
  }
}

// Helper to get status color
function getStatusColor(status: string): string {
  switch (status) {
    case 'Conflict':
      return 'status-conflict';
    case 'ResolvedLeft':
      return 'status-left';
    case 'ResolvedRight':
      return 'status-right';
    case 'ResolvedBoth':
      return 'status-both';
    case 'ResolvedCustom':
      return 'status-custom';
    default:
      return 'status-auto';
  }
}

// Initialize custom edit with current block content
function initCustomEdit(block: any) {
  showCustomEdit.value = true;
  
  // Pre-fill with left content by default
  if (block.left_lines) {
    customEditContent.value = block.left_lines.join('\n');
  }
}
</script>

<template>
  <div class="merge-view">
    <!-- Header with stats and actions -->
    <div class="merge-header">
      <div class="merge-title">
        <h3>Three-Way Merge</h3>
        <div v-if="stats" class="merge-stats">
          <span class="stat">Total blocks: {{ stats.totalBlocks }}</span>
          <span class="stat stat-conflict">Conflicts: {{ stats.conflictBlocks }}</span>
          <span class="stat stat-resolved">Resolved: {{ stats.resolvedBlocks }}</span>
        </div>
      </div>
      
      <div class="merge-actions">
        <button @click="handleUndo" :disabled="!canUndo" class="action-btn">
          ↶ Undo
        </button>
        <button @click="handleRedo" :disabled="!canRedo" class="action-btn">
          ↷ Redo
        </button>
        
        <select v-model="autoResolveStrategy" class="strategy-select">
          <option value="prefer_left">Prefer Left</option>
          <option value="prefer_right">Prefer Right</option>
          <option value="prefer_both">Prefer Both</option>
          <option value="prefer_base">Prefer Base</option>
        </select>
        <button @click="handleAutoResolve" class="action-btn action-btn--primary">
          Auto-resolve
        </button>
        
        <button @click="showMergePreview" class="action-btn">
          Preview
        </button>
        <button 
          @click="completeMerge" 
          :disabled="hasConflicts"
          class="action-btn action-btn--success"
        >
          Complete Merge
        </button>
        <button @click="cancelMerge" class="action-btn action-btn--danger">
          Cancel
        </button>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="loading-overlay">
      <div class="spinner">Initializing merge...</div>
    </div>

    <!-- Error state -->
    <div v-if="error" class="error-banner">
      {{ error }}
    </div>

    <!-- Main content -->
    <div v-if="!loading && !error" class="merge-content">
      <!-- Sidebar with block list -->
      <div class="merge-sidebar">
        <div class="sidebar-header">
          <h4>Merge Blocks ({{ blocks.length }})</h4>
        </div>
        
        <div class="block-list">
          <div
            v-for="(block, index) in blocks"
            :key="index"
            class="block-item"
            :class="{
              'block-item--selected': selectedBlockIndex === index,
              [getStatusColor(block.status)]: true,
            }"
            @click="selectBlock(index)"
          >
            <div class="block-header">
              <span class="block-index">#{{ index + 1 }}</span>
              <span class="block-status">{{ block.status }}</span>
            </div>
            <div v-if="block.conflict_type" class="block-type">
              {{ getBlockTypeLabel(block.conflict_type) }}
            </div>
            <div class="block-range">
              Lines: {{ block.base_range.start }}-{{ block.base_range.end }}
            </div>
          </div>
        </div>
      </div>

      <!-- Main merge area -->
      <div class="merge-main">
        <div v-if="currentBlock" class="block-detail">
          <div class="block-detail-header">
            <h4>Block #{{ selectedBlockIndex! + 1 }}</h4>
            <span class="conflict-badge" v-if="currentBlock.conflict_type">
              {{ getBlockTypeLabel(currentBlock.conflict_type) }}
            </span>
            <span class="status-badge" :class="getStatusColor(currentBlock.status)">
              {{ currentBlock.status }}
            </span>
          </div>

          <!-- Three-panel view -->
          <div class="three-panel">
            <!-- Base version -->
            <div class="panel panel-base">
              <div class="panel-header">
                {{ labels.base }}
              </div>
              <div class="panel-content">
                <pre v-if="currentBlock.base_lines">{{ currentBlock.base_lines.join('\n') }}</pre>
                <div v-else class="empty-content">No base content</div>
              </div>
            </div>

            <!-- Left version -->
            <div class="panel panel-left">
              <div class="panel-header">
                {{ labels.left }}
                <button 
                  v-if="currentBlock.status === 'Conflict'"
                  @click="acceptLeftForBlock(selectedBlockIndex!)"
                  class="panel-action"
                >
                  Accept
                </button>
              </div>
              <div class="panel-content">
                <pre v-if="currentBlock.left_lines">{{ currentBlock.left_lines.join('\n') }}</pre>
                <div v-else class="empty-content">No left content</div>
              </div>
            </div>

            <!-- Right version -->
            <div class="panel panel-right">
              <div class="panel-header">
                {{ labels.right }}
                <button 
                  v-if="currentBlock.status === 'Conflict'"
                  @click="acceptRightForBlock(selectedBlockIndex!)"
                  class="panel-action"
                >
                  Accept
                </button>
              </div>
              <div class="panel-content">
                <pre v-if="currentBlock.right_lines">{{ currentBlock.right_lines.join('\n') }}</pre>
                <div v-else class="empty-content">No right content</div>
              </div>
            </div>
          </div>

          <!-- Resolution actions -->
          <div v-if="currentBlock.status === 'Conflict'" class="resolution-actions">
            <button @click="acceptBothForBlock(selectedBlockIndex!)" class="resolution-btn">
              Accept Both (Left + Right)
            </button>
            <button @click="initCustomEdit(currentBlock)" class="resolution-btn">
              Custom Edit
            </button>
          </div>

          <!-- Custom edit area -->
          <div v-if="showCustomEdit" class="custom-edit">
            <div class="custom-edit-header">
              <h5>Custom Resolution</h5>
              <button @click="showCustomEdit = false" class="close-btn">✕</button>
            </div>
            <textarea 
              v-model="customEditContent"
              class="custom-textarea"
              rows="10"
              placeholder="Enter custom resolution..."
            />
            <div class="custom-edit-actions">
              <button 
                @click="acceptCustomForBlock(selectedBlockIndex!)"
                :disabled="!customEditContent"
                class="action-btn action-btn--success"
              >
                Apply Custom
              </button>
              <button @click="showCustomEdit = false" class="action-btn">
                Cancel
              </button>
            </div>
          </div>
        </div>

        <div v-else class="no-selection">
          <p>Select a block from the sidebar to view details and resolve conflicts.</p>
          <div v-if="hasConflicts" class="conflict-summary">
            <h4>⚠️ {{ conflictBlocks.length }} conflict(s) remaining</h4>
            <p>Select a conflict block to begin resolution.</p>
          </div>
          <div v-else class="success-summary">
            <h4>✓ All conflicts resolved!</h4>
            <p>Click "Complete Merge" to finalize.</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Preview modal -->
    <div v-if="showPreview" class="preview-modal">
      <div class="modal-content">
        <div class="modal-header">
          <h3>Merge Preview</h3>
          <button @click="showPreview = false" class="close-btn">✕</button>
        </div>
        <div class="modal-body">
          <pre class="preview-code">{{ previewContent }}</pre>
        </div>
        <div class="modal-footer">
          <button @click="showPreview = false" class="action-btn">Close</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.merge-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* Header */
.merge-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background: var(--color-background-soft);
  border-bottom: 2px solid var(--color-border);
}

.merge-title h3 {
  margin: 0 0 0.5rem 0;
}

.merge-stats {
  display: flex;
  gap: 1rem;
  font-size: 0.875rem;
}

.stat {
  font-weight: 500;
}

.stat-conflict {
  color: var(--color-danger, #dc2626);
}

.stat-resolved {
  color: var(--color-success, #16a34a);
}

.merge-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.action-btn {
  padding: 0.5rem 1rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn--primary {
  background: var(--color-primary, #3b82f6);
  color: white;
  border-color: var(--color-primary, #3b82f6);
}

.action-btn--success {
  background: var(--color-success, #16a34a);
  color: white;
  border-color: var(--color-success, #16a34a);
}

.action-btn--danger {
  background: var(--color-danger, #dc2626);
  color: white;
  border-color: var(--color-danger, #dc2626);
}

.strategy-select {
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
}

/* Loading/Error */
.loading-overlay {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.error-banner {
  padding: 1rem;
  background: var(--color-danger-bg, #fee);
  color: var(--color-danger, #c00);
  font-weight: 500;
}

/* Main content */
.merge-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* Sidebar */
.merge-sidebar {
  width: 250px;
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-header {
  padding: 0.75rem;
  background: var(--color-background-mute);
  border-bottom: 1px solid var(--color-border);
}

.sidebar-header h4 {
  margin: 0;
}

.block-list {
  flex: 1;
  overflow-y: auto;
}

.block-item {
  padding: 0.75rem;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
}

.block-item:hover {
  background: var(--color-background-soft);
}

.block-item--selected {
  background: var(--color-background-mute);
  border-left: 3px solid var(--color-primary, #3b82f6);
}

.block-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.25rem;
}

.block-index {
  font-weight: 600;
}

.block-status {
  font-size: 0.75rem;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  background: var(--color-background);
}

.block-type,
.block-range {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin-top: 0.25rem;
}

/* Status colors */
.status-conflict {
  border-left-color: var(--color-danger, #dc2626) !important;
}

.status-left {
  border-left-color: var(--color-primary, #3b82f6) !important;
}

.status-right {
  border-left-color: var(--color-info, #0ea5e9) !important;
}

.status-both {
  border-left-color: var(--color-success, #16a34a) !important;
}

.status-custom {
  border-left-color: var(--color-warning, #f59e0b) !important;
}

/* Main area */
.merge-main {
  flex: 1;
  overflow: auto;
  padding: 1rem;
}

.block-detail {
  max-width: 1400px;
  margin: 0 auto;
}

.block-detail-header {
  display: flex;
  gap: 1rem;
  align-items: center;
  margin-bottom: 1rem;
}

.block-detail-header h4 {
  margin: 0;
}

.conflict-badge,
.status-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 4px;
  font-size: 0.875rem;
  font-weight: 500;
}

.conflict-badge {
  background: var(--color-warning-bg, #fef3c7);
  color: var(--color-warning, #d97706);
}

/* Three-panel view */
.three-panel {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
  margin-bottom: 1rem;
}

.panel {
  border: 1px solid var(--color-border);
  border-radius: 4px;
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background: var(--color-background-mute);
  font-weight: 600;
  border-bottom: 1px solid var(--color-border);
}

.panel-action {
  padding: 0.25rem 0.75rem;
  background: var(--color-primary, #3b82f6);
  color: white;
  border: none;
  border-radius: 3px;
  cursor: pointer;
  font-size: 0.875rem;
}

.panel-content {
  padding: 0.75rem;
  max-height: 300px;
  overflow: auto;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.875rem;
  line-height: 1.5;
}

.panel-content pre {
  margin: 0;
  white-space: pre-wrap;
}

.empty-content {
  color: var(--color-text-muted);
  font-style: italic;
}

/* Resolution actions */
.resolution-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
}

.resolution-btn {
  padding: 0.5rem 1rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.resolution-btn:hover {
  background: var(--color-background-soft);
}

/* Custom edit */
.custom-edit {
  margin-top: 1rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  padding: 1rem;
  background: var(--color-background-soft);
}

.custom-edit-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.custom-edit-header h5 {
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  font-size: 1.25rem;
  cursor: pointer;
  color: var(--color-text-muted);
}

.custom-textarea {
  width: 100%;
  padding: 0.5rem;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.875rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  margin-bottom: 0.5rem;
}

.custom-edit-actions {
  display: flex;
  gap: 0.5rem;
}

/* No selection state */
.no-selection {
  text-align: center;
  padding: 3rem;
  color: var(--color-text-muted);
}

.conflict-summary,
.success-summary {
  margin-top: 2rem;
  padding: 1.5rem;
  border-radius: 8px;
}

.conflict-summary {
  background: var(--color-warning-bg, #fef3c7);
  color: var(--color-warning, #d97706);
}

.success-summary {
  background: var(--color-success-bg, #d1fae5);
  color: var(--color-success, #16a34a);
}

/* Preview modal */
.preview-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: var(--color-background);
  border-radius: 8px;
  max-width: 90vw;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  border-bottom: 1px solid var(--color-border);
}

.modal-header h3 {
  margin: 0;
}

.modal-body {
  flex: 1;
  overflow: auto;
  padding: 1rem;
}

.preview-code {
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.875rem;
  line-height: 1.5;
  background: var(--color-background-soft);
  padding: 1rem;
  border-radius: 4px;
  overflow-x: auto;
}

.modal-footer {
  padding: 1rem;
  border-top: 1px solid var(--color-border);
  display: flex;
  justify-content: flex-end;
}
</style>
