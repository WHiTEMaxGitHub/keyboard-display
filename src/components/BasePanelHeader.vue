<script setup lang="ts">
defineProps<{
  eyebrow?: string;
  title: string;
}>();
</script>

<template>
  <div class="flex items-start justify-between gap-3 mb-4">
    <div class="min-w-0">
      <p
        v-if="eyebrow"
        class="m-0 mb-1 text-text-muted text-xs font-extrabold tracking-[0.08em] uppercase"
      >
        {{ eyebrow }}
      </p>
      <h2 class="m-0 text-lg leading-6 tracking-normal text-text-primary">
        {{ title }}
      </h2>
    </div>
    <div v-if="$slots.actions" class="flex shrink-0 items-center gap-2">
      <slot name="actions" />
    </div>
  </div>
</template>
