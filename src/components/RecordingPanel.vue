<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, ref, watch } from "vue";
import type { AppConfig, RecordingConfig } from "../domain/defaultConfig";
import {
  clampRecordingFps,
  effectiveRecordingFps,
} from "../domain/recordingConfig";
import {
  estimateRawRecordingBytesPerSecond,
  formatBytesPerSecond,
} from "../domain/recordingEstimate";
import type { RecordingHotkeyConfig, RecordingHotkeyMode } from "../domain/recordingHotkeys";
import RecordingTreeNodeView from "./RecordingTreeNodeView.vue";

type RecordingInspectionEvent =
  | { frame: number; down: string }
  | { frame: number; up: string }
  | { frame: number; marker: string };

type RecordingInspectionFrame = {
  frame: number;
  keys: string[];
};

type RecordingInspection = {
  version: number;
  fps: number;
  keyIds: string[];
  events: RecordingInspectionEvent[];
  frames: RecordingInspectionFrame[];
};

type RecordingFileSummary = {
  fileName: string;
  sizeBytes: number;
  startUnixMs: number | null;
  endUnixMs: number | null;
  fps: number;
  frameCount: number;
  markerCount: number;
};

type RecordingTreeNode = {
  name: string;
  path: string;
  type: "directory" | "file";
  children: RecordingTreeNode[];
  summary: RecordingFileSummary | null;
};

const props = defineProps<{
  config: AppConfig;
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
  recordingHotkeys: RecordingHotkeyConfig;
  hotkeyCaptureTarget: "start" | "stop" | "sync" | null;
}>();

const activeRecordingFps = computed(() => effectiveRecordingFps(props.config.recording));
const estimatedRecordingBytesPerSecond = computed(() =>
  estimateRawRecordingBytesPerSecond(props.config.keys.length, activeRecordingFps.value),
);
const customFpsDraft = ref(String(props.config.recording.customFps));
const syncFeedbackDurationDraft = ref(String(props.config.recording.syncFeedbackDurationMs));
const recordingTree = ref<RecordingTreeNode | null>(null);
const recordingTreeError = ref("");
const recordingTreeLoading = ref(false);

watch(
  () => props.config.recording.customFps,
  (customFps) => {
    customFpsDraft.value = String(customFps);
  },
);

watch(
  () => props.config.recording.syncFeedbackDurationMs,
  (durationMs) => {
    syncFeedbackDurationDraft.value = String(durationMs);
  },
);

const emit = defineEmits<{
  "choose-recording-directory": [];
  "update-silent-recording": [value: boolean];
  "update-recording-config": [recording: RecordingConfig];
  "start-recording": [];
  "stop-recording": [];
  "add-sync-marker": [];
  "inspect-recording-file": [];
  "inspect-recording-path": [path: string];
  "update-recording-hotkey-mode": [mode: RecordingHotkeyMode];
  "begin-hotkey-capture": [target: "start" | "stop" | "sync"];
}>();

function chooseRecordingDirectory() {
  emit("choose-recording-directory");
}

function updateSilentRecording(event: Event) {
  emit("update-silent-recording", (event.target as HTMLInputElement).checked);
}

function selectRecordingFps(fps: number) {
  emit("update-recording-config", {
    ...props.config.recording,
    defaultFps: fps,
    customFpsEnabled: false,
  });
}

function updateCustomFpsEnabled(event: Event) {
  emit("update-recording-config", {
    ...props.config.recording,
    customFpsEnabled: (event.target as HTMLInputElement).checked,
  });
}

function updateSyncFeedbackEnabled(event: Event) {
  emit("update-recording-config", {
    ...props.config.recording,
    syncFeedbackEnabled: (event.target as HTMLInputElement).checked,
  });
}

function updateSyncFeedbackDuration(event: Event) {
  syncFeedbackDurationDraft.value = (event.target as HTMLInputElement).value;
}

function commitSyncFeedbackDuration() {
  const syncFeedbackDurationMs = Math.max(
    100,
    Math.round(Number(syncFeedbackDurationDraft.value)),
  );
  syncFeedbackDurationDraft.value = String(syncFeedbackDurationMs);
  emit("update-recording-config", {
    ...props.config.recording,
    syncFeedbackDurationMs,
  });
}

function updateCustomFps(event: Event) {
  customFpsDraft.value = (event.target as HTMLInputElement).value;
}

function commitCustomFps() {
  const customFps = clampRecordingFps(
    Number(customFpsDraft.value),
    props.config.recording.maxFps,
  );
  customFpsDraft.value = String(customFps);
  emit("update-recording-config", {
    ...props.config.recording,
    customFps,
    customFpsEnabled: true,
  });
}

function startRecording() {
  emit("start-recording");
}

function stopRecording() {
  emit("stop-recording");
}

function addSyncMarker() {
  emit("add-sync-marker");
}

function inspectRecordingFile() {
  emit("inspect-recording-file");
}

async function refreshRecordingTree() {
  const root = props.recordingDirectory || props.defaultRecordingDirectory;
  if (!root) {
    recordingTreeError.value = "Recording folder is not ready.";
    return;
  }

  recordingTreeLoading.value = true;
  recordingTreeError.value = "";

  try {
    recordingTree.value = await invoke<RecordingTreeNode>("list_recording_files", { root });
  } catch (error) {
    recordingTreeError.value = String(error);
  } finally {
    recordingTreeLoading.value = false;
  }
}

function inspectRecordingPath(path: string) {
  emit("inspect-recording-path", path);
}

function updateRecordingHotkeyMode(event: Event) {
  emit("update-recording-hotkey-mode", (event.target as HTMLSelectElement).value as RecordingHotkeyMode);
}

function beginHotkeyCapture(target: "start" | "stop" | "sync") {
  emit("begin-hotkey-capture", target);
}

function formatHotkey(keys: string[]) {
  return keys.length > 0 ? keys.join(" + ") : "Not set";
}

function formatInspectionEvent(event: RecordingInspectionEvent) {
  if ("down" in event) {
    return `frame ${event.frame} down ${event.down}`;
  }

  if ("up" in event) {
    return `frame ${event.frame} up ${event.up}`;
  }

  return `frame ${event.frame} marker ${event.marker}`;
}

function markerEvents(events: RecordingInspectionEvent[]) {
  return events.filter((event): event is Extract<RecordingInspectionEvent, { marker: string }> =>
    "marker" in event
  );
}

function formatFrameTimecode(frame: number, fps: number) {
  const totalSeconds = Math.floor(frame / fps);
  const frameInSecond = frame % fps;
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${pad2(hours)}:${pad2(minutes)}:${pad2(seconds)}:${padFrame(frameInSecond, fps)} @ ${fps}fps`;
}

function pad2(value: number) {
  return String(value).padStart(2, "0");
}

function padFrame(frame: number, fps: number) {
  return String(frame).padStart(String(Math.max(fps - 1, 0)).length, "0");
}
</script>

<template>
  <article class="panel wide-panel">
    <h2>Recording</h2>
    <div class="field-row">
      <span>Save folder</span>
      <strong>
        {{ recordingDirectory || `Default app folder: ${defaultRecordingDirectory || "loading..."}` }}
      </strong>
    </div>
    <div class="recording-actions">
      <button type="button" @click="chooseRecordingDirectory">Choose folder</button>
      <button
        type="button"
        :disabled="isRecording || recordingCountdown > 0"
        @click="startRecording"
      >
        {{ recordingCountdown > 0 ? `Starting in ${recordingCountdown}` : "Start recording" }}
      </button>
      <button type="button" :disabled="!isRecording" @click="stopRecording">
        Stop recording
      </button>
      <button type="button" :disabled="!isRecording" @click="addSyncMarker">
        Add sync marker
      </button>
    </div>
    <label class="toggle-row">
      <input
        :checked="silentRecording"
        type="checkbox"
        @change="updateSilentRecording"
      />
      Silent recording
    </label>
    <div class="fps-config-row">
      <label class="toggle-row">
        <input
          :checked="config.recording.syncFeedbackEnabled"
          type="checkbox"
          @change="updateSyncFeedbackEnabled"
        />
        Sync border flash
      </label>
      <input
        :disabled="!config.recording.syncFeedbackEnabled"
        :min="100"
        :value="syncFeedbackDurationDraft"
        class="fps-input"
        type="number"
        @blur="commitSyncFeedbackDuration"
        @change="commitSyncFeedbackDuration"
        @input="updateSyncFeedbackDuration"
      />
      <span class="write-estimate">ms</span>
    </div>
    <div class="hotkey-panel">
      <label class="settings-row">
        <span>Hotkey mode</span>
        <select
          class="select-control"
          :value="recordingHotkeys.mode"
          @change="updateRecordingHotkeyMode"
        >
          <option value="disabled">Disabled</option>
          <option value="toggle">Toggle start/stop</option>
          <option value="separate">Separate start/stop</option>
        </select>
      </label>
      <div v-if="recordingHotkeys.mode !== 'disabled'" class="hotkey-row">
        <span>Start</span>
        <strong>{{ formatHotkey(recordingHotkeys.start) }}</strong>
        <button type="button" @click="beginHotkeyCapture('start')">
          {{ hotkeyCaptureTarget === "start" ? "Press shortcut..." : "Set" }}
        </button>
      </div>
      <div v-if="recordingHotkeys.mode === 'separate'" class="hotkey-row">
        <span>Stop</span>
        <strong>{{ formatHotkey(recordingHotkeys.stop) }}</strong>
        <button type="button" @click="beginHotkeyCapture('stop')">
          {{ hotkeyCaptureTarget === "stop" ? "Press shortcut..." : "Set" }}
        </button>
      </div>
      <div class="hotkey-row">
        <span>Sync</span>
        <strong>{{ formatHotkey(recordingHotkeys.sync) }}</strong>
        <button type="button" @click="beginHotkeyCapture('sync')">
          {{ hotkeyCaptureTarget === "sync" ? "Press shortcut..." : "Set" }}
        </button>
      </div>
    </div>
    <div class="segmented" aria-label="Capture frame rate">
      <button
        v-for="fps in config.recording.fpsOptions"
        :key="fps"
        :class="{ selected: !config.recording.customFpsEnabled && fps === config.recording.defaultFps }"
        type="button"
        @click="selectRecordingFps(fps)"
      >
        {{ fps }}fps
      </button>
    </div>
    <div class="fps-config-row">
      <label class="toggle-row">
        <input
          :checked="config.recording.customFpsEnabled"
          type="checkbox"
          @change="updateCustomFpsEnabled"
        />
        Custom FPS
      </label>
      <input
        :disabled="!config.recording.customFpsEnabled"
        :max="config.recording.maxFps"
        :min="1"
        :value="customFpsDraft"
        class="fps-input"
        type="number"
        @blur="commitCustomFps"
        @change="commitCustomFps"
        @input="updateCustomFps"
      />
      <span class="write-estimate">
        {{ activeRecordingFps }}fps · {{ formatBytesPerSecond(estimatedRecordingBytesPerSecond) }} raw
      </span>
    </div>
    <div class="field-row">
      <span>Primary artifact</span>
      <strong>{{ config.recording.formatExtension }}</strong>
    </div>
    <p class="quiet">
      Input frames are stored as compact binary state, then replayed for
      rendering or export.
    </p>
    <p v-if="lastRecordingPath" class="quiet">
      Last recording: {{ lastRecordingPath }}
    </p>
    <p v-if="recordingStatusMessage" class="status-text">
      {{ recordingStatusMessage }}
    </p>
    <div class="inspection-panel">
      <div class="section-header">
        <h3>Recording inspection</h3>
        <button type="button" @click="inspectRecordingFile">
          Inspect .kbdrec
        </button>
      </div>
      <div class="section-header">
        <h3>Recording files</h3>
        <button type="button" :disabled="recordingTreeLoading" @click="refreshRecordingTree">
          {{ recordingTreeLoading ? "Loading..." : "Refresh" }}
        </button>
      </div>
      <p v-if="recordingTreeError" class="error-text">
        {{ recordingTreeError }}
      </p>
      <div v-if="recordingTree" class="recording-tree">
        <RecordingTreeNodeView
          :node="recordingTree"
          @inspect="inspectRecordingPath"
        />
      </div>
      <p v-if="inspectedRecordingPath" class="quiet">
        File: {{ inspectedRecordingPath }}
      </p>
      <p v-if="recordingInspectionError" class="error-text">
        {{ recordingInspectionError }}
      </p>
      <div v-if="recordingInspection" class="inspection-grid">
        <div class="field-row">
          <span>Version</span>
          <strong>{{ recordingInspection.version }}</strong>
        </div>
        <div class="field-row">
          <span>FPS</span>
          <strong>{{ recordingInspection.fps }}</strong>
        </div>
        <div class="field-row">
          <span>Keys</span>
          <strong>{{ recordingInspection.keyIds.length }}</strong>
        </div>
        <div class="field-row">
          <span>Events</span>
          <strong>{{ recordingInspection.events.length }}</strong>
        </div>
        <div class="field-row">
          <span>Frames</span>
          <strong>{{ recordingInspection.frames.length }}</strong>
        </div>
        <div class="field-row">
          <span>Markers</span>
          <strong>
            {{ recordingInspection.events.filter((event) => "marker" in event).length }}
          </strong>
        </div>
      </div>
      <div v-if="recordingInspection" class="inspection-lists">
        <div>
          <h4>Markers</h4>
          <div class="marker-metadata-list">
            <div
              v-for="(event, index) in markerEvents(recordingInspection.events)"
              :key="`${event.frame}-${event.marker}-${index}`"
              class="marker-metadata"
            >
              <strong>marker {{ event.marker }}</strong>
              <span>frame {{ event.frame }}</span>
              <span>time {{ formatFrameTimecode(event.frame, recordingInspection.fps) }}</span>
            </div>
          </div>
        </div>
        <div>
          <h4>Key table</h4>
          <p class="quiet">{{ recordingInspection.keyIds.join(", ") || "None" }}</p>
        </div>
        <div>
          <h4>Events</h4>
          <ol>
            <li
              v-for="(event, index) in recordingInspection.events.slice(0, 8)"
              :key="index"
            >
              {{ formatInspectionEvent(event) }}
            </li>
          </ol>
        </div>
        <div>
          <h4>Frames</h4>
          <ol>
            <li
              v-for="frame in recordingInspection.frames.slice(0, 8)"
              :key="frame.frame"
            >
              frame {{ frame.frame }}: {{ frame.keys.join(", ") || "none" }}
            </li>
          </ol>
        </div>
      </div>
    </div>
  </article>
</template>

<style scoped>
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
  margin: 0 0 16px;
  letter-spacing: 0;
}

.field-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.field-row span {
  color: #9ca7b4;
}

.field-row strong {
  min-width: 0;
  overflow-wrap: anywhere;
  text-align: right;
}

.recording-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 16px 0;
}

.recording-actions button,
.inspection-panel button,
.hotkey-row button {
  min-height: 34px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  cursor: pointer;
  font-weight: 700;
  padding: 0 10px;
}

.recording-actions button:disabled,
.inspection-panel button:disabled,
.hotkey-row button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.recording-actions button:not(:disabled):hover,
.inspection-panel button:not(:disabled):hover,
.hotkey-row button:not(:disabled):hover {
  background: #29313d;
}

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

.fps-config-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 16px;
}

.fps-config-row .toggle-row {
  margin: 0;
}

.fps-input {
  width: 96px;
  min-height: 34px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  padding: 0 10px;
}

.fps-input:disabled {
  opacity: 0.45;
}

.write-estimate {
  color: #9ca7b4;
  font-size: 13px;
  font-weight: 800;
}

.hotkey-panel {
  display: grid;
  gap: 10px;
  margin: 16px 0;
}

.settings-row {
  display: grid;
  grid-template-columns: minmax(112px, 1fr) minmax(180px, 240px);
  align-items: center;
  gap: 10px;
  margin: 0;
}

.settings-row span,
.hotkey-row span {
  color: #9ca7b4;
  font-size: 13px;
  font-weight: 800;
}

.hotkey-row {
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
}

.hotkey-row strong {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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

select:focus {
  border-color: rgba(37, 211, 102, 0.55);
  outline: 2px solid rgba(37, 211, 102, 0.14);
  outline-offset: 0;
}

.segmented {
  display: flex;
  gap: 8px;
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
  color: #9ca7b4;
}

.status-text {
  margin: 10px 0 0;
  color: #c9d1da;
  font-size: 13px;
  font-weight: 700;
}

.error-text {
  margin: 0;
  color: #ff8f8f;
  font-size: 13px;
  font-weight: 700;
}

.inspection-panel {
  display: grid;
  gap: 14px;
  margin-top: 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding-top: 18px;
}

.recording-tree {
  display: grid;
  gap: 6px;
  max-height: 360px;
  overflow: auto;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.section-header h3,
.inspection-lists h4 {
  margin: 0;
  letter-spacing: 0;
}

.section-header h3 {
  font-size: 16px;
  line-height: 22px;
}

.inspection-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0 18px;
}

.inspection-lists {
  display: grid;
  gap: 14px;
}

.marker-metadata-list {
  display: grid;
  gap: 8px;
}

.marker-metadata {
  display: grid;
  grid-template-columns: minmax(120px, 1.1fr) minmax(100px, 0.7fr) minmax(180px, 1.2fr);
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background: #151a20;
  padding: 8px 10px;
  color: #dfe5ec;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.marker-metadata strong {
  color: #eafff0;
}

.inspection-lists h4 {
  margin-bottom: 6px;
  color: #c9d1da;
  font-size: 13px;
}

.inspection-lists ol {
  display: grid;
  gap: 5px;
  margin: 0;
  padding-left: 18px;
  color: #dfe5ec;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}
</style>
