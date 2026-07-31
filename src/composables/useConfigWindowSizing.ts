import { LogicalSize, currentMonitor, getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";

const MIN_WIDTH = 1040;
const MIN_HEIGHT = 700;
const MAX_WIDTH = 1440;
const MAX_HEIGHT = 960;

function clamp(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
}

export async function applyPreferredConfigWindowSize() {
  const monitor = (await currentMonitor()) ?? (await primaryMonitor());
  if (!monitor) {
    return;
  }

  const scaleFactor = monitor.scaleFactor;
  const workArea = monitor.workArea.size.toLogical(scaleFactor);
  const width = Math.round(clamp(workArea.width * 0.72, MIN_WIDTH, MAX_WIDTH));
  const height = Math.round(clamp(workArea.height * 0.66, MIN_HEIGHT, MAX_HEIGHT));
  const currentWindow = getCurrentWindow();

  await currentWindow.setMinSize(new LogicalSize(MIN_WIDTH, MIN_HEIGHT));
  await currentWindow.setSize(new LogicalSize(width, height));
}
