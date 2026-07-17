export type SurfaceRole = "overlay" | "settings";

export type AppSurface = {
  id: "pov" | "config";
  title: string;
  role: SurfaceRole;
};

export type KeyBinding = {
  type?: "key";
  id: string;
  label: string;
  group: "mouse" | "movement" | "modifier" | "action";
  row?: number;
  widthUnit: number;
  platformLabels?: {
    macos?: string;
    windows?: string;
  };
};

export type GapBinding = {
  type: "gap" | "void";
  widthUnit: number;
};

export type OverlayRowItem = KeyBinding | GapBinding;
export type OverlayRow = OverlayRowItem[];

export type OverlayLayout = {
  unitPx: number;
  gapUnit: number;
};

export type OverlayBackgroundMode = "panel" | "transparent";

export type IdleKeyVisibility = "visible" | "faint" | "hidden";

export type OverlayStyle = {
  scale: number;
  opacity: number;
  backgroundMode: OverlayBackgroundMode;
  backgroundColor: string;
  backgroundOpacity: number;
  backgroundRadius: number;
  idleKeyVisibility: IdleKeyVisibility;
  alwaysOnTop: boolean;
  idleColor: string;
  activeColor: string;
  idleTextColor: string;
  activeTextColor: string;
};

export type RecordingConfig = {
  defaultFps: number;
  fpsOptions: number[];
  customFpsEnabled: boolean;
  customFps: number;
  maxFps: number;
  syncFeedbackEnabled: boolean;
  syncFeedbackDurationMs: number;
  formatExtension: ".kbdrec";
  primaryArtifact: "input-binary";
};

export type AppConfig = {
  surfaces: AppSurface[];
  layout: OverlayLayout;
  rows: OverlayRow[];
  keys: KeyBinding[];
  style: OverlayStyle;
  recording: RecordingConfig;
};

export function isKeyBinding(item: OverlayRowItem): item is KeyBinding {
  return item.type === undefined || item.type === "key";
}

export function flattenRowKeys(rows: OverlayRow[]): KeyBinding[] {
  return rows.flatMap((row, rowIndex) =>
    row
      .filter(isKeyBinding)
      .map((key) => ({
        ...key,
        type: "key" as const,
        row: rowIndex,
        widthUnit: key.widthUnit,
      })),
  );
}

export function createDefaultConfig(): AppConfig {
  const rows: OverlayRow[] = [
    [
      {
        type: "key",
        id: "mouse-left",
        label: "M1",
        group: "mouse",
        widthUnit: 1.35,
      },
      {
        type: "key",
        id: "mouse-right",
        label: "M2",
        group: "mouse",
        widthUnit: 1.35,
      },
    ],
    [
      { type: "gap", widthUnit: 1 },
      {
        type: "key",
        id: "w",
        label: "W",
        group: "movement",
        widthUnit: 1,
      },
      { type: "gap", widthUnit: 1 },
      {
        type: "key",
        id: "r",
        label: "R",
        group: "action",
        widthUnit: 1,
      },
    ],
    [
      {
        type: "key",
        id: "a",
        label: "A",
        group: "movement",
        widthUnit: 1,
      },
      {
        type: "key",
        id: "s",
        label: "S",
        group: "movement",
        widthUnit: 1,
      },
      {
        type: "key",
        id: "d",
        label: "D",
        group: "movement",
        widthUnit: 1,
      },
      {
        type: "key",
        id: "q",
        label: "Q",
        group: "action",
        widthUnit: 1,
      },
    ],
    [
      {
        type: "key",
        id: "shift-left",
        label: "Shift",
        group: "modifier",
        widthUnit: 1,
      },
      {
        type: "key",
        id: "ctrl-left",
        label: "Ctrl",
        group: "modifier",
        widthUnit: 1,
      },
      {
        type: "key",
        id: "e",
        label: "E",
        group: "action",
        widthUnit: 1,
      },
      { type: "gap", widthUnit: 1 },
    ],
    [
      {
        type: "key",
        id: "space",
        label: "Space",
        group: "modifier",
        widthUnit: 4,
      },
    ],
  ];

  return {
    surfaces: [
      {
        id: "pov",
        title: "POV Overlay",
        role: "overlay",
      },
      {
        id: "config",
        title: "Control Panel",
        role: "settings",
      },
    ],
    layout: {
      unitPx: 54,
      gapUnit: 0.15,
    },
    rows,
    keys: flattenRowKeys(rows),
    style: {
      scale: 1,
      opacity: 0.92,
      backgroundMode: "transparent",
      backgroundColor: "#0a0c0e",
      backgroundOpacity: 0.72,
      backgroundRadius: 8,
      idleKeyVisibility: "visible",
      alwaysOnTop: false,
      idleColor: "#121417",
      activeColor: "#25d366",
      idleTextColor: "#f5f7fa",
      activeTextColor: "#ffffff",
    },
    recording: {
      defaultFps: 60,
      fpsOptions: [30, 60, 120],
      customFpsEnabled: false,
      customFps: 600,
      maxFps: 1000,
      syncFeedbackEnabled: true,
      syncFeedbackDurationMs: 420,
      formatExtension: ".kbdrec",
      primaryArtifact: "input-binary",
    },
  };
}
