<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onMounted, onUnmounted, ref, watch, type WatchStopHandle } from "vue";
import { tauriApi } from "./api/tauri";
import AppTitlebar from "./components/AppTitlebar.vue";
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
import { useTheme } from "./composables/useTheme";
import AmbientBackground from "./components/AmbientBackground.vue";
import { buildAppConfigFile, createInitialAppConfigFile } from "./domain/appConfig";
import { type OverlayStyle } from "./domain/defaultConfig";
import { setI18nLanguage } from "./i18n";
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
  themeId,
  customThemeColors,
  customThemeTemplate,
  customThemePanelOpacity,
  loadTheme,
  setTheme,
  previewCustomThemeColor,
  setCustomThemeColor,
  setCustomThemeTemplate,
  setCustomThemePanelOpacity,
  resetCustomThemeColors,
} = useTheme();
const appConfigPath = ref("");

const {
  config,
  isOverlayVisible,
  profileName,
  profileSourcePath,
  profileChanged,
  overlayPosition,
  customOverlayPosition,
  syncFeedbackActive,
  videoExporterConfig,
  recordingBrowserDirectory,
  uiLanguage,
  applyOverlayStyle,
  applyOverlayLayout,
  applyOverlayRows,
  applyKeyIdLabels,
  markProfileChanged,
  isBackplateVisible,
  scheduleAppConfigSave,
  loadConfig,
  restoreAppConfig,
  applyConfigToOverlay,
  exportAndApplyConfig,
  overwriteAndApplyConfig,
  updateRecordingConfig,
  updateExportConfig,
  updateVideoExporterConfig,
  updateUiLanguage,
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

async function updateOverlayLayout(layout: typeof config.layout) {
  applyOverlayLayout(layout);
  markProfileChanged();
  await syncOverlayRows();
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
    ui: {
      language: uiLanguage.value,
    },
  });

  await tauriApi.saveAppConfig(`${JSON.stringify(appConfig, null, 2)}\n`);
}

function setUiLanguage(language: typeof uiLanguage.value) {
  updateUiLanguage(language);
  setI18nLanguage(uiLanguage.value);
  scheduleSave();
}

function updateRecordingHotkeyMode(mode: RecordingHotkeyMode) {
  setRecordingHotkeyMode(mode);
}

onMounted(async () => {
  loadTheme();
  if (!isOverlayWindow.value) {
    try {
      const initialAppConfig = createInitialAppConfigFile();
      const initialization = await tauriApi.initializeAppConfig(
        `${JSON.stringify(initialAppConfig, null, 2)}\n`,
      );
      appConfigPath.value = initialization.path;
      if (initialization.initialized) {
        await tauriApi.writeDebugLog(
          "app-config",
          `initialized app config at ${initialization.path}`,
        );
      }
      appConfigPath.value = await tauriApi.appConfigPath();
    } catch (error) {
      console.warn("Failed to resolve app config path", error);
    }
    await restoreAppConfig(
      overlayCallbacks,
      (recording) => {
        recordingDirectory.value = recording.directory;
        recordingBrowserDirectory.value = recording.browserDirectory;
        silentRecording.value = recording.silent;
        recordingHotkeys.value = recording.hotkeys;
      },
    );
    setI18nLanguage(uiLanguage.value);
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
    <AmbientBackground v-if="!isOverlayWindow" />
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
    <div v-else class="config-window-frame">
      <AppTitlebar />
      <ConfigPanel
        :config="config"
        :active-keys="activeKeyIds"
        :key-id-labels="config.keyIdLabels"
        :overlay-visible="isOverlayVisible"
        :profile-name="profileName"
        :profile-changed="profileChanged"
        :recording-directory="recordingDirectory"
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
        :theme-id="themeId"
        :custom-theme-colors="customThemeColors"
        :custom-theme-template="customThemeTemplate"
        :custom-theme-panel-opacity="customThemePanelOpacity"
        :ui-language="uiLanguage"
        :app-config-path="appConfigPath"
        @preview-overlay-style="previewOverlayStyle"
        @update-key-id-labels="updateKeyIdLabels"
        @update-overlay-style="updateOverlayStyle"
        @update-overlay-layout="updateOverlayLayout"
        @update-overlay-rows="updateOverlayRows"
        @update-overlay-visible="setOverlayVisible"
        @load-config="loadConfig(overlayCallbacks, scheduleSave)"
        @refresh-pov="applyConfigToOverlay(overlayCallbacks)"
        @export-and-apply-config="exportAndApplyConfig(overlayCallbacks, scheduleSave)"
        @overwrite-and-apply-config="overwriteAndApplyConfig(overlayCallbacks, scheduleSave)"
        @choose-recording-directory="chooseRecordingDirectory"
        @choose-recording-browser-directory="(async () => { await chooseRecordingBrowserDirectory(); scheduleSave(); })()"
        @update-silent-recording="updateSilentRecording"
        @update-recording-config="(r: RecordingConfig) => { updateRecordingConfig(r); scheduleSave(); }"
        @update-export-config="(e: any) => { updateExportConfig(e); scheduleSave(); }"
        @update-video-exporter-config="(v: any) => { updateVideoExporterConfig(v); scheduleSave(); }"
        @set-ui-language="setUiLanguage"
        @notify="notify"
        @dismiss-notification="dismissNotification"
        @set-theme="(id: any) => setTheme(id)"
        @preview-custom-theme-color="(key: any, color: string) => previewCustomThemeColor(key, color)"
        @set-custom-theme-color="(key: any, color: string) => setCustomThemeColor(key, color)"
        @set-custom-theme-template="(templateId: any) => setCustomThemeTemplate(templateId)"
        @set-custom-theme-panel-opacity="setCustomThemePanelOpacity"
        @reset-custom-theme-colors="resetCustomThemeColors"
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
  </div>
</template>

<style scoped>
.app-surface:not(.overlay-surface) {
  border: 1px solid var(--color-border-dim);
  border-radius: 16px;
  box-shadow: 0 22px 70px rgba(0, 0, 0, 0.34);
  clip-path: inset(0 round 16px);
  contain: paint;
  overflow: hidden;
  transform-origin: center;
  animation: app-window-enter 180ms cubic-bezier(0.16, 1, 0.3, 1) both;
}

:global(.app-window-closing) .app-surface:not(.overlay-surface) {
  pointer-events: none;
  animation: app-window-exit 140ms cubic-bezier(0.4, 0, 1, 1) both;
}

.config-window-frame {
  --app-titlebar-height: 32px;

  display: grid;
  grid-template-rows: var(--app-titlebar-height) minmax(0, 1fr);
  height: 100vh;
  min-width: 0;
  clip-path: inset(0 round 16px);
  overflow: hidden;
}

@keyframes app-window-enter {
  from {
    opacity: 0;
    transform: translateY(10px) scale(0.985);
    filter: blur(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
    filter: blur(0);
  }
}

@keyframes app-window-exit {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
    filter: blur(0);
  }
  to {
    opacity: 0;
    transform: translateY(8px) scale(0.985);
    filter: blur(8px);
  }
}

@media (prefers-reduced-motion: reduce) {
  .app-surface:not(.overlay-surface),
  :global(.app-window-closing) .app-surface:not(.overlay-surface) {
    animation-duration: 1ms;
    filter: none;
  }
}
</style>
