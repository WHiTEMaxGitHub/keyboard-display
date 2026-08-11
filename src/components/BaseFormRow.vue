<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    description?: string;
    stacked?: boolean;
  }>(),
  {
    description: "",
    stacked: false,
  },
);
</script>

<template>
  <div
    :class="[
      'grid gap-3 mb-4 text-text-secondary font-bold',
      stacked
        ? 'grid-cols-1'
        : 'grid-cols-[minmax(120px,1fr)_minmax(180px,240px)] items-center',
    ]"
  >
    <span class="min-w-0">
      <span class="block">{{ label }}</span>
      <span
        v-if="description"
        class="block mt-1 text-text-subtle text-xs font-medium"
      >
        {{ description }}
      </span>
    </span>
    <span :class="['min-w-0', !stacked && 'justify-self-end w-[min(240px,100%)]']">
      <slot />
    </span>
  </div>
</template>
