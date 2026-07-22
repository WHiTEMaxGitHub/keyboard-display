<script setup lang="ts">
defineProps<{
  checked: boolean;
  compact?: boolean;
}>();

const emit = defineEmits<{
  change: [event: Event];
}>();
</script>

<template>
  <label :class="['toggle-row', { compact }]">
    <input
      :checked="checked"
      type="checkbox"
      @change="emit('change', $event)"
    />
    <slot />
  </label>
</template>

<style scoped>
.toggle-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 0 0 16px;
  color: #c9d1da;
  font-weight: 700;
}

.toggle-row input {
  width: 18px;
  height: 18px;
  accent-color: #25d366;
}

.toggle-row.compact {
  margin: 0;
}
</style>
