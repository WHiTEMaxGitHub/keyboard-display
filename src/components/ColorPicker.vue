<script setup lang="ts">
import {
  computed,
  nextTick,
  onMounted,
  onUnmounted,
  ref,
  watch,
  type CSSProperties,
} from "vue";
import { useI18n } from "vue-i18n";
import { placeColorPopover } from "../domain/colorPopover";
import {
  hexToCssColor,
  hexToRgb,
  normalizeHexColor,
  rgbToHex,
  type RgbColor,
} from "../domain/colorPicker";

const POPOVER_GAP = 8;
const POPOVER_MARGIN = 16;
const POPOVER_MAX_HEIGHT = 460;
const POPOVER_MIN_WIDTH = 420;
const POPOVER_PREFERRED_WIDTH = 460;

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

const props = withDefaults(
  defineProps<{
    label: string;
    value: string;
    recentColors?: string[];
    alphaEnabled?: boolean;
  }>(),
  {
    recentColors: () => [],
    alphaEnabled: false,
  },
);

const emit = defineEmits<{
  "preview:value": [value: string];
  "update:value": [value: string];
  "remember-color": [value: string];
}>();

const pickerOpen = ref(false);
const pickerRoot = ref<HTMLElement | null>(null);
const colorTrigger = ref<HTMLButtonElement | null>(null);
const pickerPanel = ref<HTMLElement | null>(null);
const popoverDirection = ref<"down" | "up">("down");
const popoverStyle = ref<CSSProperties>({});
const hexDraft = ref(normalizeHexColor(props.value));
const sessionStartColor = ref(normalizeHexColor(props.value));
const hasPendingCommit = ref(false);
const { t } = useI18n();
const pickerId = `color-picker-${Math.random().toString(36).slice(2)}`;

const rgb = computed(() => hexToRgb(hexDraft.value));
const previewColorCss = computed(() => hexToCssColor(hexDraft.value));
const previewRgbaLabel = computed(() => {
  const color = rgb.value;
  const alpha = color.a === undefined ? 1 : color.a / 255;
  return `rgba(${color.r}, ${color.g}, ${color.b}, ${alpha.toFixed(2)})`;
});

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

async function openPicker() {
  hexDraft.value = normalizeHexColor(props.value, hexDraft.value);
  sessionStartColor.value = normalizePickerColor(hexDraft.value);
  hasPendingCommit.value = false;
  pickerOpen.value = true;
  updatePopoverPosition();
  await nextTick();
  updatePopoverPosition();
}

function closePicker() {
  if (pickerOpen.value) {
    commitCurrentColor(false);
  }
  pickerOpen.value = false;
}

function handleDocumentPointerDown(event: PointerEvent) {
  if (!pickerOpen.value) {
    return;
  }

  const target = event.target as Node;
  const root = pickerRoot.value;
  const panel = pickerPanel.value;

  if (root?.contains(target) || panel?.contains(target)) {
    return;
  }

  if (root || panel) {
    closePicker();
  }
}

function handleViewportChange() {
  if (pickerOpen.value) {
    updatePopoverPosition();
  }
}

onMounted(() => {
  document.addEventListener("pointerdown", handleDocumentPointerDown, true);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", handleDocumentPointerDown, true);
  stopPopoverPositionTracking();
});

watch(pickerOpen, (open) => {
  if (open) {
    startPopoverPositionTracking();
    return;
  }

  stopPopoverPositionTracking();
});

function startPopoverPositionTracking() {
  window.addEventListener("resize", handleViewportChange);
  document.addEventListener("scroll", handleViewportChange, true);
}

function stopPopoverPositionTracking() {
  window.removeEventListener("resize", handleViewportChange);
  document.removeEventListener("scroll", handleViewportChange, true);
}

function updatePopoverPosition() {
  const trigger = colorTrigger.value;
  if (!trigger) {
    popoverDirection.value = "down";
    return;
  }

  const rect = trigger.getBoundingClientRect();
  const placement = placeColorPopover(
    {
      left: rect.left,
      right: rect.right,
      top: rect.top,
      bottom: rect.bottom,
      width: rect.width,
    },
    {
      width: window.innerWidth,
      height: window.innerHeight,
    },
    {
      gap: POPOVER_GAP,
      margin: POPOVER_MARGIN,
      maxHeight: POPOVER_MAX_HEIGHT,
      minWidth: POPOVER_MIN_WIDTH,
      preferredWidth: POPOVER_PREFERRED_WIDTH,
      panelHeight: pickerPanel.value?.offsetHeight,
    },
  );

  popoverDirection.value = placement.direction;
  popoverStyle.value = {
    left: `${placement.left}px`,
    top: `${placement.top}px`,
    width: `${placement.width}px`,
    maxHeight: `${placement.maxHeight}px`,
  };
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
  hasPendingCommit.value = true;
  emit("preview:value", normalizedColor);
}

function commitCurrentColor(force: boolean) {
  const normalizedColor = normalizePickerColor(hexDraft.value, normalizeHexColor(props.value));
  hexDraft.value = normalizedColor;

  if (force || (hasPendingCommit.value && normalizedColor !== sessionStartColor.value)) {
    emit("update:value", normalizedColor);
    emit("remember-color", normalizedColor);
  }

  sessionStartColor.value = normalizedColor;
  hasPendingCommit.value = false;
}

function applyCurrentColor() {
  commitCurrentColor(true);
  pickerOpen.value = false;
}

function normalizePickerColor(color: string, fallback = normalizeHexColor(props.value)) {
  const normalizedColor = normalizeHexColor(color, fallback);
  return !props.alphaEnabled && normalizedColor.length === 9
    ? normalizedColor.slice(0, 7)
    : normalizedColor;
}
</script>

<template>
  <div ref="pickerRoot" :class="['relative min-w-0 z-1', pickerOpen && 'z-20']">
    <button
      ref="colorTrigger"
      class="color-trigger"
      type="button"
      aria-haspopup="dialog"
      :aria-controls="pickerOpen ? pickerId : undefined"
      :aria-expanded="pickerOpen"
      @click="togglePicker"
    >
      <span class="color-swatch color-checkerboard">
        <span :style="{ backgroundColor: hexToCssColor(value) }" />
      </span>
      <span>{{ label }}</span>
      <strong>{{ normalizeHexColor(value) }}</strong>
    </button>
    <Teleport to="body">
      <Transition name="picker-popover">
        <div
          v-if="pickerOpen"
          :id="pickerId"
          ref="pickerPanel"
          :class="['picker-panel', `picker-panel-${popoverDirection}`]"
          :style="popoverStyle"
          role="dialog"
          :aria-label="label"
          @keydown.esc.prevent.stop="closePicker"
        >
          <div class="picker-layout">
            <div class="picker-main">
              <label class="hex-row">
                <span>{{ t("colorPicker.hex") }}</span>
                <input
                  :value="hexDraft"
                  spellcheck="false"
                  @blur="commitHex"
                  @change="commitHex"
                  @input="updateHex"
                />
              </label>
              <div class="current-color-preview">
                <span class="preview-swatch color-checkerboard" aria-hidden="true">
                  <span :style="{ backgroundColor: previewColorCss }" />
                </span>
                <div>
                  <strong>{{ normalizeHexColor(hexDraft) }}</strong>
                  <span>{{ previewRgbaLabel }}</span>
                </div>
              </div>
            </div>
            <div class="picker-controls">
              <div class="slider-list">
                <label>
                  <span>{{ t("colorPicker.red") }}</span>
                  <input :value="rgb.r" min="0" max="255" type="range" @input="updateChannel('r', $event)" />
                </label>
                <label>
                  <span>{{ t("colorPicker.green") }}</span>
                  <input :value="rgb.g" min="0" max="255" type="range" @input="updateChannel('g', $event)" />
                </label>
                <label>
                  <span>{{ t("colorPicker.blue") }}</span>
                  <input :value="rgb.b" min="0" max="255" type="range" @input="updateChannel('b', $event)" />
                </label>
                <label v-if="alphaEnabled">
                  <span>{{ t("colorPicker.alpha") }}</span>
                  <input :value="rgb.a ?? 255" min="0" max="255" type="range" @input="updateChannel('a', $event)" />
                </label>
              </div>
              <div class="swatch-section">
                <span>{{ t("colorPicker.presets") }}</span>
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
                <span>{{ t("colorPicker.recent") }}</span>
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
            </div>
          </div>
          <div class="flex justify-end">
            <button class="apply-button" type="button" @click="applyCurrentColor">
              {{ t("colorPicker.apply") }}
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.color-trigger {
  display: grid;
  grid-template-columns: 24px minmax(68px, 1fr) auto;
  align-items: center;
  gap: 8px;
  width: 100%;
  min-height: 38px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: var(--color-surface-control);
  color: var(--color-text-body);
  cursor: pointer;
  padding: 7px 9px;
  text-align: left;
}

.color-trigger:hover {
  border-color: rgba(255, 255, 255, 0.16);
  background: var(--color-surface-control-hover);
}

.color-trigger span,
.swatch-section span,
.hex-row span,
.slider-list span {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 800;
}

.color-trigger strong {
  color: var(--color-accent-text);
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

.color-checkerboard {
  position: relative;
  overflow: hidden;
  background:
    linear-gradient(45deg, rgba(255, 255, 255, 0.18) 25%, transparent 25%),
    linear-gradient(-45deg, rgba(255, 255, 255, 0.18) 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, rgba(255, 255, 255, 0.18) 75%),
    linear-gradient(-45deg, transparent 75%, rgba(255, 255, 255, 0.18) 75%),
    #1d232b;
  background-position:
    0 0,
    0 5px,
    5px -5px,
    -5px 0;
  background-size: 10px 10px;
}

.color-checkerboard > span {
  position: absolute;
  inset: 0;
}

.picker-panel {
  position: fixed;
  z-index: 1000;
  display: grid;
  gap: 12px;
  box-sizing: border-box;
  overflow-x: hidden;
  overflow-y: auto;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  box-shadow: 0 18px 42px rgba(0, 0, 0, 0.34);
  padding: 12px;
}

.picker-layout {
  display: grid;
  grid-template-columns: minmax(150px, 0.86fr) minmax(210px, 1.14fr);
  gap: 14px;
  min-width: 0;
}

.picker-main,
.picker-controls {
  display: grid;
  align-content: start;
  gap: 12px;
  min-width: 0;
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
  color: var(--color-text-body);
  font: inherit;
  padding: 8px 10px;
}

.current-color-preview {
  display: grid;
  grid-template-columns: 64px minmax(0, 1fr);
  align-items: center;
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.035);
  padding: 8px;
}

.preview-swatch {
  width: 64px;
  height: 42px;
  border: 1px solid rgba(255, 255, 255, 0.18);
  border-radius: 7px;
  box-shadow:
    inset 0 0 0 1px rgba(0, 0, 0, 0.16),
    0 8px 18px rgba(0, 0, 0, 0.20);
}

.current-color-preview div {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.current-color-preview strong {
  overflow: hidden;
  color: var(--color-text-primary);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.current-color-preview div span {
  overflow: hidden;
  color: var(--color-text-muted);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.slider-list {
  display: grid;
  gap: 8px;
}

.slider-list label {
  grid-template-columns: 18px minmax(160px, 1fr);
  align-items: center;
}

.slider-list input {
  accent-color: var(--color-accent);
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

.apply-button {
  min-height: 32px;
  border: 1px solid var(--color-accent-border);
  border-radius: 7px;
  background: rgba(37, 211, 102, 0.12);
  color: var(--color-accent-text);
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

@media (max-width: 520px) {
  .picker-layout {
    grid-template-columns: 1fr;
  }

  .slider-list label {
    grid-template-columns: 18px minmax(0, 1fr);
  }
}
</style>
