<script setup lang="ts" generic="T extends string | number">
defineProps<{
  options: Array<{
    value: T;
    label: string;
  }>;
  modelValue: T;
  ariaLabel?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: T];
}>();
</script>

<template>
  <div class="flex gap-2" :aria-label="ariaLabel">
    <button
      v-for="option in options"
      :key="option.value"
      :class="[
        'border border-border-control rounded-radius-md bg-surface-control text-text-secondary cursor-pointer px-2.5 py-2 font-inherit font-extrabold',
        option.value === modelValue
          ? 'border-accent-soft-border bg-accent-soft-bg text-accent-text'
          : '',
      ]"
      type="button"
      @click="emit('update:modelValue', option.value)"
    >
      {{ option.label }}
    </button>
  </div>
</template>