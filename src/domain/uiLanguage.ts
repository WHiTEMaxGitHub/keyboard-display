export const SUPPORTED_UI_LANGUAGES = ["en", "zh-CN"] as const;
export type SupportedUiLanguage = typeof SUPPORTED_UI_LANGUAGES[number];
export type UiLanguage = "system" | SupportedUiLanguage;

export const DEFAULT_UI_LANGUAGE: SupportedUiLanguage = "en";

export function normalizeUiLanguage(language: unknown): UiLanguage {
  return language === "system" || isSupportedUiLanguage(language) ? language : "system";
}

export function isSupportedUiLanguage(language: unknown): language is SupportedUiLanguage {
  return typeof language === "string" &&
    (SUPPORTED_UI_LANGUAGES as readonly string[]).includes(language);
}

export function resolveRuntimeUiLanguage(
  language: UiLanguage,
  systemLanguage: string | undefined,
): SupportedUiLanguage {
  if (language !== "system") {
    return language;
  }

  const normalizedSystemLanguage = systemLanguage?.toLowerCase() ?? "";
  if (normalizedSystemLanguage.startsWith("zh")) {
    return "zh-CN";
  }
  if (normalizedSystemLanguage.startsWith("en")) {
    return "en";
  }
  return DEFAULT_UI_LANGUAGE;
}
