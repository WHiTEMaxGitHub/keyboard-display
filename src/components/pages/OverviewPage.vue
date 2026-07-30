<script setup lang="ts">
import { inject } from "vue";
import type { AppConfig } from "../../domain/defaultConfig";
import PovOverlay from "../PovOverlay.vue";
import BaseFieldRow from "../BaseFieldRow.vue";
import BaseSelect from "../BaseSelect.vue";
import BaseToggleRow from "../BaseToggleRow.vue";
import type { RecentProfile } from "../../domain/appConfig";

const config = inject<AppConfig>("config")!;
const activeKeys = inject<Set<string>>("activeKeys")!;
const keyIdLabels = inject<AppConfig["keyIdLabels"]>("keyIdLabels")!;
const overlayVisible = inject<boolean>("overlayVisible")!;
const profileName = inject<string>("profileName")!;
const profileChanged = inject<boolean>("profileChanged")!;
const recentProfiles = inject<RecentProfile[]>("recentProfiles")!;
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;

function updateOverlayVisible(event: Event) {
  emit("update-overlay-visible", (event.target as HTMLInputElement).checked);
}

function updateAlwaysOnTop(event: Event) {
  emit("update-overlay-style", {
    ...config.style,
    alwaysOnTop: (event.target as HTMLInputElement).checked,
  });
}

function loadRecentProfile(event: Event) {
  const select = event.target as HTMLSelectElement;
  const path = select.value;
  if (path) {
    emit("load-recent-profile", path);
    select.value = "";
  }
}
</script>

<template>
  <section class="page-stack">
    <section class="preview-band" aria-label="Live preview">
      <div class="preview-copy">
        <p>Live Preview</p>
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
        />
      </div>
    </section>

    <section class="panel-grid">
      <article class="panel">
        <h2 class="m-0">Profile</h2>
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
        <h2 class="m-0">Quick controls</h2>
        <BaseToggleRow :checked="overlayVisible" @change="updateOverlayVisible">
          Show POV overlay
        </BaseToggleRow>
        <BaseToggleRow :checked="config.style.alwaysOnTop" @change="updateAlwaysOnTop">
          Always on top
        </BaseToggleRow>
      </article>
    </section>
  </section>
</template>

<style scoped>
.page-stack {
  display: grid;
  gap: 16px;
}

.preview-band {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  min-width: 0;
  min-height: 250px;
  margin-bottom: 20px;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background:
    linear-gradient(90deg, rgba(37, 211, 102, 0.1), transparent 44%),
    var(--color-surface-overlay);
  padding: 24px;
}

.preview-copy {
  flex: 0 0 180px;
}

.preview-copy p {
  margin: 0 0 4px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-text-muted);
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

.panel-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.panel {
  box-sizing: border-box;
  min-height: 190px;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background: var(--color-surface-panel);
  padding: 18px;
}

.panel h2 {
  margin-bottom: 16px;
  font-size: 18px;
  line-height: 24px;
  letter-spacing: 0;
}

.recent-profile-control {
  display: grid;
  grid-template-columns: minmax(110px, 1fr) minmax(180px, 240px);
  align-items: center;
  gap: 7px;
  margin-top: 14px;
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 700;
}

.select-control {
  justify-self: end;
  width: min(240px, 100%);
}

@media (max-width: 920px) {
  .preview-band {
    align-items: flex-start;
    flex-direction: column;
  }

  .panel-grid {
    grid-template-columns: 1fr;
  }
}
</style>