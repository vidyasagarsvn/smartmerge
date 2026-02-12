<script setup lang="ts">
import { computed, ref } from "vue";
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

const props = defineProps<{ items: FolderItem[] }>();
const emit = defineEmits<{
  navigate: [string, string];
  openFile: [string, string];
}>();

const statusLabel = (status: FolderItem["status"]) => {
  switch (status) {
    case "identical":
      return "Identical";
    case "modified":
      return "Modified";
    case "added_left":
    case "folder_left_only":
      return "Left only";
    case "added_right":
    case "folder_right_only":
      return "Right only";
    case "deleted_left":
    case "deleted_right":
      return "Missing";
    default:
      return status;
  }
};

const isNavigableFolder = (item: FolderItem) =>
  item.item_type === "folder" && item.left_path && item.right_path;

const isComparableFile = (item: FolderItem) =>
  item.item_type === "file" && item.left_path && item.right_path;

const fileTypeClass = (name: string) => {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  switch (ext) {
    case "py":
      return "filetype--python";
    case "md":
      return "filetype--markdown";
    case "json":
      return "filetype--json";
    case "txt":
      return "filetype--text";
    default:
      return "filetype--generic";
  }
};

const filterState = ref({
  identical: true,
  modified: true,
  leftOnly: true,
  rightOnly: true,
  missing: true,
});

const statusKey = (status: FolderItem["status"]) => {
  switch (status) {
    case "identical":
      return "identical";
    case "modified":
      return "modified";
    case "added_left":
    case "folder_left_only":
      return "leftOnly";
    case "added_right":
    case "folder_right_only":
      return "rightOnly";
    case "deleted_left":
    case "deleted_right":
      return "missing";
    default:
      return "modified";
  }
};

const filteredItems = computed(() =>
  props.items.filter((item) => filterState.value[statusKey(item.status)])
);

const sortedItems = computed(() => {
  const items = [...filteredItems.value];
  return items.sort((a, b) => {
    if (a.item_type !== b.item_type) {
      return a.item_type === "folder" ? -1 : 1;
    }
    return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
  });
});

const onRowActivate = (item: FolderItem) => {
  if (isNavigableFolder(item)) {
    emit("navigate", item.left_path as string, item.right_path as string);
  } else if (isComparableFile(item)) {
    emit("openFile", item.left_path as string, item.right_path as string);
  }
};
</script>

<template>
  <section class="folder-compare">
    <header class="folder-filters">
      <span class="filter-label">Filter by status</span>
      <label class="filter-chip">
        <input v-model="filterState.identical" type="checkbox" />
        Identical
      </label>
      <label class="filter-chip">
        <input v-model="filterState.modified" type="checkbox" />
        Modified
      </label>
      <label class="filter-chip">
        <input v-model="filterState.leftOnly" type="checkbox" />
        Left only
      </label>
      <label class="filter-chip">
        <input v-model="filterState.rightOnly" type="checkbox" />
        Right only
      </label>
      <label class="filter-chip">
        <input v-model="filterState.missing" type="checkbox" />
        Missing
      </label>
    </header>

    <div class="folder-table">
      <div class="folder-row folder-row--head">
        <span>Name</span>
        <span>Status</span>
      </div>
      <div v-if="props.items.length === 0" class="folder-row">
        <span>No folder comparison loaded.</span>
        <span>---</span>
      </div>
      <div v-else-if="filteredItems.length === 0" class="folder-row">
        <span>No items match the current filters.</span>
        <span>---</span>
      </div>
      <div
        v-for="item in sortedItems"
        :key="item.name"
        class="folder-row"
        :class="{
          'folder-row--disabled': !isNavigableFolder(item) && !isComparableFile(item),
        }"
        :role="isNavigableFolder(item) || isComparableFile(item) ? 'button' : undefined"
        :tabindex="isNavigableFolder(item) || isComparableFile(item) ? 0 : -1"
        @dblclick="onRowActivate(item)"
      >
        <span class="folder-name">
          <span
            class="folder-icon"
            :class="item.item_type === 'folder' ? 'filetype--folder' : fileTypeClass(item.name)"
            aria-hidden="true"
          >
            <svg v-if="item.item_type === 'folder'" viewBox="0 0 20 20">
              <path d="M2 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z" />
            </svg>
            <svg v-else viewBox="0 0 20 20">
              <path d="M4 2h7l5 5v9a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zm7 1v4h4" />
            </svg>
          </span>
          {{ item.name }}
        </span>
        <span>{{ item.item_type === "folder" ? "" : statusLabel(item.status) }}</span>
      </div>
    </div>
  </section>
</template>
