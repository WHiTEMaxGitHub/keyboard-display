<script setup lang="ts">
defineProps<{
  wide?: boolean;
}>();
</script>

<template>
  <article :class="['panel', { 'wide-panel': wide }]">
    <slot />
  </article>
</template>

<style scoped>
.panel {
  box-sizing: border-box;
  width: 100%;
  min-height: 190px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #171b22;
  padding: 18px;
}

.panel h2,
.panel :deep(h2) {
  margin: 0 0 16px;
  letter-spacing: 0;
}

.wide-panel {
  width: 100%;
}
</style>
