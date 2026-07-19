import { describe, expect, it } from "vitest";
import {
  clampRecordingFps,
  effectiveRecordingFps,
  normalizeRecordingConfig,
} from "./recordingConfig";
import type { RecordingConfig } from "./defaultConfig";

describe("recording config", () => {
  it("clamps custom FPS to the configured maximum", () => {
    expect(clampRecordingFps(1200, 1000)).toBe(1000);
    expect(clampRecordingFps(0, 1000)).toBe(1);
  });

  it("uses custom FPS only when enabled", () => {
    const recording = normalizeRecordingConfig({
      defaultFps: 60,
      fpsOptions: [30, 60, 120],
      customFpsEnabled: true,
      customFps: 1200,
      maxFps: 1000,
      syncFeedbackEnabled: true,
      syncFeedbackDurationMs: 420,
      filenameTemplate: "${start}-${end}",
      formatExtension: ".kbdrec",
      primaryArtifact: "input-binary",
    });

    expect(effectiveRecordingFps(recording)).toBe(1000);
    expect(effectiveRecordingFps({ ...recording, customFpsEnabled: false })).toBe(60);
  });

  it("defaults sync feedback options for older configs", () => {
    const recording = normalizeRecordingConfig({
      defaultFps: 60,
      fpsOptions: [30, 60, 120],
      customFpsEnabled: false,
      customFps: 600,
      maxFps: 1000,
      formatExtension: ".kbdrec",
      primaryArtifact: "input-binary",
    } as RecordingConfig);

    expect(recording.syncFeedbackEnabled).toBe(true);
    expect(recording.syncFeedbackDurationMs).toBe(420);
    expect(recording.filenameTemplate).toBe("${start}-${end}");
  });
});
