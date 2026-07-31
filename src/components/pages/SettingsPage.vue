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

      <BaseFieldRow label="Config file">
        <span class="path-text">{{ appConfigPath }}</span>
      </BaseFieldRow>
      <BaseFieldRow label="Current profile">{{ profileName }}</BaseFieldRow>
      <BaseFieldRow label="Profile status">
        {{ profileChanged ? "Unsaved changes" : "Saved" }}
      </BaseFieldRow>
      <BaseFieldRow label="Recent profiles">{{ recentProfiles.length }}</BaseFieldRow>
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

@media (max-width: 640px) {
  .setting-row {
    grid-template-columns: 1fr;
  }

  .select-control {
    justify-self: stretch;
    width: 100%;
  }
}
</style>
