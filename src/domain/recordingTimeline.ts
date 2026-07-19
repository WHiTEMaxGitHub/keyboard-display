export type RecordingTimelineEvent =
  | { frame: number; down: string }
  | { frame: number; up: string }
  | { frame: number; marker: string };

export type RecordingTimelineMarker = {
  frame: number;
  name: string;
  percent: number;
  timecode: string;
};

export function buildRecordingTimelineMarkers(_input: {
  events: RecordingTimelineEvent[];
  fps: number;
  frameCount: number;
}): RecordingTimelineMarker[] {
  const markers = _input.events
    .filter((event): event is Extract<RecordingTimelineEvent, { marker: string }> =>
      "marker" in event
    )
    .map((event) => ({
      frame: event.frame,
      name: event.marker,
    }));
  const lastFrame = Math.max(
    _input.frameCount > 0 ? _input.frameCount - 1 : 0,
    ...markers.map((marker) => marker.frame),
  );
  const durationFrame = Math.max(lastFrame, 1);

  return markers.map((marker) => ({
    ...marker,
    percent: roundPercent((marker.frame / durationFrame) * 100),
    timecode: formatFrameTimecode(marker.frame, _input.fps),
  }));
}

function roundPercent(value: number) {
  return Math.round(value * 100) / 100;
}

function formatFrameTimecode(frame: number, fps: number) {
  const safeFps = Math.max(Math.floor(fps), 1);
  const totalSeconds = Math.floor(frame / safeFps);
  const frameInSecond = frame % safeFps;
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${pad2(hours)}:${pad2(minutes)}:${pad2(seconds)}:${padFrame(frameInSecond, safeFps)} @ ${safeFps}fps`;
}

function pad2(value: number) {
  return String(value).padStart(2, "0");
}

function padFrame(frame: number, fps: number) {
  return String(frame).padStart(String(Math.max(fps - 1, 0)).length, "0");
}
