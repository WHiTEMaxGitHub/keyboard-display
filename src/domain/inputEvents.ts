export const INPUT_STATE_EVENT = "input-state";
export const OVERLAY_ACTIVE_KEYS_EVENT = "overlay-active-keys";
export const OVERLAY_STYLE_EVENT = "overlay-style";
export const OVERLAY_VISIBLE_EVENT = "overlay-visible";
export const OVERLAY_CONFIG_EVENT = "overlay-config";
export const OVERLAY_READY_EVENT = "overlay-ready";
export const OVERLAY_SYNC_FEEDBACK_EVENT = "overlay-sync-feedback";
export const OVERLAY_ADJUST_MODE_EVENT = "overlay-adjust-mode";
export const RECORDING_UI_EVENT = "recording-ui";

export type InputStatePayload = {
  seq: number;
  captureSeq: number;
  tCapture: number;
  keyIds: string[];
  keyId: string;
  pressed: boolean;
};

export type OverlayActiveKeysPayload = {
  seq: number;
  captureSeq: number;
  tCapture: number;
  keyIds: string[];
  debug: boolean;
};

export function recoverMissedConfigReleases(
  previousKeyIds: Iterable<string>,
  previousSeq: number,
  payload: InputStatePayload,
): InputStatePayload[] {
  if (previousSeq === 0 || payload.seq === previousSeq + 1) {
    return [];
  }
  const latest = new Set(payload.keyIds);
  return [...previousKeyIds]
    .filter(
      (keyId) =>
        !latest.has(keyId) &&
        (payload.pressed || payload.keyId !== keyId),
    )
    .map((keyId) => ({ ...payload, keyId, pressed: false }));
}
