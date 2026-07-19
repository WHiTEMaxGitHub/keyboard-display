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
      clearSelection();
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

  clearSelection();
  await setClickThrough(false);
  await getCurrentWindow().startDragging();
}

function clearSelection() {
  window.getSelection()?.removeAllRanges();
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
  <main class="overlay-root">
    <button
      v-if="adjusting"
      class="drag-handle"
      type="button"
      draggable="false"
      title="Drag POV"
      @mousedown.prevent.stop="startDrag"
    >
      Drag
    </button>
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
  position: relative;
  display: grid;
  gap: 8px;
  width: max-content;
  justify-items: center;
  padding: 0 12px 12px 0;
  background: transparent;
}

.overlay-root.adjusting,
.overlay-root.adjusting * {
  user-select: none;
  -webkit-user-select: none;
}

.drag-handle {
  position: absolute;
  top: 0;
  right: 0;
  z-index: 5;
  min-height: 24px;
  border: 1px solid rgba(37, 211, 102, 0.56);
  border-radius: 6px;
  background: rgba(17, 19, 22, 0.9);
  color: #eafff0;
  cursor: move;
  font-size: 11px;
  font-weight: 800;
  padding: 0 8px;
}

</style>
