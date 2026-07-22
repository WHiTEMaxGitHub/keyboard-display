import { createDefaultConfig, flattenRowKeys, type AppConfig, type OverlayRow } from "./defaultConfig";
import { normalizeRecordingConfig } from "./recordingConfig";
import { normalizeRecordingHotkeyConfig, type RecordingHotkeyConfig } from "./recordingHotkeys";
import {
  createDefaultVideoExporterConfig,
  normalizeVideoExporterConfig,
  type VideoExporterConfig,
} from "./videoExporter";

export type RecentProfile = {
  name: string;
  path: string;
};

export const MAX_RECENT_PROFILES = 8;

export type CurrentProfile = {
  name: string;
  sourcePath: string | null;
  changed: boolean;
  recording: AppConfig["recording"];
  export: AppConfig["export"];
  overlay: {
    visible: boolean;
    position: string;
    layout: AppConfig["layout"];
    style: AppConfig["style"];
    rows: AppConfig["rows"];
    keys: AppConfig["keys"];
    customPosition?: {
      x: number;
      y: number;
    } | null;
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
    browserDirectory?: string | null;
    silent?: boolean;
    hotkeys: RecordingHotkeyConfig;
  };
  exporter: {
    video: VideoExporterConfig;
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
  recentProfiles,
  currentProfile,
  recording,
  exporter,
}: {
  defaultProfilePath: string;
  recentProfiles: RecentProfile[];
  currentProfile: CurrentProfile;
  recording: AppConfigFile["recording"];
  exporter: AppConfigFile["exporter"];
}): PersistedAppConfigFile {
  const nextRecentProfiles = mergeRecentProfiles(recentProfiles, currentProfile);

  return {
    version: 1,
    profiles: {
      defaultProfilePath,
      lastProfilePath: currentProfile.sourcePath,
      recentProfiles: nextRecentProfiles,
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
        customPosition: currentProfile.overlay.customPosition ?? null,
      },
    },
    recording,
    exporter: {
      video: normalizeVideoExporterConfig(exporter.video),
    },
    ui: {
      language: "system",
    },
  };
}

export function mergeRecentProfiles(
  recentProfiles: RecentProfile[],
  currentProfile: Pick<CurrentProfile, "name" | "sourcePath">,
): RecentProfile[] {
  const nextProfiles = currentProfile.sourcePath
    ? [{ name: currentProfile.name, path: currentProfile.sourcePath }, ...recentProfiles]
    : [...recentProfiles];
  const seenPaths = new Set<string>();
  const dedupedProfiles: RecentProfile[] = [];

  for (const profile of nextProfiles) {
    const path = profile.path.trim();
    if (!path || seenPaths.has(path)) {
      continue;
    }

    seenPaths.add(path);
    dedupedProfiles.push({
      name: profile.name.trim() || profileNameFromPath(path),
      path,
    });
  }

  return dedupedProfiles.slice(0, MAX_RECENT_PROFILES);
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
  const profileExport = normalizeExportConfig(
    config.currentProfile.export ?? createDefaultConfig().export,
  );

  return {
    ...config,
    currentProfile: {
      ...config.currentProfile,
      changed: config.currentProfile.changed ?? config.currentProfile.dirty ?? false,
      recording: profileRecording,
      export: profileExport,
      overlay: {
        ...config.currentProfile.overlay,
        rows,
        keys: flattenRowKeys(rows),
      },
    },
    recording: {
      ...config.recording,
      browserDirectory: cleanOptionalPath(config.recording.browserDirectory),
      hotkeys: normalizeRecordingHotkeyConfig(config.recording.hotkeys),
    },
    exporter: {
      video: normalizeVideoExporterConfig(
        config.exporter?.video ?? createDefaultVideoExporterConfig(),
      ),
    },
  };
}

function normalizeExportConfig(exportConfig: Partial<AppConfig["export"]>): AppConfig["export"] {
  const defaultExport = createDefaultConfig().export;

  return {
    defaultFormat: exportConfig.defaultFormat ?? defaultExport.defaultFormat,
    transparentFormat: exportConfig.transparentFormat ?? defaultExport.transparentFormat,
    compatibleFormat: exportConfig.compatibleFormat ?? defaultExport.compatibleFormat,
    renderMarkers: exportConfig.renderMarkers ?? true,
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

function profileNameFromPath(path: string): string {
  const fileName = path.split(/[\\/]/).pop() ?? path;
  return fileName.replace(/\.json$/i, "") || path;
}

function cleanOptionalPath(path: string | null | undefined): string | null {
  const trimmedPath = path?.trim() ?? "";
  return trimmedPath ? trimmedPath : null;
}
