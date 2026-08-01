<script setup lang="ts">
import { computed, inject, ref, type ComputedRef } from "vue";
import type { AppConfig } from "../../domain/defaultConfig";
import type { RecordingHotkeyConfig } from "../../domain/recordingHotkeys";
import type { RecordingInspection } from "../../types/recording";
import RecordingPanel from "../RecordingPanel.vue";
import RecordingBrowserPanel from "../RecordingBrowserPanel.vue";

type RecordingSubPage = "control" | "files";

const config = inject<AppConfig>("config")!;
const recordingDirectoryRef = inject<ComputedRef<string>>("recordingDirectory")!;
const defaultRecordingDirectoryRef = inject<ComputedRef<string>>("defaultRecordingDirectory")!;
const recordingBrowserDirectoryRef = inject<ComputedRef<string>>("recordingBrowserDirectory")!;
const silentRecordingRef = inject<ComputedRef<boolean>>("silentRecording")!;
const isRecordingRef = inject<ComputedRef<boolean>>("isRecording")!;
const recordingCountdownRef = inject<ComputedRef<number>>("recordingCountdown")!;
const lastRecordingPathRef = inject<ComputedRef<string>>("lastRecordingPath")!;
const recordingStatusMessageRef = inject<ComputedRef<string>>("recordingStatusMessage")!;
const currentRecordingPathRef = inject<ComputedRef<string>>("currentRecordingPath")!;
const recordingInspectionRef = inject<ComputedRef<RecordingInspection | null>>("recordingInspection")!;
const recordingInspectionErrorRef = inject<ComputedRef<string>>("recordingInspectionError")!;
const recordingHotkeysRef = inject<ComputedRef<RecordingHotkeyConfig>>("recordingHotkeys")!;
const hotkeyCaptureTargetRef = inject<ComputedRef<"start" | "stop" | "sync" | null>>("hotkeyCaptureTarget")!;
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;

const recordingSubPage = ref<RecordingSubPage>("control");
const recordingDirectory = computed(() => recordingDirectoryRef.value);
const defaultRecordingDirectory = computed(() => defaultRecordingDirectoryRef.value);
const recordingBrowserDirectory = computed(() => recordingBrowserDirectoryRef.value);
const silentRecording = computed(() => silentRecordingRef.value);
const isRecording = computed(() => isRecordingRef.value);
const recordingCountdown = computed(() => recordingCountdownRef.value);
const lastRecordingPath = computed(() => lastRecordingPathRef.value);
const recordingStatusMessage = computed(() => recordingStatusMessageRef.value);
const currentRecordingPath = computed(() => currentRecordingPathRef.value);
const recordingInspection = computed(() => recordingInspectionRef.value);
const recordingInspectionError = computed(() => recordingInspectionErrorRef.value);
const recordingHotkeys = computed(() => recordingHotkeysRef.value);
const hotkeyCaptureTarget = computed(() => hotkeyCaptureTargetRef.value);
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
