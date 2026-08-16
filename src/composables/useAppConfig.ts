import { emitTo } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { computed, reactive, ref, type ComputedRef } from "vue";
import { tauriApi } from "../api/tauri";
import { i18n } from "../i18n";
import { parseAppConfigFile } from "../domain/appConfig";
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
  setOverlayVisible: (visible: boolean, updateConfig?: boolean) => Promise<void>;
  moveOverlay: (
    position: OverlayPosition,
    updateConfig?: boolean,
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
  const overlayPosition = ref<OverlayPosition>("bottom-right");
  const customOverlayPosition = ref<OverlayCustomPosition | null>(null);
  const syncFeedbackActive = ref(false);
  const videoExporterConfig = ref<VideoExporterConfig>(
    createDefaultVideoExporterConfig(),
  );
  const recordingBrowserDirectory = ref("");
  const uiLanguage = ref<UiLanguage>("system");

  let appConfigSaveTimer: number | undefined;
  const savedProfileJson = ref(buildCurrentProfileJson());
  const profileChanged = computed(() => buildCurrentProfileJson() !== savedProfileJson.value);

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

  function setSavedProfileJson(json = buildCurrentProfileJson()) {
    savedProfileJson.value = json;
  }

  function buildCurrentProfileJson() {
    return buildConfigFileJson({
      name: profileName.value,
      visible: isOverlayVisible.value,
      position: overlayPosition.value,
      customPosition: customOverlayPosition.value,
      config,
    });
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
    isOverlayVisible.value = visible;
    setSavedProfileJson();
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

  async function restoreAppConfig(
    overlay: OverlayCallbacks,
    onRestore: (recording: {
      directory: string;
      browserDirectory: string;
      silent: boolean;
      hotkeys: any;
    }) => void,
  ) {
    const savedConfig = await tauriApi.loadAppConfig();
    if (!savedConfig) {
      return;
    }

    const appConfig = parseAppConfigFile(savedConfig);
    profileName.value = appConfig.currentProfile.name;
    profileSourcePath.value = appConfig.currentProfile.sourcePath;
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

    isOverlayVisible.value = appConfig.currentProfile.overlay.visible;
    savedProfileJson.value = await restoreSavedProfileJson(appConfig.currentProfile.sourcePath);
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
    await overlay.setOverlayVisible(isOverlayVisible.value, false);
  }

  async function exportAndApplyConfig(overlay: OverlayCallbacks, onSave: () => void) {
    await applyConfigToOverlay(overlay);

    const json = buildCurrentProfileJson();
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
    setSavedProfileJson(json);
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

    const json = buildCurrentProfileJson();
    await tauriApi.saveConfigFile(profileSourcePath.value, json);
    setSavedProfileJson(json);
    onSave();
  }

  async function restoreSavedProfileJson(sourcePath: string | null) {
    if (!sourcePath) {
      return buildCurrentProfileJson();
    }

    try {
      const text = await tauriApi.readConfigFile(sourcePath);
      const loadedConfig = parseConfigFile(text);
      return buildConfigFileJson({
        name: loadedConfig.name || profileNameFromFileName(sourcePath.split(/[\\/]/).pop() ?? sourcePath),
        config: {
          ...config,
          layout: loadedConfig.overlay.layout,
          rows: loadedConfig.overlay.rows,
          keys: loadedConfig.overlay.keys,
          keyIdLabels: loadedConfig.overlay.keyIdLabels ?? {},
          style: loadedConfig.overlay.style,
          recording: loadedConfig.recording,
          export: loadedConfig.export,
        },
        visible: loadedConfig.overlay.visible ?? true,
        position: normalizeOverlayPosition(loadedConfig.overlay.position),
        customPosition: loadedConfig.overlay.customPosition ?? null,
      });
    } catch {
      return "";
    }
  }

  function updateRecordingConfig(recording: RecordingConfig) {
    applyRecordingConfig(recording);
  }

  function updateExportConfig(exportConfig: ExportConfig) {
    applyExportConfig(exportConfig);
  }

  function updateVideoExporterConfig(exporterConfig: VideoExporterConfig) {
    videoExporterConfig.value = exporterConfig;
  }

  function updateProfileName(name: string) {
    profileName.value = name;
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
    isBackplateVisible,
    overlayStyleSyncMode,
    scheduleAppConfigSave,
    applyLoadedConfig,
    loadConfig,
    restoreAppConfig,
    applyConfigToOverlay,
    exportAndApplyConfig,
    overwriteAndApplyConfig,
    updateRecordingConfig,
    updateExportConfig,
    updateVideoExporterConfig,
    updateProfileName,
    updateUiLanguage,
    chooseRecordingBrowserDirectory,
    dispose,
  };
}
