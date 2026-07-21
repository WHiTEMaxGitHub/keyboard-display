import { invoke } from "@tauri-apps/api/core";
import type {
  RecordingInspection,
  RecordingMetadata,
  RecordingTreeNode,
} from "../types/recording";

export type StopRecordingResult = {
  path: string;
};

/// 集中管理 Tauri command 名称，避免组件直接散落 invoke 字符串。
export const tauriApi = {
  saveConfigFile(path: string, contents: string) {
    return invoke<void>("save_config_file", { path, contents });
  },

  readConfigFile(path: string) {
    return invoke<string>("read_config_file", { path });
  },

  loadAppConfig() {
    return invoke<string | null>("load_app_config");
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
};
