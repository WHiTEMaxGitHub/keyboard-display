import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { type ComputedRef, ref } from "vue";
import { tauriApi } from "../api/tauri";
import {
  INPUT_STATE_EVENT,
  OVERLAY_ACTIVE_KEYS_EVENT,
  recoverMissedConfigReleases,
  type InputStatePayload,
  type OverlayActiveKeysPayload,
} from "../domain/inputEvents";
import { PovFrameScheduler } from "../domain/povFrameScheduler";

type UseInputStateBridgeOptions = {
  isOverlayWindow: ComputedRef<boolean>;
  onConfigInput: (payload: InputStatePayload) => void;
};

export function useInputStateBridge(options: UseInputStateBridgeOptions) {
  const activeKeyIds = ref(new Set<string>());
  let unlistenInputState: UnlistenFn | undefined;
  let lastConfigSeq = 0;

  function logPovPhase(
    phase: "receive" | "paint",
    payload: OverlayActiveKeysPayload,
  ) {
    if (!payload.debug) {
      return;
    }
    void tauriApi.writeDebugLog(
      "pov-webview",
      `phase=${phase} capture_seq=${payload.captureSeq} display_seq=${payload.seq} t_capture=${payload.tCapture} webview_mono_ms=${performance.now().toFixed(3)} keys=${payload.keyIds.join(",")}`,
    ).catch(() => undefined);
  }

  const povScheduler = new PovFrameScheduler({
    apply: (payload) => {
      activeKeyIds.value = new Set(payload.keyIds);
    },
    onPhase: logPovPhase,
  });

  async function startInputBridge() {
    if (options.isOverlayWindow.value) {
      unlistenInputState = await listen<OverlayActiveKeysPayload>(
        OVERLAY_ACTIVE_KEYS_EVENT,
        (event) => {
          povScheduler.receive(event.payload);
        },
      );
      return;
    }

    unlistenInputState = await listen<InputStatePayload>(
      INPUT_STATE_EVENT,
      (event) => {
        if (event.payload.seq <= lastConfigSeq) {
          return;
        }
        const recoveredReleases = recoverMissedConfigReleases(
          activeKeyIds.value,
          lastConfigSeq,
          event.payload,
        );
        lastConfigSeq = event.payload.seq;
        activeKeyIds.value = new Set(event.payload.keyIds);

        for (const release of recoveredReleases) {
          options.onConfigInput(release);
        }
        options.onConfigInput(event.payload);
      },
    );
  }

  function stopInputBridge() {
    unlistenInputState?.();
    unlistenInputState = undefined;
    povScheduler.stop();
  }

  return {
    activeKeyIds,
    startInputBridge,
    stopInputBridge,
  };
}
