import { ref } from "vue";
import {
  buildCustomThemeCss,
  CUSTOM_THEME_COLOR_KEYS,
  DEFAULT_CUSTOM_THEME_COLORS,
  DEFAULT_CUSTOM_THEME_PANEL_OPACITY,
  DEFAULT_CUSTOM_THEME_TEMPLATE,
  THEMES,
  DEFAULT_THEME,
  type CustomThemeColorKey,
  type CustomThemeColors,
  type CustomThemeTemplateId,
  type ThemeId,
} from "../domain/theme";
import { normalizeHexColor } from "../domain/colorPicker";

const STORAGE_KEY = "keyboard-display:theme";
const CUSTOM_THEME_STORAGE_KEY = "keyboard-display:custom-theme-colors";
const CUSTOM_THEME_TEMPLATE_STORAGE_KEY = "keyboard-display:custom-theme-template";
const CUSTOM_THEME_PANEL_OPACITY_STORAGE_KEY = "keyboard-display:custom-theme-panel-opacity";
const themeId = ref<ThemeId>(DEFAULT_THEME);
const customThemeColors = ref<CustomThemeColors>({ ...DEFAULT_CUSTOM_THEME_COLORS });
const customThemeTemplate = ref<CustomThemeTemplateId>(DEFAULT_CUSTOM_THEME_TEMPLATE);
const customThemePanelOpacity = ref(DEFAULT_CUSTOM_THEME_PANEL_OPACITY);

function themeCss(id: ThemeId) {
  return id === "custom"
    ? buildCustomThemeCss(
      customThemeColors.value,
      customThemeTemplate.value,
      customThemePanelOpacity.value,
    )
    : THEMES[id].css;
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

function loadCustomThemeTemplate() {
  try {
    const raw = localStorage.getItem(CUSTOM_THEME_TEMPLATE_STORAGE_KEY);
    customThemeTemplate.value = isCustomThemeTemplate(raw) ? raw : DEFAULT_CUSTOM_THEME_TEMPLATE;
  } catch {
    customThemeTemplate.value = DEFAULT_CUSTOM_THEME_TEMPLATE;
  }
}

function loadCustomThemePanelOpacity() {
  try {
    const raw = Number(localStorage.getItem(CUSTOM_THEME_PANEL_OPACITY_STORAGE_KEY));
    customThemePanelOpacity.value = Number.isFinite(raw)
      ? clampPanelOpacity(raw)
      : DEFAULT_CUSTOM_THEME_PANEL_OPACITY;
  } catch {
    customThemePanelOpacity.value = DEFAULT_CUSTOM_THEME_PANEL_OPACITY;
  }
}

function saveCustomThemeColors() {
  try {
    localStorage.setItem(CUSTOM_THEME_STORAGE_KEY, JSON.stringify(customThemeColors.value));
  } catch {
    // non-critical
  }
}

function saveCustomThemePanelOpacity() {
  try {
    localStorage.setItem(CUSTOM_THEME_PANEL_OPACITY_STORAGE_KEY, String(customThemePanelOpacity.value));
  } catch {
    // non-critical
  }
}

function saveCustomThemeTemplate() {
  try {
    localStorage.setItem(CUSTOM_THEME_TEMPLATE_STORAGE_KEY, customThemeTemplate.value);
  } catch {
    // non-critical
  }
}

function isCustomThemeTemplate(value: string | null): value is CustomThemeTemplateId {
  return value === "vibrant" || value === "calm" || value === "still";
}

function clampPanelOpacity(value: number) {
  return Math.min(1, Math.max(0, Math.round(value * 100) / 100));
}

function sanitizeCustomThemeColors(colors: Partial<CustomThemeColors>) {
  const next = { ...DEFAULT_CUSTOM_THEME_COLORS };
  for (const { key } of CUSTOM_THEME_COLOR_KEYS) {
    next[key] = normalizeHexColor(colors[key] ?? "", DEFAULT_CUSTOM_THEME_COLORS[key]);
  }
  return next;
}

export function useTheme() {
  function loadTheme() {
    loadCustomThemeColors();
    loadCustomThemeTemplate();
    loadCustomThemePanelOpacity();
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
      [key]: normalizeHexColor(color, customThemeColors.value[key]),
    };
    saveCustomThemeColors();
    setTheme("custom");
  }

  function setCustomThemeTemplate(templateId: CustomThemeTemplateId) {
    customThemeTemplate.value = templateId;
    saveCustomThemeTemplate();
    setTheme("custom");
  }

  function setCustomThemePanelOpacity(opacity: number) {
    customThemePanelOpacity.value = clampPanelOpacity(opacity);
    saveCustomThemePanelOpacity();
    setTheme("custom");
  }

  function previewCustomThemeColor(key: CustomThemeColorKey, color: string) {
    customThemeColors.value = {
      ...customThemeColors.value,
      [key]: normalizeHexColor(color, customThemeColors.value[key]),
    };
    applyTheme("custom");
  }

  function resetCustomThemeColors() {
    customThemeColors.value = { ...DEFAULT_CUSTOM_THEME_COLORS };
    customThemeTemplate.value = DEFAULT_CUSTOM_THEME_TEMPLATE;
    customThemePanelOpacity.value = DEFAULT_CUSTOM_THEME_PANEL_OPACITY;
    saveCustomThemeColors();
    saveCustomThemeTemplate();
    saveCustomThemePanelOpacity();
    setTheme("custom");
  }

  return {
    themeId,
    customThemeColors,
    customThemeTemplate,
    customThemePanelOpacity,
    loadTheme,
    setTheme,
    previewCustomThemeColor,
    setCustomThemeColor,
    setCustomThemeTemplate,
    setCustomThemePanelOpacity,
    resetCustomThemeColors,
  };
}
