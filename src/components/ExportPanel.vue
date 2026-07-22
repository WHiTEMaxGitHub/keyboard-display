<script setup lang="ts">
import { open, save } from "@tauri-apps/plugin-dialog";
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
import BaseFieldRow from "./BaseFieldRow.vue";
import BaseToggleRow from "./BaseToggleRow.vue";

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
const inputRecordingPath = ref("");
const outputVideoPath = ref("");
const exportStatus = ref("");

const exportReady = computed(() =>
  Boolean(exporterStatus.value?.resolved && inputRecordingPath.value && outputVideoPath.value),
);

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

async function chooseInputRecording() {
  const selectedPath = await open({
    title: "Choose keyboard recording",
    filters: [{ name: "Keyboard recording", extensions: ["kbdrec"] }],
    multiple: false,
  });

  if (typeof selectedPath === "string") {
    inputRecordingPath.value = selectedPath;
    exportStatus.value = "";
  }
}

async function chooseOutputVideo() {
  const selectedPath = await save({
    title: "Save overlay video",
    defaultPath: "keyboard-overlay.webm",
    filters: [{ name: "WebM", extensions: ["webm"] }],
  });

  if (typeof selectedPath === "string") {
    outputVideoPath.value = selectedPath;
    exportStatus.value = "";
  }
}

function exportOverlayVideo() {
  if (!exportReady.value) {
    exportStatus.value = "Choose a recording, output path, and available video exporter first.";
    return;
  }

  exportStatus.value = "Export backend is ready to connect in the next step.";
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
    <BaseFieldRow label="Transparent overlay">WebM</BaseFieldRow>
    <BaseFieldRow label="Compatible video">MP4</BaseFieldRow>
    <BaseToggleRow :checked="renderMarkers" @change="emit('update-render-markers', $event)">
      Render sync markers
    </BaseToggleRow>
    <section class="export-job-panel">
      <div class="section-header">
        <h3>Overlay video</h3>
        <BaseButton
          variant="primary"
          :disabled="!exportReady"
          @click="exportOverlayVideo"
        >
          Export overlay video
        </BaseButton>
      </div>
      <BaseFieldRow label="Recording">
        {{ inputRecordingPath || "No .kbdrec selected" }}
      </BaseFieldRow>
      <BaseFieldRow label="Output">
        {{ outputVideoPath || "No .webm output selected" }}
      </BaseFieldRow>
      <div class="exporter-actions">
        <BaseButton @click="chooseInputRecording">
          Choose .kbdrec
        </BaseButton>
        <BaseButton @click="chooseOutputVideo">
          Choose output .webm
        </BaseButton>
      </div>
      <p v-if="exportStatus" class="notice-text">{{ exportStatus }}</p>
    </section>
    <div class="exporter-panel">
      <div class="section-header">
        <h3>Video exporter</h3>
        <BaseButton :disabled="exporterChecking" @click="refreshExporterStatus">
          {{ exporterChecking ? "Checking..." : "Check again" }}
        </BaseButton>
      </div>
      <BaseFieldRow label="Status">{{ resolvedExporterLabel }}</BaseFieldRow>
      <BaseFieldRow v-if="exporterStatus?.resolved" label="Using">
        {{ exporterStatus.resolved.path }}
      </BaseFieldRow>
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

.quiet {
  color: #9ca7b4;
}

.export-job-panel,
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
