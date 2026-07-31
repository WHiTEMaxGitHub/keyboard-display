<script setup lang="ts">
import { computed, inject, ref, type ComputedRef } from "vue";
import type { AppConfig } from "../../domain/defaultConfig";
import type { VideoExporterConfig } from "../../domain/videoExporter";
import { tauriApi } from "../../api/tauri";
import ExportPanel from "../ExportPanel.vue";

const config = inject<AppConfig>("config")!;
const profileNameRef = inject<ComputedRef<string>>("profileName")!;
const profileName = computed(() => profileNameRef.value);
const videoExporterConfig = inject<VideoExporterConfig>("videoExporterConfig")!;
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;

const videoExporterInstalling = ref(false);
const videoExporterUninstalling = ref(false);

function updateRenderMarkers(event: Event) {
  emit("update-export-config", {
    ...config.export,
    renderMarkers: (event.target as HTMLInputElement).checked,
  });
}

async function installAppManagedVideoExporter() {
  if (videoExporterInstalling.value) {
    return;
  }

  videoExporterInstalling.value = true;

  try {
    const result = await tauriApi.installAppManagedVideoExporter();
    emit("notify", "success", `Video exporter installed: ${result.path}`);
  } catch (error) {
    emit("notify", "error", `Video exporter install failed: ${String(error)}`);
  } finally {
    videoExporterInstalling.value = false;
  }
}

async function uninstallAppManagedVideoExporter() {
  if (videoExporterUninstalling.value) {
    return;
  }

  videoExporterUninstalling.value = true;

  try {
    await tauriApi.uninstallAppManagedVideoExporter();
    emit("notify", "success", "App-managed video exporter uninstalled.");
  } catch (error) {
    emit("notify", "error", `Video exporter uninstall failed: ${String(error)}`);
  } finally {
    videoExporterUninstalling.value = false;
  }
}
</script>

<template>
  <section class="page-stack">
    <ExportPanel
      :config="config"
      :profile-name="profileName"
      :render-markers="config.export.renderMarkers"
      :video-exporter-config="videoExporterConfig"
      :installing-app-managed-exporter="videoExporterInstalling"
      :uninstalling-app-managed-exporter="videoExporterUninstalling"
      @update-render-markers="updateRenderMarkers"
      @update-export-config="emit('update-export-config', $event)"
      @update-video-exporter-config="emit('update-video-exporter-config', $event)"
      @install-app-managed-exporter="installAppManagedVideoExporter"
      @uninstall-app-managed-exporter="uninstallAppManagedVideoExporter"
    />
  </section>
</template>

<style scoped>
.page-stack {
  display: grid;
  gap: 16px;
}
</style>
