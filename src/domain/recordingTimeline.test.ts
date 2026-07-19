import { describe, expect, it } from "vitest";
import { buildRecordingTimelineMarkers } from "./recordingTimeline";

describe("buildRecordingTimelineMarkers", () => {
  it("places markers by frame across the recording duration", () => {
    const markers = buildRecordingTimelineMarkers({
      events: [
        { frame: 0, marker: "start" },
        { frame: 50, down: "key-a" },
        { frame: 50, marker: "sync" },
        { frame: 100, marker: "end" },
      ],
      fps: 60,
      frameCount: 101,
    });

    expect(markers).toEqual([
      {
        frame: 0,
        name: "start",
        percent: 0,
        timecode: "00:00:00:00 @ 60fps",
      },
      {
        frame: 50,
        name: "sync",
        percent: 50,
        timecode: "00:00:00:50 @ 60fps",
      },
      {
        frame: 100,
        name: "end",
        percent: 100,
        timecode: "00:00:01:40 @ 60fps",
      },
    ]);
  });

  it("uses the furthest marker as the timeline duration when frames are absent", () => {
    const markers = buildRecordingTimelineMarkers({
      events: [
        { frame: 12, marker: "sync" },
        { frame: 24, marker: "finish" },
      ],
      fps: 24,
      frameCount: 0,
    });

    expect(markers.map((marker) => marker.percent)).toEqual([50, 100]);
  });

  it("keeps duplicate marker labels distinct", () => {
    const markers = buildRecordingTimelineMarkers({
      events: [
        { frame: 10, marker: "sync" },
        { frame: 20, marker: "sync" },
      ],
      fps: 30,
      frameCount: 31,
    });

    expect(markers).toHaveLength(2);
    expect(markers.map((marker) => marker.frame)).toEqual([10, 20]);
  });
});
