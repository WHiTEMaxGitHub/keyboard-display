<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onMounted, onUnmounted, watch, type WatchStopHandle } from "vue";
import { tauriApi } from "./api/tauri";
import ConfigPanel from "./components/ConfigPanel.vue";
import OverlayWindow from "./components/OverlayWindow.vue";
import {
  useOverlayWindow,
  type OverlayStyleSyncMode,
  type OverlayRuntimeConfig,
} from "./composables/useOverlayWindow";
import { useInputStateBridge } from "./composables/useInputStateBridge";
import { useRecordingController } from "./composables/useRecordingController";
import { useNotifications } from "./composables/useNotifications";
import { useAppConfig } from "./composables/useAppConfig";
import { buildAppConfigFile } from "./domain/appConfig";
import { type OverlayStyle } from "./domain/defaultConfig";
import {
  OVERLAY_CONFIG_EVENT,
  OVERLAY_READY_EVENT,
  OVERLAY_STYLE_EVENT,
  OVERLAY_SYNC_FEEDBACK_EVENT,
  OVERLAY_VISIBLE_EVENT,
  type InputStatePayload,
} from "./domain/inputEvents";
import type { RecordingHotkeyMode } from "./domain/recordingHotkeys";
import type { RecordingConfig } from "./domain/defaultConfig";

const isOverlayWindow = computed(() => {
  return new URLSearchParams(window.location.search).get("surface") === "pov";
});

const { notifications, notify, dismissNotification } = useNotifications();

const {
  config,
  isOverlayVisible,
  profileName,
  profileSourcePath,
  profileChanged,
  recentProfiles,
  overlayPosition,
  customOverlayPosition,
  syncFeedbackActive,
  videoExporterConfig,
  recordingBrowserDirectory,
  applyOverlayStyle,
  applyOverlayRows,
  applyKeyIdLabels,
  markProfileChanged,
  isBackplateVisible,
  scheduleAppConfigSave,
  loadConfig,
  loadRecentProfile,
  restoreAppConfig,
  applyConfigToOverlay,
  exportAndApplyConfig,
  overwriteAndApplyConfig,
  updateRecordingConfig,
  updateExportConfig,
  updateVideoExporterConfig,
  chooseRecordingBrowserDirectory,
  dispose,
} = useAppConfig({ isOverlayWindow });

let unlistenOverlayStyle: UnlistenFn | undefined;
let unlistenOverlayReady: UnlistenFn | undefined;
let unlistenCloseRequested: UnlistenFn | undefined;
let syncFeedbackTimer: number | undefined;
let stopAppConfigWatch: WatchStopHandle | undefined;

const scheduleSave = () => scheduleAppConfigSave(saveAppConfig);

const {
  overlayAdjusting,
  updateOverlayStyle: syncOverlayStyle,
  disposeOverlayStyleSync,
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
  scheduleAppConfigSave: scheduleSave,
});

const overlayCallbacks = {
  resizeOverlayWindow,
  syncOverlayStyle,
  syncOverlayRows,
  setOverlayVisible,
  moveOverlay,
};

const {
  activeKeyIds,
  startInputBridge,
  stopInputBridge,
} = useInputStateBridge({
  isOverlayWindow,
  onConfigInput: handleConfigInput,
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
  enabled: !isOverlayWindow.value,
  config,
  profileName,
  isOverlayWindow,
  activeKeyIds,
  isOverlayVisible,
  overlayPosition,
  destroyOverlayWindow,
  setOverlayVisible,
  moveOverlay,
  scheduleAppConfigSave: scheduleSave,
});

function handleConfigInput(payload: InputStatePayload) {
  if (hotkeyCaptureTarget.value) {
    if (payload.pressed) {
      captureHotkeyKey(payload.keyId);
    } else {
      finishHotkeyCapture();
    }
    return;
  }

  void (async () => {
    const consumed = await handleRecordingHotkeys();
    if (!consumed) {
      await recordInputIfNeeded(payload.keyId, payload.pressed);
    }
  })();
}

function overlayStyleSyncMode(
  previousStyle: OverlayStyle,
  nextStyle: OverlayStyle,
): OverlayStyleSyncMode {
  return previousStyle.scale !== nextStyle.scale ||
    previousStyle.alwaysOnTop !== nextStyle.alwaysOnTop ||
    isBackplateVisible(previousStyle) !== isBackplateVisible(nextStyle)
    ? "window"
    : "css";
}

function previewOverlayStyle(style: OverlayStyle) {
  void syncOverlayStyle(style, "css");
}

async function updateOverlayStyle(style: OverlayStyle) {
  const previousStyle = { ...config.style };
  const syncMode = overlayStyleSyncMode(previousStyle, style);
  applyOverlayStyle(style);
  markProfileChanged();
  await syncOverlayStyle(style, syncMode);
}

async function updateOverlayRows(rows: typeof config.rows) {
  applyOverlayRows(rows);
  markProfileChanged();
  await syncOverlayRows();
}

async function updateKeyIdLabels(keyIdLabels: typeof config.keyIdLabels) {
  applyKeyIdLabels(keyIdLabels);
  markProfileChanged();
  await syncOverlayRows();
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
        keyIdLabels: config.keyIdLabels,
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

function updateRecordingHotkeyMode(mode: RecordingHotkeyMode) {
  setRecordingHotkeyMode(mode);
}

onMounted(async () => {
  if (!isOverlayWindow.value) {
    await restoreAppConfig(
      overlayCallbacks,
      initializeDefaultRecordingDirectory,
      (recording) => {
        recordingDirectory.value = recording.directory;
        recordingBrowserDirectory.value = recording.browserDirectory;
        silentRecording.value = recording.silent;
        recordingHotkeys.value = recording.hotkeys;
      },
    );
    unlistenCloseRequested = await getCurrentWindow().onCloseRequested(async () => {
      await destroyOverlayWindow();
    });
  }

  await startInputBridge();

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
        config.layout = { ...event.payload.layout };
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
      scheduleSave,
      { deep: true },
    );
  }
});

onUnmounted(() => {
  dispose();
  if (syncFeedbackTimer !== undefined) {
    window.clearTimeout(syncFeedbackTimer);
  }
  stopAppConfigWatch?.();
  disposeOverlayStyleSync();
  stopInputBridge();
  unlistenOverlayStyle?.();
  unlistenOverlayReady?.();
  unlistenCloseRequested?.();
});
</script>

<template>
  <div :class="['app-surface', { 'overlay-surface': isOverlayWindow }]">
    <div v-if="isOverlayWindow" class="overlay-window">
      <OverlayWindow
        :layout="config.layout"
        :rows="config.rows"
        :keys="config.keys"
        :key-id-labels="config.keyIdLabels"
        :active-keys="activeKeyIds"
        :overlay-style="config.style"
        :sync-feedback-active="syncFeedbackActive"
      />
    </div>
    <ConfigPanel
      v-else
      :config="config"
      :active-keys="activeKeyIds"
      :key-id-labels="config.keyIdLabels"
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
      @preview-overlay-style="previewOverlayStyle"
      @update-key-id-labels="updateKeyIdLabels"
      @update-overlay-style="updateOverlayStyle"
      @update-overlay-rows="updateOverlayRows"
      @update-overlay-visible="setOverlayVisible"
      @load-config="loadConfig(overlayCallbacks, scheduleSave)"
      @refresh-pov="applyConfigToOverlay(overlayCallbacks)"
      @load-recent-profile="(path: string) => loadRecentProfile(path, overlayCallbacks, scheduleSave)"
      @export-and-apply-config="exportAndApplyConfig(overlayCallbacks, scheduleSave)"
      @overwrite-and-apply-config="overwriteAndApplyConfig(overlayCallbacks, scheduleSave)"
      @choose-recording-directory="chooseRecordingDirectory"
      @choose-recording-browser-directory="(async () => { await chooseRecordingBrowserDirectory(); scheduleSave(); })()"
      @update-silent-recording="updateSilentRecording"
      @update-recording-config="(r: RecordingConfig) => { updateRecordingConfig(r); scheduleSave(); }"
      @update-export-config="(e: any) => { updateExportConfig(e); scheduleSave(); }"
      @update-video-exporter-config="(v: any) => { updateVideoExporterConfig(v); scheduleSave(); }"
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