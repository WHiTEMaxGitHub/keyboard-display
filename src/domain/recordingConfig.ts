import type { RecordingConfig } from "./defaultConfig";

export function clampRecordingFps(fps: number, maxFps: number): number {
  return Math.min(Math.max(Math.round(fps), 1), maxFps);
}

export function effectiveRecordingFps(recording: RecordingConfig): number {
  return recording.customFpsEnabled
    ? clampRecordingFps(recording.customFps, recording.maxFps)
    : clampRecordingFps(recording.defaultFps, recording.maxFps);
}

export function normalizeRecordingConfig(recording: RecordingConfig): RecordingConfig {
  const maxFps = Math.max(1, Math.round(recording.maxFps ?? 1000));

  return {
    ...recording,
    defaultFps: clampRecordingFps(recording.defaultFps, maxFps),
    fpsOptions: recording.fpsOptions.map((fps) => clampRecordingFps(fps, maxFps)),
    customFpsEnabled: recording.customFpsEnabled ?? false,
    customFps: clampRecordingFps(recording.customFps ?? 600, maxFps),
    maxFps,
    syncFeedbackEnabled: recording.syncFeedbackEnabled ?? true,
    syncFeedbackDurationMs: Math.max(100, Math.round(recording.syncFeedbackDurationMs ?? 420)),
  };
}
