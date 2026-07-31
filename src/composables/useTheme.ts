import { ref } from "vue";
import { THEMES, DEFAULT_THEME, type ThemeId } from "../domain/theme";

const STORAGE_KEY = "keyboard-display:theme";
const themeId = ref<ThemeId>(DEFAULT_THEME);

function applyTheme(id: ThemeId) {
  const theme = THEMES[id];
  const root = document.documentElement;
  for (const [key, value] of Object.entries(theme.css)) {
    root.style.setProperty(key, value);
  }
  themeId.value = id;
}

export function useTheme() {
  function loadTheme() {
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

  return { themeId, loadTheme, setTheme };
}
