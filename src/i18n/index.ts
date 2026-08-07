import { createI18n } from "vue-i18n";
import {
  DEFAULT_UI_LANGUAGE,
  SUPPORTED_UI_LANGUAGES,
  normalizeUiLanguage,
  resolveRuntimeUiLanguage,
  type SupportedUiLanguage,
  type UiLanguage,
} from "../domain/uiLanguage";
import en from "./locales/en.json";
import zhCN from "./locales/zh-CN.json";

export {
  DEFAULT_UI_LANGUAGE as DEFAULT_LOCALE,
  SUPPORTED_UI_LANGUAGES as SUPPORTED_LOCALES,
  normalizeUiLanguage,
};

export type { SupportedUiLanguage, UiLanguage };

export const LOCALE_OPTIONS: Array<{ value: UiLanguage; labelKey: string }> = [
  { value: "system", labelKey: "settings.languageOption.system" },
  { value: "en", labelKey: "settings.languageOption.en" },
  { value: "zh-CN", labelKey: "settings.languageOption.zh-CN" },
];

const messages = {
  en,
  "zh-CN": zhCN,
};

export function resolveRuntimeLocale(
  language: UiLanguage,
  systemLanguage: string | undefined,
): SupportedUiLanguage {
  return resolveRuntimeUiLanguage(language, systemLanguage);
}

export function createAppI18n(
  language: UiLanguage,
  systemLanguage = globalThis.navigator?.language,
) {
  return createI18n({
    legacy: false,
    locale: resolveRuntimeLocale(language, systemLanguage),
    fallbackLocale: DEFAULT_UI_LANGUAGE,
    messages,
  });
}

export const i18n = createAppI18n("system");

export function setI18nLanguage(language: UiLanguage, systemLanguage = globalThis.navigator?.language) {
  i18n.global.locale.value = resolveRuntimeLocale(language, systemLanguage);
}

export function flattenLocaleKeys(value: unknown, prefix = ""): string[] {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return prefix ? [prefix] : [];
  }

  return Object.entries(value)
    .flatMap(([key, child]) => flattenLocaleKeys(child, prefix ? `${prefix}.${key}` : key))
    .sort();
}
