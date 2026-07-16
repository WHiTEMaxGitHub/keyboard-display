import { describe, expect, it } from "vitest";
import { estimateRawRecordingBytesPerSecond } from "./recordingEstimate";

describe("estimateRawRecordingBytesPerSecond", () => {
  it("estimates raw frame-state bitset write size", () => {
    expect(estimateRawRecordingBytesPerSecond(108, 600)).toBe(8400);
    expect(estimateRawRecordingBytesPerSecond(100, 600)).toBe(7800);
  });
});
