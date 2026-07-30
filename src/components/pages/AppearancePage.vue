<script setup lang="ts">
import { inject } from "vue";
import type { AppConfig } from "../../domain/defaultConfig";
import AppearancePanel from "../AppearancePanel.vue";

const config = inject<AppConfig>("config")!;
const recentColors = inject("recentColors") as any as { value: string[] };
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;

function rememberColor(color: string) {
  const normalized = color;
  recentColors.value = [
    normalized,
    ...recentColors.value.filter((c) => c !== normalized),
  ].slice(0, 8);
}
</script>

<template>
  <section class="page-stack">
    <AppearancePanel
      :config="config"
      :recent-colors="recentColors.value"
      @preview-overlay-style="emit('preview-overlay-style', $event)"
      @update-overlay-style="emit('update-overlay-style', $event)"
      @remember-color="rememberColor"
      @refresh-pov="emit('refresh-pov')"
    />
  </section>
</template>