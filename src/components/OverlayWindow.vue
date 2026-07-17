<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted } from "vue";
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

onMounted(async () => {
  const currentWindow = getCurrentWindow();
  await currentWindow.setVisibleOnAllWorkspaces(true);
  await currentWindow.setIgnoreCursorEvents(true);
});
</script>

<template>
  <main class="overlay-root">
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
  padding: 0;
  background: transparent;
}
</style>
