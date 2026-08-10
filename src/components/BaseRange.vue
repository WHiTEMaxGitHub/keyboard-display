<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
    valueLabel?: string;
    description?: string;
    disabled?: boolean;
    compact?: boolean;
  }>(),
  {
    min: 0,
    max: 100,
    step: 1,
    description: "",
    disabled: false,
    compact: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: number];
  input: [event: Event];
}>();
</script>

<template>
  <label :class="['grid gap-2 text-text-secondary font-bold', compact ? 'mb-0' : 'mb-4']">
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
    <span v-if="description" class="text-text-subtle text-xs font-medium">{{ description }}</span>
    <slot />
  </label>
</template>
