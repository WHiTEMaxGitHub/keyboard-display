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

const povOverlay = ref<InstanceType<typeof PovOverlay> | null>(null);

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
  const measured = povOverlay.value?.measure();
  if (!measured) {
    return;
  }

  await emitTo<OverlayMeasuredPayload>("config", OVERLAY_MEASURED_EVENT, measured);
}
</script>

<template>
  <main class="overlay-root">
    <PovOverlay
      ref="povOverlay"
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
