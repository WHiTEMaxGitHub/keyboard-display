import { describe, expect, it } from "vitest";
import {
  INPUT_STATE_EVENT,
  recoverMissedConfigReleases,
  type InputStatePayload,
} from "./inputEvents";

describe("input event contract", () => {
  it("uses the native input-state channel", () => {
    const payload: InputStatePayload = {
      seq: 7,
      captureSeq: 9,
      tCapture: 123,
      keyIds: ["w"],
      keyId: "w",
      pressed: true,
    };

    expect(INPUT_STATE_EVENT).toBe("input-state");
    expect(payload).toEqual({
      seq: 7,
      captureSeq: 9,
      tCapture: 123,
      keyIds: ["w"],
      keyId: "w",
      pressed: true,
    });
  });

  it("synthesizes a missed release from a complete gap snapshot", () => {
    const payload: InputStatePayload = {
      seq: 4,
      captureSeq: 8,
      tCapture: 123,
      keyIds: ["space"],
      keyId: "space",
      pressed: true,
    };

    expect(recoverMissedConfigReleases(["mouse-left"], 1, payload)).toEqual([
      { ...payload, keyId: "mouse-left", pressed: false },
    ]);
  });

  it("does not duplicate the release carried by the gap payload", () => {
    const payload: InputStatePayload = {
      seq: 4,
      captureSeq: 8,
      tCapture: 123,
      keyIds: [],
      keyId: "mouse-left",
      pressed: false,
    };

    expect(recoverMissedConfigReleases(["mouse-left"], 1, payload)).toEqual([]);
  });
});
