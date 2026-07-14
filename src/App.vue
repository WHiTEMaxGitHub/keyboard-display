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
import { open, save } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, onUnmounted, reactive, ref, watch, type WatchStopHandle } from "vue";
import ConfigPanel from "./components/ConfigPanel.vue";
import OverlayWindow from "./components/OverlayWindow.vue";
import { buildAppConfigFile, parseAppConfigFile } from "./domain/appConfig";
import { buildConfigFileJson, parseConfigFile } from "./domain/configFile";
import { createDefaultConfig, type OverlayStyle } from "./domain/defaultConfig";
import { estimateOverlaySize } from "./domain/overlaySize";
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
  type RecordingHotkeyConfig,
  type RecordingHotkeyMode,
} from "./domain/recordingHotkeys";

type OverlayPosition = "top-left" | "top-right" | "bottom-left" | "bottom-right" | "center";

const config = reactive(createDefaultConfig());
const activeKeyIds = ref(new Set<string>());
const isOverlayVisible = ref(true);
const profileName = ref("CS POV");
const overlayPosition = ref<OverlayPosition>("bottom-right");
const recordingDirectory = ref("");
const isRecording = ref(false);
const recordingCountdown = ref(0);
const recordingCountdownTimer = ref<number | null>(null);
const lastRecordingPath = ref("");
const recordingHotkeys = ref<RecordingHotkeyConfig>({
  mode: "disabled",
  start: [],
  stop: [],
});
const hotkeyCaptureTarget = ref<"start" | "stop" | null>(null);
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
  keys: typeof config.keys;
  style: OverlayStyle;
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

function applyOverlayStyle(style: OverlayStyle) {
  config.style = { ...style };
}

function applyOverlayLayout(layout: typeof config.layout) {
  config.layout = { ...layout };
}

function applyOverlayKeys(keys: typeof config.keys) {
  config.keys = [...keys];
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

  const size = estimateOverlaySize(config.layout, config.keys, config.style);
  await targetWindow.setSize(new LogicalSize(size.width, size.height));
}

async function setOverlayVisible(visible: boolean) {
  const overlayWindow = await Window.getByLabel("pov");

  if (!overlayWindow) {
    isOverlayVisible.value = false;
    return;
  }

  if (visible) {
    await resizeOverlayWindow(overlayWindow);
    await overlayWindow.show();
  } else {
    await overlayWindow.hide();
  }

  isOverlayVisible.value = visible;
  await emitTo<boolean>("pov", OVERLAY_VISIBLE_EVENT, visible);
}

async function moveOverlay(position: OverlayPosition) {
  overlayPosition.value = position;
  const overlayWindow = await Window.getByLabel("pov");
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

async function loadConfig(text: string, fileName: string) {
  const loadedConfig = parseConfigFile(text);
  profileName.value = loadedConfig.name || profileNameFromFileName(fileName);
  overlayPosition.value = (loadedConfig.overlay.position as OverlayPosition) ?? "bottom-right";

  applyOverlayLayout(loadedConfig.overlay.layout);
  applyOverlayKeys(loadedConfig.overlay.keys);
  applyOverlayStyle(loadedConfig.overlay.style);
  await resizeOverlayWindow();

  await emitTo<OverlayRuntimeConfig>("pov", OVERLAY_CONFIG_EVENT, {
    layout: loadedConfig.overlay.layout,
    keys: loadedConfig.overlay.keys,
    style: loadedConfig.overlay.style,
  });
  const visible = loadedConfig.overlay.visible ?? true;
  await setOverlayVisible(visible);
  if (visible) {
    await moveOverlay(overlayPosition.value);
  }
}

async function restoreAppConfig() {
  const savedConfig = await invoke<string | null>("load_app_config");
  if (!savedConfig) {
    return;
  }

  const appConfig = parseAppConfigFile(savedConfig);
  profileName.value = appConfig.currentProfile.name;
  overlayPosition.value = appConfig.currentProfile.overlay.position as OverlayPosition;
  recordingDirectory.value = appConfig.recording.outputDirectory ?? "";
  recordingHotkeys.value = appConfig.recording.hotkeys;

  applyOverlayLayout(appConfig.currentProfile.overlay.layout);
  applyOverlayKeys(appConfig.currentProfile.overlay.keys);
  applyOverlayStyle(appConfig.currentProfile.overlay.style);

  await emitTo<OverlayRuntimeConfig>("pov", OVERLAY_CONFIG_EVENT, {
    layout: appConfig.currentProfile.overlay.layout,
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
      sourcePath: null,
      dirty: true,
      overlay: {
        visible: isOverlayVisible.value,
        position: overlayPosition.value,
        layout: config.layout,
        style: config.style,
        keys: config.keys,
      },
    },
    recording: {
      outputDirectory: recordingDirectory.value || null,
      hotkeys: recordingHotkeys.value,
    },
  });

  await invoke("save_app_config", {
    contents: `${JSON.stringify(appConfig, null, 2)}\n`,
  });
}

async function saveAndApplyConfig() {
  await resizeOverlayWindow();
  await emitTo<OverlayRuntimeConfig>("pov", OVERLAY_CONFIG_EVENT, {
    layout: config.layout,
    keys: config.keys,
    style: config.style,
  });
  await setOverlayVisible(isOverlayVisible.value);

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
}

async function chooseRecordingDirectory() {
  const selectedPath = await open({
    title: "Choose recording folder",
    directory: true,
    multiple: false,
  });

  if (typeof selectedPath === "string") {
    recordingDirectory.value = selectedPath;
    scheduleAppConfigSave();
  }
}

async function startRecordingWithCountdown() {
  if (!recordingDirectory.value || isRecording.value || recordingCountdown.value > 0) {
    return;
  }

  recordingCountdown.value = 3;

  recordingCountdownTimer.value = window.setInterval(async () => {
    recordingCountdown.value -= 1;

    if (recordingCountdown.value <= 0) {
      cancelRecordingCountdown();
      await invoke("start_recording", { fps: config.recording.defaultFps });
      isRecording.value = true;
      lastRecordingPath.value = "";
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

async function stopRecording() {
  if (!isRecording.value) {
    return;
  }

  const result = await invoke<{ path: string }>("stop_recording", {
    outputDir: recordingDirectory.value,
  });
  isRecording.value = false;
  lastRecordingPath.value = result.path;
}

function updateRecordingHotkeyMode(mode: RecordingHotkeyMode) {
  recordingHotkeys.value = { ...recordingHotkeys.value, mode };
  scheduleAppConfigSave();
}

function beginHotkeyCapture(target: "start" | "stop") {
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

async function handleRecordingHotkeys() {
  if (recordingHotkeys.value.mode === "disabled" || hotkeyCaptureTarget.value) {
    return;
  }

  const activeSignature = normalizeHotkey(activeKeyIds.value).join("+");
  if (activeSignature === activeRecordingHotkeySignature.value) {
    return;
  }

  const matchesStart = isHotkeyMatch(activeKeyIds.value, recordingHotkeys.value.start);
  const matchesStop = isHotkeyMatch(activeKeyIds.value, recordingHotkeys.value.stop);

  if (!matchesStart && !matchesStop) {
    if (activeSignature === "") {
      activeRecordingHotkeySignature.value = "";
    }
    return;
  }

  activeRecordingHotkeySignature.value = activeSignature;

  if (recordingHotkeys.value.mode === "toggle") {
    if (recordingCountdown.value > 0) {
      cancelRecordingCountdown();
      return;
    }

    if (isRecording.value) {
      await stopRecording();
    } else {
      await startRecordingWithCountdown();
    }
    return;
  }

  if (recordingHotkeys.value.mode === "separate") {
    if (!isRecording.value && matchesStart) {
      await startRecordingWithCountdown();
    } else if (isRecording.value && matchesStop) {
      await stopRecording();
    }
  }
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
      } else {
        void handleRecordingHotkeys();
      }
      void recordInputIfNeeded(event.payload.keyId, event.payload.pressed);
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
        applyOverlayKeys(event.payload.keys);
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
      [config, isOverlayVisible, profileName, overlayPosition],
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
      :is-recording="isRecording"
      :recording-countdown="recordingCountdown"
      :last-recording-path="lastRecordingPath"
      :overlay-position="overlayPosition"
      :recording-hotkeys="recordingHotkeys"
      :hotkey-capture-target="hotkeyCaptureTarget"
      @update-overlay-style="updateOverlayStyle"
      @update-overlay-visible="setOverlayVisible"
      @load-config="loadConfig"
      @save-and-apply-config="saveAndApplyConfig"
      @choose-recording-directory="chooseRecordingDirectory"
      @start-recording="startRecordingWithCountdown"
      @stop-recording="stopRecording"
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
