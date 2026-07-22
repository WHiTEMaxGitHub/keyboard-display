<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, ref, watch } from "vue";
import { tauriApi } from "../api/tauri";
import {
  describeVideoExporter,
  normalizeVideoExporterConfig,
  type VideoExporterConfig,
  type VideoExporterStatus,
} from "../domain/videoExporter";
import BasePanel from "./BasePanel.vue";
import BaseButton from "./BaseButton.vue";

const props = defineProps<{
  renderMarkers: boolean;
  videoExporterConfig: VideoExporterConfig;
  installingAppManagedExporter: boolean;
  uninstallingAppManagedExporter: boolean;
}>();

const emit = defineEmits<{
  "update-render-markers": [event: Event];
  "update-video-exporter-config": [config: VideoExporterConfig];
  "install-app-managed-exporter": [];
  "uninstall-app-managed-exporter": [];
}>();

const exporterStatus = ref<VideoExporterStatus | null>(null);
const exporterError = ref("");
const exporterChecking = ref(false);

const resolvedExporterLabel = computed(() =>
  describeVideoExporter(exporterStatus.value?.resolved ?? null),
);

watch(
  () => [
    props.videoExporterConfig.userSelectedPath,
    props.installingAppManagedExporter,
    props.uninstallingAppManagedExporter,
  ],
  ([, installing, uninstalling], [, previousInstalling, previousUninstalling]) => {
    if (
      previousInstalling !== undefined &&
      previousInstalling &&
      !installing
    ) {
      void refreshExporterStatus();
      return;
    }

    if (
      previousUninstalling !== undefined &&
      previousUninstalling &&
      !uninstalling
    ) {
      void refreshExporterStatus();
      return;
    }

    void refreshExporterStatus();
  },
);

onMounted(() => {
  void refreshExporterStatus();
});

async function refreshExporterStatus() {
  exporterChecking.value = true;
  exporterError.value = "";

  try {
    exporterStatus.value = await tauriApi.detectVideoExporter(
      props.videoExporterConfig.userSelectedPath,
    );
  } catch (error) {
    exporterError.value = String(error);
  } finally {
    exporterChecking.value = false;
  }
}

async function chooseFfmpegPath() {
  const selectedPath = await open({
    title: "Choose ffmpeg binary",
    multiple: false,
  });

  if (typeof selectedPath !== "string") {
    return;
  }

  emit("update-video-exporter-config", normalizeVideoExporterConfig({
    ...props.videoExporterConfig,
    userSelectedPath: selectedPath,
  }));
}

function clearFfmpegPath() {
  emit("update-video-exporter-config", {
    ...props.videoExporterConfig,
    userSelectedPath: null,
  });
}

async function installAppManagedExporter() {
  if (props.installingAppManagedExporter) {
    return;
  }

  exporterError.value = "";
  emit("install-app-managed-exporter");
}

async function uninstallAppManagedExporter() {
  if (props.uninstallingAppManagedExporter) {
    return;
  }

  exporterError.value = "";
  emit("uninstall-app-managed-exporter");
}
</script>

<template>
  <BasePanel wide>
    <h2>Export</h2>
    <div class="field-row">
      <span>Transparent overlay</span>
      <strong>WebM</strong>
    </div>
    <div class="field-row">
      <span>Compatible video</span>
      <strong>MP4</strong>
    </div>
    <label class="toggle-row">
      <input
        :checked="renderMarkers"
        type="checkbox"
        @change="emit('update-render-markers', $event)"
      />
      Render sync markers
    </label>
    <div class="exporter-panel">
      <div class="section-header">
        <h3>Video exporter</h3>
        <BaseButton :disabled="exporterChecking" @click="refreshExporterStatus">
          {{ exporterChecking ? "Checking..." : "Check again" }}
        </BaseButton>
      </div>
      <div class="field-row">
        <span>Status</span>
        <strong>{{ resolvedExporterLabel }}</strong>
      </div>
      <div v-if="exporterStatus?.resolved" class="field-row">
        <span>Using</span>
        <strong>{{ exporterStatus.resolved.path }}</strong>
      </div>
      <p v-else class="notice-text">
        Video export requires ffmpeg. You can select an existing binary, use a
        PATH installation, or install an app-managed exporter later.
      </p>
      <div class="exporter-actions">
        <BaseButton @click="chooseFfmpegPath">
          Choose ffmpeg path
        </BaseButton>
        <BaseButton
          :disabled="!videoExporterConfig.userSelectedPath"
          @click="clearFfmpegPath"
        >
          Clear selected path
        </BaseButton>
        <BaseButton
          v-if="exporterStatus?.appManaged.available"
          variant="danger"
          :disabled="uninstallingAppManagedExporter"
          @click="uninstallAppManagedExporter"
        >
          {{ uninstallingAppManagedExporter ? "Uninstalling..." : "Uninstall app-managed exporter" }}
        </BaseButton>
        <BaseButton
          v-else
          variant="primary"
          :disabled="installingAppManagedExporter"
          @click="installAppManagedExporter"
        >
          {{ installingAppManagedExporter ? "Installing..." : "Install app-managed exporter" }}
        </BaseButton>
      </div>
      <div v-if="exporterStatus" class="candidate-list">
        <div class="candidate-row">
          <span>App-managed</span>
          <strong>{{ exporterStatus.appManaged.available ? "Installed" : "Not installed" }}</strong>
          <code>{{ exporterStatus.appManaged.path }}</code>
        </div>
        <div class="candidate-row">
          <span>User-selected</span>
          <strong>{{ exporterStatus.userSelected?.available ? "Available" : "Not selected" }}</strong>
          <code>{{ exporterStatus.userSelected?.path ?? "None" }}</code>
        </div>
        <div class="candidate-row">
          <span>PATH</span>
          <strong>{{ exporterStatus.path.available ? "Available" : "Not found" }}</strong>
          <code>{{ exporterStatus.path.version ?? "ffmpeg" }}</code>
        </div>
      </div>
      <p v-if="exporterError" class="error-text">{{ exporterError }}</p>
    </div>
    <p class="quiet">
      Video is generated from the input timeline, so size and format can be
      tuned after recording.
    </p>
  </BasePanel>
</template>

<style scoped>
.field-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 8px;
}

.section-header h3 {
  margin: 0;
  color: #dfe5ec;
  font-size: 16px;
  letter-spacing: 0;
  line-height: 22px;
}

.field-row span,
.quiet {
  color: #9ca7b4;
}

.field-row strong {
  min-width: 0;
  overflow-wrap: anywhere;
  text-align: right;
}

.toggle-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
  color: #c9d1da;
  font-weight: 700;
}

.toggle-row input {
  width: 18px;
  height: 18px;
  accent-color: #25d366;
}

.exporter-panel {
  display: grid;
  gap: 10px;
  margin-top: 18px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  padding: 14px;
}

.exporter-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.candidate-list {
  display: grid;
  gap: 8px;
}

.candidate-row {
  display: grid;
  grid-template-columns: 120px 110px minmax(0, 1fr);
  gap: 10px;
  align-items: center;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background: #10141a;
  padding: 8px 10px;
}

.candidate-row span {
  color: #9ca7b4;
  font-size: 13px;
  font-weight: 800;
}

.candidate-row strong,
.candidate-row code {
  min-width: 0;
  overflow-wrap: anywhere;
}

.candidate-row code {
  color: #c9d1da;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.notice-text {
  margin: 0;
  border: 1px solid rgba(255, 209, 102, 0.18);
  border-radius: 7px;
  background: rgba(255, 209, 102, 0.08);
  color: #e8cf88;
  font-size: 13px;
  font-weight: 700;
  padding: 9px 10px;
}

.error-text {
  margin: 0;
  color: #ff8f8f;
  font-size: 13px;
  font-weight: 700;
}

.quiet {
  margin: 14px 0 0;
}
</style>
