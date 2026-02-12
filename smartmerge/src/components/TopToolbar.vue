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
}>();
const emit = defineEmits<{ action: [ActionId] }>();

const themeLabel = computed(() => (props.theme === "light" ? "Dark" : "Light"));
const engineLabel = computed(() => (props.engine === "smart" ? "Smart" : "Myers"));

function onAction(action: ActionId) {
  emit("action", action);
}
</script>

<template>
  <header class="toolbar">
    <div class="toolbar__group">
      <button class="tool-btn" type="button" @click="onAction('open-files')">
        Open Files
      </button>
      <button class="tool-btn" type="button" @click="onAction('open-folders')">
        Open Folders
      </button>
      <button
        class="tool-btn"
        type="button"
        :disabled="viewMode !== 'file'"
        @click="onAction('back')"
      >
        Back to Folders
      </button>
    </div>

    <div class="toolbar__group">
      <button class="tool-btn" type="button" @click="onAction('save')">Save</button>
      <button class="tool-btn" type="button" @click="onAction('save-all')">Save All</button>
      <button class="tool-btn" type="button" :disabled="!canUndo" @click="onAction('undo')">
        Undo
      </button>
      <button class="tool-btn" type="button" :disabled="!canRedo" @click="onAction('redo')">
        Redo
      </button>
      <button class="tool-btn" type="button" @click="onAction('find')" disabled>Find</button>
    </div>

    <div class="toolbar__group">
      <button class="tool-btn" type="button" @click="onAction('prev')">Prev</button>
      <button class="tool-btn" type="button" @click="onAction('next')">Next</button>
    </div>

    <div class="toolbar__group">
      <button class="tool-btn" type="button" @click="onAction('copy-left')">Copy Left</button>
      <button class="tool-btn" type="button" @click="onAction('copy-right')">Copy Right</button>
      <button class="tool-btn" type="button" @click="onAction('copy-all-left')">
        Copy All Left
      </button>
      <button class="tool-btn" type="button" @click="onAction('copy-all-right')">
        Copy All Right
      </button>
    </div>

    <div class="toolbar__group toolbar__group--end">
      <button class="tool-btn" type="button" @click="onAction('engine')">
        Engine: {{ engineLabel }}
      </button>
      <button class="tool-btn tool-btn--accent" type="button" @click="onAction('theme')">
        {{ themeLabel }} Mode
      </button>
    </div>
  </header>
</template>
