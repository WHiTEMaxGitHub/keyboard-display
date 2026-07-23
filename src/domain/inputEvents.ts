export const INPUT_STATE_EVENT = "input-state";
export const OVERLAY_STYLE_EVENT = "overlay-style";
export const OVERLAY_VISIBLE_EVENT = "overlay-visible";
export const OVERLAY_CONFIG_EVENT = "overlay-config";
export const OVERLAY_READY_EVENT = "overlay-ready";
export const OVERLAY_SYNC_FEEDBACK_EVENT = "overlay-sync-feedback";
export const OVERLAY_ADJUST_MODE_EVENT = "overlay-adjust-mode";

export type InputStatePayload = {
  keyId: string;
  pressed: boolean;
};
