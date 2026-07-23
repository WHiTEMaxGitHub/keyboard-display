<script setup lang="ts">
import { emitTo, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, onUnmounted, reactive, ref, watch, type WatchStopHandle } from "vue";
import { tauriApi } from "./api/tauri";
import ConfigPanel from "./components/ConfigPanel.vue";
import OverlayWindow from "./components/OverlayWindow.vue";
import {
  normalizeOverlayPosition,
  useOverlayWindow,
  type OverlayPosition,
  type OverlayRuntimeConfig,
} from "./composables/useOverlayWindow";
import {
  useRecordingController,
} from "./composables/useRecordingController";
import { useNotifications } from "./composables/useNotifications";
import {
  buildAppConfigFile,
  parseAppConfigFile,
  type RecentProfile,
} from "./domain/appConfig";
import { buildConfigFileJson, parseConfigFile } from "./domain/configFile";
import {
  createDefaultConfig,
  flattenRowKeys,
  type ExportConfig,
  type OverlayCustomPosition,
  type OverlayStyle,
} from "./domain/defaultConfig";
import {
  INPUT_STATE_EVENT,
  OVERLAY_CONFIG_EVENT,
  OVERLAY_READY_EVENT,
  OVERLAY_STYLE_EVENT,
  OVERLAY_SYNC_FEEDBACK_EVENT,
  OVERLAY_VISIBLE_EVENT,
  type InputStatePayload,
} from "./domain/inputEvents";
import type { RecordingHotkeyMode } from "./domain/recordingHotkeys";
import type { RecordingConfig } from "./domain/defaultConfig";
import {
  createDefaultVideoExporterConfig,
  type VideoExporterConfig,
} from "./domain/videoExporter";

const config = reactive(createDefaultConfig());
const activeKeyIds = ref(new Set<string>());
const isOverlayVisible = ref(true);
const profileName = ref("CS POV");
const profileSourcePath = ref<string | null>(null);
const profileChanged = ref(false);
const recentProfiles = ref<RecentProfile[]>([]);
const overlayPosition = ref<OverlayPosition>("bottom-right");
const customOverlayPosition = ref<OverlayCustomPosition | null>(null);
const syncFeedbackActive = ref(false);
const videoExporterConfig = ref<VideoExporterConfig>(createDefaultVideoExporterConfig());
const recordingBrowserDirectory = ref("");
const { notifications, notify, dismissNotification } = useNotifications();

const isOverlayWindow = computed(() => {
  return new URLSearchParams(window.location.search).get("surface") === "pov";
});

let unlistenInputState: UnlistenFn | undefined;
let unlistenOverlayStyle: UnlistenFn | undefined;
let unlistenOverlayReady: UnlistenFn | undefined;
let syncFeedbackTimer: number | undefined;
let appConfigSaveTimer: number | undefined;
let stopAppConfigWatch: WatchStopHandle | undefined;

const {
  overlayAdjusting,
  updateOverlayStyle: syncOverlayStyle,
  updateOverlayRows: syncOverlayRows,
  resizeOverlayWindow,
  destroyOverlayWindow,
  setOverlayVisible,
  moveOverlay,
  startOverlayAdjust,
  handleOverlayReady,
  saveOverlayAdjust,
  cancelOverlayAdjust,
} = useOverlayWindow({
  config,
  isOverlayVisible,
  overlayPosition,
  customOverlayPosition,
  markProfileChanged,
  scheduleAppConfigSave,
});

const {
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
  initializeDefaultRecordingDirectory,
  recordInputIfNeeded,
  chooseRecordingDirectory,
  startRecordingWithCountdown,
  stopRecording,
  inspectRecordingFile,
  inspectRecordingPath,
  clearRecordingInspection,
  updateSilentRecording,
  updateRecordingHotkeyMode: setRecordingHotkeyMode,
  addSyncMarker,
  beginHotkeyCapture,
  captureHotkeyKey,
  finishHotkeyCapture,
  handleRecordingHotkeys,
} = useRecordingController({
  config,
  profileName,
  isOverlayWindow,
  activeKeyIds,
  isOverlayVisible,
  overlayPosition,
  destroyOverlayWindow,
  setOverlayVisible,
  moveOverlay,
  scheduleAppConfigSave,
});

function updateActiveKey(keyId: string, pressed: boolean) {
  const nextKeys = new Set(activeKeyIds.value);

  if (pressed) {
    nextKeys.add(keyId);
  } else {
    nextKeys.delete(keyId);
  }

  activeKeyIds.value = nextKeys;
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

function applyExportConfig(exportConfig: ExportConfig) {
  config.export = { ...exportConfig };
}

async function updateOverlayStyle(style: OverlayStyle) {
  applyOverlayStyle(style);
  markProfileChanged();
  await syncOverlayStyle(style);
}

async function updateOverlayRows(rows: typeof config.rows) {
  applyOverlayRows(rows);
  markProfileChanged();
  await syncOverlayRows();
}

function markProfileChanged() {
  profileChanged.value = true;
}

async function applyLoadedConfig(text: string, fileName: string, sourcePath: string | null) {
  const loadedConfig = parseConfigFile(text);
  profileName.value = loadedConfig.name || profileNameFromFileName(fileName);
  profileSourcePath.value = sourcePath;
  profileChanged.value = false;
  overlayPosition.value = normalizeOverlayPosition(loadedConfig.overlay.position);
  customOverlayPosition.value = loadedConfig.overlay.customPosition ?? null;

  applyOverlayLayout(loadedConfig.overlay.layout);
  applyOverlayRows(loadedConfig.overlay.rows);
  applyOverlayStyle(loadedConfig.overlay.style);
  applyRecordingConfig(loadedConfig.recording);
  applyExportConfig(loadedConfig.export);
  await resizeOverlayWindow();

  await emitTo<OverlayRuntimeConfig>("pov", OVERLAY_CONFIG_EVENT, {
    layout: loadedConfig.overlay.layout,
    rows: loadedConfig.overlay.rows,
    keys: loadedConfig.overlay.keys,
    style: loadedConfig.overlay.style,
  });
  const visible = loadedConfig.overlay.visible ?? true;
  await setOverlayVisible(visible, false);
  if (visible) {
    await moveOverlay(overlayPosition.value, false);
  }
  scheduleAppConfigSave();
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

  const text = await tauriApi.readConfigFile(selectedPath);
  await applyLoadedConfig(text, selectedPath.split(/[\\/]/).pop() ?? selectedPath, selectedPath);
}

async function loadRecentProfile(path: string) {
  const text = await tauriApi.readConfigFile(path);
  await applyLoadedConfig(text, path.split(/[\\/]/).pop() ?? path, path);
}

async function restoreAppConfig() {
  await initializeDefaultRecordingDirectory();
  const savedConfig = await tauriApi.loadAppConfig();
  if (!savedConfig) {
    return;
  }

  const appConfig = parseAppConfigFile(savedConfig);
  profileName.value = appConfig.currentProfile.name;
  profileSourcePath.value = appConfig.currentProfile.sourcePath;
  profileChanged.value = appConfig.currentProfile.changed;
  recentProfiles.value = appConfig.profiles.recentProfiles;
  overlayPosition.value = normalizeOverlayPosition(appConfig.currentProfile.overlay.position);
  customOverlayPosition.value = appConfig.currentProfile.overlay.customPosition ?? null;
  recordingDirectory.value = appConfig.recording.outputDirectory ?? "";
  recordingBrowserDirectory.value = appConfig.recording.browserDirectory ?? "";
  silentRecording.value = appConfig.recording.silent ?? false;
  recordingHotkeys.value = appConfig.recording.hotkeys;
  videoExporterConfig.value = appConfig.exporter.video;

  applyOverlayLayout(appConfig.currentProfile.overlay.layout);
  applyOverlayRows(appConfig.currentProfile.overlay.rows);
  applyOverlayStyle(appConfig.currentProfile.overlay.style);
  applyRecordingConfig(appConfig.currentProfile.recording);
  applyExportConfig(appConfig.currentProfile.export);

  await emitTo<OverlayRuntimeConfig>("pov", OVERLAY_CONFIG_EVENT, {
    layout: appConfig.currentProfile.overlay.layout,
    rows: appConfig.currentProfile.overlay.rows,
    keys: appConfig.currentProfile.overlay.keys,
    style: appConfig.currentProfile.overlay.style,
  });

  await setOverlayVisible(appConfig.currentProfile.overlay.visible, false);
  if (appConfig.currentProfile.overlay.visible) {
    await moveOverlay(overlayPosition.value, false);
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
    recentProfiles: recentProfiles.value,
    currentProfile: {
      name: profileName.value,
      sourcePath: profileSourcePath.value,
      changed: profileChanged.value,
      recording: config.recording,
      export: config.export,
      overlay: {
        visible: isOverlayVisible.value,
        position: overlayPosition.value,
        layout: config.layout,
        style: config.style,
        rows: config.rows,
        keys: config.keys,
        customPosition: customOverlayPosition.value,
      },
    },
    recording: {
      outputDirectory: recordingDirectory.value || null,
      browserDirectory: recordingBrowserDirectory.value || null,
      silent: silentRecording.value,
      hotkeys: recordingHotkeys.value,
    },
    exporter: {
      video: videoExporterConfig.value,
    },
  });
  recentProfiles.value = appConfig.profiles.recentProfiles;

  await tauriApi.saveAppConfig(`${JSON.stringify(appConfig, null, 2)}\n`);
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
    customPosition: customOverlayPosition.value,
  });
  const path = await save({
    title: "Save keyboard display config",
    defaultPath: `${profileName.value || "keyboard-display"}.json`,
    filters: [{ name: "JSON", extensions: ["json"] }],
  });

  if (!path) {
    return;
  }

  await tauriApi.saveConfigFile(path, json);
  profileSourcePath.value = path;
  profileChanged.value = false;
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
    customPosition: customOverlayPosition.value,
  });
  await tauriApi.saveConfigFile(profileSourcePath.value, json);
  profileChanged.value = false;
  scheduleAppConfigSave();
}

function updateRecordingHotkeyMode(mode: RecordingHotkeyMode) {
  setRecordingHotkeyMode(mode);
}

function updateRecordingConfig(recording: RecordingConfig) {
  applyRecordingConfig(recording);
  markProfileChanged();
  scheduleAppConfigSave();
}

function updateExportConfig(exportConfig: ExportConfig) {
  applyExportConfig(exportConfig);
  markProfileChanged();
  scheduleAppConfigSave();
}

function updateVideoExporterConfig(exporterConfig: VideoExporterConfig) {
  videoExporterConfig.value = exporterConfig;
  scheduleAppConfigSave();
}

async function chooseRecordingBrowserDirectory() {
  const selectedPath = await open({
    title: "Choose recording files folder",
    directory: true,
    multiple: false,
  });

  if (typeof selectedPath === "string") {
    recordingBrowserDirectory.value = selectedPath;
    scheduleAppConfigSave();
  }
}

function profileNameFromFileName(fileName: string): string {
  return fileName.replace(/\.json$/i, "");
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
          captureHotkeyKey(event.payload.keyId);
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
    const unlistenSyncFeedback = await listen<{ durationMs: number }>(
      OVERLAY_SYNC_FEEDBACK_EVENT,
      (event) => {
        syncFeedbackActive.value = true;
        if (syncFeedbackTimer !== undefined) {
          window.clearTimeout(syncFeedbackTimer);
        }
        syncFeedbackTimer = window.setTimeout(() => {
          syncFeedbackActive.value = false;
          syncFeedbackTimer = undefined;
        }, event.payload.durationMs);
      },
    );
    const originalUnlistenOverlayStyle = unlistenOverlayStyle;
    unlistenOverlayStyle = () => {
      originalUnlistenOverlayStyle();
      unlistenOverlayConfig();
      unlistenSyncFeedback();
    };
  } else {
    unlistenOverlayStyle = await listen<boolean>(
      OVERLAY_VISIBLE_EVENT,
      (event) => {
        isOverlayVisible.value = event.payload;
      },
    );
    unlistenOverlayReady = await listen(
      OVERLAY_READY_EVENT,
      () => {
        void handleOverlayReady();
      },
    );
  }

  if (!isOverlayWindow.value) {
    stopAppConfigWatch = watch(
      [config, isOverlayVisible, profileName, profileSourcePath, profileChanged, overlayPosition],
      // profileChanged is separate from config because it tracks whether the
      // current profile differs from sourcePath rather than the profile data.
      scheduleAppConfigSave,
      { deep: true },
    );
  }
});

onUnmounted(() => {
  if (appConfigSaveTimer !== undefined) {
    window.clearTimeout(appConfigSaveTimer);
  }
  if (syncFeedbackTimer !== undefined) {
    window.clearTimeout(syncFeedbackTimer);
  }
  stopAppConfigWatch?.();
  unlistenInputState?.();
  unlistenOverlayStyle?.();
  unlistenOverlayReady?.();
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
        :sync-feedback-active="syncFeedbackActive"
      />
    </div>
    <ConfigPanel
      v-else
      :config="config"
      :active-keys="activeKeyIds"
      :overlay-visible="isOverlayVisible"
      :profile-name="profileName"
      :profile-changed="profileChanged"
      :recent-profiles="recentProfiles"
      :recording-directory="recordingDirectory"
      :default-recording-directory="defaultRecordingDirectory"
      :recording-browser-directory="recordingBrowserDirectory"
      :silent-recording="silentRecording"
      :is-recording="isRecording"
      :recording-countdown="recordingCountdown"
      :last-recording-path="lastRecordingPath"
      :recording-status-message="recordingStatusMessage"
      :current-recording-path="currentRecordingPath"
      :recording-inspection="recordingInspection"
      :recording-inspection-error="recordingInspectionError"
      :overlay-position="overlayPosition"
      :overlay-adjusting="overlayAdjusting"
      :recording-hotkeys="recordingHotkeys"
      :hotkey-capture-target="hotkeyCaptureTarget"
      :video-exporter-config="videoExporterConfig"
      :notifications="notifications"
      @update-overlay-style="updateOverlayStyle"
      @update-overlay-rows="updateOverlayRows"
      @update-overlay-visible="setOverlayVisible"
      @load-config="loadConfig"
      @refresh-pov="applyConfigToOverlay"
      @load-recent-profile="loadRecentProfile"
      @export-and-apply-config="exportAndApplyConfig"
      @overwrite-and-apply-config="overwriteAndApplyConfig"
      @choose-recording-directory="chooseRecordingDirectory"
      @choose-recording-browser-directory="chooseRecordingBrowserDirectory"
      @update-silent-recording="updateSilentRecording"
      @update-recording-config="updateRecordingConfig"
      @update-export-config="updateExportConfig"
      @update-video-exporter-config="updateVideoExporterConfig"
      @notify="notify"
      @dismiss-notification="dismissNotification"
      @start-recording="startRecordingWithCountdown"
      @stop-recording="stopRecording"
      @add-sync-marker="addSyncMarker"
      @inspect-recording-file="inspectRecordingFile"
      @inspect-recording-path="inspectRecordingPath"
      @clear-recording-inspection="clearRecordingInspection"
      @update-recording-hotkey-mode="updateRecordingHotkeyMode"
      @begin-hotkey-capture="beginHotkeyCapture"
      @start-overlay-adjust="startOverlayAdjust"
      @save-overlay-adjust="saveOverlayAdjust"
      @cancel-overlay-adjust="cancelOverlayAdjust"
      @move-overlay="moveOverlay"
    />
  </div>
</template>
