<script setup lang="ts">
defineProps<{
  modelValue: string;
  disabled?: boolean;
  compact?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  change: [event: Event];
}>();

function handleChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  emit("update:modelValue", value);
  emit("change", event);
}
</script>

<template>
  <select
    :class="['base-select', { compact }]"
    :value="modelValue"
    :disabled="disabled"
    @change="handleChange"
  >
    <slot />
  </select>
</template>

<style scoped>
.base-select {
  min-height: 34px;
  border: 1px solid var(--color-border-control);
  border-radius: var(--radius-lg);
  appearance: none;
  background: var(--color-surface-control);
  background-image:
    linear-gradient(45deg, transparent 50%, var(--color-text-muted) 50%),
    linear-gradient(135deg, var(--color-text-muted) 50%, transparent 50%);
  background-position:
    calc(100% - 17px) 15px,
    calc(100% - 11px) 15px;
  background-repeat: no-repeat;
  background-size:
    6px 6px,
    6px 6px;
  color: var(--color-text-body);
  cursor: pointer;
  font: inherit;
  font-size: 14px;
  font-weight: 700;
  line-height: 1;
  padding: 0 34px 0 10px;
}

.base-select.compact {
  min-height: 34px;
}

.base-select:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.base-select:focus {
  border-color: var(--color-accent-focus-border);
  outline: 2px solid var(--color-accent-focus-ring);
  outline-offset: 0;
}
</style>