<script setup lang="ts">
import { normalizeHexColor } from "../domain/colorPicker";
import type { AppConfig, OverlayStyle } from "../domain/defaultConfig";
import ColorPicker from "./ColorPicker.vue";

const props = defineProps<{
  config: AppConfig;
  recentColors: string[];
}>();

const emit = defineEmits<{
  "update-overlay-style": [style: OverlayStyle];
  "remember-color": [color: string];
  "refresh-pov": [];
}>();

function updateScale(event: Event) {
  const scale = Number((event.target as HTMLInputElement).value);
  emit("update-overlay-style", { ...props.config.style, scale });
}

function formatScale(scale: number) {
  return `${scale.toFixed(2)}x`;
}

function effectiveUnitPx() {
  return Math.round(props.config.layout.unitPx * props.config.style.scale);
}

function updateOpacity(event: Event) {
  const opacity = Number((event.target as HTMLInputElement).value);
  emit("update-overlay-style", { ...props.config.style, opacity });
}

function updateBackgroundRadius(event: Event) {
  const backgroundRadius = Number((event.target as HTMLInputElement).value);
  emit("update-overlay-style", {
    ...props.config.style,
    backgroundRadius,
  });
}

function updateIdleKeyVisibility(event: Event) {
  const idleKeyVisibility = (event.target as HTMLSelectElement)
    .value as OverlayStyle["idleKeyVisibility"];
  emit("update-overlay-style", { ...props.config.style, idleKeyVisibility });
}

function updateBackplateTransparent(event: Event) {
  const transparent = (event.target as HTMLInputElement).checked;
  emit("update-overlay-style", {
    ...props.config.style,
    backgroundColor: setHexAlpha(props.config.style.backgroundColor, transparent ? 0 : 255),
  });
}

function updateStyleColor(
  field:
    | "idleColor"
    | "activeColor"
    | "idleTextColor"
    | "activeTextColor"
    | "backgroundColor",
  color: string,
) {
  const nextColor = normalizeHexColor(color, props.config.style[field]);
  emit("update-overlay-style", {
    ...props.config.style,
    [field]: nextColor,
  });
}

function isBackplateTransparent() {
  const normalizedColor = normalizeHexColor(props.config.style.backgroundColor);
  return normalizedColor.length === 9 && normalizedColor.endsWith("00");
}

function setHexAlpha(color: string, alpha: number) {
  const normalizedColor = normalizeHexColor(color);
  const rgb = normalizedColor.slice(0, 7);
  return `${rgb}${Math.min(255, Math.max(0, Math.round(alpha)))
    .toString(16)
    .padStart(2, "0")}`;
}
</script>

<template>
  <article class="panel wide-panel">
    <div class="section-header">
      <h2>Appearance</h2>
      <button class="panel-action-button" type="button" @click="emit('refresh-pov')">
        Refresh POV
      </button>
    </div>
    <label class="range-control">
      <span class="range-label">
        <span>Scale</span>
        <strong>{{ formatScale(config.style.scale) }} · {{ effectiveUnitPx() }}px unit</strong>
      </span>
      <input
        :value="config.style.scale"
        min="0.75"
        max="1.5"
        step="0.05"
        type="range"
        @input="updateScale"
      />
    </label>
    <label>
      Overlay transparency
      <input
        :value="config.style.opacity"
        min="0.35"
        max="1"
        step="0.01"
        type="range"
        @input="updateOpacity"
      />
      <span class="hint">Controls the whole POV overlay opacity.</span>
    </label>
    <div class="appearance-control-grid">
      <label>
        Backplate radius
        <input
          :value="config.style.backgroundRadius"
          min="0"
          max="24"
          step="1"
          type="range"
          @input="updateBackgroundRadius"
        />
      </label>
    </div>
    <label class="settings-row">
      <span>Idle keys</span>
      <select
        class="select-control compact-select"
        :value="config.style.idleKeyVisibility"
        @change="updateIdleKeyVisibility"
      >
        <option value="visible">Visible</option>
        <option value="hidden">Hidden until pressed</option>
      </select>
    </label>
    <div class="color-grid" aria-label="Overlay colors">
      <ColorPicker
        label="Idle key"
        :value="config.style.idleColor"
        :recent-colors="recentColors"
        alpha-enabled
        @update:value="updateStyleColor('idleColor', $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <ColorPicker
        label="Pressed key"
        :value="config.style.activeColor"
        :recent-colors="recentColors"
        alpha-enabled
        @update:value="updateStyleColor('activeColor', $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <ColorPicker
        label="Idle text"
        :value="config.style.idleTextColor"
        :recent-colors="recentColors"
        alpha-enabled
        @update:value="updateStyleColor('idleTextColor', $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <ColorPicker
        label="Pressed text"
        :value="config.style.activeTextColor"
        :recent-colors="recentColors"
        alpha-enabled
        @update:value="updateStyleColor('activeTextColor', $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <ColorPicker
        label="Backplate"
        :value="config.style.backgroundColor"
        :recent-colors="recentColors"
        alpha-enabled
        @update:value="updateStyleColor('backgroundColor', $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <label class="backplate-transparent-toggle">
        <input
          :checked="isBackplateTransparent()"
          type="checkbox"
          @change="updateBackplateTransparent"
        />
        Transparent backplate
      </label>
    </div>
  </article>
</template>

<style scoped>
.panel {
  box-sizing: border-box;
  min-height: 190px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #171b22;
  padding: 18px;
}

.wide-panel {
  width: 100%;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}

.section-header h2 {
  margin: 0;
  font-size: 18px;
  letter-spacing: 0;
  line-height: 24px;
}

.panel-action-button {
  min-height: 34px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  cursor: pointer;
  font-weight: 700;
  padding: 0 10px;
}

.panel-action-button:hover {
  background: #29313d;
}

label {
  display: grid;
  gap: 8px;
  margin-bottom: 16px;
  color: #c9d1da;
  font-weight: 700;
}

.range-label {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
}

.range-label strong {
  color: #9ca7b4;
  font-size: 12px;
  font-weight: 800;
}

.settings-row {
  display: grid;
  grid-template-columns: minmax(120px, 1fr) minmax(180px, 240px);
  align-items: center;
  gap: 12px;
}

.settings-row span {
  min-width: 0;
}

.hint {
  color: #7f8b99;
  font-size: 12px;
  font-weight: 500;
}

input[type="range"] {
  accent-color: #25d366;
}

.appearance-control-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.backplate-transparent-toggle {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 38px;
  margin: 0;
  color: #c9d1da;
  font-size: 13px;
  font-weight: 700;
}

.backplate-transparent-toggle input {
  width: 18px;
  height: 18px;
  accent-color: #25d366;
}

select {
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
  font-size: 14px;
  font-weight: 700;
  line-height: 1;
  padding: 0 34px 0 10px;
}

.select-control {
  justify-self: end;
  width: min(240px, 100%);
}

.compact-select {
  min-height: 34px;
}

select:focus {
  border-color: rgba(37, 211, 102, 0.55);
  outline: 2px solid rgba(37, 211, 102, 0.14);
  outline-offset: 0;
}

.color-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
</style>
