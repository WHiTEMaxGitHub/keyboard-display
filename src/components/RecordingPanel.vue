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
import BaseFieldRow from "./BaseFieldRow.vue";
import BasePanel from "./BasePanel.vue";
import BaseSegmentedControl from "./BaseSegmentedControl.vue";
import BaseToggleRow from "./BaseToggleRow.vue";
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
const fpsOptions = computed(() =>
  props.config.recording.fpsOptions.map((fps) => ({
    value: fps,
    label: `${fps}fps`,
  })),
);
const selectedDefaultFps = computed(() =>
  props.config.recording.customFpsEnabled ? -1 : props.config.recording.defaultFps,
);
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
    <BaseFieldRow label="Save folder">
      {{ recordingDirectory || `Default app folder: ${defaultRecordingDirectory || "loading..."}` }}
    </BaseFieldRow>
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
    <BaseToggleRow :checked="silentRecording" @change="updateSilentRecording">
      Silent recording
    </BaseToggleRow>
    <div class="fps-config-row">
      <BaseToggleRow compact :checked="config.recording.syncFeedbackEnabled" @change="updateSyncFeedbackEnabled">
        Sync border flash
      </BaseToggleRow>
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
      <BaseSegmentedControl
        :model-value="selectedDefaultFps"
        :options="fpsOptions"
        aria-label="Capture frame rate"
        @update:model-value="selectRecordingFps"
      />
      <BaseToggleRow compact :checked="config.recording.customFpsEnabled" @change="updateCustomFpsEnabled">
        Custom FPS
      </BaseToggleRow>
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
    <BaseFieldRow label="Primary artifact">{{ config.recording.formatExtension }}</BaseFieldRow>
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
.quiet {
  color: #9ca7b4;
}

.recording-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 16px 0;
}

.fps-config-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 16px;
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
