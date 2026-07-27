import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { type ComputedRef, ref } from "vue";
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
          activeKeyIds.value = new Set(event.payload.keyIds);
        },
      );
      return;
    }

    unlistenInputState = await listen<InputStatePayload>(
      INPUT_STATE_EVENT,
      (event) => {
        updateActiveKey(event.payload.keyId, event.payload.pressed);
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
    startInputBridge,
    stopInputBridge,
  };
}
