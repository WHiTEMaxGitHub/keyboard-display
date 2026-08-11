<script setup lang="ts">
import { inject, computed, type ComputedRef } from "vue";
import { useI18n } from "vue-i18n";
import type { AppConfig } from "../../domain/defaultConfig";
import PovOverlay from "../PovOverlay.vue";
import BaseFieldRow from "../BaseFieldRow.vue";
import BaseToggleRow from "../BaseToggleRow.vue";

const config = inject<AppConfig>("config")!;
const activeKeysRef = inject<ComputedRef<Set<string>>>("activeKeys")!;
const activeKeys = computed(() => activeKeysRef.value);
const keyIdLabels = inject<ComputedRef<AppConfig["keyIdLabels"]>>("keyIdLabels")!;
const overlayVisibleRef = inject<ComputedRef<boolean>>("overlayVisible")!;
const profileNameRef = inject<ComputedRef<string>>("profileName")!;
const profileChangedRef = inject<ComputedRef<boolean>>("profileChanged")!;
const overlayVisible = computed(() => overlayVisibleRef.value);
const profileName = computed(() => profileNameRef.value);
const profileChanged = computed(() => profileChangedRef.value);
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;
const { t } = useI18n();

function updateOverlayVisible(event: Event) {
  emit("update-overlay-visible", (event.target as HTMLInputElement).checked);
}

function updateAlwaysOnTop(event: Event) {
  emit("update-overlay-style", {
    ...config.style,
    alwaysOnTop: (event.target as HTMLInputElement).checked,
  });
}

</script>

<template>
  <section class="page-stack">
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
        <BaseFieldRow :label="t('overview.name')">{{ profileName }}</BaseFieldRow>
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
}
</style>
