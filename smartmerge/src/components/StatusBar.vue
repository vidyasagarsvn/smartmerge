<script setup lang="ts">
const props = defineProps<{ message: string }>();
</script>

<template>
  <footer class="status-bar">
    <span>{{ props.message }}</span>
  </footer>
</template>
