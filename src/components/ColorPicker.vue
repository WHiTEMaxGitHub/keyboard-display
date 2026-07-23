<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import {
  hexToRgb,
  normalizeHexColor,
  rgbToHex,
  type RgbColor,
} from "../domain/colorPicker";

const PRESET_COLORS = [
  "#25d366",
  "#ffffff",
  "#dfe5ec",
  "#10141a",
  "#ff5f5f",
  "#ffd166",
  "#4dabf7",
  "#b197fc",
];

const props = defineProps<{
  label: string;
  value: string;
  recentColors: string[];
  alphaEnabled?: boolean;
}>();

const emit = defineEmits<{
  "update:value": [value: string];
  "remember-color": [value: string];
}>();

const pickerOpen = ref(false);
const pickerRoot = ref<HTMLElement | null>(null);
const colorTrigger = ref<HTMLButtonElement | null>(null);
const popoverDirection = ref<"down" | "up">("down");
const hexDraft = ref(normalizeHexColor(props.value));
const sessionStartColor = ref(normalizeHexColor(props.value));
const hasPendingRecentColor = ref(false);

const rgb = computed(() => hexToRgb(hexDraft.value));

watch(
  () => props.value,
  (value) => {
    hexDraft.value = normalizeHexColor(value, hexDraft.value);
  },
);

function togglePicker() {
  if (pickerOpen.value) {
    closePicker();
    return;
  }

  openPicker();
}

function openPicker() {
  hexDraft.value = normalizeHexColor(props.value, hexDraft.value);
  sessionStartColor.value = normalizePickerColor(hexDraft.value);
  hasPendingRecentColor.value = false;
  updatePopoverDirection();
  pickerOpen.value = true;
}

function closePicker() {
  if (pickerOpen.value) {
    rememberCurrentColor(false);
  }
  pickerOpen.value = false;
}

function handleDocumentPointerDown(event: PointerEvent) {
  if (!pickerOpen.value) {
    return;
  }

  const root = pickerRoot.value;
  if (root && !root.contains(event.target as Node)) {
    closePicker();
  }
}

onMounted(() => {
  document.addEventListener("pointerdown", handleDocumentPointerDown, true);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", handleDocumentPointerDown, true);
});

function updatePopoverDirection() {
  const trigger = colorTrigger.value;
  if (!trigger) {
    popoverDirection.value = "down";
    return;
  }

  const rect = trigger.getBoundingClientRect();
  const spaceBelow = window.innerHeight - rect.bottom;
  const spaceAbove = rect.top;
  popoverDirection.value = spaceBelow < 300 && spaceAbove > spaceBelow ? "up" : "down";
}

function updateHex(event: Event) {
  hexDraft.value = (event.target as HTMLInputElement).value;
  const normalizedColor = normalizePickerColor(hexDraft.value, "");
  if (normalizedColor) {
    applyColor(normalizedColor);
  }
}

function commitHex() {
  previewColor(hexDraft.value);
}

function updateChannel(channel: keyof RgbColor, event: Event) {
  const nextRgb = {
    ...rgb.value,
    ...(props.alphaEnabled && rgb.value.a === undefined ? { a: 255 } : {}),
    [channel]: Number((event.target as HTMLInputElement).value),
  };

  previewColor(
    rgbToHex(props.alphaEnabled ? nextRgb : { r: nextRgb.r, g: nextRgb.g, b: nextRgb.b }),
  );
}

function chooseColor(color: string) {
  previewColor(color);
}

function previewColor(color: string) {
  const normalizedColor = normalizePickerColor(color, normalizeHexColor(props.value));
  applyColor(normalizedColor);
}

function applyColor(normalizedColor: string) {
  hexDraft.value = normalizedColor;
  emit("update:value", normalizedColor);
  hasPendingRecentColor.value = true;
}

function applyCurrentColor() {
  rememberCurrentColor(true);
  pickerOpen.value = false;
}

function rememberCurrentColor(force: boolean) {
  const normalizedColor = normalizePickerColor(hexDraft.value, normalizeHexColor(props.value));
  hexDraft.value = normalizedColor;
  emit("update:value", normalizedColor);

  if (force || (hasPendingRecentColor.value && normalizedColor !== sessionStartColor.value)) {
    emit("remember-color", normalizedColor);
  }

  sessionStartColor.value = normalizedColor;
  hasPendingRecentColor.value = false;
}

function normalizePickerColor(color: string, fallback = normalizeHexColor(props.value)) {
  const normalizedColor = normalizeHexColor(color, fallback);
  return !props.alphaEnabled && normalizedColor.length === 9
    ? normalizedColor.slice(0, 7)
    : normalizedColor;
}
</script>

<template>
  <div ref="pickerRoot" class="color-picker">
    <button
      ref="colorTrigger"
      class="color-trigger"
      type="button"
      :aria-expanded="pickerOpen"
      @click="togglePicker"
    >
      <span class="color-swatch" :style="{ backgroundColor: normalizeHexColor(value) }" />
      <span>{{ label }}</span>
      <strong>{{ normalizeHexColor(value) }}</strong>
    </button>
    <Transition name="picker-popover">
      <div
        v-if="pickerOpen"
        :class="['picker-panel', `picker-panel-${popoverDirection}`]"
      >
        <label class="hex-row">
          <span>HEX</span>
          <input
            :value="hexDraft"
            spellcheck="false"
            @blur="commitHex"
            @change="commitHex"
            @input="updateHex"
          />
        </label>
        <div class="slider-list">
          <label>
            <span>R</span>
            <input :value="rgb.r" min="0" max="255" type="range" @input="updateChannel('r', $event)" />
          </label>
          <label>
            <span>G</span>
            <input :value="rgb.g" min="0" max="255" type="range" @input="updateChannel('g', $event)" />
          </label>
        <label>
          <span>B</span>
          <input :value="rgb.b" min="0" max="255" type="range" @input="updateChannel('b', $event)" />
        </label>
        <label v-if="alphaEnabled">
          <span>A</span>
          <input :value="rgb.a ?? 255" min="0" max="255" type="range" @input="updateChannel('a', $event)" />
        </label>
      </div>
        <div class="swatch-section">
          <span>Presets</span>
          <div class="swatch-grid">
            <button
              v-for="color in PRESET_COLORS"
              :key="color"
              :aria-label="color"
              class="swatch-button"
              type="button"
              :style="{ backgroundColor: color }"
              @click="chooseColor(color)"
            />
          </div>
        </div>
        <div v-if="recentColors.length" class="swatch-section">
          <span>Recent</span>
          <div class="swatch-grid">
            <button
              v-for="color in recentColors"
              :key="color"
              :aria-label="color"
              class="swatch-button"
              type="button"
              :style="{ backgroundColor: color }"
              @click="chooseColor(color)"
            />
          </div>
        </div>
        <div class="picker-actions">
          <button class="apply-button" type="button" @click="applyCurrentColor">
            Apply
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.color-picker {
  position: relative;
  min-width: 0;
  z-index: 1;
}

.color-picker:has(.picker-panel) {
  z-index: 20;
}

.color-trigger {
  display: grid;
  grid-template-columns: 24px minmax(68px, 1fr) auto;
  align-items: center;
  gap: 8px;
  width: 100%;
  min-height: 38px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  cursor: pointer;
  padding: 7px 9px;
  text-align: left;
}

.color-trigger:hover {
  border-color: rgba(255, 255, 255, 0.16);
  background: #29313d;
}

.color-trigger span,
.swatch-section span,
.hex-row span,
.slider-list span {
  color: #9ca7b4;
  font-size: 12px;
  font-weight: 800;
}

.color-trigger strong {
  color: #eafff0;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.color-swatch,
.swatch-button {
  width: 22px;
  height: 22px;
  border: 1px solid rgba(255, 255, 255, 0.22);
  border-radius: 5px;
  box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.16);
}

.picker-panel {
  position: absolute;
  left: 0;
  z-index: 30;
  display: grid;
  gap: 12px;
  width: max(100%, 260px);
  max-height: min(360px, calc(100vh - 32px));
  overflow: auto;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  box-shadow: 0 18px 42px rgba(0, 0, 0, 0.34);
  padding: 12px;
}

.picker-panel-down {
  top: calc(100% + 8px);
}

.picker-panel-up {
  bottom: calc(100% + 8px);
}

.picker-popover-enter-active,
.picker-popover-leave-active {
  transition:
    opacity 150ms ease,
    transform 170ms cubic-bezier(0.16, 1, 0.3, 1);
}

.picker-popover-enter-from,
.picker-popover-leave-to {
  opacity: 0;
  transform: translateY(var(--popover-enter-y, -6px)) scale(0.98);
}

.picker-popover-enter-to,
.picker-popover-leave-from {
  opacity: 1;
  transform: translateY(0) scale(1);
}

.picker-panel-down {
  --popover-enter-y: -6px;
}

.picker-panel-up {
  --popover-enter-y: 6px;
}

.hex-row,
.slider-list label {
  display: grid;
  gap: 6px;
}

.hex-row input {
  width: 100%;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #10141a;
  color: #dfe5ec;
  font: inherit;
  padding: 8px 10px;
}

.slider-list {
  display: grid;
  gap: 8px;
}

.slider-list label {
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: center;
}

.slider-list input {
  accent-color: #25d366;
}

.swatch-section {
  display: grid;
  gap: 7px;
}

.swatch-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.swatch-button {
  cursor: pointer;
  padding: 0;
}

.swatch-button:hover {
  outline: 2px solid rgba(255, 255, 255, 0.22);
  outline-offset: 2px;
}

.picker-actions {
  display: flex;
  justify-content: flex-end;
}

.apply-button {
  min-height: 32px;
  border: 1px solid rgba(37, 211, 102, 0.42);
  border-radius: 7px;
  background: rgba(37, 211, 102, 0.12);
  color: #eafff0;
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  font-weight: 800;
  padding: 0 12px;
}

.apply-button:hover {
  border-color: rgba(37, 211, 102, 0.7);
  background: rgba(37, 211, 102, 0.18);
}
</style>
