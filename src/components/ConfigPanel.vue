<script setup lang="ts">
import {
  Clapperboard,
  Keyboard,
  MonitorUp,
  Palette,
  SlidersHorizontal,
  Video,
} from "@lucide/vue";
import type { AppConfig, OverlayStyle } from "../domain/defaultConfig";
import PovOverlay from "./PovOverlay.vue";

const props = defineProps<{
  config: AppConfig;
  activeKeys: Set<string>;
  overlayVisible: boolean;
  profileName: string;
}>();

const emit = defineEmits<{
  "update-overlay-style": [style: OverlayStyle];
  "update-overlay-visible": [visible: boolean];
  "load-config": [text: string, fileName: string];
  "save-and-apply-config": [];
  "move-overlay": [
    position: "top-left" | "top-right" | "bottom-left" | "bottom-right" | "center",
  ];
}>();

function updateScale(event: Event) {
  const scale = Number((event.target as HTMLInputElement).value);
  emit("update-overlay-style", { ...props.config.style, scale });
}

function updateOpacity(event: Event) {
  const opacity = Number((event.target as HTMLInputElement).value);
  emit("update-overlay-style", { ...props.config.style, opacity });
}

function updateBackgroundMode(event: Event) {
  const backgroundMode = (event.target as HTMLSelectElement).value as OverlayStyle["backgroundMode"];
  emit("update-overlay-style", { ...props.config.style, backgroundMode });
}

function updateIdleKeyVisibility(event: Event) {
  const idleKeyVisibility = (event.target as HTMLSelectElement)
    .value as OverlayStyle["idleKeyVisibility"];
  emit("update-overlay-style", { ...props.config.style, idleKeyVisibility });
}

function updateStyleColor(
  field:
    | "idleColor"
    | "activeColor"
    | "idleTextColor"
    | "activeTextColor"
    | "backgroundColor",
  event: Event,
) {
  emit("update-overlay-style", {
    ...props.config.style,
    [field]: (event.target as HTMLInputElement).value,
  });
}

function updateAlwaysOnTop(event: Event) {
  const alwaysOnTop = (event.target as HTMLInputElement).checked;
  emit("update-overlay-style", { ...props.config.style, alwaysOnTop });
}

function updateOverlayVisible(event: Event) {
  emit("update-overlay-visible", (event.target as HTMLInputElement).checked);
}

function moveOverlay(
  position: "top-left" | "top-right" | "bottom-left" | "bottom-right" | "center",
) {
  emit("move-overlay", position);
}

async function loadConfigFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];

  if (!file) {
    return;
  }

  emit("load-config", await file.text(), file.name);
  input.value = "";
}

function saveAndApplyConfig() {
  emit("save-and-apply-config");
}
</script>

<template>
  <main class="config-shell">
    <aside class="sidebar" aria-label="Workspace navigation">
      <div class="brand">
        <Keyboard :size="22" aria-hidden="true" />
        <div>
          <strong>Keyboard Display</strong>
          <span>Desktop POV overlay</span>
        </div>
      </div>

      <nav class="nav-list">
        <a href="#layout"><MonitorUp :size="18" /> Layout</a>
        <a href="#style"><Palette :size="18" /> Style</a>
        <a href="#recording"><Clapperboard :size="18" /> Recording</a>
        <a href="#export"><Video :size="18" /> Export</a>
      </nav>
    </aside>

    <section class="workspace">
      <header class="topbar">
        <div>
          <p>Configuration</p>
          <h1>POV overlay control panel</h1>
        </div>
        <div class="topbar-actions">
          <label class="load-config-button">
            <SlidersHorizontal :size="15" aria-hidden="true" />
            Load config
            <input accept="application/json,.json" type="file" @change="loadConfigFile" />
          </label>
          <button class="save-apply-button" type="button" @click="saveAndApplyConfig">
            Save & Apply
          </button>
        </div>
      </header>

      <section class="preview-band" aria-label="Live preview">
        <div class="preview-copy">
          <p>Live Preview</p>
          <h2>CS movement layout</h2>
        </div>
        <PovOverlay
          :layout="config.layout"
          :keys="config.keys"
          :active-keys="activeKeys"
          :overlay-style="config.style"
        />
      </section>

      <section class="panel-grid">
        <article id="layout" class="panel">
          <h2>Layout</h2>
          <div class="field-row">
            <span>Profile</span>
            <strong>{{ profileName }}</strong>
          </div>
          <div class="field-row">
            <span>Visible keys</span>
            <strong>{{ config.keys.length }}</strong>
          </div>
          <div class="key-list">
            <span v-for="key in config.keys" :key="key.id">{{ key.label }}</span>
          </div>
        </article>

        <article id="style" class="panel">
          <h2>Style</h2>
          <label class="toggle-row">
            <input
              :checked="overlayVisible"
              type="checkbox"
              @change="updateOverlayVisible"
            />
            Show POV overlay
          </label>
          <label class="toggle-row">
            <input
              :checked="config.style.alwaysOnTop"
              type="checkbox"
              @change="updateAlwaysOnTop"
            />
            Always on top
          </label>
          <label>
            Scale
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
          <label>
            Background
            <select :value="config.style.backgroundMode" @change="updateBackgroundMode">
              <option value="transparent">Transparent</option>
              <option value="panel">Rounded panel</option>
            </select>
          </label>
          <label>
            Idle keys
            <select :value="config.style.idleKeyVisibility" @change="updateIdleKeyVisibility">
              <option value="visible">Visible</option>
              <option value="faint">Faint</option>
              <option value="hidden">Hidden until pressed</option>
            </select>
          </label>
          <div class="position-control">
            <span>Position</span>
            <div class="position-grid">
              <button type="button" @click="moveOverlay('top-left')">Top left</button>
              <button type="button" @click="moveOverlay('top-right')">Top right</button>
              <button type="button" @click="moveOverlay('center')">Center</button>
              <button type="button" @click="moveOverlay('bottom-left')">Bottom left</button>
              <button type="button" @click="moveOverlay('bottom-right')">Bottom right</button>
            </div>
          </div>
          <div class="color-grid" aria-label="Overlay colors">
            <label class="color-picker">
              <span>Idle key</span>
              <input
                :value="config.style.idleColor"
                type="color"
                @input="updateStyleColor('idleColor', $event)"
              />
            </label>
            <label class="color-picker">
              <span>Pressed key</span>
              <input
                :value="config.style.activeColor"
                type="color"
                @input="updateStyleColor('activeColor', $event)"
              />
            </label>
            <label class="color-picker">
              <span>Idle text</span>
              <input
                :value="config.style.idleTextColor"
                type="color"
                @input="updateStyleColor('idleTextColor', $event)"
              />
            </label>
            <label class="color-picker">
              <span>Pressed text</span>
              <input
                :value="config.style.activeTextColor"
                type="color"
                @input="updateStyleColor('activeTextColor', $event)"
              />
            </label>
            <label class="color-picker">
              <span>Backplate</span>
              <input
                :value="config.style.backgroundColor"
                type="color"
                @input="updateStyleColor('backgroundColor', $event)"
              />
            </label>
          </div>
        </article>

        <article id="recording" class="panel">
          <h2>Recording</h2>
          <div class="segmented" aria-label="Capture frame rate">
            <button
              v-for="fps in config.recording.fpsOptions"
              :key="fps"
              :class="{ selected: fps === config.recording.defaultFps }"
              type="button"
            >
              {{ fps }}fps
            </button>
          </div>
          <div class="field-row">
            <span>Primary artifact</span>
            <strong>{{ config.recording.formatExtension }}</strong>
          </div>
          <p class="quiet">
            Input frames are stored as compact binary state, then replayed for
            rendering or export.
          </p>
        </article>

        <article id="export" class="panel">
          <h2>Export</h2>
          <div class="field-row">
            <span>Transparent overlay</span>
            <strong>WebM</strong>
          </div>
          <div class="field-row">
            <span>Compatible video</span>
            <strong>MP4</strong>
          </div>
          <p class="quiet">
            Video is generated from the input timeline, so size and format can
            be tuned after recording.
          </p>
        </article>
      </section>
    </section>
  </main>
</template>

<style scoped>
.config-shell {
  display: grid;
  height: 100vh;
  grid-template-columns: 248px minmax(0, 1fr);
  overflow: hidden;
  background: #111316;
  color: #eef2f6;
}

.sidebar {
  height: 100vh;
  overflow-y: auto;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  background: #171a1f;
  padding: 22px 18px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 28px;
}

.brand span,
.topbar p,
.preview-copy p,
.quiet,
.field-row span {
  color: #9ca7b4;
}

.brand strong,
.brand span {
  display: block;
}

.brand span {
  margin-top: 2px;
  font-size: 12px;
}

.nav-list {
  display: grid;
  gap: 6px;
}

.nav-list a {
  display: flex;
  align-items: center;
  gap: 10px;
  border-radius: 7px;
  color: #c9d1da;
  padding: 10px 11px;
  text-decoration: none;
}

.nav-list a:hover {
  background: rgba(255, 255, 255, 0.06);
}

.workspace {
  height: 100vh;
  min-width: 0;
  overflow-y: auto;
  padding: 24px;
}

.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
}

.topbar-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.topbar p,
.preview-copy p {
  margin: 0 0 4px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h1,
h2 {
  margin: 0;
  letter-spacing: 0;
}

h1 {
  font-size: 28px;
  line-height: 34px;
}

h2 {
  font-size: 18px;
  line-height: 24px;
}

.load-config-button,
.save-apply-button {
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  gap: 7px;
  height: 34px;
  margin: 0;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  appearance: none;
  background: #1d2229;
  color: #eef2f6;
  cursor: pointer;
  padding: 0 10px;
  font-size: 14px;
  font-weight: 750;
  line-height: 1;
  white-space: nowrap;
}

.load-config-button input {
  display: none;
}

.save-apply-button {
  border-color: rgba(37, 211, 102, 0.24);
  background: rgba(37, 211, 102, 0.1);
  color: #eafff0;
}

.preview-band {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  min-height: 250px;
  margin-bottom: 20px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background:
    linear-gradient(90deg, rgba(37, 211, 102, 0.1), transparent 44%),
    #151920;
  padding: 24px;
}

.panel-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.panel {
  min-height: 190px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #171b22;
  padding: 18px;
}

.panel h2 {
  margin-bottom: 16px;
}

.field-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.key-list {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
  margin-top: 16px;
}

.key-list span {
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  background: #202630;
  padding: 5px 8px;
  color: #dfe5ec;
  font-size: 13px;
  font-weight: 700;
}

label {
  display: grid;
  gap: 8px;
  margin-bottom: 16px;
  color: #c9d1da;
  font-weight: 700;
}

.toggle-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.toggle-row input {
  width: 18px;
  height: 18px;
  accent-color: #25d366;
}

.hint {
  color: #7f8b99;
  font-size: 12px;
  font-weight: 500;
}

.position-control {
  display: grid;
  gap: 8px;
  margin-bottom: 16px;
  color: #c9d1da;
  font-weight: 700;
}

.position-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.position-grid button {
  min-height: 34px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  cursor: pointer;
  font-weight: 700;
}

.position-grid button:hover {
  background: #29313d;
}

input[type="range"] {
  accent-color: #25d366;
}

select {
  min-height: 36px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  padding: 0 10px;
}

.color-grid,
.segmented {
  display: flex;
  gap: 8px;
}

.color-grid {
  flex-wrap: wrap;
}

.color-picker {
  display: grid;
  gap: 6px;
  margin: 0;
  color: #9ca7b4;
  font-size: 12px;
  font-weight: 700;
}

.color-picker input {
  width: 44px;
  height: 38px;
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 7px;
  background: transparent;
  cursor: pointer;
  padding: 0;
}

.segmented {
  margin-bottom: 16px;
}

.segmented button {
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #c9d1da;
  padding: 8px 10px;
  font-weight: 700;
}

.segmented button.selected {
  border-color: rgba(37, 211, 102, 0.52);
  background: rgba(37, 211, 102, 0.14);
  color: #eafff0;
}

.quiet {
  margin: 14px 0 0;
}

@media (max-width: 920px) {
  .config-shell {
    grid-template-columns: 1fr;
  }

  .sidebar {
    border-right: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .preview-band,
  .topbar {
    align-items: flex-start;
    flex-direction: column;
  }

  .panel-grid {
    grid-template-columns: 1fr;
  }
}
</style>
