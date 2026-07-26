import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { type ComputedRef, ref } from "vue";
import { tauriApi } from "../api/tauri";
import {
  INPUT_STATE_EVENT,
  OVERLAY_ACTIVE_KEYS_EVENT,
  type InputStatePayload,
  type OverlayActiveKeysPayload,
} from "../domain/inputEvents";

type UseInputStateBridgeOptions = {
  isOverlayWindow: ComputedRef<boolean>;
  onConfigInput: (payload: InputStatePayload) => void;
};

export function useInputStateBridge(options: UseInputStateBridgeOptions) {
  const activeKeyIds = ref(new Set<string>());
  const overlayInputDebug = ref("");
  const overlayInputDebugCount = ref(0);
  let configLogCount = 0;
  let overlayLogCount = 0;
  let unlistenInputState: UnlistenFn | undefined;

  function updateActiveKey(keyId: string, pressed: boolean) {
    const nextKeys = new Set(activeKeyIds.value);

    if (pressed) {
      nextKeys.add(keyId);
    } else {
      nextKeys.delete(keyId);
    }

    activeKeyIds.value = nextKeys;
  }

  async function startInputBridge() {
    if (options.isOverlayWindow.value) {
      unlistenInputState = await listen<OverlayActiveKeysPayload>(
        OVERLAY_ACTIVE_KEYS_EVENT,
        (event) => {
          overlayInputDebugCount.value += 1;
          overlayInputDebug.value = `${event.payload.keyIds.join("+") || "none"} #${overlayInputDebugCount.value}`;
          activeKeyIds.value = new Set(event.payload.keyIds);
          if (overlayLogCount < 80) {
            overlayLogCount += 1;
            void tauriApi.writeDebugLog(
              "frontend-pov-input",
              `active=${JSON.stringify(event.payload.keyIds)}`,
            );
          }
        },
      );
      return;
    }

    unlistenInputState = await listen<InputStatePayload>(
      INPUT_STATE_EVENT,
      (event) => {
        updateActiveKey(event.payload.keyId, event.payload.pressed);
        if (configLogCount < 80) {
          configLogCount += 1;
          void tauriApi.writeDebugLog(
            "frontend-config-input",
            `keyId=${event.payload.keyId} pressed=${event.payload.pressed} active=${JSON.stringify([...activeKeyIds.value])}`,
          );
        }
        options.onConfigInput(event.payload);
      },
    );
  }

  function stopInputBridge() {
    unlistenInputState?.();
    unlistenInputState = undefined;
  }

  return {
    activeKeyIds,
    overlayInputDebug,
    startInputBridge,
    stopInputBridge,
  };
}
