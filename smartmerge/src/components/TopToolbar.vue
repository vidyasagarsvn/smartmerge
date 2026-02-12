<script setup lang="ts">
import { computed } from "vue";

type ActionId =
  | "open-files"
  | "open-folders"
  | "back"
  | "save"
  | "save-all"
  | "undo"
  | "redo"
  | "find"
  | "next"
  | "prev"
  | "copy-left"
  | "copy-right"
  | "copy-all-left"
  | "copy-all-right"
  | "engine"
  | "theme";

const props = defineProps<{
  theme: "light" | "dark";
  viewMode: "file" | "folder";
  engine: "smart" | "myers";
  canUndo: boolean;
  canRedo: boolean;
  canBack: boolean;
  canSave: boolean;
  canSaveAll: boolean;
  canNavigate: boolean;
  canCopy: boolean;
  canCopyAll: boolean;
}>();
const emit = defineEmits<{ action: [ActionId] }>();

const themeToggleTitle = computed(() =>
  props.theme === "dark" ? "Switch to Light Mode" : "Switch to Dark Mode"
);

function onAction(action: ActionId) {
  emit("action", action);
}
</script>

<template>
  <header class="toolbar">
    <div class="toolbar__group">
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        aria-label="Open Files"
        title="Open Files"
        @click="onAction('open-files')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-open-files" viewBox="0 0 20 20">
            <path d="M4 2h7l5 5v9a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zm7 1v4h4" />
          </svg>
        </span>
      </button>
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        aria-label="Open Folders"
        title="Open Folders"
        @click="onAction('open-folders')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-open-folders" viewBox="0 0 20 20">
            <path d="M2 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z" />
          </svg>
        </span>
      </button>
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canBack"
        aria-label="Back to Folders"
        title="Back to Folders"
        @click="onAction('back')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-back" viewBox="0 0 20 20">
            <path d="M7 4 2 9l5 5v-3h8a3 3 0 0 0 0-6H7V4z" />
          </svg>
        </span>
      </button>
    </div>

    <div class="toolbar__group">
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canSave"
        aria-label="Save"
        title="Save"
        @click="onAction('save')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-save" viewBox="0 0 20 20">
            <path d="M4 2h10l4 4v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zm1 0v6h8V2H5z" />
          </svg>
        </span>
      </button>
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canSaveAll"
        aria-label="Save All"
        title="Save All"
        @click="onAction('save-all')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-save-all" viewBox="0 0 20 20">
            <path d="M3 4h9l4 4v10a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2zm0-2h8l4 4v2H5a2 2 0 0 0-2 2v10H2V4a2 2 0 0 1 2-2z" />
          </svg>
        </span>
      </button>
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canUndo"
        aria-label="Undo"
        title="Undo"
        @click="onAction('undo')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-undo" viewBox="0 0 20 20">
            <path d="M7 5 3 9l4 4V10h6a3 3 0 0 1 0 6H9v-2h4a1 1 0 1 0 0-2H7V5z" />
          </svg>
        </span>
      </button>
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canRedo"
        aria-label="Redo"
        title="Redo"
        @click="onAction('redo')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-redo" viewBox="0 0 20 20">
            <path d="M13 5v3h-6a1 1 0 1 0 0 2h6v3l4-4-4-4zm-6 11a3 3 0 0 1 0-6h2v2H7a1 1 0 1 0 0 2h2v2H7z" />
          </svg>
        </span>
      </button>
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        aria-label="Find"
        title="Find"
        @click="onAction('find')"
        disabled
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-find" viewBox="0 0 20 20">
            <path d="M8 2a6 6 0 1 0 3.7 10.7l4.3 4.3 1.4-1.4-4.3-4.3A6 6 0 0 0 8 2zm0 2a4 4 0 1 1 0 8 4 4 0 0 1 0-8z" />
          </svg>
        </span>
      </button>
    </div>

    <div class="toolbar__group">
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canNavigate"
        aria-label="Previous Change"
        title="Previous Change"
        @click="onAction('prev')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-prev" viewBox="0 0 20 20">
            <path d="M12 4 6 10l6 6v-4h6V8h-6V4z" />
          </svg>
        </span>
      </button>
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canNavigate"
        aria-label="Next Change"
        title="Next Change"
        @click="onAction('next')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-next" viewBox="0 0 20 20">
            <path d="M8 4v4H2v4h6v4l6-6-6-6z" />
          </svg>
        </span>
      </button>
    </div>

    <div class="toolbar__group">
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canCopy"
        aria-label="Copy to Left"
        title="Copy to Left"
        @click="onAction('copy-left')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-copy-left" viewBox="0 0 20 20">
            <path d="M11 4 5 10l6 6v-4h7V8h-7V4z" />
          </svg>
        </span>
      </button>
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canCopy"
        aria-label="Copy to Right"
        title="Copy to Right"
        @click="onAction('copy-right')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-copy-right" viewBox="0 0 20 20">
            <path d="M9 4v4H2v4h7v4l6-6-6-6z" />
          </svg>
        </span>
      </button>
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canCopyAll"
        aria-label="Copy All to Left"
        title="Copy All to Left"
        @click="onAction('copy-all-left')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-copy-all-left" viewBox="0 0 20 20">
            <path d="M12 3 6 9l6 6v-4h6V7h-6V3zM6 3 0 9l6 6v-4h3V7H6V3z" />
          </svg>
        </span>
      </button>
      <button
        class="tool-btn tool-btn--icon"
        type="button"
        :disabled="!canCopyAll"
        aria-label="Copy All to Right"
        title="Copy All to Right"
        @click="onAction('copy-all-right')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg class="icon-copy-all-right" viewBox="0 0 20 20">
            <path d="M8 3v4H2v4h6v4l6-6-6-6zm6 0v4h-3v4h3v4l6-6-6-6z" />
          </svg>
        </span>
      </button>
    </div>

    <div class="toolbar__group toolbar__group--end">
      <button
        class="tool-btn tool-btn--icon tool-btn--accent"
        type="button"
        :aria-label="themeToggleTitle"
        :title="themeToggleTitle"
        @click="onAction('theme')"
      >
        <span class="tool-icon" aria-hidden="true">
          <svg v-if="props.theme === 'dark'" class="icon-theme-sun" viewBox="0 0 20 20">
            <path d="M10 3a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm0-3h1v3h-1V0zm0 17h1v3h-1v-3zM0 9h3v1H0V9zm17 0h3v1h-3V9zm-2.1-6.4 2.1-2.1.7.7-2.1 2.1-.7-.7zM2.3 16.4l2.1-2.1.7.7-2.1 2.1-.7-.7zM2.3 3.5l.7-.7 2.1 2.1-.7.7-2.1-2.1zm12.8 12.9.7-.7 2.1 2.1-.7.7-2.1-2.1z" />
          </svg>
          <svg v-else class="icon-theme-moon" viewBox="0 0 20 20">
            <path d="M12 2a8 8 0 1 0 6 12.7A6.5 6.5 0 0 1 12 2z" />
          </svg>
        </span>
      </button>
    </div>
  </header>
</template>
