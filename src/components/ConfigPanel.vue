<script setup lang="ts">
import { computed, provide, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { AppConfig, ExportConfig, OverlayStyle } from "../domain/defaultConfig";
import type { UiLanguage } from "../domain/uiLanguage";
import type { AppNotification, NotificationTone } from "../composables/useNotifications";
import type { RecordingHotkeyConfig, RecordingHotkeyMode } from "../domain/recordingHotkeys";
import type { VideoExporterConfig } from "../domain/videoExporter";
import type { RecordingInspection } from "../types/recording";
import type {
  CustomThemeColorKey,
  CustomThemeColors,
  CustomThemeTemplateId,
  ThemeId,
} from "../domain/theme";
import ConfigSidebar from "./ConfigSidebar.vue";
import ConfigTopbar from "./ConfigTopbar.vue";
import NotificationStack from "./NotificationStack.vue";
import LayoutPage from "./pages/LayoutPage.vue";
import AppearancePage from "./pages/AppearancePage.vue";
import WindowPage from "./pages/WindowPage.vue";
import RecordingPage from "./pages/RecordingPage.vue";
import ExportPage from "./pages/ExportPage.vue";
import SettingsPage from "./pages/SettingsPage.vue";
import PovOverlay from "./PovOverlay.vue";
import BaseFieldRow from "./BaseFieldRow.vue";
import BaseInput from "./BaseInput.vue";
import BaseSelect from "./BaseSelect.vue";
import BaseToggleRow from "./BaseToggleRow.vue";
import { LOCALE_OPTIONS } from "../i18n";

type ConfigPage =
  | "overview"
  | "layout"
  | "appearance"
  | "window"
  | "recording"
  | "export"
  | "settings";
type LayoutSubPage = "summary" | "editor";
type RecordingSubPage = "control" | "files";

const props = defineProps<{
  config: AppConfig;
  activeKeys: Set<string>;
  keyIdLabels: AppConfig["keyIdLabels"];
  overlayVisible: boolean;
  profileName: string;
  profileChanged: boolean;
  recordingDirectory: string;
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
  themeId: ThemeId;
  customThemeColors: CustomThemeColors;
  customThemeTemplate: CustomThemeTemplateId;
  customThemePanelOpacity: number;
  uiLanguage: UiLanguage;
  appConfigPath: string;
}>();

const emit = defineEmits<{
  "preview-overlay-style": [style: OverlayStyle];
  "update-key-id-labels": [labels: AppConfig["keyIdLabels"]];
  "update-overlay-style": [style: OverlayStyle];
  "update-overlay-layout": [layout: AppConfig["layout"]];
  "update-overlay-rows": [rows: AppConfig["rows"]];
  "update-overlay-visible": [visible: boolean];
  "load-config": [];
  "refresh-pov": [];
  "export-and-apply-config": [];
  "overwrite-and-apply-config": [];
  "choose-recording-directory": [];
  "choose-recording-browser-directory": [];
  "update-silent-recording": [value: boolean];
  "update-recording-config": [recording: AppConfig["recording"]];
  "update-export-config": [exportConfig: ExportConfig];
  "update-video-exporter-config": [exporterConfig: VideoExporterConfig];
  "update-profile-name": [name: string];
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
  "set-theme": [id: ThemeId];
  "preview-custom-theme-color": [key: CustomThemeColorKey, color: string];
  "set-custom-theme-color": [key: CustomThemeColorKey, color: string];
  "set-custom-theme-template": [templateId: CustomThemeTemplateId];
  "set-custom-theme-panel-opacity": [opacity: number];
  "reset-custom-theme-colors": [];
  "set-ui-language": [language: UiLanguage];
}>();

const activePage = ref<ConfigPage>("overview");
const layoutSubPage = ref<LayoutSubPage>("summary");
const recordingSubPage = ref<RecordingSubPage>("control");
const recentColors = ref<string[]>([]);
const { t } = useI18n();
const languageSelectOptions = computed(() =>
  LOCALE_OPTIONS.map((option) => ({
    value: option.value,
    label: t(option.labelKey),
  })),
);

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

function selectLayoutSubPage(page: LayoutSubPage) {
  layoutSubPage.value = page;
}

function updateOverlayVisible(event: Event) {
  emit("update-overlay-visible", (event.target as HTMLInputElement).checked);
}

function updateAlwaysOnTop(event: Event) {
  emit("update-overlay-style", {
    ...props.config.style,
    alwaysOnTop: (event.target as HTMLInputElement).checked,
  });
}

function updateProfileName(value: string | number) {
  emit("update-profile-name", String(value));
}

function updateUiLanguage(value: string) {
  emit("set-ui-language", value as UiLanguage);
}

provide("config", props.config);
provide("activeKeys", computed(() => props.activeKeys));
provide("keyIdLabels", computed(() => props.keyIdLabels));
provide("overlayVisible", computed(() => props.overlayVisible));
provide("profileName", computed(() => props.profileName));
provide("profileChanged", computed(() => props.profileChanged));
provide("recordingDirectory", computed(() => props.recordingDirectory));
provide("recordingBrowserDirectory", computed(() => props.recordingBrowserDirectory));
provide("silentRecording", computed(() => props.silentRecording));
provide("isRecording", computed(() => props.isRecording));
provide("recordingCountdown", computed(() => props.recordingCountdown));
provide("lastRecordingPath", computed(() => props.lastRecordingPath));
provide("recordingStatusMessage", computed(() => props.recordingStatusMessage));
provide("currentRecordingPath", computed(() => props.currentRecordingPath));
provide("recordingInspection", computed(() => props.recordingInspection));
provide("recordingInspectionError", computed(() => props.recordingInspectionError));
provide("overlayPosition", computed(() => props.overlayPosition));
provide("overlayAdjusting", computed(() => props.overlayAdjusting));
provide("recordingHotkeys", computed(() => props.recordingHotkeys));
provide("hotkeyCaptureTarget", computed(() => props.hotkeyCaptureTarget));
provide("videoExporterConfig", computed(() => props.videoExporterConfig));
provide("recentColors", recentColors);
provide("layoutSubPage", computed(() => layoutSubPage.value));
provide("themeId", computed(() => props.themeId));
provide("customThemeColors", computed(() => props.customThemeColors));
provide("customThemeTemplate", computed(() => props.customThemeTemplate));
provide("customThemePanelOpacity", computed(() => props.customThemePanelOpacity));
provide("uiLanguage", computed(() => props.uiLanguage));
provide("appConfigPath", computed(() => props.appConfigPath));
provide("emit", relay);

const pageComponent = computed(() => {
  const map: Record<Exclude<ConfigPage, "overview">, any> = {
    layout: LayoutPage,
    appearance: AppearancePage,
    window: WindowPage,
    recording: RecordingPage,
    export: ExportPage,
    settings: SettingsPage,
  };
  return activePage.value === "overview" ? null : map[activePage.value];
});
</script>

<template>
  <main class="config-shell">
    <NotificationStack
      :notifications="notifications"
      @dismiss="emit('dismiss-notification', $event)"
    />
    <ConfigSidebar
      :active-page="activePage"
      :layout-sub-page="layoutSubPage"
      :recording-sub-page="recordingSubPage"
      @update-active-page="selectActivePage"
      @update-layout-sub-page="selectLayoutSubPage"
      @update-recording-sub-page="selectRecordingSubPage"
    />

    <div class="sidebar-spacer" aria-hidden="true" />

    <section class="workspace">
      <ConfigTopbar
        :profile-name="profileName"
        @load-config="emit('load-config')"
        @export-and-apply-config="emit('export-and-apply-config')"
        @overwrite-and-apply-config="emit('overwrite-and-apply-config')"
      />

      <div :key="`${activePage}-${recordingSubPage}`" class="page-container animate-[page-enter_300ms_ease-out]">
        <section v-if="activePage === 'overview'" class="page-stack">
          <section class="preview-band" :aria-label="t('common.livePreview')">
            <div class="preview-copy">
              <p>{{ t("overview.livePreview") }}</p>
              <h2 class="m-0">{{ profileName }}</h2>
            </div>
            <div class="preview-viewport">
              <PovOverlay
                :layout="config.layout"
                :rows="config.rows"
                :keys="config.keys"
                :key-id-labels="keyIdLabels"
                :active-keys="activeKeys"
                :overlay-style="config.style"
                fit-to-container
              />
            </div>
          </section>

          <section class="panel-grid">
            <article class="panel">
              <h2 class="m-0">{{ t("overview.profile") }}</h2>
              <BaseFieldRow :label="t('overview.name')">
                <BaseInput
                  block
                  :model-value="profileName"
                  :placeholder="t('overview.namePlaceholder')"
                  @update:model-value="updateProfileName"
                />
              </BaseFieldRow>
              <BaseFieldRow :label="t('overview.status')">
                {{ profileChanged ? t("overview.unsavedChanges") : t("overview.saved") }}
              </BaseFieldRow>
              <BaseFieldRow :label="t('overview.visibleKeys')">{{ config.keys.length }}</BaseFieldRow>
            </article>

            <article class="panel">
              <h2 class="m-0">{{ t("overview.quickControls") }}</h2>
              <BaseToggleRow :checked="overlayVisible" @change="updateOverlayVisible">
                {{ t("overview.showOverlay") }}
              </BaseToggleRow>
              <BaseToggleRow :checked="config.style.alwaysOnTop" @change="updateAlwaysOnTop">
                {{ t("overview.alwaysOnTop") }}
              </BaseToggleRow>
              <label class="quick-control-row">
                <span>{{ t("overview.language") }}</span>
                <BaseSelect
                  class="quick-control-select"
                  :model-value="uiLanguage"
                  :options="languageSelectOptions"
                  @update:model-value="updateUiLanguage"
                />
              </label>
            </article>
          </section>
        </section>
        <component v-else :is="pageComponent" />
      </div>
    </section>
  </main>
</template>

<style scoped>
.config-shell {
  position: relative;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: transparent;
  color: var(--color-text-primary);
}

.sidebar-spacer {
  width: 48px;
  height: 0;
  flex-shrink: 0;
}

.workspace {
  height: 100%;
  min-width: 0;
  overflow-y: auto;
  padding: 0 24px 24px;
  padding-left: calc(48px + 24px);
}

.page-container {
  display: grid;
  gap: 16px;
}

.page-stack {
  display: grid;
  gap: 16px;
}

.preview-band {
  position: relative;
  display: grid;
  align-items: center;
  min-width: 0;
  min-height: 250px;
  margin-bottom: 20px;
  border: 1px solid var(--glass-border);
  border-radius: 28px;
  background: linear-gradient(145deg, var(--glass-from), var(--glass-to));
  backdrop-filter: blur(24px) saturate(170%);
  -webkit-backdrop-filter: blur(24px) saturate(170%);
  box-shadow: var(--glass-shadow);
  padding: 24px 28px 24px;
}

.preview-copy {
  position: absolute;
  left: 24px;
  top: 50%;
  z-index: 2;
  width: 132px;
  transform: translateY(-50%);
  pointer-events: none;
}

.preview-copy p {
  margin: 0 0 4px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  white-space: nowrap;
  color: var(--color-text-muted);
}

.preview-viewport {
  --preview-available-width: calc(100vw - 220px);

  display: grid;
  align-items: center;
  justify-items: center;
  min-width: 0;
  width: 100%;
  overflow: hidden;
  scrollbar-gutter: stable both-edges;
  padding: 8px 12px 10px 148px;
}

.preview-viewport :deep(.pov-shell) {
  min-width: 0;
}

.panel-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.panel {
  box-sizing: border-box;
  min-height: 190px;
  border: 1px solid var(--glass-border);
  border-radius: 28px;
  background: linear-gradient(145deg, var(--glass-from), var(--glass-to));
  backdrop-filter: blur(24px) saturate(170%);
  -webkit-backdrop-filter: blur(24px) saturate(170%);
  box-shadow: var(--glass-shadow);
  padding: 18px;
  transition: border-color 300ms, box-shadow 300ms;
}

.panel:hover {
  border-color: var(--glass-border-hover);
  box-shadow: var(--glass-shadow-hover);
}

.panel h2 {
  margin-bottom: 16px;
  font-size: 18px;
  line-height: 24px;
  letter-spacing: 0;
  color: var(--color-text-primary);
}

.quick-control-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(150px, 210px);
  align-items: center;
  gap: 12px;
  min-height: 38px;
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 800;
}

.quick-control-select {
  width: 100%;
}

@media (max-width: 920px) {
  .preview-band {
    min-height: 280px;
  }

  .preview-copy {
    top: 22px;
    transform: none;
  }

  .preview-viewport {
    --preview-available-width: calc(100vw - 150px);

    width: 100%;
    padding: 58px 8px 8px;
  }

  .panel-grid {
    grid-template-columns: 1fr;
  }

  .quick-control-row {
    grid-template-columns: 1fr;
    align-items: stretch;
  }
}
</style>
