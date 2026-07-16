import { flattenRowKeys, type AppConfig, type OverlayRow } from "./defaultConfig";
import type { RecordingHotkeyConfig } from "./recordingHotkeys";

export type RecentProfile = {
  name: string;
  path: string;
};

export type CurrentProfile = {
  name: string;
  sourcePath: string | null;
  dirty: boolean;
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
      overlay: {
        rows?: OverlayRow[];
        keys?: AppConfig["keys"];
      };
    };
  };
  const rows =
    config.currentProfile.overlay.rows ?? rowsFromKeys(config.currentProfile.overlay.keys ?? []);

  return {
    ...config,
    currentProfile: {
      ...config.currentProfile,
      overlay: {
        ...config.currentProfile.overlay,
        rows,
        keys: flattenRowKeys(rows),
      },
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
