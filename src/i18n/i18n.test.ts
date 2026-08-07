import { describe, expect, it } from "vitest";
import en from "./locales/en.json";
import zhCN from "./locales/zh-CN.json";
import {
  DEFAULT_LOCALE,
  LOCALE_OPTIONS,
  SUPPORTED_LOCALES,
  createAppI18n,
  flattenLocaleKeys,
  normalizeUiLanguage,
  resolveRuntimeLocale,
} from "./index";

describe("i18n", () => {
  it("keeps English and Chinese locale keys aligned", () => {
    expect(flattenLocaleKeys(zhCN)).toEqual(flattenLocaleKeys(en));
  });

  it("normalizes unsupported app-config language values to system", () => {
    expect(normalizeUiLanguage("zh-CN")).toBe("zh-CN");
    expect(normalizeUiLanguage("en")).toBe("en");
    expect(normalizeUiLanguage("system")).toBe("system");
    expect(normalizeUiLanguage("fr-FR")).toBe("system");
    expect(normalizeUiLanguage(undefined)).toBe("system");
  });

  it("resolves system language to a supported runtime locale", () => {
    expect(resolveRuntimeLocale("system", "zh-Hans-CN")).toBe("zh-CN");
    expect(resolveRuntimeLocale("system", "en-US")).toBe("en");
    expect(resolveRuntimeLocale("system", "fr-FR")).toBe(DEFAULT_LOCALE);
    expect(resolveRuntimeLocale("zh-CN", "en-US")).toBe("zh-CN");
  });

  it("creates vue-i18n with fallback translation behavior", () => {
    const i18n = createAppI18n("zh-CN", "en-US");

    expect(SUPPORTED_LOCALES).toEqual(["en", "zh-CN"]);
    expect(LOCALE_OPTIONS.map((option) => option.value)).toEqual(["system", "en", "zh-CN"]);
    expect(i18n.global.t("app.name")).toBe("Keyboard Display");
    expect(i18n.global.t("settings.languageOption.system")).toBe("跟随系统");
  });

  it("contains the next configuration-page locale sections", () => {
    const keys = flattenLocaleKeys(en);

    expect(keys).toContain("layout.title");
    expect(keys).toContain("layout.editor.addRow");
    expect(keys).toContain("appearance.title");
    expect(keys).toContain("appearance.colors.idleKey");
    expect(keys).toContain("window.adjust.savePosition");
    expect(keys).toContain("window.position.bottomRight");
  });

  it("contains recording control locale sections", () => {
    const keys = flattenLocaleKeys(en);

    expect(keys).toContain("recording.title");
    expect(keys).toContain("recording.controls.start");
    expect(keys).toContain("recording.filename.variables");
    expect(keys).toContain("recording.storage.description");
    expect(keys).toContain("recording.hotkeys.mode");
    expect(keys).toContain("recording.hotkeys.capture");
  });

  it("contains export locale sections", () => {
    const keys = flattenLocaleKeys(en);

    expect(keys).toContain("export.title");
    expect(keys).toContain("export.overlayVideo.exportButton");
    expect(keys).toContain("export.exporter.appManaged");
    expect(keys).toContain("export.status.exporting");
    expect(keys).toContain("export.dialog.chooseRecording");
    expect(keys).toContain("export.notification.installSuccess");
  });
});
