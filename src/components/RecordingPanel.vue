<script setup lang="ts">
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

const props = defineProps<{
  config: AppConfig;
  recordingDirectory: string;
  defaultRecordingDirectory: string;
  silentRecording: boolean;
  isRecording: boolean;
  recordingCountdown: number;
  lastRecordingPath: string;
  recordingStatusMessage: string;
  recordingHotkeys: RecordingHotkeyConfig;
  hotkeyCaptureTarget: "start" | "stop" | "sync" | null;
}>();

const activeRecordingFps = computed(() => effectiveRecordingFps(props.config.recording));
const estimatedRecordingBytesPerSecond = computed(() =>
  estimateRawRecordingBytesPerSecond(props.config.keys.length, activeRecordingFps.value),
);
const customFpsDraft = ref(String(props.config.recording.customFps));
const syncFeedbackDurationDraft = ref(String(props.config.recording.syncFeedbackDurationMs));
const filenameTemplateDraft = ref(props.config.recording.filenameTemplate);

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

watch(
  () => props.config.recording.filenameTemplate,
  (filenameTemplate) => {
    filenameTemplateDraft.value = filenameTemplate;
  },
);

const emit = defineEmits<{
  "choose-recording-directory": [];
  "update-silent-recording": [value: boolean];
  "update-recording-config": [recording: RecordingConfig];
  "start-recording": [];
  "stop-recording": [];
  "add-sync-marker": [];
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

function updateFilenameTemplate(event: Event) {
  filenameTemplateDraft.value = (event.target as HTMLInputElement).value;
}

function commitFilenameTemplate() {
  const filenameTemplate = filenameTemplateDraft.value.trim() || "${start}-${end}";
  filenameTemplateDraft.value = filenameTemplate;
  emit("update-recording-config", {
    ...props.config.recording,
    filenameTemplate,
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

function updateRecordingHotkeyMode(event: Event) {
  emit("update-recording-hotkey-mode", (event.target as HTMLSelectElement).value as RecordingHotkeyMode);
}

function beginHotkeyCapture(target: "start" | "stop" | "sync") {
  emit("begin-hotkey-capture", target);
}

function formatHotkey(keys: string[]) {
  return keys.length > 0 ? keys.join(" + ") : "Not set";
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
    <label class="filename-template-row">
      <span>Filename template</span>
      <input
        :value="filenameTemplateDraft"
        type="text"
        placeholder="${start}-${end}"
        @blur="commitFilenameTemplate"
        @change="commitFilenameTemplate"
        @input="updateFilenameTemplate"
      />
    </label>
    <p class="quiet">
      Variables: ${start}, ${end}, ${startDate}, ${startTime}, ${endTime},
      ${duration}, ${profileName}, ${profileSlug}, ${fps}
    </p>
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

.field-row span,
.quiet {
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
.hotkey-row button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.recording-actions button:not(:disabled):hover,
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

.filename-template-row {
  display: grid;
  gap: 7px;
  margin: 16px 0 0;
  color: #c9d1da;
  font-size: 13px;
  font-weight: 700;
}

.filename-template-row input {
  width: 100%;
  min-height: 34px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  font: inherit;
  padding: 0 10px;
}

.filename-template-row input:focus {
  border-color: rgba(37, 211, 102, 0.55);
  outline: 2px solid rgba(37, 211, 102, 0.14);
  outline-offset: 0;
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
}

.status-text {
  margin: 10px 0 0;
  color: #c9d1da;
  font-size: 13px;
  font-weight: 700;
}
</style>
