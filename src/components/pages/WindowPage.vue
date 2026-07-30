<script setup lang="ts">
import { inject } from "vue";
import type { AppConfig } from "../../domain/defaultConfig";
import WindowPanel from "../WindowPanel.vue";

const config = inject<AppConfig>("config")!;
const overlayPosition = inject<string>("overlayPosition")!;
const overlayVisible = inject<boolean>("overlayVisible")!;
const overlayAdjusting = inject<boolean>("overlayAdjusting")!;
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;
</script>

<template>
  <section class="page-stack">
    <WindowPanel
      :overlay-position="overlayPosition"
      :overlay-visible="overlayVisible"
      :always-on-top="config.style.alwaysOnTop"
      :overlay-adjusting="overlayAdjusting"
      @update-overlay-visible="emit('update-overlay-visible', $event)"
      @update-always-on-top="emit('update-always-on-top', $event)"
      @start-overlay-adjust="emit('start-overlay-adjust')"
      @save-overlay-adjust="emit('save-overlay-adjust')"
      @cancel-overlay-adjust="emit('cancel-overlay-adjust')"
      @move-overlay="emit('move-overlay', $event)"
    />
  </section>
</template>