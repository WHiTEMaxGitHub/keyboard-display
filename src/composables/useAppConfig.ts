import { emitTo } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { reactive, ref, type ComputedRef } from "vue";
import { tauriApi } from "../api/tauri";
import { i18n } from "../i18n";
import {
  parseAppConfigFile,
  type RecentProfile,
} from "../domain/appConfig";
import { buildConfigFileJson, parseConfigFile } from "../domain/configFile";
import {
  createDefaultConfig,
  flattenRowKeys,
  type ExportConfig,
  type OverlayCustomPosition,
  type OverlayStyle,
} from "../domain/defaultConfig";
import {
  OVERLAY_CONFIG_EVENT,
} from "../domain/inputEvents";
import type { OverlayRuntimeConfig } from "./useOverlayWindow";
import type { RecordingConfig } from "../domain/defaultConfig";
import {
  createDefaultVideoExporterConfig,
  type VideoExporterConfig,
} from "../domain/videoExporter";
import {
  normalizeOverlayPosition,
  type OverlayPosition,
  type OverlayStyleSyncMode,
} from "./useOverlayWindow";
import { normalizeUiLanguage, type UiLanguage } from "../domain/uiLanguage";

export type OverlayCallbacks = {
  resizeOverlayWindow: () => Promise<void>;
  syncOverlayStyle: (style: OverlayStyle, mode: OverlayStyleSyncMode) => Promise<void>;
  syncOverlayRows: () => Promise<void>;
  setOverlayVisible: (visible: boolean, markChanged?: boolean) => Promise<void>;
  moveOverlay: (
    position: OverlayPosition,
    markChanged?: boolean,
    show?: boolean,
  ) => Promise<void>;
};

export type AppConfigSaveContext = {
  recordingDirectory: string;
  recordingBrowserDirectory: string;
  silentRecording: boolean;
  recordingHotkeys: any;
};

export function useAppConfig(options: {
  isOverlayWindow: ComputedRef<boolean>;
}) {
  const config = reactive(createDefaultConfig());
  const isOverlayVisible = ref(true);
  const profileName = ref("CS POV");
  const profileSourcePath = ref<string | null>(null);
  const profileChanged = ref(false);
  const recentProfiles = ref<RecentProfile[]>([]);
  const overlayPosition = ref<OverlayPosition>("bottom-right");
  const customOverlayPosition = ref<OverlayCustomPosition | null>(null);
  const syncFeedbackActive = ref(false);
  const videoExporterConfig = ref<VideoExporterConfig>(
    createDefaultVideoExporterConfig(),
  );
  const recordingBrowserDirectory = ref("");
  const uiLanguage = ref<UiLanguage>("system");

  let appConfigSaveTimer: number | undefined;

  function t(key: string, params?: Record<string, unknown>) {
    return i18n.global.t(key, params ?? {});
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

  function applyKeyIdLabels(keyIdLabels: typeof config.keyIdLabels) {
    config.keyIdLabels = { ...keyIdLabels };
  }

  function applyRecordingConfig(recording: RecordingConfig) {
    config.recording = { ...recording };
  }

  function applyExportConfig(exportConfig: ExportConfig) {
    config.export = { ...exportConfig };
  }

  function markProfileChanged() {
    profileChanged.value = true;
  }

  function profileNameFromFileName(fileName: string): string {
    return fileName.replace(/\.json$/i, "");
  }

  function isBackplateVisible(style: OverlayStyle) {
    return (
      !/^#[0-9a-fA-F]{8}$/.test(style.backgroundColor) ||
      !style.backgroundColor.endsWith("00")
    );
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

  function scheduleAppConfigSave(saveFn: () => Promise<void>) {
    if (options.isOverlayWindow.value) {
      return;
    }

    if (appConfigSaveTimer !== undefined) {
      window.clearTimeout(appConfigSaveTimer);
    }

    appConfigSaveTimer = window.setTimeout(() => {
      void saveFn();
    }, 300);
  }

  async function applyLoadedConfig(
    text: string,
    fileName: string,
    sourcePath: string | null,
    overlay: OverlayCallbacks,
    onSave: () => void,
  ) {
    const loadedConfig = parseConfigFile(text);
    profileName.value =
      loadedConfig.name || profileNameFromFileName(fileName);
    profileSourcePath.value = sourcePath;
    profileChanged.value = false;
    overlayPosition.value = normalizeOverlayPosition(
      loadedConfig.overlay.position,
    );
    customOverlayPosition.value =
      loadedConfig.overlay.customPosition ?? null;

    applyOverlayLayout(loadedConfig.overlay.layout);
    applyOverlayRows(loadedConfig.overlay.rows);
    applyKeyIdLabels(loadedConfig.overlay.keyIdLabels ?? {});
    applyOverlayStyle(loadedConfig.overlay.style);
    applyRecordingConfig(loadedConfig.recording);
    applyExportConfig(loadedConfig.export);
    await overlay.resizeOverlayWindow();

    const overlayConfig: OverlayRuntimeConfig = {
      layout: loadedConfig.overlay.layout,
      rows: loadedConfig.overlay.rows,
      keys: loadedConfig.overlay.keys,
      keyIdLabels: loadedConfig.overlay.keyIdLabels ?? {},
      style: loadedConfig.overlay.style,
    };
    await emitTo("pov", OVERLAY_CONFIG_EVENT, overlayConfig);
    const visible = loadedConfig.overlay.visible ?? true;
    await overlay.setOverlayVisible(visible, false);
    if (visible) {
      await overlay.moveOverlay(overlayPosition.value, false);
    }
    onSave();
  }

  async function loadConfig(
    overlay: OverlayCallbacks,
    onSave: () => void,
  ) {
    const selectedPath = await open({
      title: t("dialogs.loadConfig"),
      filters: [{ name: t("dialogs.jsonFilter"), extensions: ["json"] }],
      multiple: false,
    });

    if (typeof selectedPath !== "string") {
      return;
    }

    const text = await tauriApi.readConfigFile(selectedPath);
    await applyLoadedConfig(
      text,
      selectedPath.split(/[\\/]/).pop() ?? selectedPath,
      selectedPath,
      overlay,
      onSave,
    );
  }

  async function loadRecentProfile(
    path: string,
    overlay: OverlayCallbacks,
    onSave: () => void,
  ) {
    const text = await tauriApi.readConfigFile(path);
    await applyLoadedConfig(
      text,
      path.split(/[\\/]/).pop() ?? path,
      path,
      overlay,
      onSave,
    );
  }

  async function restoreAppConfig(
    overlay: OverlayCallbacks,
    initializeDefaultRecordingDirectory: () => Promise<void>,
    onRestore: (recording: {
      directory: string;
      browserDirectory: string;
      silent: boolean;
      hotkeys: any;
    }) => void,
  ) {
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
    overlayPosition.value = normalizeOverlayPosition(
      appConfig.currentProfile.overlay.position,
    );
    customOverlayPosition.value =
      appConfig.currentProfile.overlay.customPosition ?? null;
    recordingBrowserDirectory.value =
      appConfig.recording.browserDirectory ?? "";
    videoExporterConfig.value = appConfig.exporter.video;
    uiLanguage.value = appConfig.ui.language;

    onRestore({
      directory: appConfig.recording.outputDirectory ?? "",
      browserDirectory: appConfig.recording.browserDirectory ?? "",
      silent: appConfig.recording.silent ?? false,
      hotkeys: appConfig.recording.hotkeys,
    });

    applyOverlayLayout(appConfig.currentProfile.overlay.layout);
    applyOverlayRows(appConfig.currentProfile.overlay.rows);
    applyKeyIdLabels(appConfig.currentProfile.overlay.keyIdLabels);
    applyOverlayStyle(appConfig.currentProfile.overlay.style);
    applyRecordingConfig(appConfig.currentProfile.recording);
    applyExportConfig(appConfig.currentProfile.export);

    const overlayConfig: OverlayRuntimeConfig = {
      layout: appConfig.currentProfile.overlay.layout,
      rows: appConfig.currentProfile.overlay.rows,
      keys: appConfig.currentProfile.overlay.keys,
      keyIdLabels: appConfig.currentProfile.overlay.keyIdLabels,
      style: appConfig.currentProfile.overlay.style,
    };
    await emitTo("pov", OVERLAY_CONFIG_EVENT, overlayConfig);

    await overlay.setOverlayVisible(
      appConfig.currentProfile.overlay.visible,
      false,
    );
    if (appConfig.currentProfile.overlay.visible) {
      await overlay.moveOverlay(overlayPosition.value, false);
    }
  }

  async function applyConfigToOverlay(overlay: OverlayCallbacks) {
    await overlay.resizeOverlayWindow();
    const overlayConfig: OverlayRuntimeConfig = {
      layout: config.layout,
      rows: config.rows,
      keys: config.keys,
      keyIdLabels: config.keyIdLabels,
      style: config.style,
    };
    await emitTo("pov", OVERLAY_CONFIG_EVENT, overlayConfig);
    await overlay.setOverlayVisible(isOverlayVisible.value);
  }

  async function exportAndApplyConfig(overlay: OverlayCallbacks, onSave: () => void) {
    await applyConfigToOverlay(overlay);

    const json = buildConfigFileJson({
      name: profileName.value,
      config,
      visible: isOverlayVisible.value,
      position: overlayPosition.value,
      customPosition: customOverlayPosition.value,
    });
    const path = await save({
      title: t("dialogs.saveConfig"),
      defaultPath: `${profileName.value || "keyboard-display"}.json`,
      filters: [{ name: t("dialogs.jsonFilter"), extensions: ["json"] }],
    });

    if (!path) {
      return;
    }

    await tauriApi.saveConfigFile(path, json);
    profileSourcePath.value = path;
    profileChanged.value = false;
    onSave();
  }

  async function overwriteAndApplyConfig(
    overlay: OverlayCallbacks,
    onSave: () => void,
  ) {
    await applyConfigToOverlay(overlay);

    if (!profileSourcePath.value) {
      await exportAndApplyConfig(overlay, onSave);
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
    onSave();
  }

  function updateRecordingConfig(recording: RecordingConfig) {
    applyRecordingConfig(recording);
    markProfileChanged();
  }

  function updateExportConfig(exportConfig: ExportConfig) {
    applyExportConfig(exportConfig);
    markProfileChanged();
  }

  function updateVideoExporterConfig(exporterConfig: VideoExporterConfig) {
    videoExporterConfig.value = exporterConfig;
  }

  function updateUiLanguage(language: UiLanguage) {
    uiLanguage.value = normalizeUiLanguage(language);
  }

  async function chooseRecordingBrowserDirectory() {
    const selectedPath = await open({
      title: t("dialogs.recordingFilesFolder"),
      directory: true,
      multiple: false,
    });

    if (typeof selectedPath === "string") {
      recordingBrowserDirectory.value = selectedPath;
    }
  }

  function dispose() {
    if (appConfigSaveTimer !== undefined) {
      window.clearTimeout(appConfigSaveTimer);
    }
  }

  return {
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
    uiLanguage,
    applyOverlayStyle,
    applyOverlayLayout,
    applyOverlayRows,
    applyKeyIdLabels,
    markProfileChanged,
    isBackplateVisible,
    overlayStyleSyncMode,
    scheduleAppConfigSave,
    applyLoadedConfig,
    loadConfig,
    loadRecentProfile,
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
  };
}
