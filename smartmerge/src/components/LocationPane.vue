<!--
  LocationPane Component - Navigation and file location overview
  
  Provides:
  - Hierarchical view of all changes in the file
  - Quick jump to specific change locations
  - File statistics and change distribution
  - Search/filter within locations
  - Visual indicators of change density
-->

<script setup lang="ts">
import { ref, computed, withDefaults, defineEmits, defineProps } from 'vue';
import type { Opcode, DiffResult } from '@/api/smartmerge';

interface LocationItem {
  id: string;
  lineNumber: number;
  type: 'insert' | 'delete' | 'replace' | 'moved';
  blockSize: number;
  context: string;
  blockIndex: number;
  percentage: number;
}

interface Props {
  diffResult?: DiffResult;
  currentLineNumber?: number;
  lines?: string[];
  side?: 'left' | 'right';
}

const props = withDefaults(defineProps<Props>(), {
  side: 'left',
  currentLineNumber: 0,
});

const emit = defineEmits<{
  'navigate': [lineNumber: number, blockIndex: number];
}>();

// State
const searchQuery = ref('');
const expandedGroups = ref<Set<string>>(new Set(['all']));

// Compute locations from diff result
const locations = computed((): LocationItem[] => {
  if (!props.diffResult) return [];

  const items: LocationItem[] = [];
  let blockIndex = 0;

  props.diffResult.opcodes.forEach((opcode, idx) => {
    if (opcode.tag === 'equal') return;

    const lineStart = props.side === 'left' ? opcode.i1 : opcode.j1;
    const lineEnd = props.side === 'left' ? opcode.i2 : opcode.j2;
    const blockSize = lineEnd - lineStart;

    if (blockSize === 0) return;

    // Get context from adjacent lines
    let contextLine = '';
    const lines = props.side === 'left' 
      ? props.diffResult.left_lines 
      : props.diffResult.right_lines;

    if (lineStart < lines.length) {
      contextLine = lines[lineStart].substring(0, 60);
    } else if (lineStart > 0 && lineStart - 1 < lines.length) {
      contextLine = lines[lineStart - 1].substring(0, 60);
    }

    const totalLines = lines.length;
    const percentage = totalLines > 0 ? Math.round((lineStart / totalLines) * 100) : 0;

    items.push({
      id: `${idx}-${lineStart}`,
      lineNumber: lineStart + 1,
      type: opcode.tag as 'insert' | 'delete' | 'replace' | 'moved',
      blockSize,
      context: contextLine || '(no context)',
      blockIndex,
      percentage,
    });

    blockIndex++;
  });

  return items;
});

// Filtered and grouped locations
const groupedLocations = computed(() => {
  const filtered = locations.value.filter(loc => {
    if (!searchQuery.value) return true;
    const query = searchQuery.value.toLowerCase();
    return (
      loc.lineNumber.toString().includes(query) ||
      loc.context.toLowerCase().includes(query) ||
      loc.type.includes(query)
    );
  });

  // Group by change type
  const groups: Record<string, LocationItem[]> = {
    replace: [],
    delete: [],
    insert: [],
    moved: [],
  };

  filtered.forEach(item => {
    groups[item.type].push(item);
  });

  return groups;
});

// Statistics
const statistics = computed(() => {
  if (!props.diffResult || !props.lines) {
    return {
      totalLines: 0,
      totalChanges: 0,
      insertChanges: 0,
      deleteChanges: 0,
      replaceChanges: 0,
      movedChanges: 0,
      changePercentage: 0,
    };
  }

  const totalLines = props.lines.length;
  const insertChanges = locations.value.filter(l => l.type === 'insert').length;
  const deleteChanges = locations.value.filter(l => l.type === 'delete').length;
  const replaceChanges = locations.value.filter(l => l.type === 'replace').length;
  const movedChanges = locations.value.filter(l => l.type === 'moved').length;
  const totalChanges = insertChanges + deleteChanges + replaceChanges + movedChanges;
  const changePercentage = totalLines > 0 ? Math.round((totalChanges / totalLines) * 100) : 0;

  return {
    totalLines,
    totalChanges,
    insertChanges,
    deleteChanges,
    replaceChanges,
    movedChanges,
    changePercentage,
  };
});

// Methods
function toggleGroup(type: string) {
  if (expandedGroups.value.has(type)) {
    expandedGroups.value.delete(type);
  } else {
    expandedGroups.value.add(type);
  }
}

function navigateToLocation(lineNumber: number, blockIndex: number) {
  emit('navigate', lineNumber, blockIndex);
}

function navigateToNextChange() {
  const currentPercent = currentLineNumber.value / (props.lines?.length || 1);
  const nextLocation = locations.value.find(l => l.percentage > Math.round(currentPercent * 100));
  if (nextLocation) {
    navigateToLocation(nextLocation.lineNumber, nextLocation.blockIndex);
  }
}

function navigateToPrevChange() {
  const currentPercent = currentLineNumber.value / (props.lines?.length || 1);
  const prevLocation = [...locations.value]
    .reverse()
    .find(l => l.percentage < Math.round(currentPercent * 100));
  if (prevLocation) {
    navigateToLocation(prevLocation.lineNumber, prevLocation.blockIndex);
  }
}

const currentLineNumber = computed(() => props.currentLineNumber || 0);

function getTypeIcon(type: string): string {
  switch (type) {
    case 'insert':
      return '➕';
    case 'delete':
      return '➖';
    case 'replace':
      return '🔄';
    case 'moved':
      return '↔️';
    default:
      return '•';
  }
}

function getTypeColor(type: string): string {
  switch (type) {
    case 'insert':
      return 'var(--color-success, #16a34a)';
    case 'delete':
      return 'var(--color-danger, #dc2626)';
    case 'replace':
      return 'var(--color-warning, #f59e0b)';
    case 'moved':
      return 'var(--color-info, #0ea5e9)';
    default:
      return 'var(--color-text-muted)';
  }
}
</script>

<template>
  <div class="location-pane">
    <!-- Header -->
    <div class="pane-header">
      <div class="header-title">📍 Locations</div>
      <div class="header-controls">
        <button
          class="nav-btn"
          @click="navigateToPrevChange"
          title="Previous change"
          :disabled="locations.length === 0"
        >
          ▲
        </button>
        <button
          class="nav-btn"
          @click="navigateToNextChange"
          title="Next change"
          :disabled="locations.length === 0"
        >
          ▼
        </button>
      </div>
    </div>

    <!-- Statistics -->
    <div class="statistics">
      <div class="stat-item">
        <span class="stat-label">Total Lines:</span>
        <span class="stat-value">{{ statistics.totalLines }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Changes:</span>
        <span class="stat-value">{{ statistics.totalChanges }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Coverage:</span>
        <span class="stat-value">{{ statistics.changePercentage }}%</span>
      </div>
    </div>

    <!-- Search -->
    <div class="search-box">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search locations..."
        class="search-input"
      />
      <span class="search-count" v-if="searchQuery">
        {{ Object.values(groupedLocations).flat().length }} / {{ locations.length }}
      </span>
    </div>

    <!-- Location List -->
    <div class="location-list">
      <!-- Replace Changes -->
      <div class="location-group" v-if="groupedLocations.replace.length > 0">
        <div class="group-header" @click="toggleGroup('replace')">
          <span class="toggle-icon" :class="{ expanded: expandedGroups.has('replace') }">
            ▶
          </span>
          <span class="group-title">
            <span class="group-icon">🔄</span>
            Changes ({{ groupedLocations.replace.length }})
          </span>
        </div>
        <transition name="collapse">
          <div v-if="expandedGroups.has('replace')" class="group-items">
            <div
              v-for="item in groupedLocations.replace"
              :key="item.id"
              class="location-item change"
              @click="navigateToLocation(item.lineNumber, item.blockIndex)"
            >
              <span class="item-icon">{{ getTypeIcon(item.type) }}</span>
              <div class="item-content">
                <div class="item-line">📍 Line {{ item.lineNumber }}</div>
                <div class="item-context">{{ item.context }}</div>
                <div class="item-meta">{{ item.blockSize }} lines • {{ item.percentage }}%</div>
              </div>
              <span class="item-indicator">→</span>
            </div>
          </div>
        </transition>
      </div>

      <!-- Delete Changes -->
      <div class="location-group" v-if="groupedLocations.delete.length > 0">
        <div class="group-header" @click="toggleGroup('delete')">
          <span class="toggle-icon" :class="{ expanded: expandedGroups.has('delete') }">
            ▶
          </span>
          <span class="group-title">
            <span class="group-icon">➖</span>
            Deleted ({{ groupedLocations.delete.length }})
          </span>
        </div>
        <transition name="collapse">
          <div v-if="expandedGroups.has('delete')" class="group-items">
            <div
              v-for="item in groupedLocations.delete"
              :key="item.id"
              class="location-item delete"
              @click="navigateToLocation(item.lineNumber, item.blockIndex)"
            >
              <span class="item-icon">{{ getTypeIcon(item.type) }}</span>
              <div class="item-content">
                <div class="item-line">📍 Line {{ item.lineNumber }}</div>
                <div class="item-context">{{ item.context }}</div>
                <div class="item-meta">{{ item.blockSize }} lines • {{ item.percentage }}%</div>
              </div>
              <span class="item-indicator">→</span>
            </div>
          </div>
        </transition>
      </div>

      <!-- Insert Changes -->
      <div class="location-group" v-if="groupedLocations.insert.length > 0">
        <div class="group-header" @click="toggleGroup('insert')">
          <span class="toggle-icon" :class="{ expanded: expandedGroups.has('insert') }">
            ▶
          </span>
          <span class="group-title">
            <span class="group-icon">➕</span>
            Added ({{ groupedLocations.insert.length }})
          </span>
        </div>
        <transition name="collapse">
          <div v-if="expandedGroups.has('insert')" class="group-items">
            <div
              v-for="item in groupedLocations.insert"
              :key="item.id"
              class="location-item insert"
              @click="navigateToLocation(item.lineNumber, item.blockIndex)"
            >
              <span class="item-icon">{{ getTypeIcon(item.type) }}</span>
              <div class="item-content">
                <div class="item-line">📍 Line {{ item.lineNumber }}</div>
                <div class="item-context">{{ item.context }}</div>
                <div class="item-meta">{{ item.blockSize }} lines • {{ item.percentage }}%</div>
              </div>
              <span class="item-indicator">→</span>
            </div>
          </div>
        </transition>
      </div>

      <!-- Moved Changes -->
      <div class="location-group" v-if="groupedLocations.moved.length > 0">
        <div class="group-header" @click="toggleGroup('moved')">
          <span class="toggle-icon" :class="{ expanded: expandedGroups.has('moved') }">
            ▶
          </span>
          <span class="group-title">
            <span class="group-icon">↔️</span>
            Moved ({{ groupedLocations.moved.length }})
          </span>
        </div>
        <transition name="collapse">
          <div v-if="expandedGroups.has('moved')" class="group-items">
            <div
              v-for="item in groupedLocations.moved"
              :key="item.id"
              class="location-item moved"
              @click="navigateToLocation(item.lineNumber, item.blockIndex)"
            >
              <span class="item-icon">{{ getTypeIcon(item.type) }}</span>
              <div class="item-content">
                <div class="item-line">📍 Line {{ item.lineNumber }}</div>
                <div class="item-context">{{ item.context }}</div>
                <div class="item-meta">{{ item.blockSize }} lines • {{ item.percentage }}%</div>
              </div>
              <span class="item-indicator">→</span>
            </div>
          </div>
        </transition>
      </div>

      <!-- Empty state -->
      <div v-if="locations.length === 0" class="empty-state">
        <span class="empty-icon">📭</span>
        <span class="empty-text">No changes found</span>
      </div>

      <!-- No search results -->
      <div
        v-if="searchQuery && Object.values(groupedLocations).flat().length === 0"
        class="empty-state"
      >
        <span class="empty-icon">🔍</span>
        <span class="empty-text">No matches found</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.location-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

/* Header */
.pane-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  background: var(--color-background-soft);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.header-title {
  font-weight: 600;
  color: var(--color-text);
  font-size: 0.95rem;
}

.header-controls {
  display: flex;
  gap: 0.25rem;
}

.nav-btn {
  width: 28px;
  height: 28px;
  border: 1px solid var(--color-border);
  background: var(--color-background);
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.75rem;
  transition: all 0.2s;
}

.nav-btn:hover:not(:disabled) {
  background: var(--color-primary, #3b82f6);
  color: white;
  border-color: var(--color-primary, #3b82f6);
}

.nav-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Statistics */
.statistics {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1px;
  padding: 0.75rem;
  background: var(--color-border);
  border-bottom: 1px solid var(--color-border);
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  background: var(--color-background);
  padding: 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
}

.stat-label {
  color: var(--color-text-muted);
  font-weight: 500;
}

.stat-value {
  color: var(--color-primary, #3b82f6);
  font-weight: 700;
  font-size: 0.95rem;
  font-variant-numeric: tabular-nums;
}

/* Search Box */
.search-box {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  padding: 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-background);
  font-size: 0.75rem;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary, #3b82f6);
}

.search-count {
  font-size: 0.7rem;
  color: var(--color-text-muted);
  padding: 0 0.5rem;
}

/* Location List */
.location-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

/* Location Group */
.location-group {
  border: 1px solid var(--color-border);
  border-radius: 4px;
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem;
  background: var(--color-background-soft);
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
  user-select: none;
}

.group-header:hover {
  background: var(--color-background-mute);
}

.toggle-icon {
  display: inline-block;
  font-size: 0.6rem;
  transition: transform 0.2s;
  width: 14px;
  text-align: center;
}

.toggle-icon.expanded {
  transform: rotate(90deg);
}

.group-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 600;
  font-size: 0.8rem;
  color: var(--color-text);
  flex: 1;
}

.group-icon {
  font-size: 0.85rem;
}

/* Group Items */
.group-items {
  display: flex;
  flex-direction: column;
  gap: 0;
  background: var(--color-background);
}

/* Location Item */
.location-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
  border-left: 3px solid transparent;
  font-size: 0.75rem;
}

.location-item:hover {
  background: var(--color-background-mute);
  padding-left: calc(0.5rem + 2px);
  border-left-width: 1px;
}

.location-item.insert {
  border-left-color: var(--color-success, #16a34a);
}

.location-item.delete {
  border-left-color: var(--color-danger, #dc2626);
}

.location-item.replace {
  border-left-color: var(--color-warning, #f59e0b);
}

.location-item.moved {
  border-left-color: var(--color-info, #0ea5e9);
}

.item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  flex-shrink: 0;
}

.item-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
}

.item-line {
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
}

.item-context {
  color: var(--color-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 0.7rem;
}

.item-meta {
  color: var(--color-text-muted);
  font-size: 0.65rem;
  font-variant-numeric: tabular-nums;
}

.item-indicator {
  color: var(--color-text-muted);
  font-size: 0.7rem;
  flex-shrink: 0;
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem 1rem;
  color: var(--color-text-muted);
  font-size: 0.8rem;
  margin: auto;
}

.empty-icon {
  font-size: 2rem;
  opacity: 0.5;
}

.empty-text {
  text-align: center;
}

/* Animations */
.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.2s ease;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
}

.collapse-enter-to,
.collapse-leave-from {
  opacity: 1;
  max-height: 1000px;
}

/* Scrollbar styling */
.location-list::-webkit-scrollbar {
  width: 6px;
}

.location-list::-webkit-scrollbar-track {
  background: var(--color-background);
}

.location-list::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 3px;
}

.location-list::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-muted);
}
</style>
