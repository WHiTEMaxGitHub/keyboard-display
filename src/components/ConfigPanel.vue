<script setup lang="ts">
import { computed, ref } from "vue";
import { normalizeHexColor } from "../domain/colorPicker";
import {
  isKeyBinding,
  type AppConfig,
  type ExportConfig,
  type OverlayStyle,
} from "../domain/defaultConfig";
import type { RecentProfile } from "../domain/appConfig";
import type { RecordingHotkeyConfig, RecordingHotkeyMode } from "../domain/recordingHotkeys";
import PovOverlay from "./PovOverlay.vue";
import RecordingBrowserPanel from "./RecordingBrowserPanel.vue";
import ColorPicker from "./ColorPicker.vue";
import ConfigSidebar from "./ConfigSidebar.vue";
import ConfigTopbar from "./ConfigTopbar.vue";
import ExportPanel from "./ExportPanel.vue";
import LayoutEditor from "./LayoutEditor.vue";
import RecordingPanel from "./RecordingPanel.vue";
import WindowPanel from "./WindowPanel.vue";

type ConfigPage = "overview" | "layout" | "appearance" | "window" | "recording" | "export";
type LayoutSubPage = "summary" | "editor";
type RecordingSubPage = "control" | "files";

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
const layoutSubPage = ref<LayoutSubPage>("summary");
const recordingSubPage = ref<RecordingSubPage>("control");
const recentColors = ref<string[]>([]);
const sidebarCollapsed = ref(false);

const props = defineProps<{
  config: AppConfig;
  activeKeys: Set<string>;
  overlayVisible: boolean;
  profileName: string;
  profileChanged: boolean;
  recentProfiles: RecentProfile[];
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
  overlayAdjusting: boolean;
  recordingHotkeys: RecordingHotkeyConfig;
  hotkeyCaptureTarget: "start" | "stop" | "sync" | null;
}>();

const layoutRows = computed(() => {
  return props.config.rows.map((items, index) => ({ row: index + 1, items }));
});

const emit = defineEmits<{
  "update-overlay-style": [style: OverlayStyle];
  "update-overlay-rows": [rows: AppConfig["rows"]];
  "update-overlay-visible": [visible: boolean];
  "load-config": [];
  "refresh-pov": [];
  "load-recent-profile": [path: string];
  "export-and-apply-config": [];
  "overwrite-and-apply-config": [];
  "choose-recording-directory": [];
  "update-silent-recording": [value: boolean];
  "update-recording-config": [recording: AppConfig["recording"]];
  "update-export-config": [exportConfig: ExportConfig];
  "start-recording": [];
  "stop-recording": [];
  "add-sync-marker": [];
  "inspect-recording-file": [];
  "inspect-recording-path": [path: string];
  "update-recording-hotkey-mode": [mode: RecordingHotkeyMode];
  "begin-hotkey-capture": [target: "start" | "stop" | "sync"];
  "start-overlay-adjust": [];
  "save-overlay-adjust": [];
  "cancel-overlay-adjust": [];
  "move-overlay": [
    position: "top-left" | "top-right" | "bottom-left" | "bottom-right" | "custom",
  ];
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

function rememberColor(color: string) {
  const normalizedColor = normalizeHexColor(color);
  recentColors.value = [
    normalizedColor,
    ...recentColors.value.filter((recentColor) => recentColor !== normalizedColor),
  ].slice(0, 8);
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

function updateAlwaysOnTop(event: Event) {
  const alwaysOnTop = (event.target as HTMLInputElement).checked;
  emit("update-overlay-style", { ...props.config.style, alwaysOnTop });
}

function updateOverlayVisible(event: Event) {
  emit("update-overlay-visible", (event.target as HTMLInputElement).checked);
}

function moveOverlay(
  position: "top-left" | "top-right" | "bottom-left" | "bottom-right" | "custom",
) {
  emit("move-overlay", position);
}

function startOverlayAdjust() {
  emit("start-overlay-adjust");
}

function saveOverlayAdjust() {
  emit("save-overlay-adjust");
}

function cancelOverlayAdjust() {
  emit("cancel-overlay-adjust");
}

function loadConfigFile() {
  emit("load-config");
}

function refreshPov() {
  emit("refresh-pov");
}

function loadRecentProfile(event: Event) {
  const select = event.target as HTMLSelectElement;
  const path = select.value;
  if (path) {
    emit("load-recent-profile", path);
    select.value = "";
  }
}

function exportAndApplyConfig() {
  emit("export-and-apply-config");
}

function overwriteAndApplyConfig() {
  emit("overwrite-and-apply-config");
}

function updateRenderMarkers(event: Event) {
  emit("update-export-config", {
    ...props.config.export,
    renderMarkers: (event.target as HTMLInputElement).checked,
  });
}

</script>

<template>
  <main :class="['config-shell', { 'sidebar-collapsed': sidebarCollapsed }]">
    <ConfigSidebar
      :active-page="activePage"
      :recording-sub-page="recordingSubPage"
      :collapsed="sidebarCollapsed"
      @toggle-collapse="sidebarCollapsed = !sidebarCollapsed"
      @update-active-page="activePage = $event"
      @update-recording-sub-page="recordingSubPage = $event"
    />

    <section class="workspace">
      <ConfigTopbar
        @load-config="loadConfigFile"
        @export-and-apply-config="exportAndApplyConfig"
        @overwrite-and-apply-config="overwriteAndApplyConfig"
      />

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
            <label class="recent-profile-control">
              <span>Recent profiles</span>
              <select
                class="select-control"
                :disabled="recentProfiles.length === 0"
                value=""
                @change="loadRecentProfile"
              >
                <option value="">
                  {{ recentProfiles.length ? "Choose a profile" : "No recent profiles" }}
                </option>
                <option
                  v-for="profile in recentProfiles"
                  :key="profile.path"
                  :value="profile.path"
                >
                  {{ profile.name }}
                </option>
              </select>
            </label>
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
          <div class="page-tabs">
            <button
              :class="{ active: layoutSubPage === 'summary' }"
              type="button"
              @click="layoutSubPage = 'summary'"
            >
              Summary
            </button>
            <button
              :class="{ active: layoutSubPage === 'editor' }"
              type="button"
              @click="layoutSubPage = 'editor'"
            >
              Editor
            </button>
          </div>
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
          <div v-if="layoutSubPage === 'summary'" class="layout-line-list">
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
          <LayoutEditor
            v-else
            :rows="config.rows"
            @update-rows="emit('update-overlay-rows', $event)"
          />
        </article>
      </section>

      <section v-else-if="activePage === 'appearance'" class="page-stack">
        <article class="panel wide-panel">
          <div class="section-header">
            <h2>Appearance</h2>
            <button class="panel-action-button" type="button" @click="refreshPov">
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
              <option value="faint">Faint</option>
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
              @remember-color="rememberColor"
            />
            <ColorPicker
              label="Pressed key"
              :value="config.style.activeColor"
              :recent-colors="recentColors"
              alpha-enabled
              @update:value="updateStyleColor('activeColor', $event)"
              @remember-color="rememberColor"
            />
            <ColorPicker
              label="Idle text"
              :value="config.style.idleTextColor"
              :recent-colors="recentColors"
              alpha-enabled
              @update:value="updateStyleColor('idleTextColor', $event)"
              @remember-color="rememberColor"
            />
            <ColorPicker
              label="Pressed text"
              :value="config.style.activeTextColor"
              :recent-colors="recentColors"
              alpha-enabled
              @update:value="updateStyleColor('activeTextColor', $event)"
              @remember-color="rememberColor"
            />
            <ColorPicker
              label="Backplate"
              :value="config.style.backgroundColor"
              :recent-colors="recentColors"
              alpha-enabled
              @update:value="updateStyleColor('backgroundColor', $event)"
              @remember-color="rememberColor"
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
      </section>

      <section v-else-if="activePage === 'window'" class="page-stack">
        <WindowPanel
          :overlay-position="overlayPosition"
          :overlay-visible="overlayVisible"
          :always-on-top="config.style.alwaysOnTop"
          :overlay-adjusting="overlayAdjusting"
          @update-overlay-visible="updateOverlayVisible"
          @update-always-on-top="updateAlwaysOnTop"
          @start-overlay-adjust="startOverlayAdjust"
          @save-overlay-adjust="saveOverlayAdjust"
          @cancel-overlay-adjust="cancelOverlayAdjust"
          @move-overlay="moveOverlay"
        />
      </section>

      <section v-else-if="activePage === 'recording' && recordingSubPage === 'control'" class="page-stack">
        <RecordingPanel
          :config="config"
          :recording-directory="recordingDirectory"
          :default-recording-directory="defaultRecordingDirectory"
          :silent-recording="silentRecording"
          :is-recording="isRecording"
          :recording-countdown="recordingCountdown"
          :last-recording-path="lastRecordingPath"
          :recording-status-message="recordingStatusMessage"
          :recording-hotkeys="recordingHotkeys"
          :hotkey-capture-target="hotkeyCaptureTarget"
          @choose-recording-directory="emit('choose-recording-directory')"
          @update-silent-recording="emit('update-silent-recording', $event)"
          @update-recording-config="emit('update-recording-config', $event)"
          @start-recording="emit('start-recording')"
          @stop-recording="emit('stop-recording')"
          @add-sync-marker="emit('add-sync-marker')"
          @inspect-recording-file="emit('inspect-recording-file')"
          @inspect-recording-path="emit('inspect-recording-path', $event)"
          @update-recording-hotkey-mode="emit('update-recording-hotkey-mode', $event)"
          @begin-hotkey-capture="emit('begin-hotkey-capture', $event)"
        />
      </section>

      <section v-else-if="activePage === 'recording' && recordingSubPage === 'files'" class="page-stack">
        <RecordingBrowserPanel
          :recording-directory="recordingDirectory"
          :default-recording-directory="defaultRecordingDirectory"
          :inspected-recording-path="inspectedRecordingPath"
          :recording-inspection="recordingInspection"
          :recording-inspection-error="recordingInspectionError"
          @inspect-recording-file="emit('inspect-recording-file')"
          @inspect-recording-path="emit('inspect-recording-path', $event)"
        />
      </section>

      <section v-else-if="activePage === 'export'" class="page-stack">
        <ExportPanel
          :render-markers="config.export.renderMarkers"
          @update-render-markers="updateRenderMarkers"
        />
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

.config-shell.sidebar-collapsed {
  grid-template-columns: 72px minmax(0, 1fr);
}

.preview-copy p,
.quiet,
.field-row span {
  color: #9ca7b4;
}

.workspace {
  height: 100vh;
  min-width: 0;
  overflow-y: auto;
  padding: 24px;
}

.preview-copy p {
  margin: 0 0 4px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h2 {
  margin: 0;
  letter-spacing: 0;
}

h2 {
  font-size: 18px;
  line-height: 24px;
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

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}

.section-header h2 {
  margin-bottom: 0;
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

.recent-profile-control {
  display: grid;
  grid-template-columns: minmax(110px, 1fr) minmax(180px, 240px);
  align-items: center;
  gap: 7px;
  margin-top: 14px;
  color: #c9d1da;
  font-size: 13px;
  font-weight: 700;
}

.recent-profile-control select {
  justify-self: end;
  width: min(240px, 100%);
}

.recent-profile-control select:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.page-tabs {
  display: inline-flex;
  gap: 6px;
  margin-bottom: 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  padding: 4px;
}

.page-tabs button {
  min-height: 30px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: #9ca7b4;
  cursor: pointer;
  font-weight: 800;
  padding: 0 10px;
}

.page-tabs button.active,
.page-tabs button:hover {
  background: rgba(37, 211, 102, 0.14);
  color: #eafff0;
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

.quiet {
  margin: 14px 0 0;
}

@media (max-width: 920px) {
  .config-shell {
    grid-template-columns: 72px minmax(0, 1fr);
  }

  .preview-band {
    align-items: flex-start;
    flex-direction: column;
  }

  .panel-grid {
    grid-template-columns: 1fr;
  }
}
</style>
