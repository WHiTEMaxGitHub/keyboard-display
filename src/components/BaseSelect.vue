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
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  appearance: none;
  background: #202630;
  background-image:
    linear-gradient(45deg, transparent 50%, #9ca7b4 50%),
    linear-gradient(135deg, #9ca7b4 50%, transparent 50%);
  background-position:
    calc(100% - 17px) 15px,
    calc(100% - 11px) 15px;
  background-repeat: no-repeat;
  background-size:
    6px 6px,
    6px 6px;
  color: #dfe5ec;
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
  border-color: rgba(37, 211, 102, 0.55);
  outline: 2px solid rgba(37, 211, 102, 0.14);
  outline-offset: 0;
}
</style>
