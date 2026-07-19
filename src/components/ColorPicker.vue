<script setup lang="ts">
import { computed, ref, watch } from "vue";
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
}>();

const emit = defineEmits<{
  "update:value": [value: string];
  "remember-color": [value: string];
}>();

const pickerOpen = ref(false);
const hexDraft = ref(normalizeHexColor(props.value));

const rgb = computed(() => hexToRgb(hexDraft.value));

watch(
  () => props.value,
  (value) => {
    hexDraft.value = normalizeHexColor(value, hexDraft.value);
  },
);

function togglePicker() {
  pickerOpen.value = !pickerOpen.value;
}

function updateHex(event: Event) {
  hexDraft.value = (event.target as HTMLInputElement).value;
}

function commitHex() {
  commitColor(normalizeHexColor(hexDraft.value, normalizeHexColor(props.value)));
}

function updateChannel(channel: keyof RgbColor, event: Event) {
  commitColor(
    rgbToHex({
      ...rgb.value,
      [channel]: Number((event.target as HTMLInputElement).value),
    }),
  );
}

function chooseColor(color: string) {
  commitColor(color);
}

function commitColor(color: string) {
  const normalizedColor = normalizeHexColor(color, normalizeHexColor(props.value));
  hexDraft.value = normalizedColor;
  emit("update:value", normalizedColor);
  emit("remember-color", normalizedColor);
}
</script>

<template>
  <div class="color-picker">
    <button
      class="color-trigger"
      type="button"
      :aria-expanded="pickerOpen"
      @click="togglePicker"
    >
      <span class="color-swatch" :style="{ backgroundColor: normalizeHexColor(value) }" />
      <span>{{ label }}</span>
      <strong>{{ normalizeHexColor(value) }}</strong>
    </button>
    <div v-if="pickerOpen" class="picker-panel">
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
    </div>
  </div>
</template>

<style scoped>
.color-picker {
  position: relative;
  min-width: 0;
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
  display: grid;
  gap: 12px;
  margin-top: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  padding: 12px;
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
</style>
