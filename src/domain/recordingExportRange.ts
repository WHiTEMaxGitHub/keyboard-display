export type RecordingExportRangeMode = "full" | "time" | "frames";

export type RecordingExportInfo = {
  fps: number;
  frameCount: number;
};

export type ExportFrameRange = {
  startFrame: number;
  endFrameExclusive: number;
};

export function formatRecordingTime(frame: number, fps: number) {
  const totalMilliseconds = fps > 0 ? Math.floor((frame * 1000) / fps) : 0;
  const milliseconds = totalMilliseconds % 1000;
  const totalSeconds = Math.floor(totalMilliseconds / 1000);
  const seconds = totalSeconds % 60;
  const totalMinutes = Math.floor(totalSeconds / 60);
  const minutes = totalMinutes % 60;
  const hours = Math.floor(totalMinutes / 60);
  return [hours, minutes, seconds].map((value) => String(value).padStart(2, "0")).join(":") +
    `.${String(milliseconds).padStart(3, "0")}`;
}

export function formatRecordingBoundaryTime(
  frame: number,
  fps: number,
  boundary: "start" | "end",
) {
  if (!Number.isSafeInteger(frame) || frame < 0 || !Number.isSafeInteger(fps) || fps <= 0) {
    return "00:00:00.000000000";
  }
  const numerator = BigInt(frame) * 1_000_000_000n;
  const divisor = BigInt(fps);
  // Inclusive starts round upward so floor(time × fps) remains `frame`.
  // Exclusive ends round downward so ceil(time × fps) remains `frame`.
  const nanoseconds = boundary === "start"
    ? (numerator + divisor - 1n) / divisor
    : numerator / divisor;
  return formatNanoseconds(nanoseconds);
}

export function parseRecordingTime(value: string) {
  const nanoseconds = parseRecordingTimeNanoseconds(value);
  return nanoseconds === null ? null : Number(nanoseconds) / 1_000_000_000;
}

export function resolveExportFrameRange(
  mode: RecordingExportRangeMode,
  startValue: string,
  endValue: string,
  info: RecordingExportInfo,
): ExportFrameRange | null {
  if (mode === "full") {
    return null;
  }
  let startFrame: number;
  let endFrameExclusive: number;
  if (mode === "time") {
    const startNanoseconds = parseRecordingTimeNanoseconds(startValue);
    const endNanoseconds = parseRecordingTimeNanoseconds(endValue);
    if (startNanoseconds === null || endNanoseconds === null) {
      throw new Error("invalid");
    }
    const fps = BigInt(info.fps);
    const nanosecondsPerSecond = 1_000_000_000n;
    startFrame = Number((startNanoseconds * fps) / nanosecondsPerSecond);
    endFrameExclusive = Number(
      (endNanoseconds * fps + nanosecondsPerSecond - 1n) / nanosecondsPerSecond,
    );
  } else {
    startFrame = Number(startValue);
    endFrameExclusive = Number(endValue);
  }
  if (
    !Number.isSafeInteger(startFrame) ||
    !Number.isSafeInteger(endFrameExclusive) ||
    startFrame < 0 ||
    startFrame >= endFrameExclusive ||
    endFrameExclusive > info.frameCount
  ) {
    throw new Error("out-of-range");
  }
  return { startFrame, endFrameExclusive };
}

function parseRecordingTimeNanoseconds(value: string) {
  const parts = value.trim().split(":");
  if (parts.length < 1 || parts.length > 3 || parts.some((part) => part === "")) {
    return null;
  }
  const secondsMatch = /^(\d{1,2})(?:\.(\d{1,9}))?$/.exec(parts[parts.length - 1]);
  if (!secondsMatch) {
    return null;
  }
  const minuteText = parts.length >= 2 ? parts[parts.length - 2] : "0";
  const hourText = parts.length === 3 ? parts[0] : "0";
  if (!/^\d{1,2}$/.test(minuteText) || !/^\d+$/.test(hourText)) {
    return null;
  }
  const seconds = Number(secondsMatch[1]);
  const minutes = Number(minuteText);
  if (seconds >= 60 || minutes >= 60) {
    return null;
  }
  const fraction = (secondsMatch[2] ?? "").padEnd(9, "0");
  return (
    (BigInt(hourText) * 3600n + BigInt(minutes) * 60n + BigInt(seconds)) *
      1_000_000_000n +
    BigInt(fraction || "0")
  );
}

function formatNanoseconds(totalNanoseconds: bigint) {
  const nanoseconds = totalNanoseconds % 1_000_000_000n;
  const totalSeconds = totalNanoseconds / 1_000_000_000n;
  const seconds = totalSeconds % 60n;
  const totalMinutes = totalSeconds / 60n;
  const minutes = totalMinutes % 60n;
  const hours = totalMinutes / 60n;
  return [hours, minutes, seconds]
    .map((value) => value.toString().padStart(2, "0"))
    .join(":") + `.${nanoseconds.toString().padStart(9, "0")}`;
}
