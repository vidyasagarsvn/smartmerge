<!--
  TrivialDiffFilter Component - Trivial change filtering control panel
  
  Provides:
  - Statistics display (total, trivial, non-trivial changes)
  - Toggle controls for each trivial change category
  - Filter presets for quick access
  - Real-time filtering and display updates
-->

<script setup lang="ts">
import { ref, computed, withDefaults, defineEmits, defineProps } from 'vue';
import type { TrivialChangeStats } from '@/api/smartmerge';

interface FilterState {
  whitespaceOnly: boolean;
  caseOnly: boolean;
  lineEndings: boolean;
  tabSpace: boolean;
  otherTrivial: boolean;
}

interface Props {
  stats?: TrivialChangeStats;
  loading?: boolean;
  disabled?: boolean;
}

withDefaults(defineProps<Props>(), {
  loading: false,
  disabled: false,
});

const emit = defineEmits<{
  'filter-changed': [state: FilterState];
  'preset-selected': [preset: 'code-only' | 'all-changes' | 'whitespace-only'];
}>();

// Filter state
const filters = ref<FilterState>({
  whitespaceOnly: true,
  caseOnly: true,
  lineEndings: true,
  tabSpace: true,
  otherTrivial: true,
});

const isExpanded = ref(true);

// Computed properties
const allFiltersActive = computed(() => {
  return Object.values(filters.value).every(v => v === true);
});

const anyFilterActive = computed(() => {
  return Object.values(filters.value).some(v => v === true);
});

const trivialPercentage = computed(() => {
  if (!props.stats || props.stats.total_changes === 0) return 0;
  return Math.round(props.stats.trivial_percentage);
});

// Methods
function toggleFilter(key: keyof FilterState) {
  filters.value[key] = !filters.value[key];
  emitFilterChange();
}

function toggleAll() {
  const newState = !allFiltersActive.value;
  Object.keys(filters.value).forEach(key => {
    filters.value[key as keyof FilterState] = newState;
  });
  emitFilterChange();
}

function applyPreset(preset: 'code-only' | 'all-changes' | 'whitespace-only') {
  switch (preset) {
    case 'code-only':
      // Hide all trivial changes
      Object.keys(filters.value).forEach(key => {
        filters.value[key as keyof FilterState] = false;
      });
      break;
    case 'all-changes':
      // Show all trivial changes
      Object.keys(filters.value).forEach(key => {
        filters.value[key as keyof FilterState] = true;
      });
      break;
    case 'whitespace-only':
      // Show only whitespace-related changes
      filters.value.whitespaceOnly = true;
      filters.value.caseOnly = false;
      filters.value.lineEndings = true;
      filters.value.tabSpace = true;
      filters.value.otherTrivial = false;
      break;
  }
  emitFilterChange();
  emit('preset-selected', preset);
}

function emitFilterChange() {
  emit('filter-changed', filters.value);
}
</script>

<template>
  <div class="trivial-filter">
    <!-- Header -->
    <div class="filter-header" @click="isExpanded = !isExpanded">
      <div class="header-left">
        <button class="expand-btn" :class="{ expanded: isExpanded }">
          ▶
        </button>
        <span class="title">Trivial Changes Filter</span>
      </div>
      <div class="header-right" v-if="stats">
        <span class="stat-badge" :class="{ highlight: stats.trivial_count > 0 }">
          {{ stats.trivial_count }} trivial ({{ trivialPercentage }}%)
        </span>
      </div>
    </div>

    <!-- Content (collapsible) -->
    <div v-if="isExpanded" class="filter-content">
      <!-- Statistics -->
      <div v-if="stats" class="statistics">
        <div class="stat-row">
          <span class="stat-label">Total Changes:</span>
          <span class="stat-value">{{ stats.total_changes }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">Non-trivial:</span>
          <span class="stat-value stat-important">{{ stats.non_trivial_count }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">Trivial:</span>
          <span class="stat-value">{{ stats.trivial_count }}</span>
        </div>

        <!-- Progress bar -->
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: trivialPercentage + '%' }"></div>
        </div>
      </div>

      <!-- Presets -->
      <div class="presets">
        <button
          @click="applyPreset('code-only')"
          class="preset-btn"
          :disabled="disabled || loading"
          title="Hide all trivial changes"
        >
          💻 Code Only
        </button>
        <button
          @click="applyPreset('all-changes')"
          class="preset-btn"
          :disabled="disabled || loading"
          title="Show all trivial changes"
        >
          👁️ All Changes
        </button>
        <button
          @click="applyPreset('whitespace-only')"
          class="preset-btn"
          :disabled="disabled || loading"
          title="Show only whitespace-related changes"
        >
          ⎵ Whitespace
        </button>
      </div>

      <!-- Divider -->
      <div class="divider"></div>

      <!-- Filter Categories -->
      <div class="filter-categories">
        <!-- Whitespace Only -->
        <label class="filter-checkbox">
          <input
            v-model="filters.whitespaceOnly"
            type="checkbox"
            :disabled="disabled || loading"
            @change="emitFilterChange"
          />
          <span class="checkbox-label">
            <span class="icon">⎵</span>
            <span class="label-text">
              Whitespace Changes
              <span v-if="stats" class="count">({{ stats.breakdown.whitespace_only }})</span>
            </span>
          </span>
        </label>

        <!-- Case Only -->
        <label class="filter-checkbox">
          <input
            v-model="filters.caseOnly"
            type="checkbox"
            :disabled="disabled || loading"
            @change="emitFilterChange"
          />
          <span class="checkbox-label">
            <span class="icon">Aa</span>
            <span class="label-text">
              Case Changes
              <span v-if="stats" class="count">({{ stats.breakdown.case_only }})</span>
            </span>
          </span>
        </label>

        <!-- Line Endings -->
        <label class="filter-checkbox">
          <input
            v-model="filters.lineEndings"
            type="checkbox"
            :disabled="disabled || loading"
            @change="emitFilterChange"
          />
          <span class="checkbox-label">
            <span class="icon">⏎</span>
            <span class="label-text">
              Line Ending Changes
              <span v-if="stats" class="count">({{ stats.breakdown.line_endings }})</span>
            </span>
          </span>
        </label>

        <!-- Tab vs Space -->
        <label class="filter-checkbox">
          <input
            v-model="filters.tabSpace"
            type="checkbox"
            :disabled="disabled || loading"
            @change="emitFilterChange"
          />
          <span class="checkbox-label">
            <span class="icon">→</span>
            <span class="label-text">
              Tab/Space Changes
              <span v-if="stats" class="count">({{ stats.breakdown.tab_space }})</span>
            </span>
          </span>
        </label>

        <!-- Other Trivial -->
        <label class="filter-checkbox">
          <input
            v-model="filters.otherTrivial"
            type="checkbox"
            :disabled="disabled || loading"
            @change="emitFilterChange"
          />
          <span class="checkbox-label">
            <span class="icon">⚙️</span>
            <span class="label-text">
              Other Trivial
              <span v-if="stats" class="count">({{ stats.breakdown.other_trivial }})</span>
            </span>
          </span>
        </label>
      </div>

      <!-- Toggle All -->
      <div class="toggle-all">
        <button
          @click="toggleAll"
          class="toggle-btn"
          :disabled="disabled || loading"
        >
          {{ allFiltersActive ? '✓ Hide All Trivial' : '✓ Show All Trivial' }}
        </button>
      </div>
    </div>

    <!-- Loading indicator -->
    <div v-if="loading" class="loading-indicator">
      <span class="spinner"></span>
      Analyzing changes...
    </div>
  </div>
</template>

<style scoped>
.trivial-filter {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-background-soft);
  overflow: hidden;
}

/* Header */
.filter-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  background: var(--color-background);
  cursor: pointer;
  user-select: none;
  border-bottom: 1px solid var(--color-border);
}

.filter-header:hover {
  background: var(--color-background-mute);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.expand-btn {
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  color: var(--color-text);
  cursor: pointer;
  font-size: 0.75rem;
  transition: transform 0.2s;
  padding: 0;
}

.expand-btn.expanded {
  transform: rotate(90deg);
}

.title {
  font-weight: 600;
  color: var(--color-text);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.stat-badge {
  font-size: 0.75rem;
  padding: 0.25rem 0.5rem;
  background: var(--color-background-mute);
  border-radius: 4px;
  color: var(--color-text-muted);
}

.stat-badge.highlight {
  background: var(--color-warning-bg, #fef3c7);
  color: var(--color-warning, #d97706);
  font-weight: 500;
}

/* Content */
.filter-content {
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

/* Statistics */
.statistics {
  background: var(--color-background);
  border-radius: 4px;
  padding: 0.75rem;
  font-size: 0.875rem;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.stat-label {
  color: var(--color-text-muted);
  font-weight: 500;
}

.stat-value {
  color: var(--color-text);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.stat-value.stat-important {
  color: var(--color-primary, #3b82f6);
}

.progress-bar {
  width: 100%;
  height: 6px;
  background: var(--color-background-mute);
  border-radius: 3px;
  overflow: hidden;
  margin-top: 0.5rem;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--color-warning, #f59e0b), var(--color-danger, #dc2626));
  transition: width 0.3s ease;
}

/* Presets */
.presets {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.preset-btn {
  flex: 1;
  min-width: 100px;
  padding: 0.5rem 0.75rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text);
  transition: all 0.2s;
}

.preset-btn:hover:not(:disabled) {
  background: var(--color-primary, #3b82f6);
  color: white;
  border-color: var(--color-primary, #3b82f6);
}

.preset-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Divider */
.divider {
  height: 1px;
  background: var(--color-border);
}

/* Filter Categories */
.filter-categories {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.filter-checkbox {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  user-select: none;
  padding: 0.5rem;
  border-radius: 4px;
  transition: background 0.2s;
}

.filter-checkbox:hover:not(:has(input:disabled)) {
  background: var(--color-background-mute);
}

.filter-checkbox input[type='checkbox'] {
  cursor: pointer;
  accent-color: var(--color-primary, #3b82f6);
}

.filter-checkbox input[type='checkbox']:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  flex: 1;
}

.icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: var(--color-background);
  border-radius: 3px;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-muted);
}

.label-text {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.count {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  background: var(--color-background-mute);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  font-variant-numeric: tabular-nums;
}

/* Toggle All */
.toggle-all {
  display: flex;
  gap: 0.5rem;
}

.toggle-btn {
  flex: 1;
  padding: 0.5rem 0.75rem;
  background: var(--color-primary, #3b82f6);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
  font-size: 0.75rem;
  transition: opacity 0.2s;
}

.toggle-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.toggle-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Loading Indicator */
.loading-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-border);
  border-radius: 50%;
  border-top-color: var(--color-primary, #3b82f6);
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
