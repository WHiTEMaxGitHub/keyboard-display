<script setup lang="ts">
import { inject, computed, ref, type ComputedRef } from "vue";
import type { AppConfig } from "../../domain/defaultConfig";
import { isKeyBinding } from "../../domain/defaultConfig";
import BaseFieldRow from "../BaseFieldRow.vue";
import BaseSegmentedControl from "../BaseSegmentedControl.vue";
import PovOverlay from "../PovOverlay.vue";
import LayoutEditor from "../LayoutEditor.vue";

type LayoutSubPage = "summary" | "editor";

const config = inject<AppConfig>("config")!;
const activeKeysRef = inject<ComputedRef<Set<string>>>("activeKeys")!;
const activeKeys = computed(() => activeKeysRef.value);
const keyIdLabels = inject<ComputedRef<AppConfig["keyIdLabels"]>>("keyIdLabels")!;
const emit = inject<(event: string, ...args: unknown[]) => void>("emit")!;

const layoutSubPage = ref<LayoutSubPage>("summary");
const layoutSubPageOptions: Array<{ value: LayoutSubPage; label: string }> = [
  { value: "summary", label: "Summary" },
  { value: "editor", label: "Editor" },
];

const layoutRows = computed(() => {
  return config.rows.map((items, index) => ({ row: index + 1, items }));
});
</script>

<template>
  <section class="page-stack">
    <article class="panel">
      <h2 class="m-0">Layout</h2>
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
      <div v-else class="layout-editor-stack">
        <div class="layout-editor-preview">
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
        <LayoutEditor
          :rows="config.rows"
          :key-id-labels="keyIdLabels"
          @update-key-id-labels="emit('update-key-id-labels', $event)"
          @update-rows="emit('update-overlay-rows', $event)"
        />
      </div>
    </article>
  </section>
</template>

<style scoped>
.page-stack {
  display: grid;
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

.layout-line-list {
  display: grid;
  gap: 10px;
  margin-top: 16px;
}

.layout-editor-stack {
  display: grid;
  gap: 16px;
  margin-top: 16px;
}

.layout-editor-preview {
  --preview-available-width: calc(100vw - 160px);

  display: grid;
  align-items: center;
  justify-items: center;
  min-height: 160px;
  overflow: hidden;
  scrollbar-gutter: stable both-edges;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background: var(--color-surface-preview);
  padding: 14px;
}

.layout-editor-preview :deep(.pov-shell) {
  min-width: 0;
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
</style>
