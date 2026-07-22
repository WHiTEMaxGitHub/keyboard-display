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
  <div class="segmented-control" :aria-label="ariaLabel">
    <button
      v-for="option in options"
      :key="option.value"
      :class="{ selected: option.value === modelValue }"
      type="button"
      @click="emit('update:modelValue', option.value)"
    >
      {{ option.label }}
    </button>
  </div>
</template>

<style scoped>
.segmented-control {
  display: flex;
  gap: 8px;
}

.segmented-control button {
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #c9d1da;
  cursor: pointer;
  padding: 8px 10px;
  font: inherit;
  font-weight: 800;
}

.segmented-control button:hover,
.segmented-control button.selected {
  border-color: rgba(37, 211, 102, 0.52);
  background: rgba(37, 211, 102, 0.14);
  color: #eafff0;
}
</style>
