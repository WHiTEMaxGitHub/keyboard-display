export type SurfaceRole = "overlay" | "settings";

export type AppSurface = {
  id: "pov" | "config";
  title: string;
  role: SurfaceRole;
};

export type KeyBinding = {
  id: string;
  label: string;
  group: "mouse" | "movement" | "modifier" | "action";
  gridArea?: string;
  row?: number;
  widthUnit: number;
  platformLabels?: {
    macos?: string;
    windows?: string;
  };
};

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
  formatExtension: ".kbdrec";
  primaryArtifact: "input-binary";
};

export type AppConfig = {
  surfaces: AppSurface[];
  layout: OverlayLayout;
  keys: KeyBinding[];
  style: OverlayStyle;
  recording: RecordingConfig;
};

export function createDefaultConfig(): AppConfig {
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
    keys: [
      {
        id: "mouse-left",
        label: "M1",
        group: "mouse",
        gridArea: "mouseLeft",
        widthUnit: 1.35,
      },
      {
        id: "mouse-right",
        label: "M2",
        group: "mouse",
        gridArea: "mouseRight",
        widthUnit: 1.35,
      },
      {
        id: "w",
        label: "W",
        group: "movement",
        gridArea: "w",
        widthUnit: 1,
      },
      {
        id: "a",
        label: "A",
        group: "movement",
        gridArea: "a",
        widthUnit: 1,
      },
      {
        id: "s",
        label: "S",
        group: "movement",
        gridArea: "s",
        widthUnit: 1,
      },
      {
        id: "d",
        label: "D",
        group: "movement",
        gridArea: "d",
        widthUnit: 1,
      },
      {
        id: "shift-left",
        label: "Shift",
        group: "modifier",
        gridArea: "shift",
        widthUnit: 1,
      },
      {
        id: "ctrl-left",
        label: "Ctrl",
        group: "modifier",
        gridArea: "ctrl",
        widthUnit: 1,
      },
      {
        id: "space",
        label: "Space",
        group: "modifier",
        gridArea: "space",
        widthUnit: 4,
      },
      {
        id: "r",
        label: "R",
        group: "action",
        gridArea: "r",
        widthUnit: 1,
      },
      {
        id: "q",
        label: "Q",
        group: "action",
        gridArea: "q",
        widthUnit: 1,
      },
      {
        id: "e",
        label: "E",
        group: "action",
        gridArea: "e",
        widthUnit: 1,
      },
    ],
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
      formatExtension: ".kbdrec",
      primaryArtifact: "input-binary",
    },
  };
}
