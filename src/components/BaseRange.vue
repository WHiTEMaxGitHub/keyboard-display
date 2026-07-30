<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
    valueLabel?: string;
    disabled?: boolean;
  }>(),
  {
    min: 0,
    max: 100,
    step: 1,
    disabled: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: number];
  input: [event: Event];
}>();
</script>

<template>
  <label class="grid gap-2 mb-4 text-text-secondary font-bold">
    <span v-if="label || valueLabel" class="flex items-baseline justify-between gap-3">
      <span v-if="label">{{ label }}</span>
      <strong v-if="valueLabel" class="text-text-muted text-xs font-extrabold">{{ valueLabel }}</strong>
    </span>
    <input
      :value="modelValue"
      :min="min"
      :max="max"
      :step="step"
      :disabled="disabled"
      type="range"
      class="w-full"
      style="accent-color: var(--color-accent)"
      @input="emit('update:modelValue', Number(($event.target as HTMLInputElement).value)); emit('input', $event)"
    />
    <slot />
  </label>
</template>