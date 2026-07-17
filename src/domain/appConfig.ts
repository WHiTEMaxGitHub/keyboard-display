import { createDefaultConfig, flattenRowKeys, type AppConfig, type OverlayRow } from "./defaultConfig";
import { normalizeRecordingConfig } from "./recordingConfig";
import { normalizeRecordingHotkeyConfig, type RecordingHotkeyConfig } from "./recordingHotkeys";

export type RecentProfile = {
  name: string;
  path: string;
};

export type CurrentProfile = {
  name: string;
  sourcePath: string | null;
  changed: boolean;
  recording: AppConfig["recording"];
  overlay: {
    visible: boolean;
    position: string;
    layout: AppConfig["layout"];
    style: AppConfig["style"];
    rows: AppConfig["rows"];
    keys: AppConfig["keys"];
  };
};

type PersistedCurrentProfile = Omit<CurrentProfile, "overlay"> & {
  overlay: Omit<CurrentProfile["overlay"], "keys">;
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
    silent?: boolean;
    hotkeys: RecordingHotkeyConfig;
  };
  ui: {
    language: string;
  };
};

export type PersistedAppConfigFile = Omit<AppConfigFile, "currentProfile"> & {
  currentProfile: PersistedCurrentProfile;
};

export function buildAppConfigFile({
  defaultProfilePath,
  currentProfile,
  recording,
}: {
  defaultProfilePath: string;
  currentProfile: CurrentProfile;
  recording: AppConfigFile["recording"];
}): PersistedAppConfigFile {
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
    currentProfile: {
      ...currentProfile,
      recording: currentProfile.recording,
      overlay: {
        visible: currentProfile.overlay.visible,
        position: currentProfile.overlay.position,
        layout: currentProfile.overlay.layout,
        style: currentProfile.overlay.style,
        rows: currentProfile.overlay.rows,
      },
    },
    recording,
    ui: {
      language: "system",
    },
  };
}

export function parseAppConfigFile(text: string): AppConfigFile {
  const config = JSON.parse(text) as PersistedAppConfigFile & {
    currentProfile: {
      dirty?: boolean;
      changed?: boolean;
      overlay: {
        rows?: OverlayRow[];
        keys?: AppConfig["keys"];
      };
    };
  };
  const rows =
    config.currentProfile.overlay.rows ?? rowsFromKeys(config.currentProfile.overlay.keys ?? []);
  const profileRecording = normalizeRecordingConfig(
    config.currentProfile.recording ?? createDefaultConfig().recording,
  );

  return {
    ...config,
    currentProfile: {
      ...config.currentProfile,
      changed: config.currentProfile.changed ?? config.currentProfile.dirty ?? false,
      recording: profileRecording,
      overlay: {
        ...config.currentProfile.overlay,
        rows,
        keys: flattenRowKeys(rows),
      },
    },
    recording: {
      ...config.recording,
      hotkeys: normalizeRecordingHotkeyConfig(config.recording.hotkeys),
    },
  };
}

function rowsFromKeys(keys: AppConfig["keys"]): OverlayRow[] {
  const rowMap = new Map<number, AppConfig["keys"]>();

  for (const key of keys) {
    const row = key.row ?? 0;
    rowMap.set(row, [...(rowMap.get(row) ?? []), { ...key, type: "key" }]);
  }

  return [...rowMap.entries()]
    .sort(([left], [right]) => left - right)
    .map(([, row]) => row);
}
