<script setup lang="ts">
defineProps<{
  label: string;
}>();
</script>

<template>
  <div class="grid grid-cols-[minmax(90px,auto)_minmax(0,1fr)] items-center gap-3 py-2.5 border-b border-border-dim">
    <span class="min-w-0 text-text-muted">{{ label }}</span>
    <strong class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-right">
      <slot />
    </strong>
  </div>
</template>
