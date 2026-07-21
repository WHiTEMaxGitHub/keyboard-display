import { emitTo } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { ref, type ComputedRef, type Ref } from "vue";
import { tauriApi } from "../api/tauri";
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

export type RecordingTrigger = "manual" | "hotkey";
export type RecordingHotkeyTarget = "start" | "stop" | "sync";

type UseRecordingControllerOptions = {
  config: AppConfig;
  profileName: Ref<string>;
  isOverlayWindow: ComputedRef<boolean>;
  activeKeyIds: Ref<Set<string>>;
  isOverlayVisible: Ref<boolean>;
  overlayPosition: Ref<OverlayPosition>;
  destroyOverlayWindow: () => Promise<void>;
  setOverlayVisible: (visible: boolean, markChanged?: boolean) => Promise<void>;
  moveOverlay: (position: OverlayPosition, markChanged?: boolean, show?: boolean) => Promise<void>;
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
  const inspectedRecordingPath = ref("");
  const recordingInspection = ref<RecordingInspection | null>(null);
  const recordingInspectionError = ref("");
  const recordingHotkeys = ref<RecordingHotkeyConfig>(normalizeRecordingHotkeyConfig(undefined));
  const activeRecordingHotkeys = ref<RecordingHotkeyConfig | null>(null);
  const hotkeyCaptureTarget = ref<RecordingHotkeyTarget | null>(null);
  const capturedHotkeyKeys = ref(new Set<string>());
  const activeRecordingHotkeySignature = ref("");

  async function initializeDefaultRecordingDirectory() {
    defaultRecordingDirectory.value = await tauriApi.defaultRecordingDir();
  }

  async function recordInputIfNeeded(keyId: string, pressed: boolean) {
    if (options.isOverlayWindow.value || !isRecording.value) {
      return;
    }

    await tauriApi.recordInputEvent(keyId, pressed);
  }

  function effectiveRecordingHotkeys(): RecordingHotkeyConfig {
    return activeRecordingHotkeys.value ?? recordingHotkeys.value;
  }

  async function chooseRecordingDirectory() {
    const selectedPath = await open({
      title: "Choose recording folder",
      directory: true,
      multiple: false,
    });

    if (typeof selectedPath === "string") {
      recordingDirectory.value = selectedPath;
      recordingStatusMessage.value = "";
      options.scheduleAppConfigSave();
    }
  }

  async function resolveRecordingDirectory(): Promise<string> {
    if (recordingDirectory.value) {
      return recordingDirectory.value;
    }

    const defaultDirectory =
      defaultRecordingDirectory.value || (await tauriApi.defaultRecordingDir());
    defaultRecordingDirectory.value = defaultDirectory;
    recordingDirectory.value = defaultDirectory;
    recordingStatusMessage.value = `Using default save folder: ${defaultDirectory}`;
    options.scheduleAppConfigSave();

    return defaultDirectory;
  }

  /// 启动录制前保留倒计时，避免用户按下控制热键本身被录入开头帧。
  async function startRecordingWithCountdown(trigger: RecordingTrigger = "manual") {
    await resolveRecordingDirectory();

    if (isRecording.value || recordingCountdown.value > 0) {
      return;
    }

    const recordingFps = effectiveRecordingFps(options.config.recording);
    recordingStatusMessage.value = `Recording will start at ${recordingFps}fps.`;
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
        if (trigger === "hotkey") {
          await tauriApi.addRecordingMarker("hotkey-start");
        }
        isRecording.value = true;
        lastRecordingPath.value = "";
        recordingStatusMessage.value = `Recording started at ${recordingFps}fps.`;
      }
    }, 1000);
  }

  function cancelRecordingCountdown() {
    if (recordingCountdownTimer.value !== null) {
      window.clearInterval(recordingCountdownTimer.value);
      recordingCountdownTimer.value = null;
    }
    recordingCountdown.value = 0;
  }

  async function stopRecording(trigger: RecordingTrigger = "manual") {
    if (!isRecording.value) {
      return;
    }

    if (trigger === "hotkey") {
      await tauriApi.addRecordingMarker("hotkey-stop");
    }

    const result = await tauriApi.stopRecording(
      await resolveRecordingDirectory(),
      options.config.recording.filenameTemplate,
      options.profileName.value,
      effectiveRecordingFps(options.config.recording),
    );
    isRecording.value = false;
    activeRecordingHotkeys.value = null;
    lastRecordingPath.value = result.path;
    recordingStatusMessage.value = `Recording saved: ${result.path}`;

    if (silentRecording.value && restoreOverlayAfterRecording.value) {
      await options.setOverlayVisible(true);
      await options.moveOverlay(options.overlayPosition.value);
    }
    restoreOverlayAfterRecording.value = false;
  }

  async function inspectRecordingFile() {
    const selectedPath = await open({
      title: "Inspect keyboard recording",
      filters: [{ name: "Keyboard recording", extensions: ["kbdrec"] }],
      multiple: false,
    });

    if (typeof selectedPath !== "string") {
      return;
    }

    await inspectRecordingPath(selectedPath);
  }

  async function inspectRecordingPath(selectedPath: string) {
    inspectedRecordingPath.value = selectedPath;
    recordingInspection.value = null;
    recordingInspectionError.value = "";

    try {
      recordingInspection.value = await tauriApi.inspectRecordingFile(selectedPath);
    } catch (error) {
      recordingInspectionError.value = String(error);
    }
  }

  function updateSilentRecording(value: boolean) {
    silentRecording.value = value;
    options.scheduleAppConfigSave();
  }

  function updateRecordingHotkeyMode(mode: RecordingHotkeyMode) {
    recordingHotkeys.value = normalizeRecordingHotkeyConfig({
      mode,
      start: recordingHotkeys.value.start,
      stop: mode === "separate" ? undefined : recordingHotkeys.value.start,
      sync: recordingHotkeys.value.sync,
    });
    options.scheduleAppConfigSave();
  }

  async function addSyncMarker() {
    if (!isRecording.value) {
      recordingStatusMessage.value = "Start recording before adding a sync marker.";
      return;
    }

    await tauriApi.addRecordingMarker("sync");
    if (options.config.recording.syncFeedbackEnabled) {
      await emitTo("pov", OVERLAY_SYNC_FEEDBACK_EVENT, {
        durationMs: options.config.recording.syncFeedbackDurationMs,
      });
    }
    recordingStatusMessage.value = "Sync marker added.";
  }

  async function suppressRecordingHotkeyInput(keys: string[]) {
    if (!isRecording.value || keys.length === 0) {
      return;
    }

    await tauriApi.suppressRecordingKeys(normalizeHotkey(keys));
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

    activeRecordingHotkeySignature.value = activeSignature;

    if (matchesSync && isRecording.value) {
      await suppressRecordingHotkeyInput(hotkeys.sync);
      await addSyncMarker();
      return true;
    }

    if (hotkeys.mode === "disabled") {
      return false;
    }

    if (hotkeys.mode === "toggle") {
      if (recordingCountdown.value > 0) {
        cancelRecordingCountdown();
        return true;
      }

      if (isRecording.value) {
        await suppressRecordingHotkeyInput(hotkeys.stop);
        await stopRecording("hotkey");
      } else {
        await startRecordingWithCountdown("hotkey");
      }
      return true;
    }

    if (hotkeys.mode === "separate") {
      if (!isRecording.value && matchesStart) {
        await startRecordingWithCountdown("hotkey");
        return true;
      } else if (isRecording.value && matchesStop) {
        await suppressRecordingHotkeyInput(hotkeys.stop);
        await stopRecording("hotkey");
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
    inspectedRecordingPath,
    recordingInspection,
    recordingInspectionError,
    recordingHotkeys,
    hotkeyCaptureTarget,
    initializeDefaultRecordingDirectory,
    recordInputIfNeeded,
    chooseRecordingDirectory,
    startRecordingWithCountdown,
    cancelRecordingCountdown,
    stopRecording,
    inspectRecordingFile,
    inspectRecordingPath,
    updateSilentRecording,
    updateRecordingHotkeyMode,
    addSyncMarker,
    beginHotkeyCapture,
    captureHotkeyKey,
    finishHotkeyCapture,
    handleRecordingHotkeys,
  };
}
