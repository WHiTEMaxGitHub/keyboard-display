export function estimateRawRecordingBytesPerSecond(keyCount: number, fps: number): number {
  return Math.ceil(keyCount / 8) * fps;
}

export function formatBytesPerSecond(bytesPerSecond: number): string {
  if (bytesPerSecond < 1024) {
    return `${bytesPerSecond} B/s`;
  }

  return `${(bytesPerSecond / 1024).toFixed(1)} KiB/s`;
}
