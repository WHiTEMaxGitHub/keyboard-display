<script setup lang="ts">
import {
  Clapperboard,
  Eye,
  Keyboard,
  MonitorCog,
  MonitorUp,
  Palette,
  SlidersHorizontal,
  Video,
} from "@lucide/vue";
import { computed, ref } from "vue";
import {
  isKeyBinding,
  type AppConfig,
  type OverlayStyle,
} from "../domain/defaultConfig";
import type { RecordingHotkeyConfig, RecordingHotkeyMode } from "../domain/recordingHotkeys";
import PovOverlay from "./PovOverlay.vue";
import RecordingPanel from "./RecordingPanel.vue";

type ConfigPage = "overview" | "layout" | "appearance" | "window" | "recording" | "export";

type RecordingInspectionFrame = {
  frame: number;
  keys: string[];
};

type RecordingInspectionEvent =
  | { frame: number; down: string }
  | { frame: number; up: string }
  | { frame: number; marker: string };

type RecordingInspection = {
  version: number;
  fps: number;
  keyIds: string[];
  events: RecordingInspectionEvent[];
  frames: RecordingInspectionFrame[];
};

const activePage = ref<ConfigPage>("overview");

const navItems: Array<{
  id: ConfigPage;
  label: string;
  icon: typeof Eye;
}> = [
  { id: "overview", label: "Overview", icon: Eye },
  { id: "layout", label: "Layout", icon: MonitorUp },
  { id: "appearance", label: "Appearance", icon: Palette },
  { id: "window", label: "Window", icon: MonitorCog },
  { id: "recording", label: "Recording", icon: Clapperboard },
  { id: "export", label: "Export", icon: Video },
];

const props = defineProps<{
  config: AppConfig;
  activeKeys: Set<string>;
  overlayVisible: boolean;
  profileName: string;
  profileChanged: boolean;
  recordingDirectory: string;
  defaultRecordingDirectory: string;
  silentRecording: boolean;
  isRecording: boolean;
  recordingCountdown: number;
  lastRecordingPath: string;
  recordingStatusMessage: string;
  inspectedRecordingPath: string;
  recordingInspection: RecordingInspection | null;
  recordingInspectionError: string;
  overlayPosition: string;
  recordingHotkeys: RecordingHotkeyConfig;
  hotkeyCaptureTarget: "start" | "stop" | "sync" | null;
}>();

const layoutRows = computed(() => {
  return props.config.rows.map((items, index) => ({ row: index + 1, items }));
});

const emit = defineEmits<{
  "update-overlay-style": [style: OverlayStyle];
  "update-overlay-visible": [visible: boolean];
  "load-config": [];
  "export-and-apply-config": [];
  "overwrite-and-apply-config": [];
  "choose-recording-directory": [];
  "update-silent-recording": [value: boolean];
  "update-recording-config": [recording: AppConfig["recording"]];
  "start-recording": [];
  "stop-recording": [];
  "add-sync-marker": [];
  "inspect-recording-file": [];
  "update-recording-hotkey-mode": [mode: RecordingHotkeyMode];
  "begin-hotkey-capture": [target: "start" | "stop" | "sync"];
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

function loadConfigFile() {
  emit("load-config");
}

function exportAndApplyConfig() {
  emit("export-and-apply-config");
}

function overwriteAndApplyConfig() {
  emit("overwrite-and-apply-config");
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

      <nav class="nav-list" aria-label="Configuration pages">
        <button
          v-for="item in navItems"
          :key="item.id"
          :class="{ active: activePage === item.id }"
          type="button"
          @click="activePage = item.id"
        >
          <component :is="item.icon" :size="18" aria-hidden="true" />
          {{ item.label }}
        </button>
      </nav>
    </aside>

    <section class="workspace">
      <header class="topbar">
        <div>
          <p>Configuration</p>
          <h1>POV overlay control panel</h1>
        </div>
        <div class="topbar-actions">
          <button class="load-config-button" type="button" @click="loadConfigFile">
            <SlidersHorizontal :size="15" aria-hidden="true" />
            Load config
          </button>
          <button class="save-apply-button" type="button" @click="exportAndApplyConfig">
            Export & Apply
          </button>
          <button class="save-apply-button" type="button" @click="overwriteAndApplyConfig">
            Overwrite & Apply
          </button>
        </div>
      </header>

      <section v-if="activePage === 'overview'" class="page-stack">
        <section class="preview-band" aria-label="Live preview">
          <div class="preview-copy">
            <p>Live Preview</p>
            <h2>{{ profileName }}</h2>
          </div>
          <div class="preview-viewport">
            <PovOverlay
              :layout="config.layout"
              :rows="config.rows"
              :keys="config.keys"
              :active-keys="activeKeys"
              :overlay-style="config.style"
            />
          </div>
        </section>

        <section class="panel-grid">
          <article class="panel">
            <h2>Profile</h2>
            <div class="field-row">
              <span>Name</span>
              <strong>{{ profileName }}</strong>
            </div>
            <div class="field-row">
              <span>Status</span>
              <strong>{{ profileChanged ? "Unsaved changes" : "Saved" }}</strong>
            </div>
            <div class="field-row">
              <span>Visible keys</span>
              <strong>{{ config.keys.length }}</strong>
            </div>
          </article>

          <article class="panel">
            <h2>Quick controls</h2>
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
          </article>
        </section>
      </section>

      <section v-else-if="activePage === 'layout'" class="page-stack">
        <article class="panel">
          <h2>Layout</h2>
          <div class="field-row">
            <span>Unit size</span>
            <strong>{{ config.layout.unitPx }}px</strong>
          </div>
          <div class="field-row">
            <span>Gap</span>
            <strong>{{ config.layout.gapUnit }} unit</strong>
          </div>
          <div class="field-row">
            <span>Visible keys</span>
            <strong>{{ config.keys.length }}</strong>
          </div>
          <div class="layout-line-list">
            <div v-for="line in layoutRows" :key="line.row" class="layout-line">
              <span class="line-label">Line {{ line.row }}:</span>
              <span class="line-keys">
                <span
                  v-for="(item, index) in line.items"
                  :key="`${line.row}-${index}`"
                  :class="['line-key', { 'line-gap': !isKeyBinding(item) }]"
                >
                  {{ isKeyBinding(item) ? item.label : "Gap" }} · {{ item.widthUnit }}u
                </span>
              </span>
            </div>
          </div>
        </article>
      </section>

      <section v-else-if="activePage === 'appearance'" class="page-stack">
        <article class="panel wide-panel">
          <h2>Appearance</h2>
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
          <label class="settings-row">
            <span>Background</span>
            <select
              class="select-control compact-select"
              :value="config.style.backgroundMode"
              @change="updateBackgroundMode"
            >
              <option value="transparent">Transparent</option>
              <option value="panel">Rounded panel</option>
            </select>
          </label>
          <label class="settings-row">
            <span>Idle keys</span>
            <select
              class="select-control compact-select"
              :value="config.style.idleKeyVisibility"
              @change="updateIdleKeyVisibility"
            >
              <option value="visible">Visible</option>
              <option value="faint">Faint</option>
              <option value="hidden">Hidden until pressed</option>
            </select>
          </label>
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
      </section>

      <section v-else-if="activePage === 'window'" class="page-stack">
        <article class="panel wide-panel">
          <h2>Window</h2>
          <div class="field-row">
            <span>Position</span>
            <strong>{{ overlayPosition }}</strong>
          </div>
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
        </article>
      </section>

      <section v-else-if="activePage === 'recording'" class="page-stack">
        <RecordingPanel
          :config="config"
          :recording-directory="recordingDirectory"
          :default-recording-directory="defaultRecordingDirectory"
          :silent-recording="silentRecording"
          :is-recording="isRecording"
          :recording-countdown="recordingCountdown"
          :last-recording-path="lastRecordingPath"
          :recording-status-message="recordingStatusMessage"
          :inspected-recording-path="inspectedRecordingPath"
          :recording-inspection="recordingInspection"
          :recording-inspection-error="recordingInspectionError"
          :recording-hotkeys="recordingHotkeys"
          :hotkey-capture-target="hotkeyCaptureTarget"
          @choose-recording-directory="emit('choose-recording-directory')"
          @update-silent-recording="emit('update-silent-recording', $event)"
          @update-recording-config="emit('update-recording-config', $event)"
          @start-recording="emit('start-recording')"
          @stop-recording="emit('stop-recording')"
          @add-sync-marker="emit('add-sync-marker')"
          @inspect-recording-file="emit('inspect-recording-file')"
          @update-recording-hotkey-mode="emit('update-recording-hotkey-mode', $event)"
          @begin-hotkey-capture="emit('begin-hotkey-capture', $event)"
        />
      </section>

      <section v-else-if="activePage === 'export'" class="page-stack">
        <article class="panel wide-panel">
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

.nav-list button {
  display: flex;
  align-items: center;
  gap: 10px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: #c9d1da;
  cursor: pointer;
  padding: 10px 11px;
  text-align: left;
  font-weight: 700;
}

.nav-list button:hover,
.nav-list button.active {
  background: rgba(255, 255, 255, 0.06);
}

.nav-list button.active {
  color: #eafff0;
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
  min-width: 0;
  min-height: 250px;
  margin-bottom: 20px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background:
    linear-gradient(90deg, rgba(37, 211, 102, 0.1), transparent 44%),
    #151920;
  padding: 24px;
}

.preview-copy {
  flex: 0 0 180px;
}

.preview-viewport {
  display: grid;
  justify-content: start;
  min-width: 0;
  max-width: 100%;
  overflow-x: auto;
  overflow-y: hidden;
  padding: 8px 0 10px 8px;
}

.preview-viewport :deep(.pov-shell) {
  flex: 0 0 auto;
}

.page-stack {
  display: grid;
  gap: 16px;
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

.wide-panel {
  max-width: 760px;
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

.field-row strong {
  min-width: 0;
  overflow-wrap: anywhere;
  text-align: right;
}

.layout-line-list {
  display: grid;
  gap: 10px;
  margin-top: 16px;
}

.layout-line {
  display: grid;
  grid-template-columns: 76px minmax(0, 1fr);
  align-items: start;
  gap: 10px;
}

.line-label {
  color: #9ca7b4;
  font-size: 13px;
  font-weight: 800;
}

.line-keys {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.line-key {
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
