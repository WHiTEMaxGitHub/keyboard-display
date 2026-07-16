import { describe, expect, it } from "vitest";
import {
  clampRecordingFps,
  effectiveRecordingFps,
  normalizeRecordingConfig,
} from "./recordingConfig";

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
      formatExtension: ".kbdrec",
      primaryArtifact: "input-binary",
    });

    expect(effectiveRecordingFps(recording)).toBe(1000);
    expect(effectiveRecordingFps({ ...recording, customFpsEnabled: false })).toBe(60);
  });
});
