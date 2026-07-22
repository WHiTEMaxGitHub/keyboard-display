import { emitTo } from "@tauri-apps/api/event";
import {
  LogicalPosition,
  LogicalSize,
  Window,
  currentMonitor,
  primaryMonitor,
} from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ref, type Ref } from "vue";
import {
  OVERLAY_ADJUST_MODE_EVENT,
  OVERLAY_CONFIG_EVENT,
  OVERLAY_VISIBLE_EVENT,
} from "../domain/inputEvents";
import { estimateOverlaySize } from "../domain/overlaySize";
import type { AppConfig, OverlayCustomPosition, OverlayStyle } from "../domain/defaultConfig";
import {
  monitorWorkAreaToRect,
  resolveCustomOverlayPosition,
} from "../domain/overlayPosition";

export type OverlayPosition = "top-left" | "top-right" | "bottom-left" | "bottom-right" | "custom";

export type OverlayRuntimeConfig = {
  layout: AppConfig["layout"];
  rows: AppConfig["rows"];
  keys: AppConfig["keys"];
  style: OverlayStyle;
};

type OverlayAdjustModePayload = {
  enabled: boolean;
};

type UseOverlayWindowOptions = {
  config: AppConfig;
  isOverlayVisible: Ref<boolean>;
  overlayPosition: Ref<OverlayPosition>;
  customOverlayPosition: Ref<OverlayCustomPosition | null>;
  markProfileChanged: () => void;
  scheduleAppConfigSave: () => void;
};

export function normalizeOverlayPosition(position: string | null | undefined): OverlayPosition {
  return position === "top-left" ||
    position === "top-right" ||
    position === "bottom-left" ||
    position === "bottom-right" ||
    position === "custom"
    ? position
    : "bottom-right";
}

/// 管理独立 POV overlay 窗口的生命周期和位置同步。
export function useOverlayWindow(options: UseOverlayWindowOptions) {
  const overlayAdjusting = ref(false);

  async function updateOverlayStyle(style: OverlayStyle) {
    const overlayWindow = await Window.getByLabel("pov");
    await resizeOverlayWindow(overlayWindow);
    await overlayWindow?.setAlwaysOnTop(style.alwaysOnTop);
    await emitTo<OverlayStyle>("pov", "overlay-style", style);
    if (options.isOverlayVisible.value && options.overlayPosition.value !== "custom") {
      await moveOverlay(options.overlayPosition.value, false, false);
    }
  }

  async function updateOverlayRows() {
    const overlayWindow = await Window.getByLabel("pov");
    await resizeOverlayWindow(overlayWindow);
    await emitRuntimeConfig();
    if (options.isOverlayVisible.value && options.overlayPosition.value !== "custom") {
      await moveOverlay(options.overlayPosition.value, false, false);
    }
  }

  async function resizeOverlayWindow(overlayWindow?: Window | null) {
    const targetWindow = overlayWindow ?? (await Window.getByLabel("pov"));

    if (!targetWindow) {
      return;
    }

    const size = estimateOverlaySize(
      options.config.layout,
      options.config.rows,
      options.config.style,
    );
    await targetWindow.setSize(new LogicalSize(size.width, size.height));
  }

  async function ensureOverlayWindow(adjust = false): Promise<Window | null> {
    const existingWindow = await WebviewWindow.getByLabel("pov");
    if (existingWindow) {
      return existingWindow;
    }

    const size = estimateOverlaySize(
      options.config.layout,
      options.config.rows,
      options.config.style,
    );
    const createdWindow = new WebviewWindow("pov", {
      url: adjust ? "/?surface=pov&adjust=1" : "/?surface=pov",
      title: "POV Overlay",
      width: size.width,
      height: size.height,
      decorations: false,
      transparent: true,
      backgroundColor: [0, 0, 0, 0],
      shadow: false,
      alwaysOnTop: options.config.style.alwaysOnTop,
      visible: false,
      resizable: false,
      skipTaskbar: true,
      visibleOnAllWorkspaces: true,
    });

    await new Promise<void>((resolve, reject) => {
      createdWindow.once("tauri://created", () => resolve());
      createdWindow.once("tauri://error", (event) => reject(event.payload));
    });

    return createdWindow;
  }

  async function syncOverlayWindow(overlayWindow?: Window | null) {
    const targetWindow = overlayWindow ?? (await ensureOverlayWindow());
    if (!targetWindow) {
      return;
    }

    await resizeOverlayWindow(targetWindow);
    await targetWindow.setAlwaysOnTop(options.config.style.alwaysOnTop);
    await emitRuntimeConfig();
  }

  async function emitRuntimeConfig() {
    await emitTo<OverlayRuntimeConfig>("pov", OVERLAY_CONFIG_EVENT, {
      layout: options.config.layout,
      rows: options.config.rows,
      keys: options.config.keys,
      style: options.config.style,
    });
  }

  async function destroyOverlayWindow() {
    const overlayWindow = await Window.getByLabel("pov");
    await overlayWindow?.destroy();
  }

  async function setOverlayVisible(visible: boolean, markChanged = true) {
    const overlayWindow = visible ? await ensureOverlayWindow() : await Window.getByLabel("pov");

    if (!overlayWindow) {
      options.isOverlayVisible.value = false;
      return;
    }

    if (visible) {
      await syncOverlayWindow(overlayWindow);
      await overlayWindow.show();
    } else {
      await overlayWindow.hide();
    }

    options.isOverlayVisible.value = visible;
    if (markChanged) {
      options.markProfileChanged();
    }
    await emitTo<boolean>("pov", OVERLAY_VISIBLE_EVENT, visible);
  }

  async function moveOverlay(position: OverlayPosition, markChanged = true, show = true) {
    options.overlayPosition.value = position;
    if (markChanged) {
      options.markProfileChanged();
    }
    const overlayWindow = await ensureOverlayWindow();
    const monitor = (await currentMonitor()) ?? (await primaryMonitor());

    if (!overlayWindow || !monitor) {
      return;
    }

    await resizeOverlayWindow(overlayWindow);
    const overlaySize = estimateOverlaySize(
      options.config.layout,
      options.config.rows,
      options.config.style,
    );
    const workArea = monitorWorkAreaToRect(
      monitor.workArea.position.toLogical(monitor.scaleFactor),
      monitor.workArea.size.toLogical(monitor.scaleFactor),
    );

    if (position === "custom" && options.customOverlayPosition.value) {
      const mappedPosition = resolveCustomOverlayPosition(
        options.customOverlayPosition.value,
        workArea,
        overlaySize,
      );
      if (show) {
        await overlayWindow.show();
      }
      await overlayWindow.setPosition(new LogicalPosition(mappedPosition.x, mappedPosition.y));
      if (show) {
        options.isOverlayVisible.value = true;
        await emitTo<boolean>("pov", OVERLAY_VISIBLE_EVENT, true);
      }
      return;
    }
    const presetPosition = position === "custom" ? "bottom-right" : position;

    const horizontalMargin = 6;
    const topMargin = 3;
    const bottomMargin = 5;
    const overlayBleed = 12;
    const xMin = workArea.x + horizontalMargin - overlayBleed;
    const yMin = workArea.y + topMargin - overlayBleed;
    const xMax =
      workArea.x + workArea.width - overlaySize.width - horizontalMargin + overlayBleed;
    const yMax =
      workArea.y + workArea.height - overlaySize.height - bottomMargin + overlayBleed;

    const positions: Record<Exclude<OverlayPosition, "custom">, LogicalPosition> = {
      "top-left": new LogicalPosition(xMin, yMin),
      "top-right": new LogicalPosition(xMax, yMin),
      "bottom-left": new LogicalPosition(xMin, yMax),
      "bottom-right": new LogicalPosition(xMax, yMax),
    };

    if (show) {
      await overlayWindow.show();
    }
    await overlayWindow.setPosition(positions[presetPosition]);
    if (show) {
      options.isOverlayVisible.value = true;
      await emitTo<boolean>("pov", OVERLAY_VISIBLE_EVENT, true);
    }
  }

  async function startOverlayAdjust() {
    overlayAdjusting.value = true;
    const existingWindow = await Window.getByLabel("pov");
    const monitor = (await currentMonitor()) ?? (await primaryMonitor());
    const previousPosition =
      existingWindow && monitor
        ? (await existingWindow.outerPosition()).toLogical(monitor.scaleFactor)
        : null;
    await existingWindow?.destroy();
    const overlayWindow = await ensureOverlayWindow(true);
    if (!overlayWindow) {
      return;
    }

    await syncOverlayWindow(overlayWindow);
    if (previousPosition) {
      await overlayWindow.setPosition(new LogicalPosition(previousPosition.x, previousPosition.y));
    }
    await overlayWindow.show();
    options.isOverlayVisible.value = true;
    await setOverlayClickThrough(false);
    await emitAdjustMode(true);
  }

  async function handleOverlayReady() {
    const overlayWindow = await Window.getByLabel("pov");
    if (!overlayWindow) {
      return;
    }

    await syncOverlayWindow(overlayWindow);
    if (overlayAdjusting.value) {
      await setOverlayClickThrough(false);
      await emitAdjustMode(true);
    }
  }

  async function saveOverlayAdjust() {
    const overlayWindow = await Window.getByLabel("pov");
    const monitor = (await currentMonitor()) ?? (await primaryMonitor());
    if (!overlayWindow || !monitor) {
      return;
    }

    const physicalPosition = await overlayWindow.outerPosition();
    const logicalPosition = physicalPosition.toLogical(monitor.scaleFactor);
    const workArea = monitorWorkAreaToRect(
      monitor.workArea.position.toLogical(monitor.scaleFactor),
      monitor.workArea.size.toLogical(monitor.scaleFactor),
    );
    options.customOverlayPosition.value = {
      x: Math.round(logicalPosition.x),
      y: Math.round(logicalPosition.y),
      workArea,
      scaleFactor: monitor.scaleFactor,
    };
    options.overlayPosition.value = "custom";
    overlayAdjusting.value = false;
    options.markProfileChanged();
    options.scheduleAppConfigSave();
    await setOverlayClickThrough(true);
    await emitAdjustMode(false);
  }

  async function cancelOverlayAdjust() {
    overlayAdjusting.value = false;
    await setOverlayClickThrough(true);
    await emitAdjustMode(false);
    await moveOverlay(options.overlayPosition.value, false);
  }

  async function setOverlayClickThrough(enabled: boolean) {
    const overlayWindow = await Window.getByLabel("pov");
    await overlayWindow?.setIgnoreCursorEvents(enabled);
    if (!enabled) {
      window.setTimeout(() => {
        void Window.getByLabel("pov").then((window) => window?.setIgnoreCursorEvents(false));
      }, 100);
      window.setTimeout(() => {
        void Window.getByLabel("pov").then((window) => window?.setIgnoreCursorEvents(false));
      }, 250);
      window.setTimeout(() => {
        void Window.getByLabel("pov").then((window) => window?.setIgnoreCursorEvents(false));
      }, 500);
    }
  }

  async function emitAdjustMode(enabled: boolean) {
    await emitTo<OverlayAdjustModePayload>("pov", OVERLAY_ADJUST_MODE_EVENT, { enabled });
  }

  return {
    overlayAdjusting,
    updateOverlayStyle,
    updateOverlayRows,
    resizeOverlayWindow,
    syncOverlayWindow,
    destroyOverlayWindow,
    setOverlayVisible,
    moveOverlay,
    startOverlayAdjust,
    handleOverlayReady,
    saveOverlayAdjust,
    cancelOverlayAdjust,
  };
}
