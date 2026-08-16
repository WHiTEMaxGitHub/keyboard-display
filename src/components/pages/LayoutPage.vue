<script setup lang="ts">
import { inject, computed, type ComputedRef } from "vue";
import { useI18n } from "vue-i18n";
import type { AppConfig } from "../../domain/defaultConfig";
import { isKeyBinding } from "../../domain/defaultConfig";
import BasePanel from "../BasePanel.vue";
import BasePanelHeader from "../BasePanelHeader.vue";
import BaseInput from "../BaseInput.vue";
import PovOverlay from "../PovOverlay.vue";
import LayoutEditor from "../LayoutEditor.vue";

type LayoutSubPage = "summary" | "editor";

const config = inject<AppConfig>("config")!;
const keyIdLabels = inject<ComputedRef<AppConfig["keyIdLabels"]>>("keyIdLabels")!;
const profileNameRef = inject<ComputedRef<string>>("profileName")!;
const profileChangedRef = inject<ComputedRef<boolean>>("profileChanged")!;
const layoutSubPageRef = inject<ComputedRef<LayoutSubPage>>("layoutSubPage")!;
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;
const { t } = useI18n();
const staticActiveKeys = new Set<string>();

const layoutRows = computed(() => {
  return config.rows.map((items, index) => ({ row: index + 1, items }));
});

const layoutSubPage = computed(() => layoutSubPageRef.value);
const profileName = computed(() => profileNameRef.value);
const profileChanged = computed(() => profileChangedRef.value);
const totalUnits = computed(() => {
  return config.rows
    .flat()
    .reduce((sum, item) => sum + item.widthUnit, 0);
});
const totalWidthPx = computed(() => Math.round(
  totalUnits.value * config.layout.unitPx * config.style.scale,
));
const previewOverlayStyle = computed(() => ({
  ...config.style,
  idleKeyVisibility: "visible" as const,
  opacity: 1,
}));

function updateUnitPx(event: Event) {
  const unitPx = Math.max(24, Math.min(120, Math.round(Number((event.target as HTMLInputElement).value))));
  emit("update-overlay-layout", {
    ...config.layout,
    unitPx,
  });
}

function updateGapUnit(event: Event) {
  const gapUnit = Math.max(
    0,
    Math.min(1, Math.round(Number((event.target as HTMLInputElement).value) * 100) / 100),
  );
  emit("update-overlay-layout", {
    ...config.layout,
    gapUnit,
  });
}

function updateProfileName(value: string | number) {
  emit("update-profile-name", String(value));
}
</script>

<template>
  <section class="page-stack">
    <template v-if="layoutSubPage === 'summary'">
      <BasePanel wide>
        <BasePanelHeader
          :eyebrow="t('layout.views.summary')"
          :title="t('layout.title')"
        >
          <template #actions>
            <label class="layout-title-name-field">
              <span>{{ t("layout.configName") }}</span>
              <BaseInput
                block
                :model-value="profileName"
                :placeholder="t('overview.namePlaceholder')"
                @update:model-value="updateProfileName"
              />
            </label>
          </template>
        </BasePanelHeader>

        <div class="layout-metric-grid">
          <div class="layout-metric">
            <span>{{ t("layout.configStatus") }}</span>
            <strong>{{ profileChanged ? t("overview.unsavedChanges") : t("overview.saved") }}</strong>
          </div>
          <div class="layout-metric">
            <span>{{ t("layout.visibleKeys") }}</span>
            <strong>{{ config.keys.length }}</strong>
          </div>
          <div class="layout-metric">
            <span>{{ t("layout.rows") }}</span>
            <strong>{{ config.rows.length }}</strong>
          </div>
          <label class="layout-metric layout-metric-control">
            <span>{{ t("layout.unitSize") }}</span>
            <input
              :value="config.layout.unitPx"
              min="24"
              max="120"
              step="1"
              type="number"
              @blur="updateUnitPx"
              @change="updateUnitPx"
            />
          </label>
          <label class="layout-metric layout-metric-control">
            <span>{{ t("layout.gap") }}</span>
            <input
              :value="config.layout.gapUnit"
              min="0"
              max="1"
              step="0.01"
              type="number"
              @blur="updateGapUnit"
              @change="updateGapUnit"
            />
          </label>
          <div class="layout-metric">
            <span>{{ t("layout.totalWidth") }}</span>
            <strong>{{ totalUnits.toFixed(1) }}{{ t("layout.unitSuffix") }}</strong>
            <small>{{ t("layout.asPx", { px: totalWidthPx }) }}</small>
          </div>
        </div>

        <div class="layout-preview-card">
          <PovOverlay
            :layout="config.layout"
            :rows="config.rows"
            :keys="config.keys"
            :key-id-labels="keyIdLabels"
            :active-keys="staticActiveKeys"
            :overlay-style="previewOverlayStyle"
            fit-to-container
          />
        </div>
      </BasePanel>

      <BasePanel wide>
        <BasePanelHeader :title="t('layout.structure')" />
        <div class="layout-line-list">
          <div v-for="line in layoutRows" :key="line.row" class="layout-line">
            <span class="line-label">{{ t("layout.line", { number: line.row }) }}</span>
            <span class="line-keys">
              <span
                v-for="(item, index) in line.items"
                :key="`${line.row}-${index}`"
                :class="['line-key', { 'line-gap': !isKeyBinding(item) }]"
              >
                {{ isKeyBinding(item) ? item.label : t("layout.gapItem") }} · {{ item.widthUnit }}u
              </span>
            </span>
          </div>
        </div>
      </BasePanel>
    </template>

    <template v-else>
      <BasePanel wide>
        <BasePanelHeader
          :eyebrow="t('layout.views.editor')"
          :title="t('layout.preview')"
        />
        <div class="layout-preview-card">
          <PovOverlay
            :layout="config.layout"
            :rows="config.rows"
            :keys="config.keys"
            :key-id-labels="keyIdLabels"
            :active-keys="staticActiveKeys"
            :overlay-style="previewOverlayStyle"
            fit-to-container
          />
        </div>
      </BasePanel>

      <BasePanel wide>
        <BasePanelHeader :title="t('layout.editorTitle')" />
        <LayoutEditor
          :rows="config.rows"
          @update-rows="emit('update-overlay-rows', $event)"
        />
      </BasePanel>
    </template>
  </section>
</template>

<style scoped>
.page-stack {
  display: grid;
  gap: 16px;
}

.layout-metric-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 150px), 1fr));
  gap: 10px;
  margin-bottom: 16px;
}

.layout-metric {
  position: relative;
  display: grid;
  gap: 5px;
  min-width: 0;
  min-height: 76px;
  border: 1px solid var(--color-border-dim);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--color-surface-control) 72%, transparent);
  padding: 12px;
}

.layout-metric span {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 800;
}

.layout-metric strong {
  color: var(--color-text-primary);
  font-size: 18px;
  line-height: 1.2;
}

.layout-metric small {
  position: absolute;
  right: 12px;
  bottom: 10px;
  color: color-mix(in srgb, var(--color-text-muted) 78%, transparent);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.02em;
}

.layout-metric-control input {
  width: 100%;
  min-height: 30px;
  border: 1px solid var(--color-border-control);
  border-radius: var(--radius-md);
  background: var(--color-surface-control);
  color: var(--color-text-primary);
  font: inherit;
  font-size: 18px;
  font-weight: 900;
  padding: 0 9px;
}

.layout-preview-card {
  --preview-available-width: calc(100vw - 160px);

  display: grid;
  align-items: center;
  justify-items: center;
  min-height: 160px;
  overflow: hidden;
  scrollbar-gutter: stable both-edges;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background:
    linear-gradient(
      135deg,
      color-mix(in srgb, var(--color-surface-control) 42%, transparent),
      color-mix(in srgb, var(--color-surface-base) 38%, transparent)
    );
  padding: 14px;
}

.layout-title-name-field {
  display: grid;
  grid-template-columns: auto minmax(180px, 260px);
  align-items: center;
  gap: 10px;
  margin: 0;
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 900;
}

.layout-preview-card :deep(.pov-shell) {
  min-width: 0;
}

.layout-line-list {
  display: grid;
  gap: 10px;
}

.layout-line {
  display: grid;
  grid-template-columns: 76px minmax(0, 1fr);
  align-items: start;
  gap: 10px;
}

.line-label {
  color: var(--color-text-muted);
  font-size: 13px;
  font-weight: 800;
}

.line-keys {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.line-key {
  border: 1px solid var(--color-border-control);
  border-radius: var(--radius-sm);
  background: var(--color-surface-control);
  padding: 5px 8px;
  color: var(--color-text-body);
  font-size: 13px;
  font-weight: 700;
}

.line-gap {
  color: var(--color-text-muted);
  border-style: dashed;
}

@media (max-width: 720px) {
  .layout-title-name-field {
    grid-template-columns: 1fr;
    justify-self: stretch;
    width: min(100%, 320px);
  }

  .layout-line {
    grid-template-columns: 1fr;
  }
}
</style>
