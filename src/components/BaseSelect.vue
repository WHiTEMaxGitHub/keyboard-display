<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

type SelectOption = {
  value: string;
  label: string;
};

const props = defineProps<{
  modelValue: string;
  options: SelectOption[];
  disabled?: boolean;
  compact?: boolean;
  ariaLabel?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const rootRef = ref<HTMLElement | null>(null);
const open = ref(false);

const selectedOption = computed(() =>
  props.options.find((option) => option.value === props.modelValue),
);

function toggleOpen() {
  if (props.disabled) {
    return;
  }

  open.value = !open.value;
}

function close() {
  open.value = false;
}

function choose(value: string) {
  emit("update:modelValue", value);
  close();
}

function handleDocumentPointerDown(event: PointerEvent) {
  if (!rootRef.value?.contains(event.target as Node)) {
    close();
  }
}

onMounted(() => {
  document.addEventListener("pointerdown", handleDocumentPointerDown, true);
});

onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", handleDocumentPointerDown, true);
});
</script>

<template>
  <div ref="rootRef" :class="['base-select-root', { open, disabled }]">
    <button
      :class="['base-select-trigger', { compact }]"
      type="button"
      :disabled="disabled"
      aria-haspopup="listbox"
      :aria-expanded="open"
      :aria-label="ariaLabel"
      @click="toggleOpen"
      @keydown.esc.prevent.stop="close"
    >
      <span>{{ selectedOption?.label ?? modelValue }}</span>
      <i aria-hidden="true"></i>
    </button>
    <Transition name="base-select-menu">
      <div v-if="open" class="base-select-menu" role="listbox">
        <button
          v-for="option in options"
          :key="option.value"
          :class="['base-select-option', { selected: option.value === modelValue }]"
          type="button"
          role="option"
          :aria-selected="option.value === modelValue"
          @click="choose(option.value)"
        >
          <span>{{ option.label }}</span>
          <strong v-if="option.value === modelValue">✓</strong>
        </button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.base-select-root {
  position: relative;
  min-width: 0;
}

.base-select-root.open {
  z-index: 80;
}

.base-select-trigger {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 18px;
  align-items: center;
  gap: 8px;
  width: 100%;
  min-height: 34px;
  border: 1px solid var(--color-border-control);
  border-radius: var(--radius-lg);
  background: var(--color-surface-control);
  color: var(--color-text-body);
  cursor: pointer;
  font: inherit;
  font-size: 14px;
  font-weight: 700;
  line-height: 1;
  padding: 0 10px;
  text-align: left;
}

.base-select-trigger:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--color-border-control) 70%, white 20%);
  background: var(--color-surface-control-hover);
}

.base-select-trigger:focus-visible {
  border-color: var(--color-accent-focus-border);
  outline: 2px solid var(--color-accent-focus-ring);
  outline-offset: 0;
}

.base-select-trigger:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.base-select-trigger span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.base-select-trigger i {
  justify-self: end;
  width: 0;
  height: 0;
  border-top: 6px solid var(--color-text-muted);
  border-right: 5px solid transparent;
  border-left: 5px solid transparent;
  transition: transform 140ms ease;
}

.base-select-root.open .base-select-trigger i {
  transform: rotate(180deg);
}

.base-select-menu {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  left: 0;
  display: grid;
  gap: 4px;
  min-width: 180px;
  max-height: 260px;
  overflow: auto;
  border: 1px solid color-mix(in srgb, var(--color-border-default) 80%, transparent);
  border-radius: var(--radius-lg);
  background:
    linear-gradient(
      145deg,
      color-mix(in srgb, var(--color-surface-base) 88%, var(--glass-from) 12%),
      color-mix(in srgb, var(--color-surface-base) 92%, var(--glass-to) 8%)
    );
  box-shadow:
    0 18px 42px rgba(0, 0, 0, 0.34),
    inset 0 1px 0 color-mix(in srgb, white 9%, transparent);
  padding: 6px;
  backdrop-filter: blur(22px) saturate(150%);
  -webkit-backdrop-filter: blur(22px) saturate(150%);
}

.base-select-option {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
  min-height: 30px;
  border: 0;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 13px;
  font-weight: 800;
  padding: 7px 9px;
  text-align: left;
}

.base-select-option:hover,
.base-select-option.selected {
  background: color-mix(in srgb, var(--color-accent) 18%, transparent);
  color: var(--color-text-primary);
}

.base-select-option span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.base-select-option strong {
  color: var(--color-accent-text);
  font-weight: 900;
}

.base-select-menu-enter-active,
.base-select-menu-leave-active {
  transition:
    opacity 150ms ease,
    transform 170ms cubic-bezier(0.16, 1, 0.3, 1);
}

.base-select-menu-enter-from,
.base-select-menu-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.98);
}
</style>
