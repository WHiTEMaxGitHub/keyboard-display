<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
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
  silentRecording: boolean;
  isRecording: boolean;
  recordingCountdown: number;
  lastRecordingPath: string;
  recordingStatusMessage: string;
  recordingHotkeys: RecordingHotkeyConfig;
  hotkeyCaptureTarget: "start" | "stop" | "sync" | null;
}>();

const { t } = useI18n();
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
const recordingFilenameVariables = [
  { token: "${start}", descriptionKey: "recording.filename.variableHelp.start" },
  { token: "${end}", descriptionKey: "recording.filename.variableHelp.end" },
  { token: "${startDate}", descriptionKey: "recording.filename.variableHelp.startDate" },
  { token: "${startTime}", descriptionKey: "recording.filename.variableHelp.startTime" },
  { token: "${endTime}", descriptionKey: "recording.filename.variableHelp.endTime" },
  { token: "${duration}", descriptionKey: "recording.filename.variableHelp.duration" },
  { token: "${profileName}", descriptionKey: "recording.filename.variableHelp.profileName" },
  { token: "${profileSlug}", descriptionKey: "recording.filename.variableHelp.profileSlug" },
  { token: "${fps}", descriptionKey: "recording.filename.variableHelp.fps" },
];
const customFpsDraft = ref(String(props.config.recording.customFps));
const syncFeedbackDurationDraft = ref(String(props.config.recording.syncFeedbackDurationMs));
const filenameTemplateDraft = ref(props.config.recording.filenameTemplate);
const filenamePanelOpen = ref(false);

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

function toggleFilenamePanel() {
  filenamePanelOpen.value = !filenamePanelOpen.value;
}

function insertFilenameVariable(token: string) {
  filenameTemplateDraft.value = `${filenameTemplateDraft.value}${token}`;
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
    <h2 class="m-0 mb-4 text-lg leading-6 tracking-normal">{{ t("recording.title") }}</h2>
    <BaseFieldRow :label="t('recording.saveFolder')">
      {{ recordingDirectory || t("recording.noFolderSelected") }}
    </BaseFieldRow>
    <div class="flex flex-wrap gap-2 my-4">
      <BaseButton @click="chooseRecordingDirectory">{{ t("recording.controls.chooseFolder") }}</BaseButton>
      <BaseButton
        variant="primary"
        :disabled="isRecording || recordingCountdown > 0"
        @click="startRecording"
      >
        {{ recordingCountdown > 0 ? t("recording.controls.startingIn", { seconds: recordingCountdown }) : t("recording.controls.start") }}
      </BaseButton>
      <BaseButton :disabled="!isRecording" @click="stopRecording">
        {{ t("recording.controls.stop") }}
      </BaseButton>
      <BaseButton :disabled="!isRecording" @click="addSyncMarker">
        {{ t("recording.controls.addSyncMarker") }}
      </BaseButton>
    </div>
    <BaseToggleRow :checked="silentRecording" @change="updateSilentRecording">
      {{ t("recording.silent") }}
    </BaseToggleRow>
    <div class="flex items-center flex-wrap gap-2.5 mb-4">
      <BaseToggleRow compact :checked="config.recording.syncFeedbackEnabled" @change="updateSyncFeedbackEnabled">
        {{ t("recording.syncFlash") }}
      </BaseToggleRow>
      <input
        :disabled="!config.recording.syncFeedbackEnabled"
        :min="100"
        :value="syncFeedbackDurationDraft"
        class="w-24 min-h-[34px] border border-border-control rounded-md bg-surface-control text-text-body px-2.5 disabled:opacity-45"
        type="number"
        @blur="commitSyncFeedbackDuration"
        @change="commitSyncFeedbackDuration"
        @input="updateSyncFeedbackDuration"
      />
      <span class="text-text-muted text-[13px] font-extrabold">{{ t("recording.milliseconds") }}</span>
    </div>
    <RecordingHotkeysPanel
      :recording-hotkeys="recordingHotkeys"
      :hotkey-capture-target="hotkeyCaptureTarget"
      @update-recording-hotkey-mode="emit('update-recording-hotkey-mode', $event)"
      @begin-hotkey-capture="emit('begin-hotkey-capture', $event)"
    />
    <div class="flex items-center flex-wrap gap-2 mb-4">
      <BaseSegmentedControl
        :model-value="selectedDefaultFps"
        :options="fpsOptions"
        :aria-label="t('recording.captureFrameRate')"
        @update:model-value="selectRecordingFps"
      />
      <BaseToggleRow compact :checked="config.recording.customFpsEnabled" @change="updateCustomFpsEnabled">
        {{ t("recording.customFps") }}
      </BaseToggleRow>
      <input
        :disabled="!config.recording.customFpsEnabled"
        :max="config.recording.maxFps"
        :min="1"
        :value="customFpsDraft"
        class="w-24 min-h-[34px] border border-border-control rounded-md bg-surface-control text-text-body px-2.5 disabled:opacity-45"
        type="number"
        @blur="commitCustomFps"
        @change="commitCustomFps"
        @input="updateCustomFps"
      />
      <span class="text-text-muted text-[13px] font-extrabold">
        {{ t("recording.rateSummary", {
          fps: activeRecordingFps,
          bytesPerSecond: formatBytesPerSecond(estimatedRecordingBytesPerSecond),
        }) }}
      </span>
    </div>
    <BaseFieldRow :label="t('recording.primaryArtifact')">{{ config.recording.formatExtension }}</BaseFieldRow>
    <div class="filename-config">
      <span class="filename-label">{{ t("recording.filename.label") }}</span>
      <button
        class="filename-trigger"
        type="button"
        aria-haspopup="dialog"
        :aria-expanded="filenamePanelOpen"
        @click="toggleFilenamePanel"
      >
        <span>{{ t("recording.filename.configure") }}</span>
        <strong>{{ filenameTemplateDraft }}</strong>
      </button>
      <Transition name="filename-panel">
        <section
          v-if="filenamePanelOpen"
          class="filename-panel"
          role="dialog"
          :aria-label="t('recording.filename.configure')"
          @keydown.esc.prevent.stop="filenamePanelOpen = false"
        >
          <label class="filename-template-field">
            <span>{{ t("recording.filename.template") }}</span>
            <input
              :value="filenameTemplateDraft"
              type="text"
              placeholder="${start}-${end}"
              @blur="commitFilenameTemplate"
              @change="commitFilenameTemplate"
              @input="updateFilenameTemplate"
            />
          </label>
          <div class="filename-variable-list">
            <span>{{ t("recording.filename.variables") }}</span>
            <button
              v-for="variable in recordingFilenameVariables"
              :key="variable.token"
              class="filename-variable"
              type="button"
              @click="insertFilenameVariable(variable.token)"
            >
              <code>{{ variable.token }}</code>
              <span>{{ t(variable.descriptionKey) }}</span>
            </button>
          </div>
          <div class="filename-panel-actions">
            <BaseButton @click="filenamePanelOpen = false">{{ t("recording.filename.close") }}</BaseButton>
          </div>
        </section>
      </Transition>
    </div>
    <p class="notice">
      {{ t("recording.storage.description") }}
    </p>
    <p v-if="lastRecordingPath" class="notice">
      {{ t("recording.storage.lastRecording", { path: lastRecordingPath }) }}
    </p>
    <p v-if="recordingStatusMessage" class="mt-2.5 text-text-secondary text-[13px] font-bold">
      {{ recordingStatusMessage }}
    </p>
  </BasePanel>
</template>

<style scoped>
.filename-config {
  position: relative;
  display: grid;
  gap: 7px;
  margin: 16px 0 14px;
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 700;
}

.filename-label,
.filename-template-field span,
.filename-variable-list > span {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 800;
}

.filename-trigger {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  align-items: center;
  gap: 12px;
  width: 100%;
  min-height: 38px;
  border: 1px solid rgba(255, 255, 255, 0.10);
  border-radius: 7px;
  background: var(--color-surface-control);
  color: var(--color-text-body);
  cursor: pointer;
  padding: 8px 10px;
  text-align: left;
}

.filename-trigger:hover {
  border-color: rgba(255, 255, 255, 0.16);
  background: var(--color-surface-control-hover);
}

.filename-trigger strong {
  min-width: 0;
  overflow: hidden;
  color: var(--color-accent-text);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.filename-panel {
  display: grid;
  gap: 12px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  box-shadow: 0 18px 42px rgba(0, 0, 0, 0.34);
  padding: 12px;
}

.filename-panel-enter-active,
.filename-panel-leave-active {
  transition:
    opacity 150ms ease,
    transform 170ms cubic-bezier(0.16, 1, 0.3, 1);
}

.filename-panel-enter-from,
.filename-panel-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.98);
}

.filename-template-field {
  display: grid;
  gap: 6px;
}

.filename-template-field input {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid rgba(255, 255, 255, 0.10);
  border-radius: 7px;
  background: #10141a;
  color: var(--color-text-body);
  font: inherit;
  padding: 8px 10px;
}

.filename-variable-list {
  display: grid;
  gap: 7px;
}

.filename-variable {
  display: grid;
  grid-template-columns: minmax(118px, max-content) minmax(0, 1fr);
  align-items: center;
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.035);
  color: var(--color-text-secondary);
  cursor: pointer;
  padding: 8px 9px;
  text-align: left;
}

.filename-variable:hover {
  border-color: rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.06);
}

.filename-variable code {
  color: var(--color-accent-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.filename-variable span {
  color: var(--color-text-muted);
  font-size: 12px;
}

.filename-panel-actions {
  display: flex;
  justify-content: flex-end;
}

@media (max-width: 720px) {
  .filename-trigger,
  .filename-variable {
    grid-template-columns: 1fr;
  }
}
</style>
