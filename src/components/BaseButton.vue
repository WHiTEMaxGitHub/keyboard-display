<script setup lang="ts">
withDefaults(
  defineProps<{
    type?: "button" | "submit";
    variant?: "primary" | "secondary" | "danger";
    disabled?: boolean;
  }>(),
  {
    type: "button",
    variant: "secondary",
    disabled: false,
  },
);
</script>

<template>
  <button
    :type="type"
    :class="['base-button', variant]"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<style scoped>
.base-button {
  min-height: 34px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  cursor: pointer;
  font: inherit;
  font-weight: 800;
  padding: 0 12px;
  transition:
    background-color 140ms ease,
    border-color 140ms ease,
    color 140ms ease,
    opacity 140ms ease;
}

.base-button:hover:not(:disabled) {
  background: #29313d;
}

.base-button.primary {
  border-color: rgba(37, 211, 102, 0.42);
  background: #123421;
  color: #eafff0;
}

.base-button.primary:hover:not(:disabled) {
  background: #18452c;
}

.base-button.danger {
  border-color: rgba(255, 143, 143, 0.28);
  background: #2a171a;
  color: #ffd9d9;
}

.base-button.danger:hover:not(:disabled) {
  background: #3a1e22;
}

.base-button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
</style>
