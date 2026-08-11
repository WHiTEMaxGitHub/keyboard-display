import {
  createDefaultConfig,
  flattenRowKeys,
  type AppConfig,
  type OverlayCustomPosition,
  type OverlayRow,
} from "./defaultConfig";
import { normalizeRecordingConfig } from "./recordingConfig";
import { normalizeRecordingHotkeyConfig, type RecordingHotkeyConfig } from "./recordingHotkeys";
import {
  createDefaultVideoExporterConfig,
  normalizeVideoExporterConfig,
  type VideoExporterConfig,
} from "./videoExporter";
import { sanitizeExportFilenameTemplate } from "./exportFilename";
import {
  createProfileConfig,
  DEFAULT_BUILT_IN_PROFILE_ID,
  type BuiltInProfileId,
} from "./profileTemplates";
import { normalizeUiLanguage, type UiLanguage } from "./uiLanguage";

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
    keyIdLabels: AppConfig["keyIdLabels"];
    customPosition?: OverlayCustomPosition | null;
  };
};

type PersistedCurrentProfile = Omit<CurrentProfile, "overlay"> & {
  overlay: Omit<CurrentProfile["overlay"], "keys" | "keyIdLabels"> & {
    keyIdLabels?: AppConfig["keyIdLabels"];
  };
};

export type AppConfigFile = {
  version: 1;
  profiles: {
    defaultProfileId: BuiltInProfileId;
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
    language: UiLanguage;
  };
};

export type PersistedAppConfigFile = Omit<AppConfigFile, "currentProfile"> & {
  currentProfile: PersistedCurrentProfile;
};

export function buildAppConfigFile({
  defaultProfileId = DEFAULT_BUILT_IN_PROFILE_ID,
  currentProfile,
  recording,
  exporter,
  ui,
}: {
  defaultProfileId?: BuiltInProfileId;
  currentProfile: CurrentProfile;
  recording: AppConfigFile["recording"];
  exporter: AppConfigFile["exporter"];
  ui?: Partial<AppConfigFile["ui"]>;
}): PersistedAppConfigFile {
  return {
    version: 1,
    profiles: {
      defaultProfileId,
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
        keyIdLabels: currentProfile.overlay.keyIdLabels,
        customPosition: currentProfile.overlay.customPosition ?? null,
      },
    },
    recording,
    exporter: {
      video: normalizeVideoExporterConfig(exporter.video),
    },
    ui: {
      language: normalizeUiLanguage(ui?.language),
    },
  };
}

export function createInitialAppConfigFile(
  defaultProfileId: BuiltInProfileId = DEFAULT_BUILT_IN_PROFILE_ID,
): PersistedAppConfigFile {
  const profile = createProfileConfig(defaultProfileId);

  return buildAppConfigFile({
    defaultProfileId,
    currentProfile: {
      name: profile.name ?? "Keyboard Display",
      sourcePath: null,
      changed: false,
      recording: profile.recording,
      export: profile.export,
      overlay: {
        visible: profile.overlay.visible ?? true,
        position: profile.overlay.position ?? "bottom-right",
        layout: profile.overlay.layout,
        style: profile.overlay.style,
        rows: profile.overlay.rows,
        keys: profile.overlay.keys,
        keyIdLabels: profile.overlay.keyIdLabels ?? {},
        customPosition: null,
      },
    },
    recording: {
      outputDirectory: null,
      browserDirectory: null,
      silent: false,
      hotkeys: createDefaultRecordingHotkeys(),
    },
    exporter: {
      video: createDefaultVideoExporterConfig(),
    },
    ui: {
      language: "system",
    },
  });
}

export function parseAppConfigFile(text: string): AppConfigFile {
  const config = JSON.parse(text) as PersistedAppConfigFile & {
    recording?: Partial<AppConfigFile["recording"]>;
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
  const recording = config.recording ?? {};

  return {
    ...config,
    profiles: {
      defaultProfileId: normalizeBuiltInProfileId(config.profiles?.defaultProfileId),
    },
    currentProfile: {
      ...config.currentProfile,
      changed: config.currentProfile.changed ?? config.currentProfile.dirty ?? false,
      recording: profileRecording,
      export: profileExport,
      overlay: {
        ...config.currentProfile.overlay,
        rows,
        keys: flattenRowKeys(rows),
        keyIdLabels: normalizeKeyIdLabels(config.currentProfile.overlay.keyIdLabels),
      },
    },
    recording: {
      outputDirectory: cleanOptionalPath(recording.outputDirectory),
      browserDirectory: cleanOptionalPath(recording.browserDirectory),
      silent: recording.silent ?? false,
      hotkeys: normalizeRecordingHotkeyConfig(recording.hotkeys),
    },
    exporter: {
      video: normalizeVideoExporterConfig(
        config.exporter?.video ?? createDefaultVideoExporterConfig(),
      ),
    },
    ui: {
      language: normalizeUiLanguage(config.ui?.language),
    },
  };
}

function createDefaultRecordingHotkeys(): RecordingHotkeyConfig {
  return normalizeRecordingHotkeyConfig(undefined);
}

function normalizeBuiltInProfileId(value: unknown): BuiltInProfileId {
  return value === "left-keyboard" || value === "68-keyboard" || value === "default"
    ? value
    : DEFAULT_BUILT_IN_PROFILE_ID;
}

function normalizeKeyIdLabels(labels: AppConfig["keyIdLabels"] | undefined): AppConfig["keyIdLabels"] {
  if (!labels) {
    return {};
  }

  return Object.fromEntries(
    Object.entries(labels)
      .map(([keyId, label]) => [keyId.trim(), label.trim()])
      .filter(([keyId, label]) => keyId && label),
  );
}

function normalizeExportConfig(exportConfig: Partial<AppConfig["export"]>): AppConfig["export"] {
  const defaultExport = createDefaultConfig().export;

  return {
    defaultFormat: exportConfig.defaultFormat ?? defaultExport.defaultFormat,
    transparentFormat: exportConfig.transparentFormat ?? defaultExport.transparentFormat,
    compatibleFormat: exportConfig.compatibleFormat ?? defaultExport.compatibleFormat,
    renderMarkers: exportConfig.renderMarkers ?? true,
    filenameTemplate: sanitizeExportFilenameTemplate(
      exportConfig.filenameTemplate ?? defaultExport.filenameTemplate,
    ),
    fontPath: exportConfig.fontPath ?? defaultExport.fontPath,
    renderThreads: exportConfig.renderThreads ?? defaultExport.renderThreads,
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


function cleanOptionalPath(path: string | null | undefined): string | null {
  const trimmedPath = path?.trim() ?? "";
  return trimmedPath ? trimmedPath : null;
}
