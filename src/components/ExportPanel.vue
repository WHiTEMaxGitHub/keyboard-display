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
    <h2 class="m-0 mb-4 text-lg leading-6 tracking-normal">Export</h2>
    <BaseFieldRow label="Transparent overlay">WebM</BaseFieldRow>
    <BaseFieldRow label="Compatible video">MP4</BaseFieldRow>
    <BaseToggleRow :checked="renderMarkers" @change="emit('update-render-markers', $event)">
      Render sync markers
    </BaseToggleRow>
    <section class="grid gap-2 mt-4">
      <label for="export-filename-template" class="text-text-muted text-[13px] font-extrabold">Filename template</label>
      <input
        id="export-filename-template"
        class="box-border w-full border border-border-control rounded-radius-md bg-[#10141a] text-text-primary font-inherit text-[13px] px-2.5 py-2 focus:outline-none focus:border-accent-focus-border"
        type="text"
        :value="filenameTemplateDraft"
        @input="updateFilenameTemplateDraft"
        @blur="commitFilenameTemplate"
        @keydown.enter="commitFilenameTemplate"
      />
    </section>
    <section class="grid gap-2.5 mt-4 border border-border-default rounded-radius-lg bg-[#151a20] p-3.5">
      <div class="flex items-center justify-between gap-3 mb-2">
        <h3 class="m-0 text-text-body text-base leading-[22px] tracking-normal">Overlay video</h3>
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
      <div class="flex flex-wrap gap-2">
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
      <div v-if="exportInProgress || exportProgress.totalFrames > 0" class="grid gap-2">
        <div class="flex items-center justify-between gap-3 text-text-muted text-[13px] font-extrabold">
          <span>Rendering frames</span>
          <strong class="text-text-body">{{ exportProgress.renderedFrames }} / {{ exportProgress.totalFrames }}</strong>
        </div>
        <div class="h-2 overflow-hidden rounded-full bg-[#0d1117]">
          <div class="h-full rounded-full bg-accent transition-[width] duration-[120ms] ease" :style="{ width: `${exportProgressPercent}%` }"></div>
        </div>
      </div>
      <p v-if="exportStatus" class="notice-text">{{ exportStatus }}</p>
    </section>
    <div class="grid gap-2.5 mt-4 border border-border-default rounded-radius-lg bg-[#151a20] p-3.5">
      <div class="flex items-center justify-between gap-3 mb-2">
        <h3 class="m-0 text-text-body text-base leading-[22px] tracking-normal">Video exporter</h3>
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
      <div class="flex flex-wrap gap-2">
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
      <div v-if="exporterStatus" class="grid gap-2">
        <div class="grid grid-cols-[120px_110px_minmax(0,1fr)] gap-2.5 items-center border border-border-default rounded-radius-md bg-[#10141a] px-2.5 py-2">
          <span class="text-text-muted text-[13px] font-extrabold">App-managed</span>
          <strong class="min-w-0 overflow-wrap-anywhere">{{ exporterStatus.appManaged.available ? "Installed" : "Not installed" }}</strong>
          <code class="min-w-0 overflow-wrap-anywhere text-text-secondary font-mono text-xs">{{ exporterStatus.appManaged.path }}</code>
        </div>
        <div class="grid grid-cols-[120px_110px_minmax(0,1fr)] gap-2.5 items-center border border-border-default rounded-radius-md bg-[#10141a] px-2.5 py-2">
          <span class="text-text-muted text-[13px] font-extrabold">User-selected</span>
          <strong class="min-w-0 overflow-wrap-anywhere">{{ exporterStatus.userSelected?.available ? "Available" : "Not selected" }}</strong>
          <code class="min-w-0 overflow-wrap-anywhere text-text-secondary font-mono text-xs">{{ exporterStatus.userSelected?.path ?? "None" }}</code>
        </div>
        <div class="grid grid-cols-[120px_110px_minmax(0,1fr)] gap-2.5 items-center border border-border-default rounded-radius-md bg-[#10141a] px-2.5 py-2">
          <span class="text-text-muted text-[13px] font-extrabold">PATH</span>
          <strong class="min-w-0 overflow-wrap-anywhere">{{ exporterStatus.path.available ? "Available" : "Not found" }}</strong>
          <code class="min-w-0 overflow-wrap-anywhere text-text-secondary font-mono text-xs">{{ exporterStatus.path.version ?? "ffmpeg" }}</code>
        </div>
      </div>
      <p v-if="exporterError" class="error">{{ exporterError }}</p>
    </div>
    <p class="notice">
      Video is generated from the input timeline, so size and format can be
      tuned after recording.
    </p>
  </BasePanel>
</template>

<style scoped>
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
</style>