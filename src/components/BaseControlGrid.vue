<script setup lang="ts">
withDefaults(
  defineProps<{
    ariaLabel?: string;
    minColumnWidth?: string;
  }>(),
  {
    ariaLabel: "",
    minColumnWidth: "220px",
  },
);
</script>

<template>
  <div
    class="grid gap-2.5"
    :aria-label="ariaLabel || undefined"
    :style="{ gridTemplateColumns: `repeat(auto-fit, minmax(min(100%, ${minColumnWidth}), 1fr))` }"
  >
    <slot />
  </div>
</template>
