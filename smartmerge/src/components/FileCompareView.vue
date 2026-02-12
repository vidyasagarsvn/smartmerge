<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";

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

const emit = defineEmits<{
  "region-change": [
    {
      hasSelection: boolean;
      regionCount: number;
      currentIndex: number;
    },
  ];
}>();

type LineCell = {
  text: string;
  className: string;
};

type Region = {
  start: number;
  end: number;
};

type SubOp = {
  tag: "equal" | "delete" | "insert";
  i1: number;
  i2: number;
  j1: number;
  j2: number;
};

const lcsDiff = (left: string[], right: string[]): SubOp[] => {
  const m = left.length;
  const n = right.length;
  const dp: number[][] = Array.from({ length: m + 1 }, () => Array(n + 1).fill(0));

  for (let i = 1; i <= m; i += 1) {
    for (let j = 1; j <= n; j += 1) {
      if (left[i - 1] === right[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  const ops: SubOp[] = [];
  let i = m;
  let j = n;
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && left[i - 1] === right[j - 1]) {
      ops.push({ tag: "equal", i1: i - 1, i2: i, j1: j - 1, j2: j });
      i -= 1;
      j -= 1;
    } else if (i > 0 && (j === 0 || dp[i - 1][j] >= dp[i][j - 1])) {
      ops.push({ tag: "delete", i1: i - 1, i2: i, j1: j, j2: j });
      i -= 1;
    } else {
      ops.push({ tag: "insert", i1: i, i2: i, j1: j - 1, j2: j });
      j -= 1;
    }
  }

  ops.reverse();
  const merged: SubOp[] = [];
  for (const op of ops) {
    const last = merged[merged.length - 1];
    if (
      last &&
      last.tag === op.tag &&
      last.i2 === op.i1 &&
      last.j2 === op.j1
    ) {
      last.i2 = op.i2;
      last.j2 = op.j2;
    } else {
      merged.push({ ...op });
    }
  }
  return merged;
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
      const blockSize = leftBlock.length * rightBlock.length;
      const subOps = blockSize <= 40000 ? lcsDiff(leftBlock, rightBlock) : null;

      if (subOps) {
        for (const subOp of subOps) {
          if (subOp.tag === "equal") {
            for (let idx = subOp.i1; idx < subOp.i2; idx += 1) {
              const rightIdx = subOp.j1 + (idx - subOp.i1);
              leftOut.push({ text: leftBlock[idx], className: "line" });
              rightOut.push({ text: rightBlock[rightIdx], className: "line" });
              leftMap.push(opcode.i1 + idx);
              rightMap.push(opcode.j1 + rightIdx);
            }
          } else if (subOp.tag === "delete") {
            for (let idx = subOp.i1; idx < subOp.i2; idx += 1) {
              const leftLine = leftBlock[idx];
              leftOut.push({ text: leftLine, className: "line line--delete" });
              rightOut.push({ text: "", className: "line" });
              leftMap.push(opcode.i1 + idx);
              rightMap.push(-1);
            }
          } else {
            for (let idx = subOp.j1; idx < subOp.j2; idx += 1) {
              const rightLine = rightBlock[idx];
              leftOut.push({ text: "", className: "line" });
              rightOut.push({ text: rightLine, className: "line line--add" });
              leftMap.push(-1);
              rightMap.push(opcode.j1 + idx);
            }
          }
        }
      } else {
        const maxLen = Math.max(leftBlock.length, rightBlock.length);
        for (let idx = 0; idx < maxLen; idx += 1) {
          const leftLine = leftBlock[idx] ?? "";
          const rightLine = rightBlock[idx] ?? "";
          leftOut.push({ text: leftLine, className: "line line--change" });
          rightOut.push({ text: rightLine, className: "line line--change" });
          leftMap.push(leftLine === "" ? -1 : opcode.i1 + idx);
          rightMap.push(rightLine === "" ? -1 : opcode.j1 + idx);
        }
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
const regionFileRanges = computed(() => {
  return regions.value.map((region) => {
    const leftIndices = new Set<number>();
    const rightIndices = new Set<number>();
    for (let i = region.start; i <= region.end; i += 1) {
      const leftIdx = alignedLines.value.leftMap[i];
      const rightIdx = alignedLines.value.rightMap[i];
      if (leftIdx !== undefined && leftIdx >= 0) {
        leftIndices.add(leftIdx);
      }
      if (rightIdx !== undefined && rightIdx >= 0) {
        rightIndices.add(rightIdx);
      }
    }
    const leftRange = leftIndices.size
      ? { start: Math.min(...leftIndices), end: Math.max(...leftIndices) }
      : { start: -1, end: -1 };
    const rightRange = rightIndices.size
      ? { start: Math.min(...rightIndices), end: Math.max(...rightIndices) }
      : { start: -1, end: -1 };
    return { left: leftRange, right: rightRange };
  });
});
const currentRegionIndex = ref(-1);
const leftPaneRef = ref<HTMLDivElement | null>(null);
const rightPaneRef = ref<HTMLDivElement | null>(null);
const isSyncingScroll = ref(false);
const lastScrollTop = ref({ left: 0, right: 0 });
let syncFrame: number | null = null;

watch(
  () => props.diffResult,
  () => {
    if (regions.value.length === 0) {
      currentRegionIndex.value = -1;
      return;
    }
    if (currentRegionIndex.value < 0) {
      currentRegionIndex.value = 0;
    } else if (currentRegionIndex.value >= regions.value.length) {
      currentRegionIndex.value = regions.value.length - 1;
    }
    void scrollRegionIntoView();
  }
);

watch(
  [() => regions.value.length, () => currentRegionIndex.value],
  () => {
    emit("region-change", {
      hasSelection: currentRegionIndex.value >= 0 && regions.value.length > 0,
      regionCount: regions.value.length,
      currentIndex: currentRegionIndex.value,
    });
  },
  { immediate: true }
);

const scrollRegionIntoView = async () => {
  const region = regions.value[currentRegionIndex.value];
  if (!region) {
    return;
  }
  await nextTick();
  const leftPane = leftPaneRef.value;
  const rightPane = rightPaneRef.value;
  if (!leftPane || !rightPane) {
    return;
  }
  const leftTarget = leftPane.querySelector(`[data-line-idx="${region.start}"]`);
  const rightTarget = rightPane.querySelector(`[data-line-idx="${region.start}"]`);
  leftTarget?.scrollIntoView({ block: "center" });
  rightTarget?.scrollIntoView({ block: "center" });
};

watch(
  () => currentRegionIndex.value,
  () => {
    void scrollRegionIntoView();
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
  regionFileRanges: regionFileRanges.value,
  currentIndex: currentRegionIndex.value,
  leftMap: alignedLines.value.leftMap,
  rightMap: alignedLines.value.rightMap,
});

const syncScroll = (source: "left" | "right") => {
  if (isSyncingScroll.value) {
    return;
  }
  const left = leftPaneRef.value;
  const right = rightPaneRef.value;
  if (!left || !right) {
    return;
  }
  if (syncFrame !== null) {
    cancelAnimationFrame(syncFrame);
  }
  syncFrame = requestAnimationFrame(() => {
    isSyncingScroll.value = true;
    if (source === "left") {
      const nextTop = left.scrollTop;
      if (nextTop !== lastScrollTop.value.left) {
        lastScrollTop.value.left = nextTop;
        right.scrollTop = nextTop;
      }
    } else {
      const nextTop = right.scrollTop;
      if (nextTop !== lastScrollTop.value.right) {
        lastScrollTop.value.right = nextTop;
        left.scrollTop = nextTop;
      }
    }
    isSyncingScroll.value = false;
    syncFrame = null;
  });
};

defineExpose({ nextChange, prevChange, getRegionState });
</script>

<template>
  <section class="compare-grid">
    <div class="compare-pane">
      <header class="pane-header">
        <input
          class="pane-label"
          type="text"
          :value="props.leftLabel"
          readonly
          aria-label="Left file path"
        />
      </header>
      <div class="pane-body" ref="leftPaneRef" @scroll="() => syncScroll('left')">
        <div v-if="!props.diffResult" class="empty-state">
          Open two files to see the diff.
        </div>
        <div v-else class="code-list">
          <div
            v-for="(line, index) in leftLines"
            :key="`l-${index}`"
            :class="[line.className, isSelected(index) ? 'line--selected' : '']"
            :data-line-idx="index"
          >
            <span class="line-no">{{ String(index + 1).padStart(3, '0') }}</span>
            <span class="line-text">{{ line.text }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="compare-actions">
      <button class="chip" type="button" disabled>Copy to Right</button>
      <button class="chip" type="button" disabled>Copy to Left</button>
      <button class="chip" type="button" disabled>Copy All Right</button>
      <button class="chip" type="button" disabled>Copy All Left</button>
    </div>

    <div class="compare-pane">
      <header class="pane-header">
        <input
          class="pane-label"
          type="text"
          :value="props.rightLabel"
          readonly
          aria-label="Right file path"
        />
      </header>
      <div class="pane-body" ref="rightPaneRef" @scroll="() => syncScroll('right')">
        <div v-if="!props.diffResult" class="empty-state">
          The comparison output will render here.
        </div>
        <div v-else class="code-list">
          <div
            v-for="(line, index) in rightLines"
            :key="`r-${index}`"
            :class="[line.className, isSelected(index) ? 'line--selected' : '']"
            :data-line-idx="index"
          >
            <span class="line-no">{{ String(index + 1).padStart(3, '0') }}</span>
            <span class="line-text">{{ line.text }}</span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
