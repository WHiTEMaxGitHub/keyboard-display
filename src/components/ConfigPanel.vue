<script setup lang="ts">
import { computed, ref } from "vue";
import { tauriApi } from "../api/tauri";
import { normalizeHexColor } from "../domain/colorPicker";
import {
  isKeyBinding,
  type AppConfig,
  type ExportConfig,
  type OverlayStyle,
} from "../domain/defaultConfig";
import type { RecentProfile } from "../domain/appConfig";
import type { AppNotification, NotificationTone } from "../composables/useNotifications";
import type { RecordingHotkeyConfig, RecordingHotkeyMode } from "../domain/recordingHotkeys";
import type { VideoExporterConfig } from "../domain/videoExporter";
import type { RecordingInspection } from "../types/recording";
import PovOverlay from "./PovOverlay.vue";
import RecordingBrowserPanel from "./RecordingBrowserPanel.vue";
import AppearancePanel from "./AppearancePanel.vue";
import BaseFieldRow from "./BaseFieldRow.vue";
import BaseSelect from "./BaseSelect.vue";
import BaseSegmentedControl from "./BaseSegmentedControl.vue";
import BaseToggleRow from "./BaseToggleRow.vue";
import ConfigSidebar from "./ConfigSidebar.vue";
import ConfigTopbar from "./ConfigTopbar.vue";
import ExportPanel from "./ExportPanel.vue";
import LayoutEditor from "./LayoutEditor.vue";
import NotificationStack from "./NotificationStack.vue";
import RecordingPanel from "./RecordingPanel.vue";
import WindowPanel from "./WindowPanel.vue";

type ConfigPage = "overview" | "layout" | "appearance" | "window" | "recording" | "export";
type LayoutSubPage = "summary" | "editor";
type RecordingSubPage = "control" | "files";

const activePage = ref<ConfigPage>("overview");
const layoutSubPage = ref<LayoutSubPage>("summary");
const recordingSubPage = ref<RecordingSubPage>("control");
const recentColors = ref<string[]>([]);
const sidebarCollapsed = ref(false);
const videoExporterInstalling = ref(false);
const videoExporterUninstalling = ref(false);
const layoutSubPageOptions: Array<{ value: LayoutSubPage; label: string }> = [
  { value: "summary", label: "Summary" },
  { value: "editor", label: "Editor" },
];

const props = defineProps<{
  config: AppConfig;
  activeKeys: Set<string>;
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

const layoutRows = computed(() => {
  return props.config.rows.map((items, index) => ({ row: index + 1, items }));
});

const emit = defineEmits<{
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

function rememberColor(color: string) {
  const normalizedColor = normalizeHexColor(color);
  recentColors.value = [
    normalizedColor,
    ...recentColors.value.filter((recentColor) => recentColor !== normalizedColor),
  ].slice(0, 8);
}

function updateAlwaysOnTop(event: Event) {
  const alwaysOnTop = (event.target as HTMLInputElement).checked;
  emit("update-overlay-style", { ...props.config.style, alwaysOnTop });
}

function updateOverlayVisible(event: Event) {
  emit("update-overlay-visible", (event.target as HTMLInputElement).checked);
}

function moveOverlay(
  position: "top-left" | "top-right" | "bottom-left" | "bottom-right" | "custom",
) {
  emit("move-overlay", position);
}

function startOverlayAdjust() {
  emit("start-overlay-adjust");
}

function saveOverlayAdjust() {
  emit("save-overlay-adjust");
}

function cancelOverlayAdjust() {
  emit("cancel-overlay-adjust");
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

function loadConfigFile() {
  emit("load-config");
}

function refreshPov() {
  emit("refresh-pov");
}

function loadRecentProfile(event: Event) {
  const select = event.target as HTMLSelectElement;
  const path = select.value;
  if (path) {
    emit("load-recent-profile", path);
    select.value = "";
  }
}

function exportAndApplyConfig() {
  emit("export-and-apply-config");
}

function overwriteAndApplyConfig() {
  emit("overwrite-and-apply-config");
}

function updateRenderMarkers(event: Event) {
  emit("update-export-config", {
    ...props.config.export,
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
        @load-config="loadConfigFile"
        @export-and-apply-config="exportAndApplyConfig"
        @overwrite-and-apply-config="overwriteAndApplyConfig"
      />

      <div :key="`${activePage}-${recordingSubPage}`" class="page-container">
      <section v-if="activePage === 'overview'" class="page-stack">
        <section class="preview-band" aria-label="Live preview">
          <div class="preview-copy">
            <p>Live Preview</p>
            <h2>{{ profileName }}</h2>
          </div>
          <div class="preview-viewport">
            <PovOverlay
              :layout="config.layout"
              :rows="config.rows"
              :keys="config.keys"
              :active-keys="activeKeys"
              :overlay-style="config.style"
            />
          </div>
        </section>

        <section class="panel-grid">
          <article class="panel">
            <h2>Profile</h2>
            <BaseFieldRow label="Name">{{ profileName }}</BaseFieldRow>
            <BaseFieldRow label="Status">
              {{ profileChanged ? "Unsaved changes" : "Saved" }}
            </BaseFieldRow>
            <BaseFieldRow label="Visible keys">{{ config.keys.length }}</BaseFieldRow>
            <label class="recent-profile-control">
              <span>Recent profiles</span>
              <BaseSelect
                class="select-control"
                :disabled="recentProfiles.length === 0"
                model-value=""
                @change="loadRecentProfile"
              >
                <option value="">
                  {{ recentProfiles.length ? "Choose a profile" : "No recent profiles" }}
                </option>
                <option
                  v-for="profile in recentProfiles"
                  :key="profile.path"
                  :value="profile.path"
                >
                  {{ profile.name }}
                </option>
              </BaseSelect>
            </label>
          </article>

          <article class="panel">
            <h2>Quick controls</h2>
            <BaseToggleRow :checked="overlayVisible" @change="updateOverlayVisible">
              Show POV overlay
            </BaseToggleRow>
            <BaseToggleRow :checked="config.style.alwaysOnTop" @change="updateAlwaysOnTop">
              Always on top
            </BaseToggleRow>
          </article>
        </section>
      </section>

      <section v-else-if="activePage === 'layout'" class="page-stack">
        <article class="panel">
          <h2>Layout</h2>
          <BaseSegmentedControl
            v-model="layoutSubPage"
            :options="layoutSubPageOptions"
            aria-label="Layout view"
          />
          <BaseFieldRow label="Unit size">{{ config.layout.unitPx }}px</BaseFieldRow>
          <BaseFieldRow label="Gap">{{ config.layout.gapUnit }} unit</BaseFieldRow>
          <BaseFieldRow label="Visible keys">{{ config.keys.length }}</BaseFieldRow>
          <div v-if="layoutSubPage === 'summary'" class="layout-line-list">
            <div v-for="line in layoutRows" :key="line.row" class="layout-line">
              <span class="line-label">Line {{ line.row }}:</span>
              <span class="line-keys">
                <span
                  v-for="(item, index) in line.items"
                  :key="`${line.row}-${index}`"
                  :class="['line-key', { 'line-gap': !isKeyBinding(item) }]"
                >
                  {{ isKeyBinding(item) ? item.label : "Gap" }} · {{ item.widthUnit }}u
                </span>
              </span>
            </div>
          </div>
          <LayoutEditor
            v-else
            :rows="config.rows"
            @update-rows="emit('update-overlay-rows', $event)"
          />
        </article>
      </section>

      <section v-else-if="activePage === 'appearance'" class="page-stack">
        <AppearancePanel
          :config="config"
          :recent-colors="recentColors"
          @update-overlay-style="emit('update-overlay-style', $event)"
          @remember-color="rememberColor"
          @refresh-pov="refreshPov"
        />
      </section>

      <section v-else-if="activePage === 'window'" class="page-stack">
        <WindowPanel
          :overlay-position="overlayPosition"
          :overlay-visible="overlayVisible"
          :always-on-top="config.style.alwaysOnTop"
          :overlay-adjusting="overlayAdjusting"
          @update-overlay-visible="updateOverlayVisible"
          @update-always-on-top="updateAlwaysOnTop"
          @start-overlay-adjust="startOverlayAdjust"
          @save-overlay-adjust="saveOverlayAdjust"
          @cancel-overlay-adjust="cancelOverlayAdjust"
          @move-overlay="moveOverlay"
        />
      </section>

      <section v-else-if="activePage === 'recording' && recordingSubPage === 'control'" class="page-stack">
        <RecordingPanel
          :config="config"
          :recording-directory="recordingDirectory"
          :default-recording-directory="defaultRecordingDirectory"
          :silent-recording="silentRecording"
          :is-recording="isRecording"
          :recording-countdown="recordingCountdown"
          :last-recording-path="lastRecordingPath"
          :recording-status-message="recordingStatusMessage"
          :recording-hotkeys="recordingHotkeys"
          :hotkey-capture-target="hotkeyCaptureTarget"
          @choose-recording-directory="emit('choose-recording-directory')"
          @update-silent-recording="emit('update-silent-recording', $event)"
          @update-recording-config="emit('update-recording-config', $event)"
          @start-recording="emit('start-recording')"
          @stop-recording="emit('stop-recording')"
          @add-sync-marker="emit('add-sync-marker')"
          @inspect-recording-file="emit('inspect-recording-file')"
          @inspect-recording-path="emit('inspect-recording-path', $event)"
          @update-recording-hotkey-mode="emit('update-recording-hotkey-mode', $event)"
          @begin-hotkey-capture="emit('begin-hotkey-capture', $event)"
        />
      </section>

      <section v-else-if="activePage === 'recording' && recordingSubPage === 'files'" class="page-stack">
        <RecordingBrowserPanel
          :recording-browser-directory="recordingBrowserDirectory"
          :current-recording-path="currentRecordingPath"
          :recording-inspection="recordingInspection"
          :recording-inspection-error="recordingInspectionError"
          @inspect-recording-file="emit('inspect-recording-file')"
          @inspect-recording-path="emit('inspect-recording-path', $event)"
          @clear-recording-inspection="emit('clear-recording-inspection')"
          @choose-recording-browser-directory="emit('choose-recording-browser-directory')"
        />
      </section>

      <section v-else-if="activePage === 'export'" class="page-stack">
        <ExportPanel
          :render-markers="config.export.renderMarkers"
          :video-exporter-config="videoExporterConfig"
          :installing-app-managed-exporter="videoExporterInstalling"
          :uninstalling-app-managed-exporter="videoExporterUninstalling"
          @update-render-markers="updateRenderMarkers"
          @update-video-exporter-config="emit('update-video-exporter-config', $event)"
          @notify="emit('notify', $event.tone, $event.message)"
          @update-installing-app-managed-exporter="videoExporterInstalling = $event"
          @install-app-managed-exporter="installAppManagedVideoExporter"
          @uninstall-app-managed-exporter="uninstallAppManagedVideoExporter"
        />
      </section>
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
  background: #111316;
  color: #eef2f6;
}

.config-shell.sidebar-collapsed {
  grid-template-columns: 72px minmax(0, 1fr);
}

.preview-copy p,
.quiet {
  color: #9ca7b4;
}

.workspace {
  height: 100vh;
  min-width: 0;
  overflow-y: auto;
  padding: 24px;
}

.preview-copy p {
  margin: 0 0 4px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h2 {
  margin: 0;
  letter-spacing: 0;
}

h2 {
  font-size: 18px;
  line-height: 24px;
}

.preview-band {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  min-width: 0;
  min-height: 250px;
  margin-bottom: 20px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background:
    linear-gradient(90deg, rgba(37, 211, 102, 0.1), transparent 44%),
    #151920;
  padding: 24px;
}

.preview-copy {
  flex: 0 0 180px;
}

.preview-viewport {
  display: grid;
  justify-content: start;
  min-width: 0;
  max-width: 100%;
  overflow-x: auto;
  overflow-y: hidden;
  padding: 8px 0 10px 8px;
}

.preview-viewport :deep(.pov-shell) {
  flex: 0 0 auto;
}

.page-stack {
  display: grid;
  gap: 16px;
}

.page-container {
  display: grid;
  gap: 16px;
}

.panel-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.panel {
  box-sizing: border-box;
  min-height: 190px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #171b22;
  padding: 18px;
}

.panel h2 {
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}

.section-header h2 {
  margin-bottom: 0;
}

.recent-profile-control {
  display: grid;
  grid-template-columns: minmax(110px, 1fr) minmax(180px, 240px);
  align-items: center;
  gap: 7px;
  margin-top: 14px;
  color: #c9d1da;
  font-size: 13px;
  font-weight: 700;
}

.recent-profile-control select {
  justify-self: end;
  width: min(240px, 100%);
}

.recent-profile-control select:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.layout-line-list {
  display: grid;
  gap: 10px;
  margin-top: 16px;
}

.layout-line {
  display: grid;
  grid-template-columns: 76px minmax(0, 1fr);
  align-items: start;
  gap: 10px;
}

.line-label {
  color: #9ca7b4;
  font-size: 13px;
  font-weight: 800;
}

.line-keys {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.line-key {
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  background: #202630;
  padding: 5px 8px;
  color: #dfe5ec;
  font-size: 13px;
  font-weight: 700;
}

label {
  display: grid;
  gap: 8px;
  margin-bottom: 16px;
  color: #c9d1da;
  font-weight: 700;
}

.select-control {
  justify-self: end;
  width: min(240px, 100%);
}

.quiet {
  margin: 14px 0 0;
}

@media (max-width: 920px) {
  .config-shell {
    grid-template-columns: 72px minmax(0, 1fr);
  }

  .preview-band {
    align-items: flex-start;
    flex-direction: column;
  }

  .panel-grid {
    grid-template-columns: 1fr;
  }
}
</style>
