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
import BaseButton from "./BaseButton.vue";
import BasePanel from "./BasePanel.vue";
import RecordingHotkeysPanel from "./RecordingHotkeysPanel.vue";

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

</script>

<template>
  <BasePanel wide>
    <h2>Recording</h2>
    <div class="field-row">
      <span>Save folder</span>
      <strong>
        {{ recordingDirectory || `Default app folder: ${defaultRecordingDirectory || "loading..."}` }}
      </strong>
    </div>
    <div class="recording-actions">
      <BaseButton @click="chooseRecordingDirectory">Choose folder</BaseButton>
      <BaseButton
        variant="primary"
        :disabled="isRecording || recordingCountdown > 0"
        @click="startRecording"
      >
        {{ recordingCountdown > 0 ? `Starting in ${recordingCountdown}` : "Start recording" }}
      </BaseButton>
      <BaseButton :disabled="!isRecording" @click="stopRecording">
        Stop recording
      </BaseButton>
      <BaseButton :disabled="!isRecording" @click="addSyncMarker">
        Add sync marker
      </BaseButton>
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
    <RecordingHotkeysPanel
      :recording-hotkeys="recordingHotkeys"
      :hotkey-capture-target="hotkeyCaptureTarget"
      @update-recording-hotkey-mode="emit('update-recording-hotkey-mode', $event)"
      @begin-hotkey-capture="emit('begin-hotkey-capture', $event)"
    />
    <div class="fps-picker-row" aria-label="Capture frame rate">
      <div class="segmented">
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
  </BasePanel>
</template>

<style scoped>
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

.fps-picker-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 16px;
}

.segmented {
  display: flex;
  gap: 8px;
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
