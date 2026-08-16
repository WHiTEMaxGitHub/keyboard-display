import { emitTo } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { ref, type ComputedRef, type Ref } from "vue";
import { tauriApi } from "../api/tauri";
import { i18n } from "../i18n";
import { OVERLAY_SYNC_FEEDBACK_EVENT } from "../domain/inputEvents";
import { effectiveRecordingFps } from "../domain/recordingConfig";
import {
  isHotkeyMatch,
  normalizeHotkey,
  normalizeRecordingHotkeyConfig,
  type RecordingHotkeyConfig,
  type RecordingHotkeyMode,
} from "../domain/recordingHotkeys";
import type { AppConfig } from "../domain/defaultConfig";
import type { RecordingInspection } from "../types/recording";
import type { OverlayPosition } from "./useOverlayWindow";

export type RecordingHotkeyTarget = "start" | "stop" | "sync";

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
  const activeRecordingHotkeySignature = ref("");

  function t(key: string, params?: Record<string, unknown>) {
    return i18n.global.t(key, params ?? {});
  }

  async function recordInputIfNeeded(keyId: string, pressed: boolean) {
    if (!options.enabled || options.isOverlayWindow.value || !isRecording.value) {
      return;
    }

    await tauriApi.recordInputEvent(keyId, pressed);
  }

  function effectiveRecordingHotkeys(): RecordingHotkeyConfig {
    return activeRecordingHotkeys.value ?? recordingHotkeys.value;
  }

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

  async function suppressRecordingHotkeyInput(keys: string[]) {
    if (!isRecording.value || keys.length === 0) {
      return;
    }

    const normalizedKeys = normalizeHotkey(keys);
    await tauriApi.suppressRecordingKeys(normalizedKeys);
    releaseSuppressedHotkeyKeys(normalizedKeys);
  }

  function releaseSuppressedHotkeyKeys(keys: string[]) {
    const nextActiveKeys = new Set(options.activeKeyIds.value);
    keys.forEach((key) => nextActiveKeys.delete(key));
    options.activeKeyIds.value = nextActiveKeys;
    activeRecordingHotkeySignature.value = "";
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
    if (!options.enabled) {
      return false;
    }

    if (hotkeyCaptureTarget.value) {
      return false;
    }

    const activeSignature = normalizeHotkey(options.activeKeyIds.value).join("+");
    if (activeSignature === activeRecordingHotkeySignature.value) {
      return false;
    }

    const hotkeys = effectiveRecordingHotkeys();
    const matchesStart = isHotkeyMatch(options.activeKeyIds.value, hotkeys.start);
    const matchesStop = isHotkeyMatch(options.activeKeyIds.value, hotkeys.stop);
    const matchesSync = isHotkeyMatch(options.activeKeyIds.value, hotkeys.sync);

    if (!matchesStart && !matchesStop && !matchesSync) {
      if (activeSignature === "") {
        activeRecordingHotkeySignature.value = "";
      }
      return false;
    }

    if (matchesSync && isRecording.value) {
      activeRecordingHotkeySignature.value = activeSignature;
      await suppressRecordingHotkeyInput(hotkeys.sync);
      await addSyncMarker();
      return true;
    }

    if (hotkeys.mode === "disabled") {
      return false;
    }

    if (hotkeys.mode === "toggle") {
      if (recordingCountdown.value > 0) {
        activeRecordingHotkeySignature.value = activeSignature;
        cancelRecordingCountdown();
        return true;
      }

      if (isRecording.value) {
        activeRecordingHotkeySignature.value = activeSignature;
        await suppressRecordingHotkeyInput(hotkeys.stop);
        await stopRecording();
      } else {
        activeRecordingHotkeySignature.value = activeSignature;
        await startRecordingWithCountdown();
      }
      return true;
    }

    if (hotkeys.mode === "separate") {
      if (!isRecording.value && matchesStart) {
        activeRecordingHotkeySignature.value = activeSignature;
        await startRecordingWithCountdown();
        return true;
      } else if (isRecording.value && matchesStop) {
        activeRecordingHotkeySignature.value = activeSignature;
        await suppressRecordingHotkeyInput(hotkeys.stop);
        await stopRecording();
        return true;
      }
    }

    return false;
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
  };
}
