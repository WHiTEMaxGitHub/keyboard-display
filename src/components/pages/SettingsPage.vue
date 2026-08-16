<script setup lang="ts">
import { computed, inject, type ComputedRef } from "vue";
import { useI18n } from "vue-i18n";
import BaseButton from "../BaseButton.vue";
import BaseFieldRow from "../BaseFieldRow.vue";
import BasePanel from "../BasePanel.vue";
import BaseSelect from "../BaseSelect.vue";
import ColorPicker from "../ColorPicker.vue";
import { hexToCssColor } from "../../domain/colorPicker";
import type { UiLanguage } from "../../domain/uiLanguage";
import {
  CUSTOM_THEME_COLOR_KEYS,
  CUSTOM_THEME_TEMPLATES,
  THEMES,
  type CustomThemeColorKey,
  type CustomThemeColors,
  type CustomThemeTemplateId,
  type ThemeId,
} from "../../domain/theme";
import { LOCALE_OPTIONS } from "../../i18n";

const appConfigPathRef = inject<ComputedRef<string>>("appConfigPath")!;
const profileNameRef = inject<ComputedRef<string>>("profileName")!;
const profileChangedRef = inject<ComputedRef<boolean>>("profileChanged")!;
const themeIdRef = inject<ComputedRef<ThemeId>>("themeId")!;
const customThemeColorsRef = inject<ComputedRef<CustomThemeColors>>("customThemeColors")!;
const customThemeTemplateRef = inject<ComputedRef<CustomThemeTemplateId>>("customThemeTemplate")!;
const customThemePanelOpacityRef = inject<ComputedRef<number>>("customThemePanelOpacity")!;
const uiLanguageRef = inject<ComputedRef<UiLanguage>>("uiLanguage")!;
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;
const { t } = useI18n();

const appConfigPath = computed(() => appConfigPathRef.value || "Resolving...");
const profileName = computed(() => profileNameRef.value);
const profileChanged = computed(() => profileChangedRef.value);
const themeId = computed(() => themeIdRef.value);
const customThemeColors = computed(() => customThemeColorsRef.value);
const customThemeTemplate = computed(() => customThemeTemplateRef.value);
const customThemePanelOpacity = computed(() => customThemePanelOpacityRef.value);
const customThemePanelOpacityPercent = computed(() => Math.round(customThemePanelOpacity.value * 100));
const uiLanguage = computed(() => uiLanguageRef.value);
const themeOptions = Object.values(THEMES);
const customThemeTemplateOptions = Object.values(CUSTOM_THEME_TEMPLATES);
const themeSelectOptions = computed(() =>
  themeOptions.map((theme) => ({
    value: theme.id,
    label: themeLabel(theme.id),
  })),
);
const languageSelectOptions = computed(() =>
  LOCALE_OPTIONS.map((option) => ({
    value: option.value,
    label: t(option.labelKey),
  })),
);
const customThemeTemplateSelectOptions = computed(() =>
  customThemeTemplateOptions.map((template) => ({
    value: template.id,
    label: customThemeTemplateLabel(template.id),
  })),
);
const selectedTheme = computed(() => THEMES[themeId.value]);
const isCustomThemeActive = computed(() => themeId.value === "custom");
const customThemeColorOptions: Array<{
  key: CustomThemeColorKey;
  labelKey: string;
}> = CUSTOM_THEME_COLOR_KEYS.map(({ key }) => ({
  key,
  labelKey: `settings.customThemeColor.${key}`,
}));
const visibleCustomThemeColorOptions = computed(() => {
  const visibleKeys = new Set(CUSTOM_THEME_TEMPLATES[customThemeTemplate.value].colorKeys);
  return customThemeColorOptions.filter((option) => visibleKeys.has(option.key));
});
const activeThemeSwatchOptions = computed(() => {
  return isCustomThemeActive.value ? visibleCustomThemeColorOptions.value : customThemeColorOptions;
});
const customThemePreviewStyle = computed(() => ({
  "--preview-primary": hexToCssColor(customThemeColors.value.accent),
  "--preview-blue": hexToCssColor(customThemeColors.value.accentBlue),
  "--preview-violet": hexToCssColor(customThemeColors.value.accentViolet),
  "--preview-green": hexToCssColor(customThemeColors.value.accentEmerald),
  "--preview-panel-opacity": customThemePanelOpacity.value,
}));
const configDirectory = computed(() => {
  const path = appConfigPathRef.value;
  const separatorIndex = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return separatorIndex >= 0 ? path.slice(0, separatorIndex) : path || "Resolving...";
});

function updateTheme(value: string) {
  emit("set-theme", value as ThemeId);
}

function updateUiLanguage(value: string) {
  emit("set-ui-language", value as UiLanguage);
}

function previewCustomThemeColor(key: CustomThemeColorKey, color: string) {
  emit("preview-custom-theme-color", key, color);
}

function updateCustomThemeColor(key: CustomThemeColorKey, color: string) {
  emit("set-custom-theme-color", key, color);
}

function updateCustomThemeTemplate(value: string) {
  emit("set-custom-theme-template", value as CustomThemeTemplateId);
}

function updateCustomThemePanelOpacity(event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  const percent = Number.isFinite(value) ? Math.min(100, Math.max(0, Math.round(value))) : 100;
  emit("set-custom-theme-panel-opacity", percent / 100);
}

function activateCustomTheme() {
  emit("set-theme", "custom");
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

function customThemeTemplateLabel(id: CustomThemeTemplateId) {
  return t(`settings.customThemeTemplateOption.${id}`);
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
        <BaseSelect
          class="select-control"
          :model-value="themeId"
          :options="themeSelectOptions"
          @update:model-value="updateTheme"
        />
      </label>

      <label class="setting-row mt-3">
        <span>{{ t("settings.language") }}</span>
        <BaseSelect
          class="select-control"
          :model-value="uiLanguage"
          :options="languageSelectOptions"
          @update:model-value="updateUiLanguage"
        />
      </label>

      <div class="theme-preview">
        <div class="theme-swatch-row" aria-hidden="true">
          <span
            v-for="option in activeThemeSwatchOptions"
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
            <span>
              {{ isCustomThemeActive ? t("settings.customThemeDescription") : t("settings.customThemeInactiveDescription") }}
            </span>
          </div>
          <BaseButton v-if="!isCustomThemeActive" size="sm" @click="activateCustomTheme">
            {{ t("settings.activateCustomTheme") }}
          </BaseButton>
          <BaseButton v-else size="sm" @click="resetCustomThemeColors">
            {{ t("settings.resetCustomTheme") }}
          </BaseButton>
        </div>

        <template v-if="isCustomThemeActive">
          <label class="setting-row custom-template-row">
            <span>{{ t("settings.customThemeTemplate") }}</span>
            <BaseSelect
              class="select-control"
              :model-value="customThemeTemplate"
              :options="customThemeTemplateSelectOptions"
              @update:model-value="updateCustomThemeTemplate"
            />
          </label>

          <label class="setting-row custom-opacity-row">
            <span>{{ t("settings.customThemePanelOpacity") }}</span>
            <span class="number-field">
              <input
                :value="customThemePanelOpacityPercent"
                min="0"
                max="100"
                step="1"
                type="number"
                @blur="updateCustomThemePanelOpacity"
                @change="updateCustomThemePanelOpacity"
              />
              <span>%</span>
            </span>
          </label>
          <p class="custom-opacity-description">
            {{ t("settings.customThemePanelOpacityDescription") }}
          </p>

          <div class="custom-theme-live-preview" :style="customThemePreviewStyle">
            <div class="preview-stage">
              <div class="preview-panel">
                <div>
                  <span>{{ t("settings.customThemePreview") }}</span>
                  <strong>{{ customThemeTemplateLabel(customThemeTemplate) }}</strong>
                </div>
                <button type="button">{{ t("topbar.exportApply") }}</button>
              </div>
              <div class="preview-key-row" aria-hidden="true">
                <span class="preview-key preview-key-primary">A</span>
                <span class="preview-key preview-key-blue">S</span>
                <span class="preview-key preview-key-violet">D</span>
                <span class="preview-key preview-key-green">F</span>
              </div>
            </div>
            <div class="preview-channel-list">
              <span
                v-for="option in visibleCustomThemeColorOptions"
                :key="option.key"
              >
                <i :style="{ background: hexToCssColor(customThemeColors[option.key]) }"></i>
                {{ t(option.labelKey) }}
              </span>
            </div>
          </div>

          <div class="custom-color-section">
            <span>{{ t("settings.customThemeAccentColors") }}</span>
            <div class="custom-color-grid">
              <ColorPicker
                v-for="option in visibleCustomThemeColorOptions"
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
        </template>
      </div>
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
  grid-template-columns: repeat(2, minmax(0, 1fr));
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
.theme-preview strong {
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
.custom-color-section > span,
.custom-opacity-description {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 700;
}

.custom-opacity-row {
  margin-top: -2px;
}

.number-field {
  justify-self: end;
  display: grid;
  grid-template-columns: minmax(0, 84px) auto;
  align-items: center;
  gap: 7px;
  color: var(--color-text-muted);
  font-weight: 900;
}

.number-field input {
  width: 84px;
  min-height: 34px;
  border: 1px solid var(--color-border-control);
  border-radius: var(--radius-md);
  background: var(--color-surface-control);
  color: var(--color-text-primary);
  font: inherit;
  font-weight: 900;
  padding: 0 9px;
}

.custom-opacity-description {
  margin: -8px 0 0;
}

.custom-theme-live-preview {
  display: grid;
  gap: 10px;
  overflow: hidden;
  border: 1px solid var(--color-border-control);
  border-radius: var(--radius-xl);
  background:
    radial-gradient(circle at 18% 16%, color-mix(in srgb, var(--preview-blue) 38%, transparent), transparent 34%),
    radial-gradient(circle at 76% 20%, color-mix(in srgb, var(--preview-violet) 42%, transparent), transparent 34%),
    radial-gradient(circle at 62% 88%, color-mix(in srgb, var(--preview-green) 28%, transparent), transparent 38%),
    color-mix(in srgb, var(--color-surface-base) 78%, transparent);
  padding: 12px;
}

.preview-stage {
  display: grid;
  gap: 12px;
  min-height: 132px;
  border-radius: var(--radius-lg);
  background:
    linear-gradient(
      135deg,
      color-mix(in srgb, var(--preview-primary) 18%, transparent),
      color-mix(in srgb, var(--preview-violet) 16%, transparent)
    );
  padding: 14px;
}

.preview-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  border: 1px solid color-mix(in srgb, white 16%, transparent);
  border-radius: var(--radius-lg);
  background:
    linear-gradient(
      135deg,
      rgba(24, 30, 42, var(--preview-panel-opacity)),
      rgba(9, 12, 18, var(--preview-panel-opacity))
    );
  padding: 12px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.22);
}

.preview-panel div {
  display: grid;
  gap: 3px;
}

.preview-panel span,
.preview-channel-list {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 800;
}

.preview-panel strong {
  color: var(--color-text-primary);
  font-size: 15px;
}

.preview-panel button {
  min-height: 32px;
  border: 1px solid color-mix(in srgb, var(--preview-primary) 60%, white 14%);
  border-radius: var(--radius-md);
  background:
    linear-gradient(
      135deg,
      color-mix(in srgb, var(--preview-primary) 82%, white 6%),
      color-mix(in srgb, var(--preview-blue) 72%, black 4%)
    );
  color: white;
  font: inherit;
  font-size: 12px;
  font-weight: 900;
  padding: 0 12px;
}

.preview-key-row {
  display: flex;
  gap: 8px;
}

.preview-key {
  display: grid;
  place-items: center;
  width: 42px;
  height: 38px;
  border: 1px solid color-mix(in srgb, white 18%, transparent);
  border-radius: var(--radius-md);
  color: white;
  font-weight: 900;
  box-shadow:
    inset 0 -3px 0 rgba(0, 0, 0, 0.34),
    0 10px 22px rgba(0, 0, 0, 0.22);
}

.preview-key-primary {
  background: var(--preview-primary);
}

.preview-key-blue {
  background: var(--preview-blue);
}

.preview-key-violet {
  background: var(--preview-violet);
}

.preview-key-green {
  background: var(--preview-green);
}

.preview-channel-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 12px;
}

.preview-channel-list span {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.preview-channel-list i {
  width: 13px;
  height: 13px;
  border: 1px solid var(--color-border-control);
  border-radius: 999px;
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

  .number-field {
    justify-self: stretch;
  }

  .custom-theme-header {
    display: grid;
  }
}
</style>
