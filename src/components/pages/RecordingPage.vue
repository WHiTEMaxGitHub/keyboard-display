<script setup lang="ts">
import { inject, ref } from "vue";
import type { AppConfig } from "../../domain/defaultConfig";
import type { RecordingHotkeyConfig } from "../../domain/recordingHotkeys";
import type { RecordingInspection } from "../../types/recording";
import RecordingPanel from "../RecordingPanel.vue";
import RecordingBrowserPanel from "../RecordingBrowserPanel.vue";

type RecordingSubPage = "control" | "files";

const config = inject<AppConfig>("config")!;
const recordingDirectory = inject<string>("recordingDirectory")!;
const defaultRecordingDirectory = inject<string>("defaultRecordingDirectory")!;
const recordingBrowserDirectory = inject<string>("recordingBrowserDirectory")!;
const silentRecording = inject<boolean>("silentRecording")!;
const isRecording = inject<boolean>("isRecording")!;
const recordingCountdown = inject<number>("recordingCountdown")!;
const lastRecordingPath = inject<string>("lastRecordingPath")!;
const recordingStatusMessage = inject<string>("recordingStatusMessage")!;
const currentRecordingPath = inject<string>("currentRecordingPath")!;
const recordingInspection = inject<RecordingInspection | null>("recordingInspection")!;
const recordingInspectionError = inject<string>("recordingInspectionError")!;
const recordingHotkeys = inject<RecordingHotkeyConfig>("recordingHotkeys")!;
const hotkeyCaptureTarget = inject<"start" | "stop" | "sync" | null>("hotkeyCaptureTarget")!;
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;

const recordingSubPage = ref<RecordingSubPage>("control");
</script>

<template>
  <section v-if="recordingSubPage === 'control'" class="page-stack">
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

  <section v-else-if="recordingSubPage === 'files'" class="page-stack">
    <RecordingBrowserPanel
      :recording-browser-directory="recordingBrowserDirectory"
      :current-recording-path="currentRecordingPath"
      :recording-inspection="recordingInspection"
      :recording-inspection-error="recordingInspectionError"
      @inspect-recording-file="emit('inspect-recording-file')"
      @inspect-recording-path="emit('inspect-recording-path', $event)"
      @clear-recording-inspection="emit('clear-recording-inspection')"
      @choose-recording-browser-directory="emit('choose-recording-browser-directory')"
    />
  </section>
</template>