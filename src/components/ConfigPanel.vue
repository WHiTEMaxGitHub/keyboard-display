<script setup lang="ts">
import { computed, provide, ref } from "vue";
import type { AppConfig, ExportConfig, OverlayStyle } from "../domain/defaultConfig";
import type { RecentProfile } from "../domain/appConfig";
import type { AppNotification, NotificationTone } from "../composables/useNotifications";
import type { RecordingHotkeyConfig, RecordingHotkeyMode } from "../domain/recordingHotkeys";
import type { VideoExporterConfig } from "../domain/videoExporter";
import type { RecordingInspection } from "../types/recording";
import ConfigSidebar from "./ConfigSidebar.vue";
import ConfigTopbar from "./ConfigTopbar.vue";
import NotificationStack from "./NotificationStack.vue";
import OverviewPage from "./pages/OverviewPage.vue";
import LayoutPage from "./pages/LayoutPage.vue";
import AppearancePage from "./pages/AppearancePage.vue";
import WindowPage from "./pages/WindowPage.vue";
import RecordingPage from "./pages/RecordingPage.vue";
import ExportPage from "./pages/ExportPage.vue";

type ConfigPage = "overview" | "layout" | "appearance" | "window" | "recording" | "export";
type RecordingSubPage = "control" | "files";

const props = defineProps<{
  config: AppConfig;
  activeKeys: Set<string>;
  keyIdLabels: AppConfig["keyIdLabels"];
  overlayVisible: boolean;
  profileName: string;
  profileChanged: boolean;
  recentProfiles: RecentProfile[];
  recordingDirectory: string;
  defaultRecordingDirectory: string;
  recordingBrowserDirectory: string;
  silentRecording: boolean;
  isRecording: boolean;
  recordingCountdown: number;
  lastRecordingPath: string;
  recordingStatusMessage: string;
  currentRecordingPath: string;
  recordingInspection: RecordingInspection | null;
  recordingInspectionError: string;
  overlayPosition: string;
  overlayAdjusting: boolean;
  recordingHotkeys: RecordingHotkeyConfig;
  hotkeyCaptureTarget: "start" | "stop" | "sync" | null;
  videoExporterConfig: VideoExporterConfig;
  notifications: AppNotification[];
}>();

const emit = defineEmits<{
  "preview-overlay-style": [style: OverlayStyle];
  "update-key-id-labels": [labels: AppConfig["keyIdLabels"]];
  "update-overlay-style": [style: OverlayStyle];
  "update-overlay-rows": [rows: AppConfig["rows"]];
  "update-overlay-visible": [visible: boolean];
  "load-config": [];
  "refresh-pov": [];
  "load-recent-profile": [path: string];
  "export-and-apply-config": [];
  "overwrite-and-apply-config": [];
  "choose-recording-directory": [];
  "choose-recording-browser-directory": [];
  "update-silent-recording": [value: boolean];
  "update-recording-config": [recording: AppConfig["recording"]];
  "update-export-config": [exportConfig: ExportConfig];
  "update-video-exporter-config": [exporterConfig: VideoExporterConfig];
  notify: [tone: NotificationTone, message: string];
  "dismiss-notification": [id: number];
  "start-recording": [];
  "stop-recording": [];
  "add-sync-marker": [];
  "inspect-recording-file": [];
  "inspect-recording-path": [path: string];
  "clear-recording-inspection": [];
  "update-recording-hotkey-mode": [mode: RecordingHotkeyMode];
  "begin-hotkey-capture": [target: "start" | "stop" | "sync"];
  "start-overlay-adjust": [];
  "save-overlay-adjust": [];
  "cancel-overlay-adjust": [];
  "move-overlay": [
    position: "top-left" | "top-right" | "bottom-left" | "bottom-right" | "custom",
  ];
}>();

const activePage = ref<ConfigPage>("overview");
const recordingSubPage = ref<RecordingSubPage>("control");
const recentColors = ref<string[]>([]);
const sidebarCollapsed = ref(false);

function relay(event: string, ...args: unknown[]) {
  (emit as any)(event, ...args);
}

function selectActivePage(page: ConfigPage) {
  if (activePage.value !== page) {
    emit("clear-recording-inspection");
  }
  activePage.value = page;
}

function selectRecordingSubPage(page: RecordingSubPage) {
  if (recordingSubPage.value !== page) {
    emit("clear-recording-inspection");
  }
  recordingSubPage.value = page;
}

provide("config", props.config);
provide("activeKeys", props.activeKeys);
provide("keyIdLabels", props.keyIdLabels);
provide("overlayVisible", props.overlayVisible);
provide("profileName", props.profileName);
provide("profileChanged", props.profileChanged);
provide("recentProfiles", props.recentProfiles);
provide("recordingDirectory", props.recordingDirectory);
provide("defaultRecordingDirectory", props.defaultRecordingDirectory);
provide("recordingBrowserDirectory", props.recordingBrowserDirectory);
provide("silentRecording", props.silentRecording);
provide("isRecording", props.isRecording);
provide("recordingCountdown", props.recordingCountdown);
provide("lastRecordingPath", props.lastRecordingPath);
provide("recordingStatusMessage", props.recordingStatusMessage);
provide("currentRecordingPath", props.currentRecordingPath);
provide("recordingInspection", props.recordingInspection);
provide("recordingInspectionError", props.recordingInspectionError);
provide("overlayPosition", props.overlayPosition);
provide("overlayAdjusting", props.overlayAdjusting);
provide("recordingHotkeys", props.recordingHotkeys);
provide("hotkeyCaptureTarget", props.hotkeyCaptureTarget);
provide("videoExporterConfig", props.videoExporterConfig);
provide("recentColors", recentColors);
provide("emit", relay);

const pageComponent = computed(() => {
  const map: Record<ConfigPage, any> = {
    overview: OverviewPage,
    layout: LayoutPage,
    appearance: AppearancePage,
    window: WindowPage,
    recording: RecordingPage,
    export: ExportPage,
  };
  return map[activePage.value];
});
</script>

<template>
  <main :class="['config-shell', { 'sidebar-collapsed': sidebarCollapsed }]">
    <NotificationStack
      :notifications="notifications"
      @dismiss="emit('dismiss-notification', $event)"
    />
    <ConfigSidebar
      :active-page="activePage"
      :recording-sub-page="recordingSubPage"
      :collapsed="sidebarCollapsed"
      @toggle-collapse="sidebarCollapsed = !sidebarCollapsed"
      @update-active-page="selectActivePage"
      @update-recording-sub-page="selectRecordingSubPage"
    />

    <section class="workspace">
      <ConfigTopbar
        @load-config="emit('load-config')"
        @export-and-apply-config="emit('export-and-apply-config')"
        @overwrite-and-apply-config="emit('overwrite-and-apply-config')"
      />

      <div :key="`${activePage}-${recordingSubPage}`" class="page-container">
        <component :is="pageComponent" />
      </div>
    </section>
  </main>
</template>

<style scoped>
.config-shell {
  display: grid;
  height: 100vh;
  grid-template-columns: 248px minmax(0, 1fr);
  overflow: hidden;
  background: var(--color-surface-base);
  color: var(--color-text-primary);
}

.config-shell.sidebar-collapsed {
  grid-template-columns: 72px minmax(0, 1fr);
}

.workspace {
  height: 100vh;
  min-width: 0;
  overflow-y: auto;
  padding: 24px;
}

.page-container {
  display: grid;
  gap: 16px;
}

@media (max-width: 920px) {
  .config-shell {
    grid-template-columns: 72px minmax(0, 1fr);
  }
}
</style>