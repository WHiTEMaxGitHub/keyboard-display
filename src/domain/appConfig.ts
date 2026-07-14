import type { AppConfig } from "./defaultConfig";
import type { RecordingHotkeyConfig } from "./recordingHotkeys";

export type RecentProfile = {
  name: string;
  path: string;
};

export type CurrentProfile = {
  name: string;
  sourcePath: string | null;
  dirty: boolean;
  overlay: {
    visible: boolean;
    position: string;
    layout: AppConfig["layout"];
    style: AppConfig["style"];
    keys: AppConfig["keys"];
  };
};

export type AppConfigFile = {
  version: 1;
  profiles: {
    defaultProfilePath: string;
    lastProfilePath: string | null;
    recentProfiles: RecentProfile[];
  };
  currentProfile: CurrentProfile;
  recording: {
    outputDirectory: string | null;
    hotkeys: RecordingHotkeyConfig;
  };
  ui: {
    language: string;
  };
};

export function buildAppConfigFile({
  defaultProfilePath,
  currentProfile,
  recording,
}: {
  defaultProfilePath: string;
  currentProfile: CurrentProfile;
  recording: AppConfigFile["recording"];
}): AppConfigFile {
  const recentProfiles = currentProfile.sourcePath
    ? [{ name: currentProfile.name, path: currentProfile.sourcePath }]
    : [];

  return {
    version: 1,
    profiles: {
      defaultProfilePath,
      lastProfilePath: currentProfile.sourcePath,
      recentProfiles,
    },
    currentProfile,
    recording,
    ui: {
      language: "system",
    },
  };
}

export function parseAppConfigFile(text: string): AppConfigFile {
  return JSON.parse(text) as AppConfigFile;
}
