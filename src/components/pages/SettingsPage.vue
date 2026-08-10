<script setup lang="ts">
import { computed, inject, type ComputedRef } from "vue";
import { useI18n } from "vue-i18n";
import BaseButton from "../BaseButton.vue";
import BaseFieldRow from "../BaseFieldRow.vue";
import BasePanel from "../BasePanel.vue";
import BaseSelect from "../BaseSelect.vue";
import ColorPicker from "../ColorPicker.vue";
import type { RecentProfile } from "../../domain/appConfig";
import type { UiLanguage } from "../../domain/uiLanguage";
import {
  CUSTOM_THEME_COLOR_KEYS,
  THEMES,
  type CustomThemeColorKey,
  type CustomThemeColors,
  type ThemeId,
} from "../../domain/theme";
import { LOCALE_OPTIONS } from "../../i18n";

const appConfigPathRef = inject<ComputedRef<string>>("appConfigPath")!;
const profileNameRef = inject<ComputedRef<string>>("profileName")!;
const profileChangedRef = inject<ComputedRef<boolean>>("profileChanged")!;
const recentProfilesRef = inject<ComputedRef<RecentProfile[]>>("recentProfiles")!;
const themeIdRef = inject<ComputedRef<ThemeId>>("themeId")!;
const customThemeColorsRef = inject<ComputedRef<CustomThemeColors>>("customThemeColors")!;
const uiLanguageRef = inject<ComputedRef<UiLanguage>>("uiLanguage")!;
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;
const { t } = useI18n();

const appConfigPath = computed(() => appConfigPathRef.value || "Resolving...");
const profileName = computed(() => profileNameRef.value);
const profileChanged = computed(() => profileChangedRef.value);
const recentProfiles = computed(() => recentProfilesRef.value);
const themeId = computed(() => themeIdRef.value);
const customThemeColors = computed(() => customThemeColorsRef.value);
const uiLanguage = computed(() => uiLanguageRef.value);
const themeOptions = Object.values(THEMES);
const selectedTheme = computed(() => THEMES[themeId.value]);
const customThemeColorOptions: Array<{
  key: CustomThemeColorKey;
  labelKey: string;
}> = CUSTOM_THEME_COLOR_KEYS.map(({ key }) => ({
  key,
  labelKey: `settings.customThemeColor.${key}`,
}));
const configDirectory = computed(() => {
  const path = appConfigPathRef.value;
  const separatorIndex = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return separatorIndex >= 0 ? path.slice(0, separatorIndex) : path || "Resolving...";
});
const displayedRecentProfiles = computed(() => recentProfiles.value.slice(0, 5));

function updateTheme(event: Event) {
  emit("set-theme", (event.target as HTMLSelectElement).value as ThemeId);
}

function updateUiLanguage(event: Event) {
  emit("set-ui-language", (event.target as HTMLSelectElement).value as UiLanguage);
}

function previewCustomThemeColor(key: CustomThemeColorKey, color: string) {
  emit("preview-custom-theme-color", key, color);
}

function updateCustomThemeColor(key: CustomThemeColorKey, color: string) {
  emit("set-custom-theme-color", key, color);
}

function resetCustomThemeColors() {
  emit("reset-custom-theme-colors");
}

function themeSwatchColor(key: CustomThemeColorKey) {
  if (themeId.value === "custom") {
    return customThemeColors.value[key];
  }

  const cssVar = CUSTOM_THEME_COLOR_KEYS.find((item) => item.key === key)?.cssVar ?? "--theme-accent";
  return selectedTheme.value.css[cssVar];
}

function themeLabel(id: ThemeId) {
  return t(`settings.themeOption.${id}`);
}
</script>

<template>
  <section class="page-stack">
    <BasePanel>
      <div class="section-header">
        <div>
          <p class="eyebrow">{{ t("settings.application") }}</p>
          <h2 class="m-0">{{ t("settings.appConfig") }}</h2>
        </div>
      </div>

      <div class="config-path-card">
        <span class="config-path-label">{{ t("settings.configFile") }}</span>
        <code class="config-path">{{ appConfigPath }}</code>
        <span class="config-path-dir">{{ configDirectory }}</span>
      </div>

      <div class="metric-grid">
        <div class="metric-card">
          <span>{{ t("settings.currentProfile") }}</span>
          <strong>{{ profileName }}</strong>
        </div>
        <div class="metric-card">
          <span>{{ t("settings.profileStatus") }}</span>
          <strong>{{ profileChanged ? t("overview.unsavedChanges") : t("overview.saved") }}</strong>
        </div>
        <div class="metric-card">
          <span>{{ t("settings.recentProfiles") }}</span>
          <strong>{{ recentProfiles.length }}</strong>
        </div>
      </div>
    </BasePanel>

    <BasePanel>
      <div class="section-header">
        <div>
          <p class="eyebrow">{{ t("settings.interface") }}</p>
          <h2 class="m-0">{{ t("settings.theme") }}</h2>
        </div>
      </div>

      <label class="setting-row">
        <span>{{ t("settings.theme") }}</span>
        <BaseSelect class="select-control" :model-value="themeId" @change="updateTheme">
          <option
            v-for="theme in themeOptions"
            :key="theme.id"
            :value="theme.id"
          >
            {{ themeLabel(theme.id) }}
          </option>
        </BaseSelect>
      </label>

      <label class="setting-row mt-3">
        <span>{{ t("settings.language") }}</span>
        <BaseSelect class="select-control" :model-value="uiLanguage" @change="updateUiLanguage">
          <option
            v-for="option in LOCALE_OPTIONS"
            :key="option.value"
            :value="option.value"
          >
            {{ t(option.labelKey) }}
          </option>
        </BaseSelect>
      </label>

      <div class="theme-preview">
        <div class="theme-swatch-row" aria-hidden="true">
          <span
            v-for="option in customThemeColorOptions"
            :key="option.key"
            class="theme-swatch"
            :style="{ background: themeSwatchColor(option.key) }"
          />
        </div>
        <div>
          <strong>{{ themeLabel(selectedTheme.id) }}</strong>
          <span>{{ t("settings.activeTheme") }}</span>
        </div>
      </div>

      <div class="custom-theme-card">
        <div class="custom-theme-header">
          <div>
            <strong>{{ t("settings.customTheme") }}</strong>
            <span>{{ t("settings.customThemeDescription") }}</span>
          </div>
          <BaseButton size="sm" @click="resetCustomThemeColors">
            {{ t("settings.resetCustomTheme") }}
          </BaseButton>
        </div>

        <div class="custom-color-section">
          <span>{{ t("settings.customThemeAccentColors") }}</span>
          <div class="custom-color-grid">
            <ColorPicker
              v-for="option in customThemeColorOptions"
              :key="option.key"
              :label="t(option.labelKey)"
              :value="customThemeColors[option.key]"
              :recent-colors="[]"
              alpha-enabled
              @preview:value="previewCustomThemeColor(option.key, $event)"
              @update:value="updateCustomThemeColor(option.key, $event)"
            />
          </div>
        </div>
      </div>
    </BasePanel>

    <BasePanel>
      <div class="section-header">
        <div>
          <p class="eyebrow">{{ t("overview.profile") }}</p>
          <h2 class="m-0">{{ t("settings.recentConfigs") }}</h2>
        </div>
      </div>

      <div v-if="displayedRecentProfiles.length" class="recent-list">
        <div
          v-for="profile in displayedRecentProfiles"
          :key="profile.path"
          class="recent-item"
        >
          <strong>{{ profile.name }}</strong>
          <span>{{ profile.path }}</span>
        </div>
      </div>
      <p v-else class="empty-state">{{ t("settings.noRecentProfiles") }}</p>
    </BasePanel>

    <BasePanel>
      <div class="section-header">
        <div>
          <p class="eyebrow">{{ t("settings.future") }}</p>
          <h2 class="m-0">{{ t("settings.appLevelSettings") }}</h2>
        </div>
      </div>

      <div class="settings-roadmap">
        <BaseFieldRow :label="t('settings.recordingDefaults')">{{ t("settings.storedInAppConfig") }}</BaseFieldRow>
        <BaseFieldRow :label="t('settings.exporterSettings')">{{ t("settings.storedInAppConfig") }}</BaseFieldRow>
        <BaseFieldRow :label="t('settings.interfaceSettings')">{{ t("settings.readyForExpansion") }}</BaseFieldRow>
      </div>
    </BasePanel>
  </section>
</template>

<style scoped>
.page-stack {
  display: grid;
  gap: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}

.eyebrow {
  margin: 0 0 4px;
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.path-text {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.config-path-card {
  display: grid;
  gap: 6px;
  margin-bottom: 16px;
  border: 1px solid var(--color-border-control);
  border-radius: var(--radius-xl);
  background: var(--color-surface-control);
  padding: 14px;
}

.config-path-label,
.config-path-dir,
.metric-card span,
.theme-preview span,
.recent-item span,
.empty-state {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 700;
}

.config-path {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 800;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.metric-card {
  display: grid;
  gap: 5px;
  min-width: 0;
  border: 1px solid var(--color-border-dim);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--color-surface-control) 72%, transparent);
  padding: 12px;
}

.metric-card strong,
.theme-preview strong,
.recent-item strong {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-primary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.setting-row {
  display: grid;
  grid-template-columns: minmax(110px, 1fr) minmax(180px, 240px);
  align-items: center;
  gap: 7px;
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 700;
}

.select-control {
  justify-self: end;
  width: min(240px, 100%);
}

.theme-preview {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 14px;
  margin-top: 16px;
  border: 1px solid var(--color-border-control);
  border-radius: var(--radius-xl);
  background: var(--color-surface-control);
  padding: 14px;
}

.theme-swatch-row {
  display: flex;
  gap: 6px;
}

.theme-swatch {
  width: 22px;
  height: 22px;
  border: 1px solid var(--color-border-control);
  border-radius: 999px;
}

.theme-preview div:last-child {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.custom-theme-card {
  display: grid;
  gap: 14px;
  margin-top: 12px;
  border: 1px solid var(--color-border-dim);
  border-radius: var(--radius-xl);
  background: color-mix(in srgb, var(--color-surface-control) 72%, transparent);
  padding: 14px;
}

.custom-theme-header {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: 12px;
}

.custom-theme-header div {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.custom-theme-header strong {
  color: var(--color-text-primary);
}

.custom-theme-header span,
.custom-color-section > span {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 700;
}

.custom-color-section {
  display: grid;
  gap: 8px;
}

.custom-color-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 220px), 1fr));
  gap: 10px;
}

.recent-list {
  display: grid;
  gap: 8px;
}

.recent-item {
  display: grid;
  gap: 4px;
  min-width: 0;
  border: 1px solid var(--color-border-dim);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--color-surface-control) 70%, transparent);
  padding: 10px 12px;
}

.recent-item span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-state {
  margin: 0;
}

.settings-roadmap {
  display: grid;
}

@media (max-width: 640px) {
  .metric-grid {
    grid-template-columns: 1fr;
  }

  .setting-row {
    grid-template-columns: 1fr;
  }

  .select-control {
    justify-self: stretch;
    width: 100%;
  }

  .custom-theme-header {
    display: grid;
  }
}
</style>
