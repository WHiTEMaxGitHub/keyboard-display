<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted, onUnmounted, ref } from "vue";
import { OVERLAY_ADJUST_MODE_EVENT } from "../domain/inputEvents";
import type { KeyBinding, OverlayLayout, OverlayRow, OverlayStyle } from "../domain/defaultConfig";
import PovOverlay from "./PovOverlay.vue";

defineProps<{
  layout: OverlayLayout;
  rows: OverlayRow[];
  keys: KeyBinding[];
  activeKeys: Set<string>;
  overlayStyle: OverlayStyle;
  syncFeedbackActive?: boolean;
}>();

const adjusting = ref(false);
let unlistenAdjustMode: UnlistenFn | undefined;

onMounted(async () => {
  const currentWindow = getCurrentWindow();
  await currentWindow.setVisibleOnAllWorkspaces(true);
  await currentWindow.setIgnoreCursorEvents(true);
  unlistenAdjustMode = await listen<{ enabled: boolean }>(
    OVERLAY_ADJUST_MODE_EVENT,
    async (event) => {
      adjusting.value = event.payload.enabled;
      await setClickThrough(!event.payload.enabled);
    },
  );
});

onUnmounted(() => {
  unlistenAdjustMode?.();
});

async function startDrag() {
  if (!adjusting.value) {
    return;
  }

  await setClickThrough(false);
  await getCurrentWindow().startDragging();
}

async function setClickThrough(enabled: boolean) {
  const currentWindow = getCurrentWindow();
  await currentWindow.setIgnoreCursorEvents(enabled);
  if (!enabled) {
    window.setTimeout(() => {
      void currentWindow.setIgnoreCursorEvents(false);
    }, 80);
    window.setTimeout(() => {
      void currentWindow.setIgnoreCursorEvents(false);
    }, 180);
  }
}
</script>

<template>
  <main
    class="overlay-root"
    @mousedown.prevent.stop="startDrag"
  >
    <PovOverlay
      :layout="layout"
      :rows="rows"
      :keys="keys"
      :active-keys="activeKeys"
      :overlay-style="overlayStyle"
      :sync-feedback-active="syncFeedbackActive"
      :adjusting="adjusting"
    />
  </main>
</template>

<style scoped>
.overlay-root {
  display: grid;
  gap: 8px;
  width: max-content;
  justify-items: center;
  padding: 0 12px 12px 0;
  background: transparent;
}

</style>
