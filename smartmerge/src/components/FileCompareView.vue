<script setup lang="ts">
import { computed, ref, watch } from "vue";

type Opcode = {
  tag: string;
  i1: number;
  i2: number;
  j1: number;
  j2: number;
};

type DiffResult = {
  left_lines: string[];
  right_lines: string[];
  opcodes: Opcode[];
} | null;

const props = defineProps<{
  leftLabel: string;
  rightLabel: string;
  diffResult: DiffResult;
  engine: "smart" | "myers";
}>();

type LineCell = {
  text: string;
  className: string;
};

type Region = {
  start: number;
  end: number;
};

const alignedLines = computed(() => {
  if (!props.diffResult) {
    return {
      left: [],
      right: [],
      leftMap: [],
      rightMap: [],
      regions: [],
    } as {
      left: LineCell[];
      right: LineCell[];
      leftMap: number[];
      rightMap: number[];
      regions: Region[];
    };
  }

  const leftOut: LineCell[] = [];
  const rightOut: LineCell[] = [];
  const leftMap: number[] = [];
  const rightMap: number[] = [];

  for (const opcode of props.diffResult.opcodes) {
    const leftBlock = props.diffResult.left_lines.slice(opcode.i1, opcode.i2);
    const rightBlock = props.diffResult.right_lines.slice(opcode.j1, opcode.j2);

    if (opcode.tag === "equal") {
      for (let idx = 0; idx < leftBlock.length; idx += 1) {
        leftOut.push({ text: leftBlock[idx], className: "line" });
        rightOut.push({ text: rightBlock[idx], className: "line" });
        leftMap.push(opcode.i1 + idx);
        rightMap.push(opcode.j1 + idx);
      }
    } else if (opcode.tag === "replace") {
      const maxLen = Math.max(leftBlock.length, rightBlock.length);
      for (let idx = 0; idx < maxLen; idx += 1) {
        const leftLine = leftBlock[idx] ?? "";
        const rightLine = rightBlock[idx] ?? "";
        leftOut.push({ text: leftLine, className: "line line--change" });
        rightOut.push({ text: rightLine, className: "line line--change" });
        leftMap.push(leftLine === "" ? -1 : opcode.i1 + idx);
        rightMap.push(rightLine === "" ? -1 : opcode.j1 + idx);
      }
    } else if (opcode.tag === "insert") {
      for (let idx = 0; idx < rightBlock.length; idx += 1) {
        const rightLine = rightBlock[idx];
        leftOut.push({ text: "", className: "line" });
        rightOut.push({ text: rightLine, className: "line line--add" });
        leftMap.push(-1);
        rightMap.push(opcode.j1 + idx);
      }
    } else if (opcode.tag === "delete") {
      for (let idx = 0; idx < leftBlock.length; idx += 1) {
        const leftLine = leftBlock[idx];
        leftOut.push({ text: leftLine, className: "line line--delete" });
        rightOut.push({ text: "", className: "line" });
        leftMap.push(opcode.i1 + idx);
        rightMap.push(-1);
      }
    }
  }

  const changeIndices: number[] = [];
  leftOut.forEach((line, idx) => {
    if (line.className !== "line" || rightOut[idx].className !== "line") {
      changeIndices.push(idx);
    }
  });

  const regions: Region[] = [];
  if (changeIndices.length > 0) {
    let start = changeIndices[0];
    let end = changeIndices[0];
    for (const idx of changeIndices.slice(1)) {
      if (idx === end + 1) {
        end = idx;
      } else {
        regions.push({ start, end });
        start = idx;
        end = idx;
      }
    }
    regions.push({ start, end });
  }

  return { left: leftOut, right: rightOut, leftMap, rightMap, regions };
});

const leftLines = computed(() => alignedLines.value.left);
const rightLines = computed(() => alignedLines.value.right);
const regions = computed(() => alignedLines.value.regions);
const currentRegionIndex = ref(-1);

watch(
  () => props.diffResult,
  () => {
    currentRegionIndex.value = regions.value.length > 0 ? 0 : -1;
  }
);

const isSelected = (index: number) => {
  const region = regions.value[currentRegionIndex.value];
  if (!region) {
    return false;
  }
  return index >= region.start && index <= region.end;
};

const nextChange = () => {
  if (regions.value.length === 0) {
    return;
  }
  if (currentRegionIndex.value < regions.value.length - 1) {
    currentRegionIndex.value += 1;
  }
};

const prevChange = () => {
  if (regions.value.length === 0) {
    return;
  }
  if (currentRegionIndex.value > 0) {
    currentRegionIndex.value -= 1;
  }
};

const getRegionState = () => ({
  regions: regions.value,
  currentIndex: currentRegionIndex.value,
  leftMap: alignedLines.value.leftMap,
  rightMap: alignedLines.value.rightMap,
});

defineExpose({ nextChange, prevChange, getRegionState });
</script>

<template>
  <section class="compare-grid">
    <div class="compare-pane">
      <header class="pane-header">
        <span>{{ props.leftLabel }}</span>
      </header>
      <div class="pane-body">
        <div v-if="!props.diffResult" class="empty-state">
          Open two files to see the diff.
        </div>
        <div v-else class="code-list">
          <div
            v-for="(line, index) in leftLines"
            :key="`l-${index}`"
            :class="[line.className, isSelected(index) ? 'line--selected' : '']"
          >
            <span class="line-no">{{ String(index + 1).padStart(3, '0') }}</span>
            <span class="line-text">{{ line.text }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="compare-actions">
      <div class="engine-pill">Engine: {{ props.engine }}</div>
      <button class="chip" type="button" disabled>Copy to Right</button>
      <button class="chip" type="button" disabled>Copy to Left</button>
      <button class="chip" type="button" disabled>Copy All Right</button>
      <button class="chip" type="button" disabled>Copy All Left</button>
    </div>

    <div class="compare-pane">
      <header class="pane-header">
        <span>{{ props.rightLabel }}</span>
      </header>
      <div class="pane-body">
        <div v-if="!props.diffResult" class="empty-state">
          The comparison output will render here.
        </div>
        <div v-else class="code-list">
          <div
            v-for="(line, index) in rightLines"
            :key="`r-${index}`"
            :class="[line.className, isSelected(index) ? 'line--selected' : '']"
          >
            <span class="line-no">{{ String(index + 1).padStart(3, '0') }}</span>
            <span class="line-text">{{ line.text }}</span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
