import { ref } from "vue";
import {
  buildCustomThemeCss,
  CUSTOM_THEME_COLOR_KEYS,
  DEFAULT_CUSTOM_THEME_COLORS,
  THEMES,
  DEFAULT_THEME,
  type CustomThemeColorKey,
  type CustomThemeColors,
  type ThemeId,
} from "../domain/theme";
import { normalizeHexColor } from "../domain/colorPicker";

const STORAGE_KEY = "keyboard-display:theme";
const CUSTOM_THEME_STORAGE_KEY = "keyboard-display:custom-theme-colors";
const themeId = ref<ThemeId>(DEFAULT_THEME);
const customThemeColors = ref<CustomThemeColors>({ ...DEFAULT_CUSTOM_THEME_COLORS });

function themeCss(id: ThemeId) {
  return id === "custom" ? buildCustomThemeCss(customThemeColors.value) : THEMES[id].css;
}

function applyTheme(id: ThemeId) {
  const theme = THEMES[id];
  const root = document.documentElement;
  for (const [key, value] of Object.entries(themeCss(theme.id))) {
    root.style.setProperty(key, value);
  }
  themeId.value = id;
}

function loadCustomThemeColors() {
  try {
    const raw = localStorage.getItem(CUSTOM_THEME_STORAGE_KEY);
    if (!raw) {
      customThemeColors.value = { ...DEFAULT_CUSTOM_THEME_COLORS };
      return;
    }

    const parsed = JSON.parse(raw) as Partial<CustomThemeColors>;
    customThemeColors.value = sanitizeCustomThemeColors(parsed);
  } catch {
    customThemeColors.value = { ...DEFAULT_CUSTOM_THEME_COLORS };
  }
}

function saveCustomThemeColors() {
  try {
    localStorage.setItem(CUSTOM_THEME_STORAGE_KEY, JSON.stringify(customThemeColors.value));
  } catch {
    // non-critical
  }
}

function sanitizeCustomThemeColors(colors: Partial<CustomThemeColors>) {
  const next = { ...DEFAULT_CUSTOM_THEME_COLORS };
  for (const { key } of CUSTOM_THEME_COLOR_KEYS) {
    next[key] = normalizeHexColor(colors[key] ?? "", DEFAULT_CUSTOM_THEME_COLORS[key]).slice(0, 7);
  }
  return next;
}

export function useTheme() {
  function loadTheme() {
    loadCustomThemeColors();
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored && THEMES[stored as ThemeId]) {
        applyTheme(stored as ThemeId);
        return;
      }
    } catch {
      // fall through
    }
    applyTheme(DEFAULT_THEME);
  }

  function setTheme(id: ThemeId) {
    applyTheme(id);
    try {
      localStorage.setItem(STORAGE_KEY, id);
    } catch {
      // non-critical
    }
  }

  function setCustomThemeColor(key: CustomThemeColorKey, color: string) {
    customThemeColors.value = {
      ...customThemeColors.value,
      [key]: normalizeHexColor(color, customThemeColors.value[key]).slice(0, 7),
    };
    saveCustomThemeColors();
    setTheme("custom");
  }

  function resetCustomThemeColors() {
    customThemeColors.value = { ...DEFAULT_CUSTOM_THEME_COLORS };
    saveCustomThemeColors();
    setTheme("custom");
  }

  return {
    themeId,
    customThemeColors,
    loadTheme,
    setTheme,
    setCustomThemeColor,
    resetCustomThemeColors,
  };
}
