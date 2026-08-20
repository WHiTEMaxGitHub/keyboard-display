<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted, onUnmounted, ref } from "vue";
import { OVERLAY_ADJUST_MODE_EVENT } from "../domain/inputEvents";
import type {
  KeyBinding,
  KeyIdLabelRegistry,
  OverlayLayout,
  OverlayRow,
  OverlayStyle,
} from "../domain/defaultConfig";
import PovOverlay from "./PovOverlay.vue";

defineProps<{
  visible?: boolean;
  layout: OverlayLayout;
  rows: OverlayRow[];
  keys: KeyBinding[];
  keyIdLabels?: KeyIdLabelRegistry;
  activeKeys: Set<string>;
  overlayStyle: OverlayStyle;
  syncFeedbackActive?: boolean;
}>();

const adjusting = ref(false);
const startsAdjusting = new URLSearchParams(window.location.search).get("adjust") === "1";
let unlistenAdjustMode: UnlistenFn | undefined;

onMounted(async () => {
  const currentWindow = getCurrentWindow();
  await currentWindow.setVisibleOnAllWorkspaces(true);
  adjusting.value = startsAdjusting;
  await currentWindow.setIgnoreCursorEvents(!startsAdjusting);
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

function clearSelection() {
  window.getSelection()?.removeAllRanges();
}

async function startDrag() {
  if (!adjusting.value) {
    return;
  }

  clearSelection();
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
    :class="['overlay-root', { adjusting, visible }]"
    @mousedown.prevent.stop="startDrag"
  >
    <PovOverlay
      :layout="layout"
      :rows="rows"
      :keys="keys"
      :key-id-labels="keyIdLabels"
      :active-keys="activeKeys"
      :overlay-style="overlayStyle"
      :sync-feedback-active="syncFeedbackActive"
      :adjusting="adjusting"
      @start-drag="startDrag"
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
  padding: 12px;
  background: transparent;
  opacity: 0;
  transform: translateY(6px) scale(0.985);
  filter: blur(5px);
  transition:
    opacity 520ms cubic-bezier(0.16, 1, 0.3, 1),
    transform 520ms cubic-bezier(0.16, 1, 0.3, 1),
    filter 520ms cubic-bezier(0.16, 1, 0.3, 1);
}

.overlay-root.visible {
  opacity: 1;
  transform: translateY(0) scale(1);
  filter: blur(0);
}

.overlay-root.adjusting,
.overlay-root.adjusting * {
  user-select: none;
  -webkit-user-select: none;
}

@media (prefers-reduced-motion: reduce) {
  .overlay-root,
  .overlay-root.visible {
    opacity: 1;
    transform: none;
    filter: none;
    transition-duration: 1ms;
  }
}
</style>
