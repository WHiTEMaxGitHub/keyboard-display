<script setup lang="ts">
import { computed, inject, type ComputedRef } from "vue";
import BaseFieldRow from "../BaseFieldRow.vue";
import BasePanel from "../BasePanel.vue";
import BaseSelect from "../BaseSelect.vue";
import type { RecentProfile } from "../../domain/appConfig";
import { THEMES, type ThemeId } from "../../domain/theme";

const appConfigPathRef = inject<ComputedRef<string>>("appConfigPath")!;
const profileNameRef = inject<ComputedRef<string>>("profileName")!;
const profileChangedRef = inject<ComputedRef<boolean>>("profileChanged")!;
const recentProfilesRef = inject<ComputedRef<RecentProfile[]>>("recentProfiles")!;
const themeIdRef = inject<ComputedRef<ThemeId>>("themeId")!;
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;

const appConfigPath = computed(() => appConfigPathRef.value || "Resolving...");
const profileName = computed(() => profileNameRef.value);
const profileChanged = computed(() => profileChangedRef.value);
const recentProfiles = computed(() => recentProfilesRef.value);
const themeId = computed(() => themeIdRef.value);
const themeOptions = Object.values(THEMES);
const selectedTheme = computed(() => THEMES[themeId.value]);
const configDirectory = computed(() => {
  const path = appConfigPathRef.value;
  const separatorIndex = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return separatorIndex >= 0 ? path.slice(0, separatorIndex) : path || "Resolving...";
});
const displayedRecentProfiles = computed(() => recentProfiles.value.slice(0, 5));

function updateTheme(event: Event) {
  emit("set-theme", (event.target as HTMLSelectElement).value as ThemeId);
}
</script>

<template>
  <section class="page-stack">
    <BasePanel>
      <div class="section-header">
        <div>
          <p class="eyebrow">Application</p>
          <h2 class="m-0">App Config</h2>
        </div>
      </div>

      <div class="config-path-card">
        <span class="config-path-label">Config file</span>
        <code class="config-path">{{ appConfigPath }}</code>
        <span class="config-path-dir">{{ configDirectory }}</span>
      </div>

      <div class="metric-grid">
        <div class="metric-card">
          <span>Current profile</span>
          <strong>{{ profileName }}</strong>
        </div>
        <div class="metric-card">
          <span>Profile status</span>
          <strong>{{ profileChanged ? "Unsaved" : "Saved" }}</strong>
        </div>
        <div class="metric-card">
          <span>Recent profiles</span>
          <strong>{{ recentProfiles.length }}</strong>
        </div>
      </div>
    </BasePanel>

    <BasePanel>
      <div class="section-header">
        <div>
          <p class="eyebrow">Interface</p>
          <h2 class="m-0">Theme</h2>
        </div>
      </div>

      <label class="setting-row">
        <span>Theme</span>
        <BaseSelect class="select-control" :model-value="themeId" @change="updateTheme">
          <option
            v-for="theme in themeOptions"
            :key="theme.id"
            :value="theme.id"
          >
            {{ theme.label }}
          </option>
        </BaseSelect>
      </label>

      <div class="theme-preview">
        <div class="theme-swatch-row" aria-hidden="true">
          <span
            class="theme-swatch"
            :style="{ background: selectedTheme.css['--theme-accent'] }"
          />
          <span
            class="theme-swatch"
            :style="{ background: selectedTheme.css['--theme-accent-blue'] }"
          />
          <span
            class="theme-swatch"
            :style="{ background: selectedTheme.css['--theme-accent-violet'] }"
          />
          <span
            class="theme-swatch"
            :style="{ background: selectedTheme.css['--theme-accent-emerald'] }"
          />
        </div>
        <div>
          <strong>{{ selectedTheme.label }}</strong>
          <span>Active interface theme</span>
        </div>
      </div>
    </BasePanel>

    <BasePanel>
      <div class="section-header">
        <div>
          <p class="eyebrow">Profiles</p>
          <h2 class="m-0">Recent Configs</h2>
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
      <p v-else class="empty-state">No recent profiles stored yet.</p>
    </BasePanel>

    <BasePanel>
      <div class="section-header">
        <div>
          <p class="eyebrow">Future</p>
          <h2 class="m-0">App-Level Settings</h2>
        </div>
      </div>

      <div class="settings-roadmap">
        <BaseFieldRow label="Recording defaults">Stored in app config</BaseFieldRow>
        <BaseFieldRow label="Exporter settings">Stored in app config</BaseFieldRow>
        <BaseFieldRow label="Interface settings">Ready for expansion</BaseFieldRow>
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
}
</style>
