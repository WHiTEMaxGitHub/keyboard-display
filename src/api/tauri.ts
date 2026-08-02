import { invoke } from "@tauri-apps/api/core";
import type {
  RecordingInspection,
  RecordingMetadata,
  RecordingTreeNode,
} from "../types/recording";
import type { AppConfig } from "../domain/defaultConfig";
import type { VideoExporterStatus } from "../domain/videoExporter";

export type StopRecordingResult = {
  path: string;
};

export type InstallVideoExporterResult = {
  path: string;
};

export type ExportOverlayVideoResult = {
  outputPath: string;
  frameCount: number;
  width: number;
  height: number;
  fps: number;
};

/// 集中管理 Tauri command 名称，避免组件直接散落 invoke 字符串。
export const tauriApi = {
  saveConfigFile(path: string, contents: string) {
    return invoke<void>("save_config_file", { path, contents });
  },

  writeDebugLog(source: string, message: string) {
    return invoke<void>("write_debug_log", { source, message });
  },

  readConfigFile(path: string) {
    return invoke<string>("read_config_file", { path });
  },

  loadAppConfig() {
    return invoke<string | null>("load_app_config");
  },

  appConfigPath() {
    return invoke<string>("app_config_path_string");
  },

  saveAppConfig(contents: string) {
    return invoke<void>("save_app_config", { contents });
  },

  defaultRecordingDir() {
    return invoke<string>("default_recording_dir");
  },

  startRecording(fps: number) {
    return invoke<void>("start_recording", { fps });
  },

  recordInputEvent(keyId: string, pressed: boolean) {
    return invoke<void>("record_input_event", { keyId, pressed });
  },

  addRecordingMarker(name: string) {
    return invoke<void>("add_recording_marker", { name });
  },

  suppressRecordingKeys(keyIds: string[]) {
    return invoke<void>("suppress_recording_keys", { keyIds });
  },

  stopRecording(
    outputDir: string,
    filenameTemplate: string,
    profileName: string,
    fps: number,
  ) {
    return invoke<StopRecordingResult>("stop_recording", {
      outputDir,
      filenameTemplate,
      profileName,
      fps,
    });
  },

  inspectRecordingFile(path: string) {
    return invoke<RecordingInspection>("inspect_recording_file", { path });
  },

  listRecordingFiles(root: string) {
    return invoke<RecordingTreeNode>("list_recording_files", { root });
  },

  createRecordingFolder(root: string, folderName: string) {
    return invoke<RecordingTreeNode>("create_recording_folder", { root, folderName });
  },

  readRecordingMetadata(path: string) {
    return invoke<RecordingMetadata>("read_recording_metadata", { path });
  },

  saveRecordingMetadata(path: string, metadata: RecordingMetadata) {
    return invoke<RecordingMetadata>("save_recording_metadata", { path, metadata });
  },

  detectVideoExporter(userSelectedPath: string | null) {
    return invoke<VideoExporterStatus>("detect_video_exporter", { userSelectedPath });
  },

  installAppManagedVideoExporter() {
    return invoke<InstallVideoExporterResult>("install_app_managed_video_exporter");
  },

  uninstallAppManagedVideoExporter() {
    return invoke<void>("uninstall_app_managed_video_exporter");
  },

  exportOverlayVideo(
    recordingPath: string,
    outputPath: string,
    ffmpegPath: string,
    profile: Pick<AppConfig, "layout" | "rows" | "style" | "export" | "recording">,
  ) {
    return invoke<ExportOverlayVideoResult>("export_overlay_video", {
      recordingPath,
      outputPath,
      ffmpegPath,
      profile,
    });
  },

  copyFontFile(sourcePath: string) {
    return invoke<string>("copy_font_file", { sourcePath });
  },
};
