<script setup lang="ts">
import { emitTo, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import {
  OVERLAY_ADJUST_MODE_EVENT,
  OVERLAY_MEASURED_EVENT,
  type OverlayMeasuredPayload,
} from "../domain/inputEvents";
import type { KeyBinding, OverlayLayout, OverlayRow, OverlayStyle } from "../domain/defaultConfig";
import PovOverlay from "./PovOverlay.vue";

const props = defineProps<{
  layout: OverlayLayout;
  rows: OverlayRow[];
  keys: KeyBinding[];
  activeKeys: Set<string>;
  overlayStyle: OverlayStyle;
  syncFeedbackActive?: boolean;
}>();

const overlayRoot = ref<HTMLElement | null>(null);
const adjusting = ref(false);
let unlistenAdjustMode: UnlistenFn | undefined;

onMounted(async () => {
  const currentWindow = getCurrentWindow();
  await currentWindow.setVisibleOnAllWorkspaces(true);
  await currentWindow.setIgnoreCursorEvents(true);
  unlistenAdjustMode = await listen<{ enabled: boolean }>(
    OVERLAY_ADJUST_MODE_EVENT,
    (event) => {
      adjusting.value = event.payload.enabled;
    },
  );
  await reportMeasuredSize();
});

onUnmounted(() => {
  unlistenAdjustMode?.();
});

watch(
  () => [props.layout, props.rows, props.overlayStyle],
  () => {
    void reportMeasuredSize();
  },
  { deep: true },
);

async function reportMeasuredSize() {
  await nextTick();
  await nextAnimationFrame();
  await nextAnimationFrame();
  const rect = overlayRoot.value?.getBoundingClientRect();
  if (!rect) {
    return;
  }

  await emitTo<OverlayMeasuredPayload>("config", OVERLAY_MEASURED_EVENT, {
    width: Math.ceil(rect.width),
    height: Math.ceil(rect.height),
  });
}

function nextAnimationFrame() {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => resolve());
  });
}

async function startDrag() {
  if (!adjusting.value) {
    return;
  }

  await getCurrentWindow().startDragging();
}
</script>

<template>
  <main
    ref="overlayRoot"
    :class="['overlay-root', { adjusting }]"
    @pointerdown="startDrag"
  >
    <PovOverlay
      :layout="layout"
      :rows="rows"
      :keys="keys"
      :active-keys="activeKeys"
      :overlay-style="overlayStyle"
      :sync-feedback-active="syncFeedbackActive"
    />
  </main>
</template>

<style scoped>
.overlay-root {
  display: grid;
  gap: 8px;
  width: max-content;
  justify-items: center;
  padding: 14px;
  background: transparent;
}

.overlay-root.adjusting {
  cursor: move;
  outline: 2px solid rgba(37, 211, 102, 0.75);
  outline-offset: -2px;
}
</style>
