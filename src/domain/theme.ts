export type ThemeId = "vibrant" | "dark" | "midnight" | "light" | "custom";

export type CustomThemeColorKey =
  | "accent"
  | "accentBlue"
  | "accentViolet"
  | "accentEmerald";

export type CustomThemeColors = Record<CustomThemeColorKey, string>;

export type Theme = {
  id: ThemeId;
  label: string;
  css: Record<string, string>;
};

export const THEMES: Record<ThemeId, Theme> = {
  vibrant: {
    id: "vibrant",
    label: "Vibrant",
    css: {
      "--theme-bg-base": "linear-gradient(135deg, #0c1026, #101a35 46%, #1b102a)",
      "--theme-flow-1": "rgba(45, 184, 220, 0.26)",
      "--theme-flow-2": "rgba(105, 86, 216, 0.23)",
      "--theme-flow-3": "rgba(214, 82, 146, 0.17)",
      "--theme-flow-4": "rgba(224, 166, 76, 0.14)",
      "--theme-orb-1": "rgba(45, 184, 220, 0.13)",
      "--theme-orb-2": "rgba(105, 86, 216, 0.12)",
      "--theme-orb-3": "rgba(214, 82, 146, 0.10)",
      "--theme-glass-from": "rgba(255, 255, 255, 0.125)",
      "--theme-glass-to": "rgba(255, 255, 255, 0.055)",
      "--theme-glass-border": "rgba(255, 255, 255, 0.16)",
      "--theme-glass-inset": "rgba(255, 255, 255, 0.18)",
      "--theme-accent": "#2db8dc",
      "--theme-accent-blue": "#4f8ed6",
      "--theme-accent-violet": "#6956d8",
      "--theme-accent-emerald": "#2fc99e",
      "--theme-text-primary": "#f8f6ff",
      "--theme-text-body": "rgba(248, 246, 255, 0.86)",
      "--theme-text-secondary": "rgba(248, 246, 255, 0.72)",
      "--theme-text-muted": "rgba(248, 246, 255, 0.56)",
      "--theme-text-subtle": "rgba(248, 246, 255, 0.38)",
      "--theme-surface-base": "#0b1020",
      "--theme-surface-control": "rgba(255, 255, 255, 0.085)",
      "--theme-surface-control-hover": "rgba(255, 255, 255, 0.14)",
      "--theme-surface-panel": "var(--theme-glass-from, rgba(255, 255, 255, 0.12))",
      "--theme-grid-line": "rgba(255, 255, 255, 0.07)",
      "--theme-border-dim": "rgba(255, 255, 255, 0.07)",
      "--theme-border-default": "rgba(255, 255, 255, 0.13)",
      "--theme-border-control": "rgba(255, 255, 255, 0.16)",
    },
  },
  dark: {
    id: "dark",
    label: "Dark",
    css: {
      "--theme-bg-base": "linear-gradient(135deg, #07090f, #101522 52%, #160f12)",
      "--theme-flow-1": "rgba(68, 112, 184, 0.13)",
      "--theme-flow-2": "rgba(198, 115, 68, 0.12)",
      "--theme-flow-3": "rgba(78, 170, 146, 0.075)",
      "--theme-flow-4": "rgba(124, 94, 188, 0.09)",
      "--theme-orb-1": "rgba(68, 112, 184, 0.07)",
      "--theme-orb-2": "rgba(198, 115, 68, 0.065)",
      "--theme-orb-3": "rgba(78, 170, 146, 0.045)",
      "--theme-glass-from": "rgba(30, 36, 48, 0.72)",
      "--theme-glass-to": "rgba(12, 15, 22, 0.56)",
      "--theme-glass-border": "rgba(255, 255, 255, 0.085)",
      "--theme-glass-inset": "rgba(255, 255, 255, 0.06)",
      "--theme-accent": "#c67344",
      "--theme-accent-blue": "#4470b8",
      "--theme-accent-violet": "#7c5ebc",
      "--theme-accent-emerald": "#4eaa92",
      "--theme-text-primary": "#edeff5",
      "--theme-text-body": "rgba(237, 239, 245, 0.86)",
      "--theme-text-secondary": "rgba(237, 239, 245, 0.68)",
      "--theme-text-muted": "rgba(237, 239, 245, 0.48)",
      "--theme-text-subtle": "rgba(237, 239, 245, 0.32)",
      "--theme-surface-base": "#090b11",
      "--theme-surface-control": "rgba(255, 255, 255, 0.065)",
      "--theme-surface-control-hover": "rgba(255, 255, 255, 0.105)",
      "--theme-surface-panel": "rgba(31, 37, 49, 0.72)",
      "--theme-grid-line": "rgba(255, 255, 255, 0.035)",
      "--theme-border-dim": "rgba(255, 255, 255, 0.05)",
      "--theme-border-default": "rgba(255, 255, 255, 0.085)",
      "--theme-border-control": "rgba(255, 255, 255, 0.11)",
    },
  },
  midnight: {
    id: "midnight",
    label: "Midnight",
    css: {
      "--theme-bg-base": "linear-gradient(135deg, #000104, #02050a 52%, #05070d)",
      "--theme-flow-1": "rgba(18, 126, 118, 0.18)",
      "--theme-flow-2": "rgba(24, 94, 132, 0.14)",
      "--theme-flow-3": "rgba(64, 146, 118, 0.10)",
      "--theme-flow-4": "rgba(12, 58, 76, 0.15)",
      "--theme-orb-1": "rgba(18, 126, 118, 0.065)",
      "--theme-orb-2": "rgba(24, 94, 132, 0.052)",
      "--theme-orb-3": "rgba(64, 146, 118, 0.045)",
      "--theme-glass-from": "rgba(14, 18, 22, 0.76)",
      "--theme-glass-to": "rgba(4, 7, 10, 0.58)",
      "--theme-glass-border": "rgba(255, 255, 255, 0.055)",
      "--theme-glass-inset": "rgba(255, 255, 255, 0.03)",
      "--theme-accent": "#4d928a",
      "--theme-accent-blue": "#486f8c",
      "--theme-accent-violet": "#68727c",
      "--theme-accent-emerald": "#5e8879",
      "--theme-text-primary": "#b9c0c8",
      "--theme-text-body": "rgba(185, 192, 200, 0.84)",
      "--theme-text-secondary": "rgba(185, 192, 200, 0.62)",
      "--theme-text-muted": "rgba(185, 192, 200, 0.40)",
      "--theme-text-subtle": "rgba(185, 192, 200, 0.27)",
      "--theme-surface-base": "#000000",
      "--theme-surface-control": "rgba(255, 255, 255, 0.050)",
      "--theme-surface-control-hover": "rgba(255, 255, 255, 0.085)",
      "--theme-surface-panel": "rgba(16, 16, 22, 0.74)",
      "--theme-grid-line": "rgba(255, 255, 255, 0.022)",
      "--theme-border-dim": "rgba(255, 255, 255, 0.03)",
      "--theme-border-default": "rgba(255, 255, 255, 0.055)",
      "--theme-border-control": "rgba(255, 255, 255, 0.07)",
    },
  },
  light: {
    id: "light",
    label: "Light",
    css: {
      "--theme-bg-base": "linear-gradient(135deg, #f4f7ff, #fff7ed 48%, #ecfbf6)",
      "--theme-flow-1": "rgba(58, 103, 202, 0.14)",
      "--theme-flow-2": "rgba(211, 122, 72, 0.10)",
      "--theme-flow-3": "rgba(38, 146, 124, 0.10)",
      "--theme-flow-4": "rgba(114, 88, 190, 0.085)",
      "--theme-orb-1": "rgba(58, 103, 202, 0.085)",
      "--theme-orb-2": "rgba(211, 122, 72, 0.065)",
      "--theme-orb-3": "rgba(38, 146, 124, 0.065)",
      "--theme-glass-from": "rgba(255, 255, 255, 0.82)",
      "--theme-glass-to": "rgba(255, 255, 255, 0.58)",
      "--theme-glass-border": "rgba(38, 48, 72, 0.10)",
      "--theme-glass-inset": "rgba(255, 255, 255, 0.76)",
      "--theme-accent": "#2f579f",
      "--theme-accent-blue": "#3a67ca",
      "--theme-accent-violet": "#6650b8",
      "--theme-accent-emerald": "#247b69",
      "--theme-text-primary": "#171b26",
      "--theme-text-body": "rgba(23, 27, 38, 0.88)",
      "--theme-text-secondary": "rgba(23, 27, 38, 0.66)",
      "--theme-text-muted": "rgba(23, 27, 38, 0.48)",
      "--theme-text-subtle": "rgba(23, 27, 38, 0.32)",
      "--theme-surface-base": "#f6f7fb",
      "--theme-surface-control": "rgba(36, 46, 68, 0.060)",
      "--theme-surface-control-hover": "rgba(36, 46, 68, 0.095)",
      "--theme-surface-panel": "rgba(255, 255, 255, 0.82)",
      "--theme-grid-line": "rgba(38, 48, 72, 0.035)",
      "--theme-border-dim": "rgba(38, 48, 72, 0.045)",
      "--theme-border-default": "rgba(38, 48, 72, 0.085)",
      "--theme-border-control": "rgba(38, 48, 72, 0.12)",
    },
  },
  custom: {
    id: "custom",
    label: "Custom",
    css: {
      "--theme-bg-base": "linear-gradient(135deg, #07090f, #101522 52%, #160f12)",
      "--theme-flow-1": "rgba(65, 116, 128, 0.13)",
      "--theme-flow-2": "rgba(78, 104, 132, 0.10)",
      "--theme-flow-3": "rgba(76, 130, 112, 0.075)",
      "--theme-flow-4": "rgba(104, 94, 128, 0.08)",
      "--theme-orb-1": "rgba(65, 116, 128, 0.07)",
      "--theme-orb-2": "rgba(78, 104, 132, 0.055)",
      "--theme-orb-3": "rgba(76, 130, 112, 0.045)",
      "--theme-glass-from": "rgba(30, 36, 48, 0.72)",
      "--theme-glass-to": "rgba(12, 15, 22, 0.56)",
      "--theme-glass-border": "rgba(255, 255, 255, 0.085)",
      "--theme-glass-inset": "rgba(255, 255, 255, 0.06)",
      "--theme-accent": "#417480",
      "--theme-accent-blue": "#4e6884",
      "--theme-accent-violet": "#685e80",
      "--theme-accent-emerald": "#4c8270",
      "--theme-text-primary": "#edeff5",
      "--theme-text-body": "rgba(237, 239, 245, 0.86)",
      "--theme-text-secondary": "rgba(237, 239, 245, 0.68)",
      "--theme-text-muted": "rgba(237, 239, 245, 0.48)",
      "--theme-text-subtle": "rgba(237, 239, 245, 0.32)",
      "--theme-surface-base": "#090b11",
      "--theme-surface-control": "rgba(255, 255, 255, 0.065)",
      "--theme-surface-control-hover": "rgba(255, 255, 255, 0.105)",
      "--theme-surface-panel": "rgba(31, 37, 49, 0.72)",
      "--theme-grid-line": "rgba(255, 255, 255, 0.035)",
      "--theme-border-dim": "rgba(255, 255, 255, 0.05)",
      "--theme-border-default": "rgba(255, 255, 255, 0.085)",
      "--theme-border-control": "rgba(255, 255, 255, 0.11)",
    },
  },
};

export const DEFAULT_THEME: ThemeId = "vibrant";

export const DEFAULT_CUSTOM_THEME_COLORS: CustomThemeColors = {
  accent: THEMES.custom.css["--theme-accent"],
  accentBlue: THEMES.custom.css["--theme-accent-blue"],
  accentViolet: THEMES.custom.css["--theme-accent-violet"],
  accentEmerald: THEMES.custom.css["--theme-accent-emerald"],
};

export const CUSTOM_THEME_COLOR_KEYS: Array<{
  key: CustomThemeColorKey;
  cssVar: string;
}> = [
  { key: "accent", cssVar: "--theme-accent" },
  { key: "accentBlue", cssVar: "--theme-accent-blue" },
  { key: "accentViolet", cssVar: "--theme-accent-violet" },
  { key: "accentEmerald", cssVar: "--theme-accent-emerald" },
];

export function buildCustomThemeCss(colors: CustomThemeColors) {
  return {
    ...THEMES.custom.css,
    "--theme-accent": colors.accent,
    "--theme-accent-blue": colors.accentBlue,
    "--theme-accent-violet": colors.accentViolet,
    "--theme-accent-emerald": colors.accentEmerald,
  };
}
