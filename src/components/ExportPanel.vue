<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { tauriApi } from "../api/tauri";
import type { AppConfig } from "../domain/defaultConfig";
import { formatExportFileName } from "../domain/exportFilename";
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
  config: AppConfig;
  profileName: string;
  renderMarkers: boolean;
  videoExporterConfig: VideoExporterConfig;
  installingAppManagedExporter: boolean;
  uninstallingAppManagedExporter: boolean;
}>();

const emit = defineEmits<{
  "update-render-markers": [event: Event];
  "update-export-config": [exportConfig: AppConfig["export"]];
  "update-video-exporter-config": [config: VideoExporterConfig];
  "install-app-managed-exporter": [];
  "uninstall-app-managed-exporter": [];
}>();

const exporterStatus = ref<VideoExporterStatus | null>(null);
const exporterError = ref("");
const exporterChecking = ref(false);
const inputRecordingPath = ref("");
const filenameTemplateDraft = ref(props.config.export.filenameTemplate);
const exportStatus = ref("");
const exportInProgress = ref(false);
const exportProgress = ref({ renderedFrames: 0, totalFrames: 0 });
let unlistenExportProgress: UnlistenFn | undefined;

const exportReady = computed(() =>
  Boolean(exporterStatus.value?.resolved && inputRecordingPath.value && props.videoExporterConfig.outputDirectory),
);

const outputVideoPath = computed(() => {
  if (!props.videoExporterConfig.outputDirectory || !inputRecordingPath.value) {
    return "";
  }

  return joinPath(
    props.videoExporterConfig.outputDirectory,
    defaultOutputFileName(inputRecordingPath.value),
  );
});

const resolvedExporterLabel = computed(() =>
  describeVideoExporter(exporterStatus.value?.resolved ?? null),
);

const exportProgressPercent = computed(() => {
  if (exportProgress.value.totalFrames <= 0) {
    return 0;
  }

  return Math.min(
    100,
    Math.round((exportProgress.value.renderedFrames / exportProgress.value.totalFrames) * 100),
  );
});

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

onMounted(async () => {
  void refreshExporterStatus();
  unlistenExportProgress = await listen<{
    renderedFrames: number;
    totalFrames: number;
  }>("export-progress", (event) => {
    exportProgress.value = event.payload;
  });
});

onUnmounted(() => {
  unlistenExportProgress?.();
});

watch(
  () => props.config.export.filenameTemplate,
  (filenameTemplate) => {
    filenameTemplateDraft.value = filenameTemplate;
  },
);

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

function updateFilenameTemplateDraft(event: Event) {
  filenameTemplateDraft.value = (event.target as HTMLInputElement).value;
}

function commitFilenameTemplate() {
  const filenameTemplate = filenameTemplateDraft.value.trim() ||
    "${profileSlug}-${recordingName}-overlay";
  filenameTemplateDraft.value = filenameTemplate;
  emit("update-export-config", {
    ...props.config.export,
    filenameTemplate,
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

async function chooseOutputDirectory() {
  const selectedPath = await open({
    title: "Choose export folder",
    directory: true,
    multiple: false,
  });

  if (typeof selectedPath === "string") {
    emit("update-video-exporter-config", normalizeVideoExporterConfig({
      ...props.videoExporterConfig,
      outputDirectory: selectedPath,
    }));
    exportStatus.value = "";
  }
}

async function exportOverlayVideo() {
  if (exportInProgress.value) {
    return;
  }

  if (!exportReady.value) {
    exportStatus.value = "Choose a recording, export folder, and available video exporter first.";
    return;
  }

  const exporterPath = exporterStatus.value?.resolved?.path;
  if (!exporterPath) {
    exportStatus.value = "Choose an available video exporter first.";
    return;
  }

  exportInProgress.value = true;
  exportProgress.value = { renderedFrames: 0, totalFrames: 0 };
  exportStatus.value = "Exporting overlay video...";

  try {
    const result = await tauriApi.exportOverlayVideo(
      inputRecordingPath.value,
      outputVideoPath.value,
      exporterPath,
      {
        layout: props.config.layout,
        rows: props.config.rows,
        style: props.config.style,
        export: props.config.export,
        recording: props.config.recording,
      },
    );
    exportStatus.value =
      `Exported ${result.frameCount} frames at ${result.width}x${result.height} @ ${result.fps}fps.`;
  } catch (error) {
    exportStatus.value = `Export failed: ${String(error)}`;
  } finally {
    exportInProgress.value = false;
  }
}

async function openOutputDirectory() {
  if (!props.videoExporterConfig.outputDirectory) {
    exportStatus.value = "Choose an export folder first.";
    return;
  }

  try {
    await openPath(props.videoExporterConfig.outputDirectory);
  } catch (error) {
    exportStatus.value = `Failed to open export folder: ${String(error)}`;
  }
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

function defaultOutputFileName(recordingPath: string) {
  return formatExportFileName({
    template: props.config.export.filenameTemplate,
    recordingPath,
    profileName: props.profileName,
    fps: props.config.recording.customFpsEnabled
      ? props.config.recording.customFps
      : props.config.recording.defaultFps,
  });
}

function joinPath(directory: string, fileName: string) {
  const separator = directory.includes("\\") ? "\\" : "/";
  return `${directory.replace(/[\\/]+$/, "")}${separator}${fileName}`;
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
    <section class="template-panel">
      <label for="export-filename-template">Filename template</label>
      <input
        id="export-filename-template"
        class="template-input"
        type="text"
        :value="filenameTemplateDraft"
        @input="updateFilenameTemplateDraft"
        @blur="commitFilenameTemplate"
        @keydown.enter="commitFilenameTemplate"
      />
    </section>
    <section class="export-job-panel">
      <div class="section-header">
        <h3>Overlay video</h3>
        <BaseButton
          variant="primary"
          :disabled="!exportReady || exportInProgress"
          @click="exportOverlayVideo"
        >
          {{ exportInProgress ? "Exporting..." : "Export overlay video" }}
        </BaseButton>
      </div>
      <BaseFieldRow label="Recording">
        {{ inputRecordingPath || "No .kbdrec selected" }}
      </BaseFieldRow>
      <BaseFieldRow label="Output">
        {{ outputVideoPath || "No export folder selected" }}
      </BaseFieldRow>
      <div class="exporter-actions">
        <BaseButton @click="chooseInputRecording">
          Choose .kbdrec
        </BaseButton>
        <BaseButton @click="chooseOutputDirectory">
          Choose output folder
        </BaseButton>
        <BaseButton
          :disabled="!videoExporterConfig.outputDirectory"
          @click="openOutputDirectory"
        >
          Open output folder
        </BaseButton>
      </div>
      <div v-if="exportInProgress || exportProgress.totalFrames > 0" class="export-progress">
        <div class="progress-copy">
          <span>Rendering frames</span>
          <strong>{{ exportProgress.renderedFrames }} / {{ exportProgress.totalFrames }}</strong>
        </div>
        <div class="progress-track" aria-hidden="true">
          <div class="progress-fill" :style="{ width: `${exportProgressPercent}%` }"></div>
        </div>
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

.template-panel {
  display: grid;
  gap: 8px;
}

.template-panel label {
  color: #9ca7b4;
  font-size: 13px;
  font-weight: 800;
}

.export-progress {
  display: grid;
  gap: 8px;
}

.progress-copy {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  color: #9ca7b4;
  font-size: 13px;
  font-weight: 800;
}

.progress-copy strong {
  color: #dfe5ec;
}

.progress-track {
  height: 8px;
  overflow: hidden;
  border-radius: 999px;
  background: #0d1117;
}

.progress-fill {
  height: 100%;
  border-radius: inherit;
  background: #25d366;
  transition: width 120ms ease;
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

.template-input {
  box-sizing: border-box;
  width: 100%;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #10141a;
  color: #eef2f6;
  font: inherit;
  font-size: 13px;
  padding: 8px 10px;
}

.template-input:focus {
  outline: none;
  border-color: rgba(37, 211, 102, 0.55);
}

.quiet {
  margin: 14px 0 0;
}
</style>
