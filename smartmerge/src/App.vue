<script setup lang="ts">
import { ref, watchEffect, computed, onMounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import TopToolbar from "./components/TopToolbar.vue";
import FileCompareView from "./components/FileCompareView.vue";
import FolderCompareView from "./components/FolderCompareView.vue";
import StatusBar from "./components/StatusBar.vue";

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
};

type FolderItem = {
  name: string;
  item_type: "file" | "folder";
  status:
    | "identical"
    | "modified"
    | "added_left"
    | "added_right"
    | "deleted_left"
    | "deleted_right"
    | "folder_left_only"
    | "folder_right_only";
  left_path: string | null;
  right_path: string | null;
};

const viewMode = ref<"file" | "folder">("file");
const theme = ref<"light" | "dark">("light");
const statusText = ref("Ready. Open files or folders to begin.");
const diffEngine = ref<"smart" | "myers">("smart");
const leftPath = ref<string | null>(null);
const rightPath = ref<string | null>(null);
const diffResult = ref<DiffResult | null>(null);
const folderItems = ref<FolderItem[]>([]);
const leftLines = ref<string[]>([]);
const rightLines = ref<string[]>([]);
const savedLeftLines = ref<string[]>([]);
const savedRightLines = ref<string[]>([]);
const isLeftModified = ref(false);
const isRightModified = ref(false);
const undoStack = ref<{ left: string[]; right: string[] }[]>([]);
const redoStack = ref<{ left: string[]; right: string[] }[]>([]);
const navigationStack = ref<{ left: string; right: string }[]>([]);
const viewingFromFolder = ref(false);
const fileCompareRef = ref<InstanceType<typeof FileCompareView> | null>(null);

const canUndo = computed(() => undoStack.value.length > 0);
const canRedo = computed(() => redoStack.value.length > 0);

watchEffect(() => {
  document.documentElement.dataset.theme = theme.value;
  localStorage.setItem("smartmerge.theme", theme.value);
});

watchEffect(() => {
  localStorage.setItem("smartmerge.engine", diffEngine.value);
});

const updateModifiedFlags = () => {
  isLeftModified.value =
    savedLeftLines.value.length > 0 &&
    leftLines.value.join("") !== savedLeftLines.value.join("");
  isRightModified.value =
    savedRightLines.value.length > 0 &&
    rightLines.value.join("") !== savedRightLines.value.join("");
};

const refreshDiffFromLines = async () => {
  try {
    const opcodes = await invoke<Opcode[]>("compare_lines", {
      leftLines: leftLines.value,
      rightLines: rightLines.value,
      engine: diffEngine.value,
    });
    diffResult.value = {
      left_lines: [...leftLines.value],
      right_lines: [...rightLines.value],
      opcodes,
    };
    updateModifiedFlags();
  } catch (error) {
    statusText.value = `Failed to update diff: ${String(error)}`;
  }
};

const pushUndo = () => {
  undoStack.value.push({
    left: [...leftLines.value],
    right: [...rightLines.value],
  });
  redoStack.value = [];
};

const handleUndo = async () => {
  if (!canUndo.value) {
    return;
  }
  const snapshot = undoStack.value.pop();
  if (!snapshot) {
    return;
  }
  redoStack.value.push({
    left: [...leftLines.value],
    right: [...rightLines.value],
  });
  leftLines.value = [...snapshot.left];
  rightLines.value = [...snapshot.right];
  await refreshDiffFromLines();
};

const handleRedo = async () => {
  if (!canRedo.value) {
    return;
  }
  const snapshot = redoStack.value.pop();
  if (!snapshot) {
    return;
  }
  undoStack.value.push({
    left: [...leftLines.value],
    right: [...rightLines.value],
  });
  leftLines.value = [...snapshot.left];
  rightLines.value = [...snapshot.right];
  await refreshDiffFromLines();
};

async function openFilePair() {
  const left = await open({
    multiple: false,
    filters: [{ name: "All Files", extensions: ["*"] }],
  });
  if (!left || Array.isArray(left)) {
    statusText.value = "Left file selection cancelled.";
    return;
  }

  const right = await open({
    multiple: false,
    filters: [{ name: "All Files", extensions: ["*"] }],
  });
  if (!right || Array.isArray(right)) {
    statusText.value = "Right file selection cancelled.";
    return;
  }

  leftPath.value = left;
  rightPath.value = right;
  localStorage.setItem("smartmerge.lastLeftPath", left);
  localStorage.setItem("smartmerge.lastRightPath", right);
  await loadFileDiff();
}

async function loadFileDiff() {
  if (!leftPath.value || !rightPath.value) {
    statusText.value = "Select left and right files first.";
    return;
  }

  try {
    diffResult.value = await invoke<DiffResult>("compare_files", {
      leftPath: leftPath.value,
      rightPath: rightPath.value,
      engine: diffEngine.value,
    });
    leftLines.value = [...diffResult.value.left_lines];
    rightLines.value = [...diffResult.value.right_lines];
    savedLeftLines.value = [...diffResult.value.left_lines];
    savedRightLines.value = [...diffResult.value.right_lines];
    isLeftModified.value = false;
    isRightModified.value = false;
    viewMode.value = "file";
    viewingFromFolder.value = false;
    statusText.value = `Comparing ${leftPath.value} and ${rightPath.value}`;
  } catch (error) {
    statusText.value = `Failed to compare files: ${String(error)}`;
  }
}

async function openFolderPair() {
  const left = await open({ directory: true, multiple: false });
  if (!left || Array.isArray(left)) {
    statusText.value = "Left folder selection cancelled.";
    return;
  }

  const right = await open({ directory: true, multiple: false });
  if (!right || Array.isArray(right)) {
    statusText.value = "Right folder selection cancelled.";
    return;
  }

  leftPath.value = left;
  rightPath.value = right;
  localStorage.setItem("smartmerge.lastLeftPath", left);
  localStorage.setItem("smartmerge.lastRightPath", right);
  navigationStack.value = [];
  await loadFolderCompare();
}

async function loadFolderCompare() {
  if (!leftPath.value || !rightPath.value) {
    statusText.value = "Select left and right folders first.";
    return;
  }

  try {
    folderItems.value = await invoke<FolderItem[]>("compare_folders_command", {
      leftPath: leftPath.value,
      rightPath: rightPath.value,
    });
    viewMode.value = "folder";
    statusText.value = `Comparing folders ${leftPath.value} and ${rightPath.value}`;
  } catch (error) {
    statusText.value = `Failed to compare folders: ${String(error)}`;
  }
}

const handleFolderNavigate = async (left: string, right: string) => {
  if (leftPath.value && rightPath.value) {
    navigationStack.value.push({ left: leftPath.value, right: rightPath.value });
  }
  leftPath.value = left;
  rightPath.value = right;
  await loadFolderCompare();
};

const handleFolderFileOpen = async (left: string, right: string) => {
  leftPath.value = left;
  rightPath.value = right;
  viewingFromFolder.value = true;
  await loadFileDiff();
};

const toggleEngine = async () => {
  diffEngine.value = diffEngine.value === "smart" ? "myers" : "smart";
  if (diffResult.value) {
    await refreshDiffFromLines();
  }
};

const handleCopyAllToRight = async () => {
  if (!diffResult.value) {
    return;
  }
  pushUndo();
  rightLines.value = [...leftLines.value];
  isRightModified.value = true;
  await refreshDiffFromLines();
};

const handleCopyAllToLeft = async () => {
  if (!diffResult.value) {
    return;
  }
  pushUndo();
  leftLines.value = [...rightLines.value];
  isLeftModified.value = true;
  await refreshDiffFromLines();
};

const getRegionRanges = () => {
  const regionState = fileCompareRef.value?.getRegionState();
  if (!regionState || regionState.currentIndex < 0) {
    return null;
  }
  const region = regionState.regions[regionState.currentIndex];
  if (!region) {
    return null;
  }
  const leftIndices: number[] = [];
  const rightIndices: number[] = [];
  for (let i = region.start; i <= region.end; i += 1) {
    const leftIdx = regionState.leftMap[i];
    const rightIdx = regionState.rightMap[i];
    if (leftIdx !== undefined && leftIdx >= 0) {
      leftIndices.push(leftIdx);
    }
    if (rightIdx !== undefined && rightIdx >= 0) {
      rightIndices.push(rightIdx);
    }
  }
  const leftRange = leftIndices.length
    ? { start: Math.min(...leftIndices), end: Math.max(...leftIndices) }
    : null;
  const rightRange = rightIndices.length
    ? { start: Math.min(...rightIndices), end: Math.max(...rightIndices) }
    : null;
  return { region, leftRange, rightRange, regionState };
};

const handleCopyToRight = async () => {
  if (!diffResult.value) {
    return;
  }
  const rangeInfo = getRegionRanges();
  if (!rangeInfo) {
    statusText.value = "No change region selected.";
    return;
  }
  pushUndo();
  const { leftRange, rightRange, regionState, region } = rangeInfo;
  if (leftRange) {
    if (rightRange) {
      rightLines.value.splice(
        rightRange.start,
        rightRange.end - rightRange.start + 1,
        ...leftLines.value.slice(leftRange.start, leftRange.end + 1)
      );
    } else {
      const insertPos = findInsertPosition(
        regionState.rightMap,
        region.end + 1,
        rightLines.value.length
      );
      rightLines.value.splice(
        insertPos,
        0,
        ...leftLines.value.slice(leftRange.start, leftRange.end + 1)
      );
    }
  } else if (rightRange) {
    rightLines.value.splice(rightRange.start, rightRange.end - rightRange.start + 1);
  }
  isRightModified.value = true;
  await refreshDiffFromLines();
};

const handleCopyToLeft = async () => {
  if (!diffResult.value) {
    return;
  }
  const rangeInfo = getRegionRanges();
  if (!rangeInfo) {
    statusText.value = "No change region selected.";
    return;
  }
  pushUndo();
  const { leftRange, rightRange, regionState, region } = rangeInfo;
  if (rightRange) {
    if (leftRange) {
      leftLines.value.splice(
        leftRange.start,
        leftRange.end - leftRange.start + 1,
        ...rightLines.value.slice(rightRange.start, rightRange.end + 1)
      );
    } else {
      const insertPos = findInsertPosition(
        regionState.leftMap,
        region.end + 1,
        leftLines.value.length
      );
      leftLines.value.splice(
        insertPos,
        0,
        ...rightLines.value.slice(rightRange.start, rightRange.end + 1)
      );
    }
  } else if (leftRange) {
    leftLines.value.splice(leftRange.start, leftRange.end - leftRange.start + 1);
  }
  isLeftModified.value = true;
  await refreshDiffFromLines();
};

const findInsertPosition = (map: number[], startIndex: number, fallback: number) => {
  for (let i = startIndex; i < map.length; i += 1) {
    if (map[i] >= 0) {
      return map[i];
    }
  }
  return fallback;
};

const handleSave = async () => {
  if (!leftPath.value || !rightPath.value) {
    statusText.value = "No files to save.";
    return;
  }
  const tasks: Promise<void>[] = [];
  if (isLeftModified.value) {
    tasks.push(invoke("save_file", { path: leftPath.value, lines: leftLines.value }) as Promise<void>);
  }
  if (isRightModified.value) {
    tasks.push(invoke("save_file", { path: rightPath.value, lines: rightLines.value }) as Promise<void>);
  }
  if (tasks.length === 0) {
    statusText.value = "No modified files to save.";
    return;
  }
  try {
    await Promise.all(tasks);
    savedLeftLines.value = [...leftLines.value];
    savedRightLines.value = [...rightLines.value];
    updateModifiedFlags();
    statusText.value = "Saved modified files.";
  } catch (error) {
    statusText.value = `Save failed: ${String(error)}`;
  }
};

const handleSaveAll = async () => {
  await handleSave();
};

const handleNextChange = () => {
  fileCompareRef.value?.nextChange();
};

const handlePrevChange = () => {
  fileCompareRef.value?.prevChange();
};

const handleBack = async () => {
  if (viewMode.value === "file" && viewingFromFolder.value) {
    viewMode.value = "folder";
    viewingFromFolder.value = false;
    statusText.value = "Back to folder view.";
    return;
  }
  if (viewMode.value === "folder" && navigationStack.value.length > 0) {
    const previous = navigationStack.value.pop();
    if (previous) {
      leftPath.value = previous.left;
      rightPath.value = previous.right;
      await loadFolderCompare();
    }
  }
};

const onKeydown = (event: KeyboardEvent) => {
  const isMac = navigator.platform.toLowerCase().includes("mac");
  const mod = isMac ? event.metaKey : event.ctrlKey;

  const key = event.key.toLowerCase();
  if (mod && key === "o" && !event.shiftKey) {
    event.preventDefault();
    openFilePair();
  } else if (mod && key === "o" && event.shiftKey) {
    event.preventDefault();
    openFolderPair();
  } else if (mod && key === "s" && !event.shiftKey) {
    event.preventDefault();
    handleSave();
  } else if (mod && key === "s" && event.shiftKey) {
    event.preventDefault();
    handleSaveAll();
  } else if (mod && key === "z" && !event.shiftKey) {
    event.preventDefault();
    handleUndo();
  } else if (mod && key === "z" && event.shiftKey) {
    event.preventDefault();
    handleRedo();
  } else if (event.altKey && key === "arrowdown") {
    event.preventDefault();
    handleNextChange();
  } else if (event.altKey && key === "arrowup") {
    event.preventDefault();
    handlePrevChange();
  } else if (event.altKey && key === "arrowright") {
    event.preventDefault();
    handleCopyToRight();
  } else if (event.altKey && key === "arrowleft") {
    event.preventDefault();
    handleCopyToLeft();
  } else if (mod && event.altKey && key === "arrowright") {
    event.preventDefault();
    handleCopyAllToRight();
  } else if (mod && event.altKey && key === "arrowleft") {
    event.preventDefault();
    handleCopyAllToLeft();
  }
};

onMounted(() => {
  const storedTheme = localStorage.getItem("smartmerge.theme");
  const storedEngine = localStorage.getItem("smartmerge.engine");
  const storedLeft = localStorage.getItem("smartmerge.lastLeftPath");
  const storedRight = localStorage.getItem("smartmerge.lastRightPath");

  if (storedTheme === "light" || storedTheme === "dark") {
    theme.value = storedTheme;
  }
  if (storedEngine === "smart" || storedEngine === "myers") {
    diffEngine.value = storedEngine;
  }
  if (storedLeft) {
    leftPath.value = storedLeft;
  }
  if (storedRight) {
    rightPath.value = storedRight;
  }

  window.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
});

function handleToolbarAction(action: string) {
  switch (action) {
    case "open-files":
      openFilePair();
      break;
    case "open-folders":
      openFolderPair();
      break;
    case "back":
      handleBack();
      break;
    case "save":
      handleSave();
      break;
    case "save-all":
      handleSaveAll();
      break;
    case "undo":
      handleUndo();
      break;
    case "redo":
      handleRedo();
      break;
    case "theme":
      theme.value = theme.value === "light" ? "dark" : "light";
      statusText.value = `Switched to ${theme.value} mode.`;
      break;
    case "engine":
      toggleEngine();
      statusText.value = `Engine set to ${diffEngine.value}.`;
      break;
    case "next":
      handleNextChange();
      break;
    case "prev":
      handlePrevChange();
      break;
    case "copy-left":
      handleCopyToLeft();
      break;
    case "copy-right":
      handleCopyToRight();
      break;
    case "copy-all-left":
      handleCopyAllToLeft();
      break;
    case "copy-all-right":
      handleCopyAllToRight();
      break;
    default:
      statusText.value = `Action: ${action}`;
  }
}
</script>

<template>
  <div class="app-shell">
    <TopToolbar
      :theme="theme"
      :view-mode="viewMode"
      :engine="diffEngine"
      :can-undo="canUndo"
      :can-redo="canRedo"
      @action="handleToolbarAction"
    />

    <main class="content-area">
      <FileCompareView
        v-if="viewMode === 'file'"
        ref="fileCompareRef"
        :left-label="leftPath ?? 'Left file'"
        :right-label="rightPath ?? 'Right file'"
        :diff-result="diffResult"
        :engine="diffEngine"
      />
      <FolderCompareView
        v-else
        :items="folderItems"
        @navigate="handleFolderNavigate"
        @open-file="handleFolderFileOpen"
      />
    </main>

    <StatusBar :message="statusText" />
  </div>
</template>