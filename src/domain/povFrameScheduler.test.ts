import { describe, expect, it } from "vitest";
import type { AnimationFrameDriver } from "./povFrameScheduler";
import { PovFrameScheduler } from "./povFrameScheduler";
import type { OverlayActiveKeysPayload } from "./inputEvents";

class ManualFrames implements AnimationFrameDriver {
  private nextId = 1;
  private callbacks = new Map<number, FrameRequestCallback>();

  request(callback: FrameRequestCallback) {
    const id = this.nextId++;
    this.callbacks.set(id, callback);
    return id;
  }

  cancel(handle: number) {
    this.callbacks.delete(handle);
  }

  runFrame() {
    const callbacks = [...this.callbacks.values()];
    this.callbacks.clear();
    callbacks.forEach((callback) => callback(0));
  }

  get size() {
    return this.callbacks.size;
  }
}

function snapshot(seq: number, keyIds: string[]): OverlayActiveKeysPayload {
  return { seq, captureSeq: seq, tCapture: seq * 10, keyIds, debug: false };
}

function setup(preserveShortPresses = true) {
  const frames = new ManualFrames();
  const paints: string[][] = [];
  const applied: string[][] = [];
  const scheduler = new PovFrameScheduler({
    frames,
    apply: (payload) => applied.push(payload.keyIds),
    onPhase: (phase, payload) => {
      if (phase === "paint") {
        paints.push(payload.keyIds);
      }
    },
    preserveShortPresses: () => preserveShortPresses,
  });
  return { frames, paints, applied, scheduler };
}

describe("PovFrameScheduler", () => {
  it("keeps a same-frame short press visible for one paint", () => {
    const { frames, paints, applied, scheduler } = setup();
    scheduler.receive(snapshot(1, ["w"]));
    scheduler.receive(snapshot(2, []));

    expect(applied).toEqual([["w"]]);
    frames.runFrame();
    expect(paints).toEqual([["w"]]);
    expect(applied).toEqual([["w"]]);

    frames.runFrame();
    expect(applied).toEqual([["w"], []]);
  });

  it("does not add a fixed delay to a long press release", () => {
    const { frames, applied, scheduler } = setup();
    scheduler.receive(snapshot(1, ["w"]));
    frames.runFrame();

    scheduler.receive(snapshot(2, []));
    expect(applied).toEqual([["w"], []]);
  });

  it("applies every latest state immediately when short-press preservation is disabled", () => {
    const { applied, scheduler } = setup(false);
    scheduler.receive(snapshot(1, ["w"]));
    scheduler.receive(snapshot(2, []));
    scheduler.receive(snapshot(3, ["space"]));
    scheduler.receive(snapshot(4, []));

    expect(applied).toEqual([["w"], [], ["space"], []]);
  });

  it("ignores old and out-of-order sequence numbers", () => {
    const { frames, applied, scheduler } = setup();
    scheduler.receive(snapshot(5, ["w"]));
    frames.runFrame();
    scheduler.receive(snapshot(4, []));
    scheduler.receive(snapshot(5, []));

    expect(applied).toEqual([["w"]]);
  });

  it("preserves an unpainted Down across a sequence gap", () => {
    const { frames, paints, applied, scheduler } = setup();
    scheduler.receive(snapshot(1, ["w"]));
    scheduler.receive(snapshot(4, []));

    expect(applied).toEqual([["w"]]);
    frames.runFrame();
    expect(paints).toEqual([["w"]]);
    frames.runFrame();
    expect(applied).toEqual([["w"], []]);
  });

  it("recovers immediately from a gap when no paint is pending", () => {
    const { frames, applied, scheduler } = setup();
    scheduler.receive(snapshot(1, ["w"]));
    frames.runFrame();
    scheduler.receive(snapshot(4, []));

    expect(applied).toEqual([["w"], []]);
  });

  it("paints Space once while LMB is waiting for its first paint", () => {
    const { frames, paints, applied, scheduler } = setup();
    scheduler.receive(snapshot(1, ["mouse-left"]));
    scheduler.receive(snapshot(2, ["mouse-left", "space"]));
    scheduler.receive(snapshot(3, ["mouse-left"]));

    frames.runFrame();
    frames.runFrame();
    frames.runFrame();
    frames.runFrame();

    expect(paints).toContainEqual(["mouse-left", "space"]);
    expect(applied[applied.length - 1]).toEqual(["mouse-left"]);
  });

  it("coalesces interleaved short keys into one bounded synthetic paint", () => {
    const { frames, paints, applied, scheduler } = setup();
    scheduler.receive(snapshot(1, ["mouse-left"]));
    scheduler.receive(snapshot(2, ["mouse-left", "space"]));
    scheduler.receive(snapshot(3, ["mouse-left"]));
    scheduler.receive(snapshot(4, ["mouse-left", "w"]));
    scheduler.receive(snapshot(5, ["mouse-left"]));

    frames.runFrame();
    frames.runFrame();
    frames.runFrame();
    frames.runFrame();

    expect(paints).toContainEqual(["mouse-left", "space", "w"]);
    expect(applied).toHaveLength(3);
    expect(applied[applied.length - 1]).toEqual(["mouse-left"]);
  });

  it("cancels pending animation frames when stopped", () => {
    const { frames, scheduler } = setup();
    scheduler.receive(snapshot(1, ["w"]));
    scheduler.receive(snapshot(2, []));
    scheduler.stop();

    expect(frames.size).toBe(0);
  });
});
