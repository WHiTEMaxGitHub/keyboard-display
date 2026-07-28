import { describe, expect, it } from "vitest";
import {
  DEFAULT_EXPORT_FILENAME_TEMPLATE,
  formatExportFileName,
  sanitizeExportFilenameTemplate,
} from "./exportFilename";

describe("export filename templates", () => {
  it("builds a readable overlay video name from recording and profile context", () => {
    expect(
      formatExportFileName({
        template: DEFAULT_EXPORT_FILENAME_TEMPLATE,
        recordingPath: "/recordings/2026-07-28-session.kbdrec",
        profileName: "CS POV / Aim",
        fps: 120,
      }),
    ).toBe("cs-pov-aim-2026-07-28-session-overlay.webm");
  });

  it("expands supported export variables", () => {
    expect(
      formatExportFileName({
        template: "${profileName}-${recordingName}-${fps}",
        recordingPath: "/recordings/input.kbdrec",
        profileName: "Aim Warmup",
        fps: 60,
      }),
    ).toBe("Aim Warmup-input-60.webm");
  });

  it("sanitizes unsafe output names and empty templates", () => {
    expect(sanitizeExportFilenameTemplate("   ")).toBe(DEFAULT_EXPORT_FILENAME_TEMPLATE);
    expect(
      formatExportFileName({
        template: "../${profileName}\n${recordingName}",
        recordingPath: "/recordings/a/b.kbdrec",
        profileName: "CS/POV",
        fps: 60,
      }),
    ).toBe("..-CS-POV-b.webm");
  });
});
