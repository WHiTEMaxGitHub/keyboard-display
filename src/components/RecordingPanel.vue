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
  defaultRecordingDirectory: string;
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
    <h2 class="m-0 mb-4 text-lg leading-6 tracking-normal">{{ t("recording.title") }}</h2>
    <BaseFieldRow :label="t('recording.saveFolder')">
      {{
        recordingDirectory ||
          t("recording.defaultFolder", {
            path: defaultRecordingDirectory || t("recording.loading"),
          })
      }}
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
    <label class="grid gap-[7px] mt-4 mb-4 text-text-secondary text-[13px] font-bold">
      <span>{{ t("recording.filename.label") }}</span>
      <input
        :value="filenameTemplateDraft"
        type="text"
        placeholder="${start}-${end}"
        class="w-full min-h-[34px] border border-border-control rounded-md bg-surface-control text-text-body font-inherit px-2.5 focus:border-accent-focus-border focus:outline-2 focus:outline-accent-focus-ring focus:outline-offset-0"
        @blur="commitFilenameTemplate"
        @change="commitFilenameTemplate"
        @input="updateFilenameTemplate"
      />
    </label>
    <p class="notice">
      {{ t("recording.filename.variables") }}
    </p>
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
