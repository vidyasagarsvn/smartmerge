<!--
  Minimap Component - Visual code map showing all changes
  
  Provides:
  - Miniature visual representation of entire file
  - Color-coded change indicators
  - Current viewport indicator
  - Quick navigation via clicking
  - Change density heat map
  - Zoom controls
-->

<script setup lang="ts">
import { ref, computed, withDefaults, defineEmits, defineProps } from 'vue';
import type { DiffResult } from '@/api/smartmerge';

interface Props {
  diffResult?: DiffResult;
  currentViewportStart?: number;
  currentViewportEnd?: number;
  lines?: string[];
}

const props = withDefaults(defineProps<Props>(), {
  currentViewportStart: 0,
  currentViewportEnd: 0,
});

const emit = defineEmits<{
  'navigate': [lineNumber: number];
}>();

// State
const zoomLevel = ref(1);
const showLegend = ref(true);
const showDensity = ref(true);

// Constants
const PIXEL_HEIGHT_PER_LINE = 1; // Height of each line in the minimap
const MINIMAP_WIDTH = 40;
const MINIMAP_MAX_HEIGHT = 400;

// Compute minimap height based on file size
const minimapHeight = computed(() => {
  if (!props.lines) return 100;
  const height = Math.min((props.lines.length * PIXEL_HEIGHT_PER_LINE) / zoomLevel.value, MINIMAP_MAX_HEIGHT);
  return Math.max(height, 100);
});

// Compute minimap blocks from diff result
const minimapBlocks = computed(() => {
  if (!props.diffResult || !props.lines) return [];

  const totalLines = props.lines.length;
  const blocks: any[] = [];
  let position = 0;

  props.diffResult.opcodes.forEach((opcode) => {
    const lineStart = opcode.i1;
    const lineEnd = opcode.i2;
    const blockSize = lineEnd - lineStart;

    if (blockSize === 0) return;

    blocks.push({
      type: opcode.tag,
      start: lineStart,
      end: lineEnd,
      size: blockSize,
      percentage: (lineStart / totalLines) * 100,
      height: Math.max((blockSize / totalLines) * minimapHeight.value, 1),
      isTrivial: opcode.is_trivial || false,
    });
  });

  return blocks;
});

// Compute viewport indicator position and size
const viewportIndicator = computed(() => {
  if (!props.lines) return { top: '0%', height: '0%' };

  const totalLines = props.lines.length;
  const viewportStart = props.currentViewportStart || 0;
  const viewportEnd = Math.min(props.currentViewportEnd || 0, totalLines);
  const viewportSize = viewportEnd - viewportStart;

  const top = (viewportStart / totalLines) * 100;
  const height = (viewportSize / totalLines) * 100;

  return { top: `${top}%`, height: `${Math.max(height, 2)}%` };
});

// Compute change density heat map
const densityMap = computed(() => {
  if (!props.diffResult || !props.lines) return [];

  const totalLines = props.lines.length;
  const bucketSize = Math.ceil(totalLines / 20); // 20 buckets
  const density: number[] = new Array(20).fill(0);

  props.diffResult.opcodes.forEach((opcode) => {
    if (opcode.tag === 'equal') return;

    const lineStart = opcode.i1;
    const lineEnd = opcode.i2;

    for (let i = lineStart; i < lineEnd; i++) {
      const bucketIndex = Math.min(Math.floor(i / bucketSize), 19);
      density[bucketIndex]++;
    }
  });

  // Normalize to 0-100
  const maxDensity = Math.max(...density);
  return density.map(d => (maxDensity > 0 ? (d / maxDensity) * 100 : 0));
});

// Methods
function onMinimapClick(event: MouseEvent) {
  const rect = (event.target as HTMLElement).getBoundingClientRect();
  const clickY = event.clientY - rect.top;
  const percentage = (clickY / rect.height) * 100;
  
  if (props.lines) {
    const lineNumber = Math.floor((percentage / 100) * props.lines.length);
    emit('navigate', lineNumber);
  }
}

function zoomIn() {
  zoomLevel.value = Math.min(zoomLevel.value + 0.5, 3);
}

function zoomOut() {
  zoomLevel.value = Math.max(zoomLevel.value - 0.5, 0.5);
}

function resetZoom() {
  zoomLevel.value = 1;
}

function getBlockColor(type: string, isTrivial: boolean): string {
  if (isTrivial) {
    return '#d1d5db'; // Gray for trivial
  }

  switch (type) {
    case 'insert':
      return '#16a34a'; // Green
    case 'delete':
      return '#dc2626'; // Red
    case 'replace':
      return '#f59e0b'; // Orange/Yellow
    case 'moved':
      return '#0ea5e9'; // Blue
    default:
      return '#6b7280'; // Gray
  }
}

function getDensityColor(percentage: number): string {
  if (percentage === 0) return 'transparent';
  if (percentage < 20) return 'rgba(100, 200, 100, 0.1)';
  if (percentage < 40) return 'rgba(100, 200, 100, 0.2)';
  if (percentage < 60) return 'rgba(255, 193, 7, 0.3)';
  if (percentage < 80) return 'rgba(255, 152, 0, 0.4)';
  return 'rgba(244, 67, 54, 0.5)';
}

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
</script>

<template>
  <div class="minimap-container">
    <!-- Header -->
    <div class="minimap-header">
      <span class="title">🗺️ Minimap</span>
      <div class="controls">
        <button
          class="control-btn"
          @click="zoomOut"
          title="Zoom out"
          :disabled="zoomLevel <= 0.5"
        >
          −
        </button>
        <span class="zoom-level">{{ zoomLevel.toFixed(1) }}x</span>
        <button
          class="control-btn"
          @click="zoomIn"
          title="Zoom in"
          :disabled="zoomLevel >= 3"
        >
          +
        </button>
        <button
          class="control-btn reset"
          @click="resetZoom"
          title="Reset zoom"
        >
          ⟲
        </button>
      </div>
    </div>

    <!-- Legend and Stats -->
    <div class="legend-section" v-if="showLegend">
      <div class="legend-item">
        <span class="legend-color" style="background: #16a34a"></span>
        <span class="legend-text">Added</span>
      </div>
      <div class="legend-item">
        <span class="legend-color" style="background: #dc2626"></span>
        <span class="legend-text">Deleted</span>
      </div>
      <div class="legend-item">
        <span class="legend-color" style="background: #f59e0b"></span>
        <span class="legend-text">Changed</span>
      </div>
      <div class="legend-item">
        <span class="legend-color" style="background: #0ea5e9"></span>
        <span class="legend-text">Moved</span>
      </div>
      <div class="legend-item">
        <span class="legend-color" style="background: #d1d5db"></span>
        <span class="legend-text">Trivial</span>
      </div>
    </div>

    <!-- Density Heat Map -->
    <div v-if="showDensity" class="density-map">
      <div
        v-for="(density, idx) in densityMap"
        :key="idx"
        class="density-bar"
        :style="{ backgroundColor: getDensityColor(density), height: '100%' }"
        :title="`Density: ${density.toFixed(0)}%`"
      ></div>
    </div>

    <!-- Minimap -->
    <div class="minimap-wrapper">
      <div
        class="minimap"
        :style="{ height: minimapHeight + 'px' }"
        @click="onMinimapClick"
      >
        <!-- Blocks -->
        <div
          v-for="(block, idx) in minimapBlocks"
          :key="idx"
          class="minimap-block"
          :style="{
            backgroundColor: getBlockColor(block.type, block.isTrivial),
            height: block.height + 'px',
            cursor: 'pointer',
            opacity: block.isTrivial ? 0.5 : 1,
          }"
          :title="`${block.type}: ${block.size} lines`"
        ></div>
      </div>

      <!-- Viewport Indicator -->
      <div
        class="viewport-indicator"
        :style="{
          top: viewportIndicator.top,
          height: viewportIndicator.height,
        }"
      ></div>
    </div>

    <!-- Footer Stats -->
    <div class="minimap-footer" v-if="lines">
      <div class="stat">
        <span class="stat-label">Total:</span>
        <span class="stat-value">{{ lines.length }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Changes:</span>
        <span class="stat-value">{{ minimapBlocks.length }}</span>
      </div>
    </div>

    <!-- Toggle Legend/Density -->
    <div class="toggle-section">
      <label class="toggle-item">
        <input type="checkbox" v-model="showLegend" />
        <span>Legend</span>
      </label>
      <label class="toggle-item">
        <input type="checkbox" v-model="showDensity" />
        <span>Density</span>
      </label>
    </div>
  </div>
</template>

<style scoped>
.minimap-container {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 0.75rem;
}

/* Header */
.minimap-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
}

.title {
  font-weight: 600;
  color: var(--color-text);
}

.controls {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.control-btn {
  width: 24px;
  height: 24px;
  border: 1px solid var(--color-border);
  background: var(--color-background-soft);
  border-radius: 3px;
  cursor: pointer;
  font-size: 0.75rem;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  color: var(--color-text);
}

.control-btn:hover:not(:disabled) {
  background: var(--color-primary, #3b82f6);
  border-color: var(--color-primary, #3b82f6);
  color: white;
}

.control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.control-btn.reset {
  background: var(--color-background);
}

.zoom-level {
  display: inline-block;
  min-width: 32px;
  text-align: center;
  color: var(--color-text-muted);
  font-size: 0.65rem;
  font-weight: 500;
}

/* Legend */
.legend-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.25rem;
  padding: 0.5rem;
  background: var(--color-background-soft);
  border-radius: 4px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 2px;
  flex-shrink: 0;
}

.legend-text {
  font-size: 0.65rem;
  color: var(--color-text-muted);
}

/* Density Map */
.density-map {
  display: flex;
  gap: 1px;
  height: 20px;
  background: var(--color-background-soft);
  border-radius: 4px;
  padding: 2px;
}

.density-bar {
  flex: 1;
  border-radius: 2px;
  transition: background-color 0.2s;
}

.density-bar:hover {
  outline: 1px solid var(--color-primary, #3b82f6);
}

/* Minimap */
.minimap-wrapper {
  position: relative;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  overflow: hidden;
  background: var(--color-background-mute);
}

.minimap {
  width: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-background);
  min-height: 100px;
}

.minimap-block {
  width: 100%;
  transition: opacity 0.2s;
  border: none;
}

.minimap-block:hover {
  opacity: 1 !important;
  outline: 1px solid rgba(0, 0, 0, 0.2);
}

/* Viewport Indicator */
.viewport-indicator {
  position: absolute;
  left: 0;
  right: 0;
  background: rgba(59, 130, 246, 0.2);
  border-top: 2px solid var(--color-primary, #3b82f6);
  border-bottom: 2px solid var(--color-primary, #3b82f6);
  pointer-events: none;
  z-index: 10;
  transition: all 0.2s;
}

/* Footer Stats */
.minimap-footer {
  display: flex;
  justify-content: space-around;
  padding: 0.5rem;
  background: var(--color-background-soft);
  border-radius: 4px;
  border: 1px solid var(--color-border);
}

.stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.125rem;
}

.stat-label {
  color: var(--color-text-muted);
  font-size: 0.6rem;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.stat-value {
  color: var(--color-primary, #3b82f6);
  font-weight: 700;
  font-size: 0.8rem;
  font-variant-numeric: tabular-nums;
}

/* Toggle Section */
.toggle-section {
  display: flex;
  gap: 0.5rem;
  padding: 0.5rem;
  background: var(--color-background-soft);
  border-radius: 4px;
}

.toggle-item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  cursor: pointer;
  user-select: none;
  font-size: 0.7rem;
  color: var(--color-text-muted);
}

.toggle-item input[type="checkbox"] {
  width: 14px;
  height: 14px;
  cursor: pointer;
  accent-color: var(--color-primary, #3b82f6);
}

.toggle-item:hover {
  color: var(--color-text);
}

/* Responsive adjustments */
@media (max-width: 600px) {
  .minimap-container {
    padding: 0.5rem;
    gap: 0.25rem;
  }

  .legend-section {
    grid-template-columns: 1fr;
    padding: 0.375rem;
  }

  .minimap-footer {
    padding: 0.375rem;
  }
}
</style>
