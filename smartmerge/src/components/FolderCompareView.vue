<script setup lang="ts">
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
        <input type="checkbox" checked />
        Identical
      </label>
      <label class="filter-chip">
        <input type="checkbox" checked />
        Modified
      </label>
      <label class="filter-chip">
        <input type="checkbox" checked />
        Left only
      </label>
      <label class="filter-chip">
        <input type="checkbox" checked />
        Right only
      </label>
      <label class="filter-chip">
        <input type="checkbox" checked />
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
      <div
        v-for="item in props.items"
        :key="item.name"
        class="folder-row"
        role="button"
        tabindex="0"
        @dblclick="onRowActivate(item)"
      >
        <span>{{ item.name }}</span>
        <span>{{ statusLabel(item.status) }}</span>
      </div>
    </div>
  </section>
</template>
