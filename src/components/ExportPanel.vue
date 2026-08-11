<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { tauriApi } from "../api/tauri";
import type { AppConfig } from "../domain/defaultConfig";
import { formatExportFileName } from "../domain/exportFilename";
import {
  normalizeVideoExporterConfig,
  type VideoExporterCandidate,
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
const { t } = useI18n();
const exporterError = ref("");
const exporterChecking = ref(false);
const inputRecordingPath = ref("");
const filenameTemplateDraft = ref(props.config.export.filenameTemplate);
const fontPathDraft = ref(props.config.export.fontPath);
const renderThreadsDraft = ref(props.config.export.renderThreads);
const exportStatus = ref("");
const exportInProgress = ref(false);
const defaultExportVideoDirectory = ref("");
const renderThreadsNotice = ref("");
const exportProgress = ref({
  renderedFrames: 0,
  totalFrames: 0,
  currentFrame: 0,
  activeKeyIds: [] as string[],
});
let unlistenExportProgress: UnlistenFn | undefined;

const effectiveOutputDirectory = computed(() =>
  props.videoExporterConfig.outputDirectory || defaultExportVideoDirectory.value,
);
const detectedCpuCores = computed(() => navigator.hardwareConcurrency || 4);
const maxRenderThreads = computed(() => Math.max(1, detectedCpuCores.value * 4));

const exportReady = computed(() =>
  Boolean(exporterStatus.value?.resolved && inputRecordingPath.value && effectiveOutputDirectory.value),
);

const outputVideoPath = computed(() => {
  if (!effectiveOutputDirectory.value || !inputRecordingPath.value) {
    return "";
  }

  return joinPath(
    effectiveOutputDirectory.value,
    defaultOutputFileName(inputRecordingPath.value),
  );
});

const resolvedExporterLabel = computed(() => describeExporter(exporterStatus.value?.resolved ?? null));

const exportProgressPercent = computed(() => {
  if (exportProgress.value.totalFrames <= 0) {
    return 0;
  }

  return Math.min(
    100,
    Math.round((exportProgress.value.renderedFrames / exportProgress.value.totalFrames) * 100),
  );
});

const currentFrameKeyStatus = computed(() => {
  if (exportProgress.value.totalFrames <= 0) {
    return t("export.overlayVideo.waitingForFrames");
  }

  const keys = exportProgress.value.activeKeyIds;
  if (!keys.length) {
    return t("export.overlayVideo.frameNoKeys", { frame: exportProgress.value.currentFrame });
  }

  return t("export.overlayVideo.frameWithKeys", {
    frame: exportProgress.value.currentFrame,
    keys: keys.join(" + "),
  });
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
  void resolveDefaultExportVideoDirectory();
  unlistenExportProgress = await listen<{
    renderedFrames: number;
    totalFrames: number;
    currentFrame: number;
    activeKeyIds: string[];
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

watch(
  () => props.config.export.fontPath,
  (fontPath) => {
    fontPathDraft.value = fontPath;
  },
);

watch(
  () => props.config.export.renderThreads,
  (renderThreads) => {
    renderThreadsDraft.value = renderThreads;
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
    title: t("export.dialog.chooseFfmpeg"),
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
    title: t("export.dialog.chooseRecording"),
    filters: [{ name: t("export.dialog.recordingFilter"), extensions: ["kbdrec"] }],
    multiple: false,
  });

  if (typeof selectedPath === "string") {
    inputRecordingPath.value = selectedPath;
    exportStatus.value = "";
  }
}

async function chooseFontFile() {
  const selectedPath = await open({
    title: t("export.dialog.chooseFont"),
    filters: [
      { name: t("export.dialog.fontFilter"), extensions: ["ttf", "otf", "ttc"] },
    ],
    multiple: false,
  });

  if (typeof selectedPath === "string") {
    try {
      const copiedPath = await tauriApi.copyFontFile(selectedPath);
      fontPathDraft.value = copiedPath;
      emit("update-export-config", {
        ...props.config.export,
        fontPath: copiedPath,
      });
    } catch (error) {
      exportStatus.value = t("export.status.copyFontFailed", { error: String(error) });
    }
  }
}

function clearFontFile() {
  fontPathDraft.value = null;
  emit("update-export-config", {
    ...props.config.export,
    fontPath: null,
  });
}

function onRenderThreadsInput(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  renderThreadsDraft.value = value === "" ? null : Math.max(-1, parseInt(value, 10) || 0);
  renderThreadsNotice.value = "";
}

function commitRenderThreads() {
  let renderThreads = renderThreadsDraft.value === null
    ? null
    : Math.max(-1, Math.round(renderThreadsDraft.value));

  if (renderThreads !== null && renderThreads > maxRenderThreads.value) {
    renderThreadsNotice.value = t("export.overlayVideo.renderThreadsCapped", {
      max: maxRenderThreads.value,
    });
    renderThreads = maxRenderThreads.value;
  }

  renderThreadsDraft.value = renderThreads;
  emit("update-export-config", {
    ...props.config.export,
    renderThreads,
  });
}

async function chooseOutputDirectory() {
  const selectedPath = await open({
    title: t("export.dialog.chooseFolder"),
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

async function resolveDefaultExportVideoDirectory() {
  if (!defaultExportVideoDirectory.value) {
    defaultExportVideoDirectory.value = await tauriApi.defaultExportVideoDir();
  }

  return defaultExportVideoDirectory.value;
}

async function resolveOutputDirectory() {
  if (props.videoExporterConfig.outputDirectory) {
    return props.videoExporterConfig.outputDirectory;
  }

  const outputDirectory = await resolveDefaultExportVideoDirectory();
  emit("update-video-exporter-config", normalizeVideoExporterConfig({
    ...props.videoExporterConfig,
    outputDirectory,
  }));
  return outputDirectory;
}

async function exportOverlayVideo() {
  if (exportInProgress.value) {
    return;
  }

  if (!exportReady.value) {
    exportStatus.value = t("export.status.missingExportRequirements");
    return;
  }

  const exporterPath = exporterStatus.value?.resolved?.path;
  if (!exporterPath) {
    exportStatus.value = t("export.status.missingExporter");
    return;
  }

  const outputDirectory = await resolveOutputDirectory();
  const resolvedOutputVideoPath = joinPath(
    outputDirectory,
    defaultOutputFileName(inputRecordingPath.value),
  );

  exportInProgress.value = true;
  exportProgress.value = {
    renderedFrames: 0,
    totalFrames: 0,
    currentFrame: 0,
    activeKeyIds: [],
  };
  exportStatus.value = t("export.status.exporting");

  try {
      await tauriApi.exportOverlayVideo(
      inputRecordingPath.value,
      resolvedOutputVideoPath,
      exporterPath,
      {
        layout: props.config.layout,
        rows: props.config.rows,
        style: props.config.style,
        export: props.config.export,
        recording: props.config.recording,
      },
    );
    exportStatus.value = "";
  } catch (error) {
    exportStatus.value = t("export.status.exportFailed", { error: String(error) });
  } finally {
    exportInProgress.value = false;
  }
}

async function openOutputDirectory() {
  const outputDirectory = await resolveOutputDirectory();
  if (!outputDirectory) {
    exportStatus.value = t("export.status.missingOutputFolder");
    return;
  }

  try {
    await tauriApi.openDirectory(outputDirectory);
  } catch (error) {
    exportStatus.value = t("export.status.openFolderFailed", { error: String(error) });
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

function describeExporter(candidate: VideoExporterCandidate | null) {
  if (!candidate) {
    return t("export.exporter.notInstalled");
  }

  switch (candidate.source) {
    case "app-managed":
      return t("export.exporter.appManagedExporter");
    case "user-selected":
      return t("export.exporter.userSelectedFfmpeg");
    case "path":
      return t("export.exporter.systemFfmpeg");
  }
}
</script>

<template>
  <BasePanel wide>
    <h2 class="m-0 mb-4 text-lg leading-6 tracking-normal">{{ t("export.title") }}</h2>
    <BaseFieldRow :label="t('export.transparentOverlay')">WebM</BaseFieldRow>
    <BaseFieldRow :label="t('export.compatibleVideo')">MP4</BaseFieldRow>
    <BaseToggleRow :checked="renderMarkers" @change="emit('update-render-markers', $event)">
      {{ t("export.renderMarkers") }}
    </BaseToggleRow>
    <section class="grid gap-2 mt-4">
      <label for="export-filename-template" class="text-text-muted text-[13px] font-extrabold">{{ t("export.filenameTemplate") }}</label>
      <input
        id="export-filename-template"
        class="box-border w-full border border-border-control rounded-md bg-surface-control text-text-primary font-inherit text-[13px] px-2.5 py-2 focus:outline-none focus:border-accent-focus-border"
        type="text"
        :value="filenameTemplateDraft"
        @input="updateFilenameTemplateDraft"
        @blur="commitFilenameTemplate"
        @keydown.enter="commitFilenameTemplate"
      />
    </section>
    <section class="grid gap-2 mt-4">
      <label class="text-text-muted text-[13px] font-extrabold">{{ t("export.font") }}</label>
      <div class="flex items-center gap-2">
        <span class="text-text-secondary text-[13px] min-w-0 overflow-hidden text-ellipsis whitespace-nowrap flex-1">
          {{ fontPathDraft || t("export.systemDefault") }}
        </span>
        <BaseButton @click="chooseFontFile">{{ t("export.chooseFont") }}</BaseButton>
        <BaseButton v-if="fontPathDraft" @click="clearFontFile">{{ t("export.reset") }}</BaseButton>
      </div>
    </section>
    <section class="grid gap-2.5 mt-4 border border-border-control rounded-lg bg-surface-control p-3.5">
      <div class="flex items-center justify-between gap-3 mb-2">
        <h3 class="m-0 text-text-body text-base leading-[22px] tracking-normal">{{ t("export.overlayVideo.title") }}</h3>
        <BaseButton
          variant="primary"
          :disabled="!exportReady || exportInProgress"
          @click="exportOverlayVideo"
        >
          {{ exportInProgress ? t("export.overlayVideo.exportingButton") : t("export.overlayVideo.exportButton") }}
        </BaseButton>
      </div>
      <BaseFieldRow :label="t('export.overlayVideo.recording')">
        {{ inputRecordingPath || t("export.overlayVideo.noRecording") }}
      </BaseFieldRow>
      <BaseFieldRow :label="t('export.overlayVideo.output')">
        {{ outputVideoPath || t("export.overlayVideo.noOutputFolder") }}
      </BaseFieldRow>
      <div class="flex flex-wrap gap-2">
        <BaseButton @click="chooseInputRecording">
          {{ t("export.overlayVideo.chooseRecording") }}
        </BaseButton>
        <BaseButton @click="chooseOutputDirectory">
          {{ t("export.overlayVideo.chooseOutputFolder") }}
        </BaseButton>
        <BaseButton
          :disabled="!effectiveOutputDirectory"
          @click="openOutputDirectory"
        >
          {{ t("export.overlayVideo.openOutputFolder") }}
        </BaseButton>
      </div>
      <section class="grid gap-2 mt-3">
        <label class="text-text-muted text-[13px] font-extrabold">{{ t("export.overlayVideo.renderThreads") }}</label>
        <div class="flex items-center gap-2">
          <input
            class="box-border w-20 border border-border-control rounded-md bg-surface-control text-text-primary font-inherit text-[13px] px-2.5 py-2 focus:outline-none focus:border-accent-focus-border"
            type="number"
            min="-1"
            :max="maxRenderThreads"
            :value="renderThreadsDraft ?? ''"
            :placeholder="t('export.overlayVideo.autoPlaceholder')"
            @input="onRenderThreadsInput"
            @blur="commitRenderThreads"
            @keydown.enter="commitRenderThreads"
          />
          <span class="text-text-secondary text-[13px]">{{ t("export.overlayVideo.autoThreads") }}</span>
        </div>
        <p v-if="renderThreadsNotice" class="notice-text">{{ renderThreadsNotice }}</p>
      </section>
      <div v-if="exportInProgress" class="export-progress-stack">
        <div class="flex items-center justify-between gap-3 text-text-muted text-[13px] font-extrabold">
          <span>{{ t("export.overlayVideo.renderingFrames") }}</span>
          <strong class="text-text-body">{{ exportProgress.renderedFrames }} / {{ exportProgress.totalFrames }}</strong>
        </div>
        <div class="glass-progress" :aria-label="t('export.overlayVideo.progress')">
          <div class="glass-progress-fill" :style="{ width: `${exportProgressPercent}%` }"></div>
          <div class="glass-progress-shine" aria-hidden="true"></div>
        </div>
        <p class="frame-status">{{ currentFrameKeyStatus }}</p>
      </div>
      <p v-if="exportStatus" class="notice-text">{{ exportStatus }}</p>
    </section>
    <div class="grid gap-2.5 mt-4 border border-border-control rounded-lg bg-surface-control p-3.5">
      <div class="flex items-center justify-between gap-3 mb-2">
        <h3 class="m-0 text-text-body text-base leading-[22px] tracking-normal">{{ t("export.exporter.title") }}</h3>
        <BaseButton :disabled="exporterChecking" @click="refreshExporterStatus">
          {{ exporterChecking ? t("export.exporter.checking") : t("export.exporter.checkAgain") }}
        </BaseButton>
      </div>
      <BaseFieldRow :label="t('export.exporter.status')">{{ resolvedExporterLabel }}</BaseFieldRow>
      <BaseFieldRow v-if="exporterStatus?.resolved" :label="t('export.exporter.using')">
        {{ exporterStatus.resolved.path }}
      </BaseFieldRow>
      <p v-else class="notice-text">
        {{ t("export.exporter.description") }}
      </p>
      <div class="flex flex-wrap gap-2">
        <BaseButton @click="chooseFfmpegPath">
          {{ t("export.exporter.choosePath") }}
        </BaseButton>
        <BaseButton
          :disabled="!videoExporterConfig.userSelectedPath"
          @click="clearFfmpegPath"
        >
          {{ t("export.exporter.clearPath") }}
        </BaseButton>
        <BaseButton
          v-if="exporterStatus?.appManaged.available"
          variant="danger"
          :disabled="uninstallingAppManagedExporter"
          @click="uninstallAppManagedExporter"
        >
          {{ uninstallingAppManagedExporter ? t("export.exporter.uninstalling") : t("export.exporter.uninstall") }}
        </BaseButton>
        <BaseButton
          v-else
          variant="primary"
          :disabled="installingAppManagedExporter"
          @click="installAppManagedExporter"
        >
          {{ installingAppManagedExporter ? t("export.exporter.installing") : t("export.exporter.install") }}
        </BaseButton>
      </div>
      <div v-if="exporterStatus" class="grid gap-2">
        <div class="grid grid-cols-[120px_110px_minmax(0,1fr)] gap-2.5 items-center border border-border-control rounded-md bg-surface-control px-2.5 py-2">
          <span class="text-text-muted text-[13px] font-extrabold">{{ t("export.exporter.appManaged") }}</span>
          <strong class="min-w-0 overflow-wrap-anywhere">{{ exporterStatus.appManaged.available ? t("export.exporter.installed") : t("export.exporter.notInstalledShort") }}</strong>
          <code class="min-w-0 overflow-wrap-anywhere text-text-secondary font-mono text-xs">{{ exporterStatus.appManaged.path }}</code>
        </div>
        <div class="grid grid-cols-[120px_110px_minmax(0,1fr)] gap-2.5 items-center border border-border-control rounded-md bg-surface-control px-2.5 py-2">
          <span class="text-text-muted text-[13px] font-extrabold">{{ t("export.exporter.userSelected") }}</span>
          <strong class="min-w-0 overflow-wrap-anywhere">{{ exporterStatus.userSelected?.available ? t("export.exporter.available") : t("export.exporter.notSelected") }}</strong>
          <code class="min-w-0 overflow-wrap-anywhere text-text-secondary font-mono text-xs">{{ exporterStatus.userSelected?.path ?? t("export.exporter.none") }}</code>
        </div>
        <div class="grid grid-cols-[120px_110px_minmax(0,1fr)] gap-2.5 items-center border border-border-control rounded-md bg-surface-control px-2.5 py-2">
          <span class="text-text-muted text-[13px] font-extrabold">{{ t("export.exporter.path") }}</span>
          <strong class="min-w-0 overflow-wrap-anywhere">{{ exporterStatus.path.available ? t("export.exporter.available") : t("export.exporter.notFound") }}</strong>
          <code class="min-w-0 overflow-wrap-anywhere text-text-secondary font-mono text-xs">{{ exporterStatus.path.version ?? "ffmpeg" }}</code>
        </div>
      </div>
      <p v-if="exporterError" class="error">{{ exporterError }}</p>
    </div>
    <p class="notice">
      {{ t("export.note") }}
    </p>
  </BasePanel>
</template>

<style scoped>
.export-progress-stack {
  display: grid;
  gap: 8px;
}

.glass-progress {
  position: relative;
  height: 14px;
  overflow: hidden;
  border: 1px solid var(--glass-border);
  border-radius: 999px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.12), rgba(255, 255, 255, 0.035)),
    color-mix(in srgb, var(--color-surface-control) 78%, black 22%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.22),
    inset 0 -8px 16px rgba(0, 0, 0, 0.22),
    0 8px 28px rgba(0, 0, 0, 0.20);
}

.glass-progress-fill {
  position: absolute;
  inset: 2px auto 2px 2px;
  min-width: 8px;
  border-radius: inherit;
  background:
    linear-gradient(90deg, var(--color-accent), var(--color-accent-blue), var(--color-accent-violet));
  box-shadow:
    0 0 18px var(--color-accent-glow),
    inset 0 1px 0 rgba(255, 255, 255, 0.42);
  transition: width 160ms ease-out;
}

.glass-progress-shine {
  position: absolute;
  inset: 2px 8px auto;
  height: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.30);
  opacity: 0.72;
  pointer-events: none;
}

.frame-status {
  min-height: 18px;
  margin: 0;
  overflow: hidden;
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 800;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.notice-text {
  margin: 0;
  border: 1px solid rgba(255, 209, 102, 0.14);
  border-radius: 7px;
  background: rgba(255, 209, 102, 0.06);
  color: #d4c070;
  font-size: 13px;
  font-weight: 700;
  padding: 9px 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.dismiss-btn {
  border: none;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: #d4c070;
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  padding: 2px 8px;
  flex-shrink: 0;
}

.dismiss-btn:hover {
  background: rgba(255, 255, 255, 0.16);
}
</style>
