<!-- 
  Updated FileCompareView using SmartMerge Backend API
  
  This is a reference implementation showing how to integrate the backend API.
  It replaces the client-side diff logic with backend-powered diff and highlighting.
-->

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { useDiff, useHighlighting } from '@/composables/useSmartMerge';
import type { DiffAlgorithm, Theme, TrivialChangeStats, GhostLineLayout } from '@/api/smartmerge';
import { TrivialDiffAPI, GhostLineAPI } from '@/api/smartmerge';
import TrivialDiffFilter from './TrivialDiffFilter.vue';
import LocationPane from './LocationPane.vue';
import Minimap from './Minimap.vue';

// Props
const props = defineProps<{
  leftLabel: string;
  rightLabel: string;
  leftPath?: string;
  rightPath?: string;
  leftLines?: string[];
  rightLines?: string[];
  algorithm?: DiffAlgorithm;
  theme?: Theme;
}>();

// Emits
const emit = defineEmits<{
  'region-change': [{ hasSelection: boolean; regionCount: number; currentIndex: number }];
}>();

// Composables
const { diffResult, loading: diffLoading, compareFiles, compareLines } = useDiff();
const { highlightResult, loading: highlightLoading, highlightDiff } = useHighlighting();

// Algorithm selection
const selectedAlgorithm = ref<DiffAlgorithm>(props.algorithm || 'myers');
const selectedTheme = ref<Theme>(props.theme || 'auto');

// Watch for algorithm changes from parent
watch(() => props.algorithm, (newAlgorithm) => {
  if (newAlgorithm) {
    selectedAlgorithm.value = newAlgorithm;
  }
});

// Trivial diff filtering
const trivialStats = ref<TrivialChangeStats | null>(null);
const analyzingTrivial = ref(false);
const trivialFilterState = ref({
  whitespaceOnly: true,
  caseOnly: true,
  lineEndings: true,
  tabSpace: true,
  otherTrivial: true,
});

// Ghost line mappings (WinMerge-style)
const ghostLineMappings = ref<GhostLineLayout[]>([]);

// Location pane and minimap tracking
const currentViewportLine = ref(0);
const showLocationPane = ref(true);
const showMinimap = ref(true);

// Refs for pane scrolling
const leftPaneRef = ref<HTMLElement | null>(null);
const rightPaneRef = ref<HTMLElement | null>(null);
const isSyncingScroll = ref(false);

// Loading state
const loading = computed(() => diffLoading.value || highlightLoading.value);

// Perform comparison
async function performComparison() {
  // If paths are provided, compare files
  if (props.leftPath && props.rightPath) {
    console.log('FileCompareView: comparing files', props.leftPath, props.rightPath, selectedAlgorithm.value);
    await compareFiles(props.leftPath, props.rightPath, selectedAlgorithm.value);
    console.log('FileCompareView: diffResult after compareFiles', diffResult.value);
  }
  // If lines are provided, compare lines
  else if (props.leftLines && props.rightLines) {
    console.log('FileCompareView: comparing lines');
    await compareLines(props.leftLines, props.rightLines, selectedAlgorithm.value);
  }

  // After diff is complete, generate highlights
  if (diffResult.value) {
    console.log('FileCompareView: generating highlights');
    await highlightDiff(
      diffResult.value.left_lines,
      diffResult.value.right_lines,
      selectedAlgorithm.value,
      selectedTheme.value
    );

    // Compute ghost line mappings using WinMerge-style algorithm
    try {
      const leftLines = diffResult.value.left_lines;
      const rightLines = diffResult.value.right_lines;
      console.log('FileCompareView: calling GhostLineAPI with', leftLines.length, 'left lines,', rightLines.length, 'right lines,', diffResult.value.opcodes.length, 'opcodes');
      const layouts = await GhostLineAPI.computeGhostLineMappings(leftLines, rightLines, diffResult.value.opcodes);
      ghostLineMappings.value = layouts;
      console.log('FileCompareView: computed ghost line mappings - got', layouts.length, 'blocks');
      if (layouts.length > 0) {
        console.log('FileCompareView: first layout:', layouts[0]);
        console.log('FileCompareView: left_mappings count:', layouts[0].left_mappings.length, 'right_mappings count:', layouts[0].right_mappings.length);
      }
    } catch (error) {
      console.error('Failed to compute ghost line mappings:', error);
      ghostLineMappings.value = [];
    }

    // Analyze for trivial changes
    await analyzeTrivialChanges();
  } else {
    console.error('FileCompareView: diffResult is null after comparison');
  }
}

// Analyze trivial changes
async function analyzeTrivialChanges() {
  if (!diffResult.value) return;

  try {
    analyzingTrivial.value = true;
    const stats = await TrivialDiffAPI.getTrivialStats(diffResult.value, false);
    trivialStats.value = stats;
  } catch (error) {
    console.error('Failed to analyze trivial changes:', error);
  } finally {
    analyzingTrivial.value = false;
  }
}

// Handle filter state change
function onFilterChange(filterState: typeof trivialFilterState.value) {
  trivialFilterState.value = filterState;
}

// Handle navigation from LocationPane
function onLocationNavigate(lineNumber: number, blockIndex: number) {
  currentViewportLine.value = lineNumber;
  currentRegionIndex.value = blockIndex;
  scrollToRegion(blockIndex);
}

// Handle navigation from Minimap
function onMinimapNavigate(lineNumber: number) {
  currentViewportLine.value = lineNumber;
  
  // Scroll to the line in both panes
  if (leftPaneRef.value && rightPaneRef.value) {
    // Refs now point directly to pane-content elements
    const leftPaneContent = leftPaneRef.value;
    const rightPaneContent = rightPaneRef.value;
    
    if (leftPaneContent && rightPaneContent) {
      // Find line divs matching the line number (line numbers are 0-based internally, 1-based in display)
      const leftLines = leftPaneContent.querySelectorAll('div[class*="line"]');
      const rightLines = rightPaneContent.querySelectorAll('div[class*="line"]');
      
      let leftTargetDiv: Element | null = null;
      let rightTargetDiv: Element | null = null;
      
      for (const div of leftLines) {
        const lineNumSpan = div.querySelector('.line-number');
        if (lineNumSpan && parseInt(lineNumSpan.textContent || '0') === lineNumber + 1) {
          leftTargetDiv = div;
          break;
        }
      }
      
      for (const div of rightLines) {
        const lineNumSpan = div.querySelector('.line-number');
        if (lineNumSpan && parseInt(lineNumSpan.textContent || '0') === lineNumber + 1) {
          rightTargetDiv = div;
          break;
        }
      }
      
      isSyncingScroll.value = true;
      
      if (leftTargetDiv) {
        leftTargetDiv.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
      
      if (rightTargetDiv) {
        rightTargetDiv.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
      
      setTimeout(() => {
        isSyncingScroll.value = false;
      }, 500);
    }
  }
  
  // Find the block index for this line
  let blockIndex = 0;
  for (let i = 0; i < diffResult.value?.opcodes.length; i++) {
    const op = diffResult.value!.opcodes[i];
    if (op.i1 <= lineNumber && lineNumber < op.i2) {
      break;
    }
    if (op.tag !== 'equal') {
      blockIndex++;
    }
  }
  currentRegionIndex.value = blockIndex;
}

// Handle scroll events to update viewport position and sync scrolling
// With ghost lines, both panes have the same number of display lines, so direct pixel sync works
function onPaneScroll(event: Event, isLeftPane: boolean) {
  if (isSyncingScroll.value) return; // Prevent infinite loop
  
  const target = event.target as HTMLElement;
  const scrollTop = target.scrollTop;
  
  // Update viewport line based on scroll position
  const lineHeight = 24; // Approximate line height
  const estimatedLine = Math.floor(scrollTop / lineHeight);
  currentViewportLine.value = estimatedLine;
  
  // Sync scroll to the other pane - with ghost lines, we can use direct pixel sync
  isSyncingScroll.value = true;
  try {
    const otherPane = isLeftPane ? rightPaneRef.value : leftPaneRef.value;
    if (otherPane) {
      // Direct pixel synchronization works because ghost lines make both panes equal height
      otherPane.scrollTop = scrollTop;
    }
  } finally {
    // Use nextTick to reset the flag
    setTimeout(() => {
      isSyncingScroll.value = false;
    }, 0);
  }
}

// Check if a line should be visible based on trivial filters
function shouldShowLine(lineIndex: number, isLeftPane: boolean): boolean {
  if (!diffResult.value) return true;

  // Find the opcode that contains this line
  const opcode = diffResult.value.opcodes.find(op => {
    if (isLeftPane) {
      return lineIndex >= op.i1 && lineIndex < op.i2;
    } else {
      return lineIndex >= op.j1 && lineIndex < op.j2;
    }
  });

  if (!opcode || opcode.tag === 'equal') return true;
  if (!opcode.is_trivial) return true; // Show non-trivial changes

  // If it's trivial, check the filter state based on type
  // For now, we'll use a simplified check - in reality, we'd need to know the trivial type
  if (!trivialFilterState.value.whitespaceOnly &&
      !trivialFilterState.value.caseOnly &&
      !trivialFilterState.value.lineEndings &&
      !trivialFilterState.value.tabSpace &&
      !trivialFilterState.value.otherTrivial) {
    return false; // Hide all trivial changes
  }

  return true; // Show trivial changes
}

// Watch for prop changes
watch(
  () => [props.leftPath, props.rightPath, props.leftLines, props.rightLines, selectedAlgorithm.value],
  () => {
    performComparison();
  },
  { immediate: true }
);

// Computed properties for rendering with ghost lines (WinMerge-style)
const leftLineData = computed(() => {
  if (!highlightResult.value || !diffResult.value) return [];
  
  const lines: any[] = [];
  let displayLineNumber = 0;
  
  // If we don't have ghost line mappings yet, fall back to simple rendering
  if (ghostLineMappings.value.length === 0) {
    console.log('leftLineData: using fallback rendering (no ghost mappings)');
    for (let i = 0; i < diffResult.value.left_lines.length; i++) {
      const text = diffResult.value.left_lines[i];
      const highlight = highlightResult.value.left_highlights.find(
        (h: any) => h.line_number === i
      );
      lines.push({
        number: i + 1,
        displayNumber: displayLineNumber++,
        text,
        className: getClassNameFromHighlight(highlight),
        style: highlight?.style,
        inlineHighlights: highlight?.inline_highlights || [],
        visible: shouldShowLine(i, true),
        isGhost: false,
      });
    }
    return lines.filter(line => line.visible);
  }
  
  // Process ghost line mappings (WinMerge-style intelligent alignment)
  for (let blockIndex = 0; blockIndex < ghostLineMappings.value.length; blockIndex++) {
    const layout = ghostLineMappings.value[blockIndex];
    
    for (const mapping of layout.left_mappings) {
      if (mapping.maps_to === null) {
        // This is a ghost line - no real content
        lines.push({
          number: null,
          displayNumber: displayLineNumber++,
          text: '',
          className: 'line line-ghost',
          style: {},
          inlineHighlights: [],
          visible: true,
          isGhost: true,
        });
      } else {
        // Real line from left file
        const realIndex = mapping.index;
        const text = diffResult.value.left_lines[realIndex];
        const highlight = highlightResult.value.left_highlights.find(
          (h: any) => h.line_number === realIndex
        );
        
        lines.push({
          number: realIndex + 1,  // 1-based for display
          displayNumber: displayLineNumber++,
          text,
          className: getClassNameFromHighlight(highlight),
          style: highlight?.style,
          inlineHighlights: highlight?.inline_highlights || [],
          visible: shouldShowLine(realIndex, true),
          isGhost: false,
        });
      }
    }
  }
  
  return lines.filter(line => line.visible);
});

const rightLineData = computed(() => {
  if (!highlightResult.value || !diffResult.value) return [];
  
  const lines: any[] = [];
  let displayLineNumber = 0;
  
  // If we don't have ghost line mappings yet, fall back to simple rendering
  if (ghostLineMappings.value.length === 0) {
    for (let j = 0; j < diffResult.value.right_lines.length; j++) {
      const text = diffResult.value.right_lines[j];
      const highlight = highlightResult.value.right_highlights.find(
        (h: any) => h.line_number === j
      );
      lines.push({
        number: j + 1,
        displayNumber: displayLineNumber++,
        text,
        className: getClassNameFromHighlight(highlight),
        style: highlight?.style,
        inlineHighlights: highlight?.inline_highlights || [],
        visible: shouldShowLine(j, false),
        isGhost: false,
      });
    }
    return lines.filter(line => line.visible);
  }
  
  // Process ghost line mappings (WinMerge-style intelligent alignment)
  for (let blockIndex = 0; blockIndex < ghostLineMappings.value.length; blockIndex++) {
    const layout = ghostLineMappings.value[blockIndex];
    
    for (const mapping of layout.right_mappings) {
      if (mapping.maps_to === null) {
        // This is a ghost line - no real content
        lines.push({
          number: null,
          displayNumber: displayLineNumber++,
          text: '',
          className: 'line line-ghost',
          style: {},
          inlineHighlights: [],
          visible: true,
          isGhost: true,
        });
      } else {
        // Real line from right file
        const realIndex = mapping.index;
        const text = diffResult.value.right_lines[realIndex];
        const highlight = highlightResult.value.right_highlights.find(
          (h: any) => h.line_number === realIndex
        );
        
        lines.push({
          number: realIndex + 1,  // 1-based for display
          displayNumber: displayLineNumber++,
          text,
          className: getClassNameFromHighlight(highlight),
          style: highlight?.style,
          inlineHighlights: highlight?.inline_highlights || [],
          visible: shouldShowLine(realIndex, false),
          isGhost: false,
        });
      }
    }
  }
  
  return lines.filter(line => line.visible);
});

// Helper to convert highlight to CSS class
function getClassNameFromHighlight(highlight: any): string {
  if (!highlight) return 'line-equal';
  
  const blockType = highlight.block_type?.toLowerCase();
  const classes = ['line'];
  
  switch (blockType) {
    case 'insert':
      classes.push('line-added');
      break;
    case 'delete':
      classes.push('line-deleted');
      break;
    case 'replace':
      classes.push('line-modified');
      break;
    case 'moved':
      classes.push('line-moved');
      break;
    default:
      classes.push('line-equal');
  }
  
  if (highlight.is_trivial) {
    classes.push('line-trivial');
  }
  
  return classes.join(' ');
}

// Diff regions for navigation with linked list (WinMerge-style)
const diffRegions = computed(() => {
  if (!diffResult.value) return [];
  
  const regions = diffResult.value.opcodes
    .map((op, absIndex) => ({
      absIndex,        // Index in full opcode list
      leftStart: op.i1,
      leftEnd: op.i2,
      rightStart: op.j1,
rightEnd: op.j2,
      type: op.tag,
      opcode: op,
      isSignificant: op.tag !== 'equal' && !op.is_trivial,
    }))
    .filter(r => r.isSignificant);
  
  // Build doubly-linked list for fast navigation
  for (let i = 0; i < regions.length; i++) {
    regions[i].prevIndex = i > 0 ? i - 1 : -1;
    regions[i].nextIndex = i < regions.length - 1 ? i + 1 : -1;
  }
  
  return regions;
});

// Current region index
const currentRegionIndex = ref(0);

// Navigation functions (WinMerge-style: check if current diff is visible first)
function nextDiff() {
  const regions = diffRegions.value;
  if (regions.length === 0) return;
  
  // If we're not at a diff or already past current, find next from cursor
  const currentLine = currentViewportLine.value;
  let targetIndex = -1;
  
  // Find first diff that starts after current viewport line
  for (let i = 0; i < regions.length; i++) {
    if (regions[i].leftStart > currentLine) {
      targetIndex = i;
      break;
    }
  }
  
  // If no diff found after current line, wrap to first
  if (targetIndex === -1 && regions.length > 0) {
    targetIndex = 0;
  }
  
  if (targetIndex !== -1) {
    currentRegionIndex.value = targetIndex;
    scrollToRegion(targetIndex);
  }
}

function prevDiff() {
  const regions = diffRegions.value;
  if (regions.length === 0) return;
  
  const currentLine = currentViewportLine.value;
  let targetIndex = -1;
  
  // Find last diff that starts before current viewport line
  for (let i = regions.length - 1; i >= 0; i--) {
    if (regions[i].leftStart < currentLine) {
      targetIndex = i;
      break;
    }
  }
  
  // If no diff found before current line, wrap to last
  if (targetIndex === -1 && regions.length > 0) {
    targetIndex = regions.length - 1;
  }
  
  if (targetIndex !== -1) {
    currentRegionIndex.value = targetIndex;
    scrollToRegion(targetIndex);
  }
}

// WinMerge-style: Show 5 lines of context above, 3 below
const CONTEXT_LINES_ABOVE = 5;
const CONTEXT_LINES_BELOW = 3;

function scrollToRegion(index: number) {
  const region = diffRegions.value[index];
  if (!region || !leftPaneRef.value || !rightPaneRef.value) return;
  
  const leftLineNumber = region.leftStart;
  const rightLineNumber = region.rightStart;
  
  // Add visual highlight to the current diff region
  currentRegionIndex.value = index;
  currentViewportLine.value = leftLineNumber;
  
  // Refs now point directly to pane-content elements
  const leftPaneContent = leftPaneRef.value;
  const rightPaneContent = rightPaneRef.value;
  
  if (leftPaneContent && rightPaneContent) {
    const leftLines = leftPaneContent.querySelectorAll('div[class*="line"]') as NodeListOf<HTMLElement>;
    const rightLines = rightPaneContent.querySelectorAll('div[class*="line"]') as NodeListOf<HTMLElement>;
    
    // Clear previous highlights
    leftLines.forEach(div => div.classList.remove('current-diff'));
    rightLines.forEach(div => div.classList.remove('current-diff'));
    
    let leftTargetDiv: HTMLElement | null = null;
    let rightTargetDiv: HTMLElement | null = null;
    
    // Find and highlight the diff region
    for (const div of leftLines) {
      const lineNumSpan = div.querySelector('.line-number');
      const lineNum = parseInt(lineNumSpan?.textContent || '0') - 1; // Convert to 0-based
      
      if (lineNum >= region.leftStart && lineNum < region.leftEnd) {
        div.classList.add('current-diff');
        if (!leftTargetDiv) leftTargetDiv = div;
      }
    }
    
    for (const div of rightLines) {
      const lineNumSpan = div.querySelector('.line-number');
      const lineNum = parseInt(lineNumSpan?.textContent || '0') - 1;
      
      if (lineNum >= region.rightStart && lineNum < region.rightEnd) {
        div.classList.add('current-diff');
        if (!rightTargetDiv) rightTargetDiv = div;
      }
    }
    
    // Scroll with context (WinMerge shows 5 lines above for context)
    isSyncingScroll.value = true;
    
    if (leftTargetDiv) {
      const targetTop = leftTargetDiv.offsetTop - (CONTEXT_LINES_ABOVE * 24);
      leftPaneRef.value.scrollTop = Math.max(0, targetTop);
    }
    
    if (rightTargetDiv) {
      const targetTop = rightTargetDiv.offsetTop - (CONTEXT_LINES_ABOVE * 24);
      rightPaneRef.value.scrollTop = Math.max(0, targetTop);
    }
    
    setTimeout(() => {
      isSyncingScroll.value = false;
    }, 100);
  }
  
  emit('region-change', {
    hasSelection: true,
    regionCount: diffRegions.value.length,
    currentIndex: index,
  });
}

// Algorithm change handler
function onAlgorithmChange(algorithm: DiffAlgorithm) {
  selectedAlgorithm.value = algorithm;
  performComparison();
}

// Theme change handler
function onThemeChange(theme: Theme) {
  selectedTheme.value = theme;
  performComparison();
}

// Expose methods and data for parent
defineExpose({
  nextDiff,
  prevDiff,
  diffRegions,
  currentRegionIndex,
  onAlgorithmChange,
  onThemeChange,
  refresh: performComparison,
  trivialStats,
  trivialFilterState,
  onFilterChange,
  currentViewportLine,
  showLocationPane,
  onLocationNavigate,
  onPaneScroll,
  showMinimap,
  onMinimapNavigate,
});
</script>

<template>
  <div class="file-compare-view">
    <!-- Loading indicator -->
    <div v-if="loading" class="loading-overlay">
      <div class="spinner">Loading...</div>
    </div>

    <!-- Toolbar -->
    <div class="toolbar">
      <div class="toolbar-section navigation">
        <button @click="prevDiff" :disabled="currentRegionIndex === 0">
          Previous
        </button>
        <span class="region-counter">
          {{ currentRegionIndex + 1 }} / {{ diffRegions.length }}
        </span>
        <button @click="nextDiff" :disabled="currentRegionIndex >= diffRegions.length - 1">
          Next
        </button>
      </div>
      
      <div class="toolbar-section stats" v-if="diffResult">
        <span>Changes: {{ diffResult.total_changes ?? 0 }}</span>
        <span v-if="diffResult.moved_blocks">Moved: {{ diffResult.moved_blocks.length }}</span>
      </div>
    </div>

    <!-- Empty state message -->
    <div v-if="!diffResult" class="empty-state">
      <div class="empty-state-content">
        <h2>No Comparison</h2>
        <p>Open two files to begin comparing.</p>
        <p style="font-size: 0.9rem; color: #888;">Use the toolbar buttons to open files or folders.</p>
      </div>
    </div>

    <!-- Trivial Diff Filter (hidden from main UI) -->
    <!-- <TrivialDiffFilter
      v-if="diffResult"
      :stats="trivialStats"
      :loading="analyzingTrivial"
      @filter-changed="onFilterChange"
      style="margin: 0.5rem"
    /> -->

    <!-- Split view -->
    <div v-if="diffResult" class="split-view">
      <!-- Left pane -->
      <div class="pane left-pane">
        <div class="pane-header">{{ leftLabel }}</div>
        <div ref="leftPaneRef" class="pane-content" @scroll="onPaneScroll($event, true)">
          <div
            v-for="line in leftLineData"
            :key="line.isGhost ? `ghost-${line.displayNumber}` : `line-${line.number}`"
            :class="line.className"
            :style="{
              backgroundColor: line.style?.background,
              color: line.style?.foreground,
            }"
          >
            <span class="line-number" v-if="!line.isGhost">{{ line.number }}</span>
            <span class="line-number line-number-ghost" v-else>·</span>
            <span class="line-content">
              <template v-if="!line.isGhost && line.inlineHighlights.length > 0">
                <!-- Render with inline highlights -->
                <span
                  v-for="(part, idx) in splitTextWithHighlights(line.text, line.inlineHighlights)"
                  :key="idx"
                  :style="part.highlight ? { backgroundColor: part.highlight.background } : {}"
                >
                  {{ part.text }}
                </span>
              </template>
              <template v-else>
                {{ line.text }}
              </template>
            </span>
          </div>
        </div>
      </div>

      <!-- Right pane -->
      <div class="pane right-pane">
        <div class="pane-header">{{ rightLabel }}</div>
        <div ref="rightPaneRef" class="pane-content" @scroll="onPaneScroll($event, false)">
          <div
            v-for="line in rightLineData"
            :key="line.isGhost ? `ghost-${line.displayNumber}` : `line-${line.number}`"
            :class="line.className"
            :style="{
              backgroundColor: line.style?.background,
              color: line.style?.foreground,
            }"
          >
            <span class="line-number" v-if="!line.isGhost">{{ line.number }}</span>
            <span class="line-number line-number-ghost" v-else>·</span>
            <span class="line-content">
              <template v-if="!line.isGhost && line.inlineHighlights.length > 0">
                <!-- Render with inline highlights -->
                <span
                  v-for="(part, idx) in splitTextWithHighlights(line.text, line.inlineHighlights)"
                  :key="idx"
                  :style="part.highlight ? { backgroundColor: part.highlight.background } : {}"
                >
                  {{ part.text }}
                </span>
              </template>
              <template v-else>
                {{ line.text }}
              </template>
            </span>
          </div>
        </div>
      </div>

      <!-- Sidebar: Minimap + Location Pane -->
      <div class="sidebar-panel">
        <!-- Minimap -->
        <div v-if="showMinimap" class="minimap-wrapper-outer">
          <Minimap
            :diffResult="diffResult"
            :currentViewportStart="Math.max(0, currentViewportLine - 10)"
            :currentViewportEnd="Math.min(diffResult?.left_lines.length || 0, currentViewportLine + 50)"
            :lines="diffResult?.left_lines"
            @navigate="onMinimapNavigate"
          />
        </div>

        <!-- Location Pane -->
        <div v-if="showLocationPane" class="location-pane-wrapper-outer">
          <LocationPane
            :diffResult="diffResult"
            :currentLineNumber="currentViewportLine"
            :lines="diffResult?.left_lines"
            side="left"
            @navigate="onLocationNavigate"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
// Helper function to split text with inline highlights
function splitTextWithHighlights(text: string, highlights: any[]) {
  if (highlights.length === 0) {
    return [{ text, highlight: null }];
  }

  const parts: { text: string; highlight: any }[] = [];
  let lastEnd = 0;

  for (const highlight of highlights) {
    // Add text before highlight
    if (highlight.start > lastEnd) {
      parts.push({
        text: text.substring(lastEnd, highlight.start),
        highlight: null,
      });
    }

    // Add highlighted text
    parts.push({
      text: text.substring(highlight.start, highlight.end),
      highlight,
    });

    lastEnd = highlight.end;
  }

  // Add remaining text
  if (lastEnd < text.length) {
    parts.push({
      text: text.substring(lastEnd),
      highlight: null,
    });
  }

  return parts;
}
</script>

<style scoped>
.file-compare-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
}

.loading-overlay {
  position: absolute;
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

.toolbar {
  display: flex;
  gap: 1rem;
  padding: 0.5rem;
  background: var(--color-background-soft);
  border-bottom: 1px solid var(--color-border);
}

.toolbar-section {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.navigation button {
  padding: 0.25rem 0.75rem;
}

.region-counter {
  min-width: 4rem;
  text-align: center;
}

.stats {
  margin-left: auto;
  gap: 1rem;
}

.split-view {
  display: flex;
  flex: 1;
  overflow: hidden;
  gap: 0.5rem;
}

.pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.pane-header {
  padding: 0.5rem;
  background: var(--color-background-mute);
  border-bottom: 1px solid var(--color-border);
  font-weight: 600;
}

.pane-content {
  flex: 1;
  overflow: auto;
  font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.line {
  display: flex;
  padding: 0 0.5rem;
}

.line-number {
  min-width: 3rem;
  text-align: right;
  padding-right: 1rem;
  user-select: none;
  opacity: 0.5;
}

.line-content {
  flex: 1;
  white-space: pre;
}

/* Line highlighting styles */
.line-added {
  background-color: var(--color-diff-added-bg, #d4f4dd);
  color: var(--color-diff-added-fg, #0d5a0d);
}

.line-deleted {
  background-color: var(--color-diff-deleted-bg, #ffd7d5);
  color: var(--color-diff-deleted-fg, #82071e);
}

.line-modified {
  background-color: var(--color-diff-changed-bg, #fff5b1);
  color: var(--color-diff-changed-fg, #7d4e00);
}

.line-moved {
  background-color: var(--color-diff-moved-bg, #d8e5ff);
  color: var(--color-diff-moved-fg, #0550ae);
}

.line-trivial {
  opacity: 0.7;
}

.line-equal {
  background-color: transparent;
}

.sidebar-panel {
  width: 280px;
  min-width: 200px;
  max-width: 400px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-left: 1px solid var(--color-border);
  padding: 0.5rem;
  gap: 0.75rem;
}

.minimap-wrapper-outer {
  flex: 0 0 auto;
  min-height: 150px;
  overflow: hidden;
}

.location-pane-wrapper-outer {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.location-pane-wrapper {
  width: 280px;
  min-width: 200px;
  max-width: 400px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-left: 1px solid var(--color-border);
  padding: 0.5rem;
}

.location-pane-wrapper > :first-child {
  flex: 1;
  overflow: hidden;
}
</style>
