<script setup lang="ts">
defineProps<{
  wide?: boolean;
}>();
</script>

<template>
  <article :class="['box-border min-h-[190px] border border-border-default rounded-radius-lg bg-surface-panel p-[18px]', wide ? 'w-full' : 'w-full']">
    <slot />
  </article>
</template>