import { describe, expect, it } from "vitest";
import {
  formatRecordingBoundaryTime,
  formatRecordingTime,
  parseRecordingTime,
  resolveExportFrameRange,
} from "./recordingExportRange";

describe("recording export range", () => {
  it("formats and parses user-facing timecodes", () => {
    expect(formatRecordingTime(90, 60)).toBe("00:00:01.500");
    expect(parseRecordingTime("01:02:03.250")).toBe(3723.25);
    expect(parseRecordingTime("01:99:00")).toBeNull();
  });

  it("converts time to a half-open frame range", () => {
    expect(resolveExportFrameRange("time", "00:00:01.000", "00:00:02.001", {
      fps: 60,
      frameCount: 300,
    })).toEqual({ startFrame: 60, endFrameExclusive: 121 });
  });

  it.each([24, 60, 240])(
    "round-trips non-millisecond frame boundaries at %ifps",
    (fps) => {
      const frameCount = fps * 7 + 1;
      const startFrame = 1;
      expect(resolveExportFrameRange(
        "time",
        formatRecordingBoundaryTime(startFrame, fps, "start"),
        formatRecordingBoundaryTime(frameCount, fps, "end"),
        { fps, frameCount },
      )).toEqual({ startFrame, endFrameExclusive: frameCount });
    },
  );

  it("rejects empty and out-of-bounds frame ranges", () => {
    const info = { fps: 60, frameCount: 120 };
    expect(() => resolveExportFrameRange("frames", "20", "20", info)).toThrow();
    expect(() => resolveExportFrameRange("frames", "0", "121", info)).toThrow();
  });
});
