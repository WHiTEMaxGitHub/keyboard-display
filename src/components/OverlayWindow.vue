<script setup lang="ts">
import { emitTo } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { nextTick, onMounted, ref, watch } from "vue";
import { OVERLAY_MEASURED_EVENT, type OverlayMeasuredPayload } from "../domain/inputEvents";
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

onMounted(async () => {
  const currentWindow = getCurrentWindow();
  await currentWindow.setVisibleOnAllWorkspaces(true);
  await currentWindow.setIgnoreCursorEvents(true);
  await reportMeasuredSize();
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
</script>

<template>
  <main ref="overlayRoot" class="overlay-root">
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
</style>
