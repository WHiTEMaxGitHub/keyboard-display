<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { emitTo, listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  LogicalSize,
  PhysicalPosition,
  Window,
  currentMonitor,
  primaryMonitor,
} from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open, save } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, onUnmounted, reactive, ref, watch, type WatchStopHandle } from "vue";
import ConfigPanel from "./components/ConfigPanel.vue";
import OverlayWindow from "./components/OverlayWindow.vue";
import { buildAppConfigFile, parseAppConfigFile } from "./domain/appConfig";
import { buildConfigFileJson, parseConfigFile } from "./domain/configFile";
import {
  createDefaultConfig,
  flattenRowKeys,
  type OverlayStyle,
} from "./domain/defaultConfig";
import { estimateOverlaySize } from "./domain/overlaySize";
import { effectiveRecordingFps } from "./domain/recordingConfig";
import {
  INPUT_STATE_EVENT,
  OVERLAY_CONFIG_EVENT,
  OVERLAY_STYLE_EVENT,
  OVERLAY_VISIBLE_EVENT,
  keyIdFromKeyboardCode,
  keyIdFromMouseButton,
  type InputStatePayload,
} from "./domain/inputEvents";
import {
  isHotkeyMatch,
  normalizeHotkey,
  normalizeRecordingHotkeyConfig,
  type RecordingHotkeyConfig,
  type RecordingHotkeyMode,
} from "./domain/recordingHotkeys";
import type { RecordingConfig } from "./domain/defaultConfig";

type OverlayPosition = "top-left" | "top-right" | "bottom-left" | "bottom-right" | "center";

const config = reactive(createDefaultConfig());
const activeKeyIds = ref(new Set<string>());
const isOverlayVisible = ref(true);
const profileName = ref("CS POV");
const profileSourcePath = ref<string | null>(null);
const overlayPosition = ref<OverlayPosition>("bottom-right");
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
const hotkeyCaptureTarget = ref<"start" | "stop" | "sync" | null>(null);
const capturedHotkeyKeys = ref(new Set<string>());
const activeRecordingHotkeySignature = ref("");

const isOverlayWindow = computed(() => {
  return new URLSearchParams(window.location.search).get("surface") === "pov";
});

let unlistenInputState: UnlistenFn | undefined;
let unlistenOverlayStyle: UnlistenFn | undefined;
let appConfigSaveTimer: number | undefined;
let stopAppConfigWatch: WatchStopHandle | undefined;

type OverlayRuntimeConfig = {
  layout: typeof config.layout;
  rows: typeof config.rows;
  keys: typeof config.keys;
  style: OverlayStyle;
};

type RecordingInspectionEvent =
  | { t: number; down: string }
  | { t: number; up: string }
  | { t: number; marker: string };

type RecordingInspectionFrame = {
  t: number;
  keys: string[];
};

type RecordingInspection = {
  version: number;
  fps: number;
  keyIds: string[];
  events: RecordingInspectionEvent[];
  frames: RecordingInspectionFrame[];
};

function updateActiveKey(keyId: string, pressed: boolean) {
  const nextKeys = new Set(activeKeyIds.value);

  if (pressed) {
    nextKeys.add(keyId);
  } else {
    nextKeys.delete(keyId);
  }

  activeKeyIds.value = nextKeys;
}

async function recordInputIfNeeded(keyId: string, pressed: boolean) {
  if (isOverlayWindow.value || !isRecording.value) {
    return;
  }

  await invoke("record_input_event", { keyId, pressed });
}

function effectiveRecordingHotkeys(): RecordingHotkeyConfig {
  return activeRecordingHotkeys.value ?? recordingHotkeys.value;
}

function applyOverlayStyle(style: OverlayStyle) {
  config.style = { ...style };
}

function applyOverlayLayout(layout: typeof config.layout) {
  config.layout = { ...layout };
}

function applyOverlayRows(rows: typeof config.rows) {
  config.rows = rows.map((row) => row.map((item) => ({ ...item })));
  config.keys = flattenRowKeys(config.rows);
}

function applyRecordingConfig(recording: RecordingConfig) {
  config.recording = { ...recording };
}

async function updateOverlayStyle(style: OverlayStyle) {
  applyOverlayStyle(style);
  const overlayWindow = await Window.getByLabel("pov");
  await resizeOverlayWindow(overlayWindow);
  await overlayWindow?.setAlwaysOnTop(style.alwaysOnTop);
  await emitTo<OverlayStyle>("pov", OVERLAY_STYLE_EVENT, style);
}

async function resizeOverlayWindow(overlayWindow?: Window | null) {
  const targetWindow = overlayWindow ?? (await Window.getByLabel("pov"));

  if (!targetWindow) {
    return;
  }

  const size = estimateOverlaySize(config.layout, config.rows, config.style);
  await targetWindow.setSize(new LogicalSize(size.width, size.height));
}

async function ensureOverlayWindow(): Promise<Window | null> {
  const existingWindow = await WebviewWindow.getByLabel("pov");
  if (existingWindow) {
    return existingWindow;
  }

  const size = estimateOverlaySize(config.layout, config.rows, config.style);
  const createdWindow = new WebviewWindow("pov", {
    url: "/?surface=pov",
    title: "POV Overlay",
    width: size.width,
    height: size.height,
    decorations: false,
    transparent: true,
    backgroundColor: [0, 0, 0, 0],
    shadow: false,
    alwaysOnTop: config.style.alwaysOnTop,
    visible: false,
    resizable: false,
    skipTaskbar: true,
    visibleOnAllWorkspaces: true,
  });

  await new Promise<void>((resolve, reject) => {
    createdWindow.once("tauri://created", () => resolve());
    createdWindow.once("tauri://error", (event) => reject(event.payload));
  });

  return createdWindow;
}

async function syncOverlayWindow(overlayWindow?: Window | null) {
  const targetWindow = overlayWindow ?? (await ensureOverlayWindow());
  if (!targetWindow) {
    return;
  }

  await resizeOverlayWindow(targetWindow);
  await targetWindow.setAlwaysOnTop(config.style.alwaysOnTop);
  await emitTo<OverlayRuntimeConfig>("pov", OVERLAY_CONFIG_EVENT, {
    layout: config.layout,
    rows: config.rows,
    keys: config.keys,
    style: config.style,
  });
}

async function destroyOverlayWindow() {
  const overlayWindow = await Window.getByLabel("pov");
  await overlayWindow?.destroy();
}

async function setOverlayVisible(visible: boolean) {
  const overlayWindow = visible ? await ensureOverlayWindow() : await Window.getByLabel("pov");

  if (!overlayWindow) {
    isOverlayVisible.value = false;
    return;
  }

  if (visible) {
    await syncOverlayWindow(overlayWindow);
    await overlayWindow.show();
  } else {
    await overlayWindow.hide();
  }

  isOverlayVisible.value = visible;
  await emitTo<boolean>("pov", OVERLAY_VISIBLE_EVENT, visible);
}

async function moveOverlay(position: OverlayPosition) {
  overlayPosition.value = position;
  const overlayWindow = await ensureOverlayWindow();
  const monitor = (await currentMonitor()) ?? (await primaryMonitor());

  if (!overlayWindow || !monitor) {
    return;
  }

  await resizeOverlayWindow(overlayWindow);

  const margin = 24;
  const overlaySize = await overlayWindow.outerSize();
  const workArea = monitor.workArea;
  const xMin = workArea.position.x + margin;
  const yMin = workArea.position.y + margin;
  const xMax = workArea.position.x + workArea.size.width - overlaySize.width - margin;
  const yMax = workArea.position.y + workArea.size.height - overlaySize.height - margin;

  const positions: Record<OverlayPosition, PhysicalPosition> = {
    "top-left": new PhysicalPosition(xMin, yMin),
    "top-right": new PhysicalPosition(xMax, yMin),
    "bottom-left": new PhysicalPosition(xMin, yMax),
    "bottom-right": new PhysicalPosition(xMax, yMax),
    center: new PhysicalPosition((xMin + xMax) / 2, (yMin + yMax) / 2),
  };

  await overlayWindow.show();
  await overlayWindow.setPosition(positions[position]);
  isOverlayVisible.value = true;
  await emitTo<boolean>("pov", OVERLAY_VISIBLE_EVENT, true);
}

async function applyLoadedConfig(text: string, fileName: string, sourcePath: string | null) {
  const loadedConfig = parseConfigFile(text);
  profileName.value = loadedConfig.name || profileNameFromFileName(fileName);
  profileSourcePath.value = sourcePath;
  overlayPosition.value = (loadedConfig.overlay.position as OverlayPosition) ?? "bottom-right";

  applyOverlayLayout(loadedConfig.overlay.layout);
  applyOverlayRows(loadedConfig.overlay.rows);
  applyOverlayStyle(loadedConfig.overlay.style);
  applyRecordingConfig(loadedConfig.recording);
  await resizeOverlayWindow();

  await emitTo<OverlayRuntimeConfig>("pov", OVERLAY_CONFIG_EVENT, {
    layout: loadedConfig.overlay.layout,
    rows: loadedConfig.overlay.rows,
    keys: loadedConfig.overlay.keys,
    style: loadedConfig.overlay.style,
  });
  const visible = loadedConfig.overlay.visible ?? true;
  await setOverlayVisible(visible);
  if (visible) {
    await moveOverlay(overlayPosition.value);
  }
}

async function loadConfig() {
  const selectedPath = await open({
    title: "Load keyboard display config",
    filters: [{ name: "JSON", extensions: ["json"] }],
    multiple: false,
  });

  if (typeof selectedPath !== "string") {
    return;
  }

  const text = await invoke<string>("read_config_file", { path: selectedPath });
  await applyLoadedConfig(text, selectedPath.split(/[\\/]/).pop() ?? selectedPath, selectedPath);
}

async function restoreAppConfig() {
  defaultRecordingDirectory.value = await invoke<string>("default_recording_dir");
  const savedConfig = await invoke<string | null>("load_app_config");
  if (!savedConfig) {
    return;
  }

  const appConfig = parseAppConfigFile(savedConfig);
  profileName.value = appConfig.currentProfile.name;
  profileSourcePath.value = appConfig.currentProfile.sourcePath;
  overlayPosition.value = appConfig.currentProfile.overlay.position as OverlayPosition;
  recordingDirectory.value = appConfig.recording.outputDirectory ?? "";
  silentRecording.value = appConfig.recording.silent ?? false;
  recordingHotkeys.value = appConfig.recording.hotkeys;

  applyOverlayLayout(appConfig.currentProfile.overlay.layout);
  applyOverlayRows(appConfig.currentProfile.overlay.rows);
  applyOverlayStyle(appConfig.currentProfile.overlay.style);
  applyRecordingConfig(appConfig.currentProfile.recording);

  await emitTo<OverlayRuntimeConfig>("pov", OVERLAY_CONFIG_EVENT, {
    layout: appConfig.currentProfile.overlay.layout,
    rows: appConfig.currentProfile.overlay.rows,
    keys: appConfig.currentProfile.overlay.keys,
    style: appConfig.currentProfile.overlay.style,
  });

  await setOverlayVisible(appConfig.currentProfile.overlay.visible);
  if (appConfig.currentProfile.overlay.visible) {
    await moveOverlay(overlayPosition.value);
  }
}

function scheduleAppConfigSave() {
  if (isOverlayWindow.value) {
    return;
  }

  if (appConfigSaveTimer !== undefined) {
    window.clearTimeout(appConfigSaveTimer);
  }

  appConfigSaveTimer = window.setTimeout(() => {
    void saveAppConfig();
  }, 300);
}

async function saveAppConfig() {
  const appConfig = buildAppConfigFile({
    defaultProfilePath: "docs/default-config.json",
    currentProfile: {
      name: profileName.value,
      sourcePath: profileSourcePath.value,
      dirty: true,
      recording: config.recording,
      overlay: {
        visible: isOverlayVisible.value,
        position: overlayPosition.value,
        layout: config.layout,
        style: config.style,
        rows: config.rows,
        keys: config.keys,
      },
    },
    recording: {
      outputDirectory: recordingDirectory.value || null,
      silent: silentRecording.value,
      hotkeys: recordingHotkeys.value,
    },
  });

  await invoke("save_app_config", {
    contents: `${JSON.stringify(appConfig, null, 2)}\n`,
  });
}

async function applyConfigToOverlay() {
  await resizeOverlayWindow();
  await emitTo<OverlayRuntimeConfig>("pov", OVERLAY_CONFIG_EVENT, {
    layout: config.layout,
    rows: config.rows,
    keys: config.keys,
    style: config.style,
  });
  await setOverlayVisible(isOverlayVisible.value);
}

async function exportAndApplyConfig() {
  await applyConfigToOverlay();

  const json = buildConfigFileJson({
    name: profileName.value,
    config,
    visible: isOverlayVisible.value,
    position: overlayPosition.value,
  });
  const path = await save({
    title: "Save keyboard display config",
    defaultPath: `${profileName.value || "keyboard-display"}.json`,
    filters: [{ name: "JSON", extensions: ["json"] }],
  });

  if (!path) {
    return;
  }

  await invoke("save_config_file", { path, contents: json });
  profileSourcePath.value = path;
  scheduleAppConfigSave();
}

async function overwriteAndApplyConfig() {
  await applyConfigToOverlay();

  if (!profileSourcePath.value) {
    await exportAndApplyConfig();
    return;
  }

  const json = buildConfigFileJson({
    name: profileName.value,
    config,
    visible: isOverlayVisible.value,
    position: overlayPosition.value,
  });
  await invoke("save_config_file", { path: profileSourcePath.value, contents: json });
  scheduleAppConfigSave();
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
    scheduleAppConfigSave();
  }
}

async function resolveRecordingDirectory(): Promise<string> {
  if (recordingDirectory.value) {
    return recordingDirectory.value;
  }

  const defaultDirectory =
    defaultRecordingDirectory.value || (await invoke<string>("default_recording_dir"));
  defaultRecordingDirectory.value = defaultDirectory;
  recordingDirectory.value = defaultDirectory;
  recordingStatusMessage.value = `Using default save folder: ${defaultDirectory}`;
  scheduleAppConfigSave();

  return defaultDirectory;
}

async function startRecordingWithCountdown(trigger: "manual" | "hotkey" = "manual") {
  await resolveRecordingDirectory();

  if (isRecording.value || recordingCountdown.value > 0) {
    return;
  }

  recordingStatusMessage.value = "";
  recordingCountdown.value = 3;

  recordingCountdownTimer.value = window.setInterval(async () => {
    recordingCountdown.value -= 1;

    if (recordingCountdown.value <= 0) {
      cancelRecordingCountdown();
      activeRecordingHotkeys.value = { ...recordingHotkeys.value };
      await invoke("start_recording", { fps: effectiveRecordingFps(config.recording) });
      restoreOverlayAfterRecording.value = isOverlayVisible.value;
      if (silentRecording.value) {
        await destroyOverlayWindow();
      }
      if (trigger === "hotkey") {
        await invoke("add_recording_marker", { name: "hotkey-start" });
      }
      isRecording.value = true;
      lastRecordingPath.value = "";
      recordingStatusMessage.value = "Recording started.";
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

async function stopRecording(trigger: "manual" | "hotkey" = "manual") {
  if (!isRecording.value) {
    return;
  }

  if (trigger === "hotkey") {
    await invoke("add_recording_marker", { name: "hotkey-stop" });
  }

  const result = await invoke<{ path: string }>("stop_recording", {
    outputDir: await resolveRecordingDirectory(),
  });
  isRecording.value = false;
  activeRecordingHotkeys.value = null;
  lastRecordingPath.value = result.path;
  recordingStatusMessage.value = `Recording saved: ${result.path}`;

  if (silentRecording.value && restoreOverlayAfterRecording.value) {
    await setOverlayVisible(true);
    await moveOverlay(overlayPosition.value);
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

  inspectedRecordingPath.value = selectedPath;
  recordingInspection.value = null;
  recordingInspectionError.value = "";

  try {
    recordingInspection.value = await invoke<RecordingInspection>("inspect_recording_file", {
      path: selectedPath,
    });
  } catch (error) {
    recordingInspectionError.value = String(error);
  }
}

function updateSilentRecording(value: boolean) {
  silentRecording.value = value;
  scheduleAppConfigSave();
}

function updateRecordingHotkeyMode(mode: RecordingHotkeyMode) {
  recordingHotkeys.value = normalizeRecordingHotkeyConfig({
    mode,
    start: recordingHotkeys.value.start,
    stop: mode === "separate" ? undefined : recordingHotkeys.value.start,
    sync: recordingHotkeys.value.sync,
  });
  scheduleAppConfigSave();
}

function updateRecordingConfig(recording: RecordingConfig) {
  applyRecordingConfig(recording);
  scheduleAppConfigSave();
}

async function addSyncMarker() {
  if (!isRecording.value) {
    recordingStatusMessage.value = "Start recording before adding a sync marker.";
    return;
  }

  await invoke("add_recording_marker", { name: "sync" });
  recordingStatusMessage.value = "Sync marker added.";
}

async function suppressRecordingHotkeyInput(keys: string[]) {
  if (!isRecording.value || keys.length === 0) {
    return;
  }

  await invoke("suppress_recording_keys", { keyIds: normalizeHotkey(keys) });
}

function beginHotkeyCapture(target: "start" | "stop" | "sync") {
  capturedHotkeyKeys.value = new Set();
  hotkeyCaptureTarget.value = target;
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
  scheduleAppConfigSave();
  hotkeyCaptureTarget.value = null;
  capturedHotkeyKeys.value = new Set();
}

async function handleRecordingHotkeys(): Promise<boolean> {
  if (hotkeyCaptureTarget.value) {
    return false;
  }

  const activeSignature = normalizeHotkey(activeKeyIds.value).join("+");
  if (activeSignature === activeRecordingHotkeySignature.value) {
    return false;
  }

  const hotkeys = effectiveRecordingHotkeys();
  const matchesStart = isHotkeyMatch(activeKeyIds.value, hotkeys.start);
  const matchesStop = isHotkeyMatch(activeKeyIds.value, hotkeys.stop);
  const matchesSync = isHotkeyMatch(activeKeyIds.value, hotkeys.sync);

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

function profileNameFromFileName(fileName: string): string {
  return fileName.replace(/\.json$/i, "");
}

function handleKeydown(event: KeyboardEvent) {
  const keyId = keyIdFromKeyboardCode(event.code);

  if (keyId) {
    updateActiveKey(keyId, true);
    if (hotkeyCaptureTarget.value) {
      capturedHotkeyKeys.value = new Set([...capturedHotkeyKeys.value, keyId]);
    } else {
      void handleRecordingHotkeys();
    }
  }
}

function handleKeyup(event: KeyboardEvent) {
  const keyId = keyIdFromKeyboardCode(event.code);

  if (keyId) {
    updateActiveKey(keyId, false);
    if (hotkeyCaptureTarget.value) {
      finishHotkeyCapture();
    } else {
      void handleRecordingHotkeys();
    }
  }
}

function handleMousedown(event: MouseEvent) {
  if (!isOverlayWindow.value) {
    return;
  }

  const keyId = keyIdFromMouseButton(event.button);
  if (keyId) {
    updateActiveKey(keyId, true);
  }
}

function handleMouseup(event: MouseEvent) {
  if (!isOverlayWindow.value) {
    return;
  }

  const keyId = keyIdFromMouseButton(event.button);
  if (keyId) {
    updateActiveKey(keyId, false);
  }
}

onMounted(async () => {
  if (!isOverlayWindow.value) {
    await restoreAppConfig();
  }

  unlistenInputState = await listen<InputStatePayload>(
    INPUT_STATE_EVENT,
    (event) => {
      updateActiveKey(event.payload.keyId, event.payload.pressed);
      if (hotkeyCaptureTarget.value) {
        if (event.payload.pressed) {
          capturedHotkeyKeys.value = new Set([
            ...capturedHotkeyKeys.value,
            event.payload.keyId,
          ]);
        } else {
          finishHotkeyCapture();
        }
        return;
      }

      void (async () => {
        const consumed = await handleRecordingHotkeys();
        if (!consumed) {
          await recordInputIfNeeded(event.payload.keyId, event.payload.pressed);
        }
      })();
    },
  );

  if (isOverlayWindow.value) {
    unlistenOverlayStyle = await listen<OverlayStyle>(
      OVERLAY_STYLE_EVENT,
      (event) => {
        applyOverlayStyle(event.payload);
      },
    );
    const unlistenOverlayConfig = await listen<OverlayRuntimeConfig>(
      OVERLAY_CONFIG_EVENT,
      (event) => {
        applyOverlayLayout(event.payload.layout);
        applyOverlayRows(event.payload.rows);
        applyOverlayStyle(event.payload.style);
      },
    );
    const originalUnlistenOverlayStyle = unlistenOverlayStyle;
    unlistenOverlayStyle = () => {
      originalUnlistenOverlayStyle();
      unlistenOverlayConfig();
    };
  } else {
    unlistenOverlayStyle = await listen<boolean>(
      OVERLAY_VISIBLE_EVENT,
      (event) => {
        isOverlayVisible.value = event.payload;
      },
    );
  }

  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("keyup", handleKeyup);
  window.addEventListener("mousedown", handleMousedown);
  window.addEventListener("mouseup", handleMouseup);

  if (!isOverlayWindow.value) {
    stopAppConfigWatch = watch(
      [config, isOverlayVisible, profileName, profileSourcePath, overlayPosition],
      scheduleAppConfigSave,
      { deep: true },
    );
  }
});

onUnmounted(() => {
  if (appConfigSaveTimer !== undefined) {
    window.clearTimeout(appConfigSaveTimer);
  }
  stopAppConfigWatch?.();
  unlistenInputState?.();
  unlistenOverlayStyle?.();
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("keyup", handleKeyup);
  window.removeEventListener("mousedown", handleMousedown);
  window.removeEventListener("mouseup", handleMouseup);
});
</script>

<template>
  <div :class="['app-surface', { 'overlay-surface': isOverlayWindow }]">
    <div v-if="isOverlayWindow" class="overlay-window">
      <OverlayWindow
        :layout="config.layout"
        :rows="config.rows"
        :keys="config.keys"
        :active-keys="activeKeyIds"
        :overlay-style="config.style"
      />
    </div>
    <ConfigPanel
      v-else
      :config="config"
      :active-keys="activeKeyIds"
      :overlay-visible="isOverlayVisible"
      :profile-name="profileName"
      :recording-directory="recordingDirectory"
      :default-recording-directory="defaultRecordingDirectory"
      :silent-recording="silentRecording"
      :is-recording="isRecording"
      :recording-countdown="recordingCountdown"
      :last-recording-path="lastRecordingPath"
      :recording-status-message="recordingStatusMessage"
      :inspected-recording-path="inspectedRecordingPath"
      :recording-inspection="recordingInspection"
      :recording-inspection-error="recordingInspectionError"
      :overlay-position="overlayPosition"
      :recording-hotkeys="recordingHotkeys"
      :hotkey-capture-target="hotkeyCaptureTarget"
      @update-overlay-style="updateOverlayStyle"
      @update-overlay-visible="setOverlayVisible"
      @load-config="loadConfig"
      @export-and-apply-config="exportAndApplyConfig"
      @overwrite-and-apply-config="overwriteAndApplyConfig"
      @choose-recording-directory="chooseRecordingDirectory"
      @update-silent-recording="updateSilentRecording"
      @update-recording-config="updateRecordingConfig"
      @start-recording="startRecordingWithCountdown"
      @stop-recording="stopRecording"
      @add-sync-marker="addSyncMarker"
      @inspect-recording-file="inspectRecordingFile"
      @update-recording-hotkey-mode="updateRecordingHotkeyMode"
      @begin-hotkey-capture="beginHotkeyCapture"
      @move-overlay="moveOverlay"
    />
  </div>
</template>

<style>
:root {
  font-family:
    Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI",
    sans-serif;
  font-size: 16px;
  line-height: 1.5;
  font-weight: 400;
  color: #eef2f6;
  background-color: transparent;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

* {
  box-sizing: border-box;
}

html,
body,
#app {
  margin: 0;
  width: 100%;
  min-height: 100%;
  overflow: hidden;
}

body {
  overflow: hidden;
}

button,
input {
  font: inherit;
}

.app-surface {
  min-height: 100vh;
}

.overlay-surface {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.overlay-window {
  display: flex;
  width: 100vw;
  min-height: 100vh;
  align-items: center;
  justify-content: center;
  background: transparent;
  padding: 14px;
}
</style>
