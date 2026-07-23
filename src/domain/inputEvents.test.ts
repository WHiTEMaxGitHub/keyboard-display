import { describe, expect, it } from "vitest";
import { INPUT_STATE_EVENT, type InputStatePayload } from "./inputEvents";

describe("input event contract", () => {
  it("uses the native input-state channel", () => {
    const payload: InputStatePayload = { keyId: "w", pressed: true };

    expect(INPUT_STATE_EVENT).toBe("input-state");
    expect(payload).toEqual({ keyId: "w", pressed: true });
  });
});
