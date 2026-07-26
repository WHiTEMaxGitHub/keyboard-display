<script setup lang="ts">
import { emitTo, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted, onUnmounted, ref } from "vue";
import { OVERLAY_ADJUST_MODE_EVENT, OVERLAY_READY_EVENT } from "../domain/inputEvents";
import type {
  KeyBinding,
  KeyIdLabelRegistry,
  OverlayLayout,
  OverlayRow,
  OverlayStyle,
} from "../domain/defaultConfig";
import PovOverlay from "./PovOverlay.vue";

defineProps<{
  layout: OverlayLayout;
  rows: OverlayRow[];
  keys: KeyBinding[];
  keyIdLabels?: KeyIdLabelRegistry;
  activeKeys: Set<string>;
  overlayStyle: OverlayStyle;
  syncFeedbackActive?: boolean;
  inputDebug?: string;
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
  await emitTo("config", OVERLAY_READY_EVENT);
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
    :class="['overlay-root', { adjusting }]"
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
    <span v-if="inputDebug" class="input-debug">{{ inputDebug }}</span>
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
}

.input-debug {
  position: absolute;
  top: 0;
  right: 0;
  z-index: 20;
  border: 1px solid rgba(255, 255, 255, 0.24);
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.72);
  color: #eafff0;
  font: 700 11px/1.2 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  padding: 3px 5px;
  pointer-events: none;
}

.overlay-root.adjusting,
.overlay-root.adjusting * {
  user-select: none;
  -webkit-user-select: none;
}

</style>
