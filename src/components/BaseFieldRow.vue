<script setup lang="ts">
defineProps<{
  label: string;
}>();
</script>

<template>
  <div class="flex justify-between gap-3 py-2.5 border-b border-border-dim">
    <span class="text-text-muted">{{ label }}</span>
    <strong class="min-w-0 overflow-wrap-anywhere text-right">
      <slot />
    </strong>
  </div>
</template>