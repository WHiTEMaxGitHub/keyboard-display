import type { OverlayActiveKeysPayload } from "./inputEvents";

export type AnimationFrameDriver = {
  request(callback: FrameRequestCallback): number;
  cancel(handle: number): void;
};

export type PovFramePhase = "receive" | "paint";

type SchedulerOptions = {
  apply: (payload: OverlayActiveKeysPayload) => void;
  onPhase?: (phase: PovFramePhase, payload: OverlayActiveKeysPayload) => void;
  frames?: AnimationFrameDriver;
};

const browserFrames: AnimationFrameDriver = {
  request: (callback) => window.requestAnimationFrame(callback),
  cancel: (handle) => window.cancelAnimationFrame(handle),
};

/**
 * Applies full POV snapshots in sequence order. If a Down snapshot is replaced
 * by Up before its first paint, Up is deferred to the following animation
 * frame so the press remains visible for one actual browser paint.
 */
export class PovFrameScheduler {
  private readonly applySnapshot: SchedulerOptions["apply"];
  private readonly onPhase?: SchedulerOptions["onPhase"];
  private readonly frames: AnimationFrameDriver;
  private lastSeq = 0;
  private awaitingPaint = false;
  private current: OverlayActiveKeysPayload | null = null;
  private latest: OverlayActiveKeysPayload | null = null;
  private readonly unpaintedPressKeys = new Set<string>();
  private paintFrame: number | null = null;
  private deferredFrame: number | null = null;
  private stopped = false;

  constructor(options: SchedulerOptions) {
    this.applySnapshot = options.apply;
    this.onPhase = options.onPhase;
    this.frames = options.frames ?? browserFrames;
  }

  receive(payload: OverlayActiveKeysPayload): boolean {
    if (this.stopped || payload.seq <= this.lastSeq) {
      return false;
    }

    this.onPhase?.("receive", payload);
    const hasGap = this.lastSeq !== 0 && payload.seq !== this.lastSeq + 1;
    this.lastSeq = payload.seq;

    if (hasGap) {
      this.latest = payload;
      this.unpaintedPressKeys.clear();
      if (this.awaitingPaint) {
        // The currently applied state has not reached a paint yet. Preserve
        // that one frame, then recover from the newest complete snapshot.
        return true;
      }
      this.cancelDeferredFrame();
      this.applyIfVisuallyChanged(payload);
      return true;
    }

    const previousKeys = new Set(this.latest?.keyIds ?? []);
    for (const keyId of payload.keyIds) {
      if (!previousKeys.has(keyId)) {
        this.unpaintedPressKeys.add(keyId);
      }
    }
    this.latest = payload;

    if (!this.awaitingPaint && this.deferredFrame === null) {
      this.applyIfVisuallyChanged(this.composedTarget());
    }
    return true;
  }

  stop() {
    this.stopped = true;
    this.latest = null;
    this.unpaintedPressKeys.clear();
    this.cancelFrames();
  }

  private apply(payload: OverlayActiveKeysPayload) {
    this.current = payload;
    this.applySnapshot(payload);
    this.awaitingPaint = true;
    this.paintFrame = this.frames.request(() => this.confirmPaint());
  }

  private confirmPaint() {
    this.paintFrame = null;
    this.awaitingPaint = false;
    if (this.current) {
      this.onPhase?.("paint", this.current);
      for (const keyId of this.current.keyIds) {
        this.unpaintedPressKeys.delete(keyId);
      }
    }

    const target = this.composedTarget();
    if (target && !sameKeys(this.current, target)) {
      this.deferredFrame = this.frames.request(() => {
        this.deferredFrame = null;
        this.applyIfVisuallyChanged(this.composedTarget());
      });
    }
  }

  private composedTarget(): OverlayActiveKeysPayload | null {
    if (!this.latest) {
      return null;
    }
    const keys = new Set(this.latest.keyIds);
    for (const keyId of this.unpaintedPressKeys) {
      keys.add(keyId);
    }
    const keyIds = [...keys].sort();
    return sameKeyArrays(keyIds, this.latest.keyIds)
      ? this.latest
      : { ...this.latest, keyIds };
  }

  private applyIfVisuallyChanged(payload: OverlayActiveKeysPayload | null) {
    if (payload && !sameKeys(this.current, payload)) {
      this.apply(payload);
    }
  }

  private cancelDeferredFrame() {
    if (this.deferredFrame !== null) {
      this.frames.cancel(this.deferredFrame);
      this.deferredFrame = null;
    }
  }

  private cancelFrames() {
    if (this.paintFrame !== null) {
      this.frames.cancel(this.paintFrame);
      this.paintFrame = null;
    }
    this.cancelDeferredFrame();
  }
}

function sameKeys(
  left: OverlayActiveKeysPayload | null,
  right: OverlayActiveKeysPayload,
) {
  return left !== null && sameKeyArrays(left.keyIds, right.keyIds);
}

function sameKeyArrays(left: string[], right: string[]) {
  return left.length === right.length && left.every((key, index) => key === right[index]);
}
