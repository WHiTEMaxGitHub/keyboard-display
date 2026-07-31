<script setup lang="ts">
defineProps<{
  wide?: boolean;
}>();
</script>

<template>
  <article :class="['box-border min-h-[190px] border border-[var(--glass-border)] rounded-[28px] bg-gradient-to-br from-[var(--glass-from)] to-[var(--glass-to)] backdrop-blur-2xl backdrop-saturate-[170%] shadow-[var(--glass-shadow)] p-[18px] transition-[border-color,box-shadow] duration-300 hover:border-[color-mix(in_srgb,var(--glass-border),white_10%)] hover:shadow-[var(--glass-shadow-hover)]', wide ? 'w-full' : 'w-full']">
    <slot />
  </article>
</template>