import { describe, expect, it } from "vitest";
import {
  DEFAULT_RECORDING_FILENAME_TEMPLATE,
  formatRecordingFileName,
  sanitizeRecordingFilenameTemplate,
} from "./recordingFilename";

describe("recording filename templates", () => {
  it("keeps the existing machine-friendly default filename", () => {
    expect(
      formatRecordingFileName({
        template: DEFAULT_RECORDING_FILENAME_TEMPLATE,
        start: 1000,
        end: 2000,
        profileName: "CS POV",
        fps: 60,
      }),
    ).toBe("1000-2000.kbdrec");
  });

  it("expands supported template variables", () => {
    expect(
      formatRecordingFileName({
        template: "${profileName}-${fps}-${start}",
        start: 1000,
        end: 2000,
        profileName: "Aim Warmup",
        fps: 120,
      }),
    ).toBe("Aim Warmup-120-1000.kbdrec");
  });

  it("expands readable time and slug variables", () => {
    const start = new Date(2026, 6, 19, 16, 42, 8).getTime();
    const end = start + 40_000;

    expect(
      formatRecordingFileName({
        template: "${profileSlug}-${startDate}-${startTime}-${endTime}-${duration}-${fps}fps",
        start,
        end,
        profileName: "CS POV / Aim",
        fps: 120,
      }),
    ).toBe("cs-pov-aim-2026-07-19-16-42-08-16-42-48-00-00-40-120fps.kbdrec");
  });

  it("removes path separators and control characters", () => {
    expect(
      formatRecordingFileName({
        template: "../${profileName}\n${end}",
        start: 1000,
        end: 2000,
        profileName: "CS/POV",
        fps: 60,
      }),
    ).toBe("..-CS-POV-2000.kbdrec");
  });

  it("falls back to the default template when the template is empty", () => {
    expect(sanitizeRecordingFilenameTemplate("   ")).toBe(
      DEFAULT_RECORDING_FILENAME_TEMPLATE,
    );
  });
});
