import { emitTo, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { ref, watch, type ComputedRef, type Ref } from "vue";
import { tauriApi } from "../api/tauri";
import { i18n } from "../i18n";
import { OVERLAY_SYNC_FEEDBACK_EVENT, RECORDING_UI_EVENT } from "../domain/inputEvents";
import { effectiveRecordingFps } from "../domain/recordingConfig";
import {
  normalizeHotkey,
  normalizeRecordingHotkeyConfig,
  type RecordingHotkeyConfig,
  type RecordingHotkeyMode,
} from "../domain/recordingHotkeys";
import type { AppConfig } from "../domain/defaultConfig";
import type { RecordingInspection } from "../types/recording";
import type { OverlayPosition } from "./useOverlayWindow";

export type RecordingHotkeyTarget = "start" | "stop" | "sync";

type RecordingUiPayload =
  | { type: "countdown"; remaining: number; fps: number }
  | { type: "started"; fps: number; silent: boolean }
  | { type: "stopped"; path: string; silent: boolean }
  | { type: "sync" }
  | { type: "countdownCancelled" }
  | { type: "error"; message: string };

type UseRecordingControllerOptions = {
  enabled: boolean;
  config: AppConfig;
  profileName: Ref<string>;
  isOverlayWindow: ComputedRef<boolean>;
  activeKeyIds: Ref<Set<string>>;
  isOverlayVisible: Ref<boolean>;
  overlayPosition: Ref<OverlayPosition>;
  destroyOverlayWindow: () => Promise<void>;
  setOverlayVisible: (visible: boolean, updateConfig?: boolean) => Promise<void>;
  moveOverlay: (position: OverlayPosition, updateConfig?: boolean, show?: boolean) => Promise<void>;
  scheduleAppConfigSave: () => void;
};

export function useRecordingController(options: UseRecordingControllerOptions) {
  const recordingDirectory = ref("");
  const defaultRecordingDirectory = ref("");
  const silentRecording = ref(false);
  const restoreOverlayAfterRecording = ref(false);
  const isRecording = ref(false);
  const recordingCountdown = ref(0);
  const recordingCountdownTimer = ref<number | null>(null);
  const lastRecordingPath = ref("");
  const recordingStatusMessage = ref("");
  const currentRecordingPath = ref("");
  const recordingInspection = ref<RecordingInspection | null>(null);
  const recordingInspectionError = ref("");
  const recordingHotkeys = ref<RecordingHotkeyConfig>(normalizeRecordingHotkeyConfig(undefined));
  const activeRecordingHotkeys = ref<RecordingHotkeyConfig | null>(null);
  const activeRecordingFps = ref<number | null>(null);
  const hotkeyCaptureTarget = ref<RecordingHotkeyTarget | null>(null);
  const capturedHotkeyKeys = ref(new Set<string>());
  let unlistenRecordingUi: UnlistenFn | undefined;

  function t(key: string, params?: Record<string, unknown>) {
    return i18n.global.t(key, params ?? {});
  }

  async function pushRecordingRuntime() {
    if (!options.enabled || options.isOverlayWindow.value) {
      return;
    }

    await tauriApi.syncRecordingRuntime({
      hotkeys: recordingHotkeys.value,
      outputDirectory: recordingDirectory.value,
      filenameTemplate: options.config.recording.filenameTemplate,
      profileName: options.profileName.value,
      fps: effectiveRecordingFps(options.config.recording),
      silent: silentRecording.value,
      syncFeedbackEnabled: options.config.recording.syncFeedbackEnabled,
      syncFeedbackDurationMs: options.config.recording.syncFeedbackDurationMs,
    });
  }

  async function applyRecordingUiEvent(payload: RecordingUiPayload) {
    if (!options.enabled || options.isOverlayWindow.value) {
      return;
    }

    if (payload.type === "countdown") {
      recordingCountdown.value = payload.remaining;
      recordingStatusMessage.value = t("recording.status.willStart", { fps: payload.fps });
      return;
    }

    if (payload.type === "started") {
      cancelRecordingCountdown();
      activeRecordingHotkeys.value = { ...recordingHotkeys.value };
      activeRecordingFps.value = payload.fps;
      isRecording.value = true;
      lastRecordingPath.value = "";
      recordingStatusMessage.value = t("recording.status.started", { fps: payload.fps });
      restoreOverlayAfterRecording.value = options.isOverlayVisible.value;
      if (payload.silent && silentRecording.value) {
        await options.destroyOverlayWindow();
      }
      return;
    }

    if (payload.type === "stopped") {
      isRecording.value = false;
      activeRecordingHotkeys.value = null;
      activeRecordingFps.value = null;
      lastRecordingPath.value = payload.path;
      recordingStatusMessage.value = t("recording.status.saved", { path: payload.path });
      if (payload.silent && silentRecording.value && restoreOverlayAfterRecording.value) {
        await options.setOverlayVisible(true, false);
        await options.moveOverlay(options.overlayPosition.value, false);
      }
      restoreOverlayAfterRecording.value = false;
      return;
    }

    if (payload.type === "sync") {
      recordingStatusMessage.value = t("recording.status.syncMarkerAdded");
      return;
    }

    if (payload.type === "countdownCancelled") {
      cancelRecordingCountdown();
      recordingStatusMessage.value = "";
      return;
    }

    recordingStatusMessage.value = payload.message;
  }

  // Key events are recorded in Rust from InputStateBridge::apply_key, at the
  // same timestamp as the overlay snapshot. Do not invoke record_input_event
  // here — that hop delayed jump-throw keys relative to the live overlay.
  async function recordInputIfNeeded(_keyId: string, _pressed: boolean) {  }

  async function resolveDefaultRecordingDirectory() {
    if (!defaultRecordingDirectory.value) {
      defaultRecordingDirectory.value = await tauriApi.defaultRecordingDir();
    }

    return defaultRecordingDirectory.value;
  }

  async function chooseRecordingDirectory(): Promise<boolean> {
    if (!options.enabled) {
      return false;
    }

    const selectedPath = await open({
      title: t("recording.dialog.chooseFolder"),
      directory: true,
      multiple: false,
    });

    if (typeof selectedPath === "string") {
      recordingDirectory.value = selectedPath;
      recordingStatusMessage.value = "";
      options.scheduleAppConfigSave();
      return true;
    }

    return false;
  }

  async function resolveRecordingDirectory(): Promise<string> {
    if (recordingDirectory.value) {
      return recordingDirectory.value;
    }

    const defaultDirectory = await resolveDefaultRecordingDirectory();
    recordingDirectory.value = defaultDirectory;
    recordingStatusMessage.value = t("recording.status.usingDefaultFolder", {
      path: defaultDirectory,
    });
    options.scheduleAppConfigSave();

    return defaultDirectory;
  }

  /// 启动录制前保留倒计时，避免用户按下控制热键本身被录入开头帧。
  async function startRecordingWithCountdown() {
    if (!options.enabled) {
      return;
    }

    await resolveRecordingDirectory();
    await pushRecordingRuntime();

    if (isRecording.value || recordingCountdown.value > 0) {
      return;
    }

    const recordingFps = effectiveRecordingFps(options.config.recording);
    recordingStatusMessage.value = t("recording.status.willStart", { fps: recordingFps });
    recordingCountdown.value = 3;

    recordingCountdownTimer.value = window.setInterval(async () => {
      recordingCountdown.value -= 1;

      if (recordingCountdown.value <= 0) {
        cancelRecordingCountdown();
        activeRecordingHotkeys.value = { ...recordingHotkeys.value };
        await tauriApi.startRecording(recordingFps);
        restoreOverlayAfterRecording.value = options.isOverlayVisible.value;
        if (silentRecording.value) {
          await options.destroyOverlayWindow();
        }
        activeRecordingFps.value = recordingFps;
        isRecording.value = true;
        lastRecordingPath.value = "";
        recordingStatusMessage.value = t("recording.status.started", { fps: recordingFps });
      }
    }, 1000);
  }

  function cancelRecordingCountdown() {
    if (recordingCountdownTimer.value !== null) {
      window.clearInterval(recordingCountdownTimer.value);
      recordingCountdownTimer.value = null;
    }
    recordingCountdown.value = 0;
    activeRecordingFps.value = null;
  }

  async function stopRecording() {
    if (!options.enabled || !isRecording.value) {
      return;
    }

    const recordingFps = activeRecordingFps.value ?? effectiveRecordingFps(options.config.recording);
    const recordingOutputDirectory = await resolveRecordingDirectory();
    const result = await tauriApi.stopRecording(
      recordingOutputDirectory,
      options.config.recording.filenameTemplate,
      options.profileName.value,
      recordingFps,
    );
    isRecording.value = false;
    activeRecordingHotkeys.value = null;
    activeRecordingFps.value = null;
    lastRecordingPath.value = result.path;
    recordingStatusMessage.value = t("recording.status.saved", { path: result.path });

    if (silentRecording.value && restoreOverlayAfterRecording.value) {
      await options.setOverlayVisible(true, false);
      await options.moveOverlay(options.overlayPosition.value, false);
    }
    restoreOverlayAfterRecording.value = false;
  }

  async function inspectRecordingFile() {
    if (!options.enabled) {
      return;
    }

    const selectedPath = await open({
      title: t("recording.dialog.inspectRecording"),
      filters: [{ name: t("recording.dialog.recordingFilter"), extensions: ["kbdrec"] }],
      multiple: false,
    });

    if (typeof selectedPath !== "string") {
      return;
    }

    await inspectRecordingPath(selectedPath);
  }

  async function inspectRecordingPath(selectedPath: string) {
    if (!options.enabled) {
      return;
    }

    currentRecordingPath.value = selectedPath;
    recordingInspection.value = null;
    recordingInspectionError.value = "";

    try {
      recordingInspection.value = await tauriApi.inspectRecordingFile(selectedPath);
    } catch (error) {
      recordingInspectionError.value = String(error);
    }
  }

  function clearRecordingInspection() {
    currentRecordingPath.value = "";
    recordingInspection.value = null;
    recordingInspectionError.value = "";
  }

  function updateSilentRecording(value: boolean) {
    if (!options.enabled) {
      return;
    }

    silentRecording.value = value;
    options.scheduleAppConfigSave();
  }

  function updateRecordingHotkeyMode(mode: RecordingHotkeyMode) {
    if (!options.enabled) {
      return;
    }

    recordingHotkeys.value = normalizeRecordingHotkeyConfig({
      mode,
      start: recordingHotkeys.value.start,
      stop: mode === "separate" ? undefined : recordingHotkeys.value.start,
      sync: recordingHotkeys.value.sync,
    });
    options.scheduleAppConfigSave();
  }

  async function addSyncMarker() {
    if (!options.enabled) {
      return;
    }

    if (!isRecording.value) {
      recordingStatusMessage.value = t("recording.status.startBeforeMarker");
      return;
    }

    await tauriApi.addRecordingMarker("sync");
    if (options.config.recording.syncFeedbackEnabled) {
      await emitTo("pov", OVERLAY_SYNC_FEEDBACK_EVENT, {
        durationMs: options.config.recording.syncFeedbackDurationMs,
      });
    }
    recordingStatusMessage.value = t("recording.status.syncMarkerAdded");
  }

  function beginHotkeyCapture(target: RecordingHotkeyTarget) {
    capturedHotkeyKeys.value = new Set();
    hotkeyCaptureTarget.value = target;
  }

  function captureHotkeyKey(keyId: string) {
    if (hotkeyCaptureTarget.value) {
      capturedHotkeyKeys.value = new Set([...capturedHotkeyKeys.value, keyId]);
    }
  }

  function finishHotkeyCapture() {
    const target = hotkeyCaptureTarget.value;

    if (!target || capturedHotkeyKeys.value.size === 0) {
      hotkeyCaptureTarget.value = null;
      capturedHotkeyKeys.value = new Set();
      return;
    }

    recordingHotkeys.value = {
      ...recordingHotkeys.value,
      [target]: normalizeHotkey(capturedHotkeyKeys.value),
    };
    options.scheduleAppConfigSave();
    hotkeyCaptureTarget.value = null;
    capturedHotkeyKeys.value = new Set();
  }

  async function handleRecordingHotkeys(): Promise<boolean> {
    // Start/stop/sync are evaluated in Rust from keyboard Raw Input (same
    // apply_key path as the overlay) so they work while CS2 is focused.
    // Config-window input-state must not also match: a focused app would
    // start the Rust countdown then immediately cancel it via toggle.
    return false;
  }

  if (options.enabled && !options.isOverlayWindow.value) {
    watch(
      () => ({
        hotkeys: recordingHotkeys.value,
        outputDirectory: recordingDirectory.value,
        silent: silentRecording.value,
        profileName: options.profileName.value,
        filenameTemplate: options.config.recording.filenameTemplate,
        fps: effectiveRecordingFps(options.config.recording),
        syncFeedbackEnabled: options.config.recording.syncFeedbackEnabled,
        syncFeedbackDurationMs: options.config.recording.syncFeedbackDurationMs,
      }),
      () => {
        void pushRecordingRuntime();
      },
      { deep: true },
    );

    void listen<RecordingUiPayload>(RECORDING_UI_EVENT, (event) => {
      void applyRecordingUiEvent(event.payload);
    }).then((unlisten) => {
      unlistenRecordingUi = unlisten;
    });
  }

  function stopRecordingUiBridge() {
    unlistenRecordingUi?.();
    unlistenRecordingUi = undefined;
  }

  return {
    recordingDirectory,
    defaultRecordingDirectory,
    silentRecording,
    isRecording,
    recordingCountdown,
    lastRecordingPath,
    recordingStatusMessage,
    currentRecordingPath,
    recordingInspection,
    recordingInspectionError,
    recordingHotkeys,
    hotkeyCaptureTarget,
    recordInputIfNeeded,
    chooseRecordingDirectory,
    startRecordingWithCountdown,
    cancelRecordingCountdown,
    stopRecording,
    inspectRecordingFile,
    inspectRecordingPath,
    clearRecordingInspection,
    updateSilentRecording,
    updateRecordingHotkeyMode,
    addSyncMarker,
    beginHotkeyCapture,
    captureHotkeyKey,
    finishHotkeyCapture,
    handleRecordingHotkeys,
    stopRecordingUiBridge,
  };
}
