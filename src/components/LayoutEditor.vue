<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import BaseButton from "./BaseButton.vue";
import {
  addRow,
  addGapToRow,
  addKeyToRow,
  moveRow,
  moveRowItem,
  removeRow,
  removeRowItem,
  updateRowItem,
  validateKeyId,
} from "../domain/layoutEditor";
import {
  isKeyBinding,
  type GapBinding,
  type KeyBinding,
  type OverlayRow,
  type OverlayRowItem,
} from "../domain/defaultConfig";
import { INPUT_STATE_EVENT, type InputStatePayload } from "../domain/inputEvents";

const props = defineProps<{
  rows: OverlayRow[];
}>();

const emit = defineEmits<{
  "update-rows": [rows: OverlayRow[]];
}>();

const { t } = useI18n();
const collapsedRows = reactive(new Set<number>());
const captureTarget = ref<{ rowIndex: number; itemIndex: number; currentId: string } | null>(null);
const widthDrafts = reactive(new Map<string, string>());
const textDrafts = reactive(new Map<string, string>());
const idErrors = reactive(new Map<string, string>());
let unlistenInputState: UnlistenFn | undefined;
let collapsedRowsInitialized = false;
let previousRowCount = 0;

watch(
  () => props.rows,
  (rows) => {
    syncCollapsedRows(rows);
    widthDrafts.clear();
    textDrafts.clear();
    idErrors.clear();
  },
  { immediate: true },
);

function syncCollapsedRows(rows: OverlayRow[]) {
  if (!collapsedRowsInitialized) {
    rows.forEach((_row, rowIndex) => collapsedRows.add(rowIndex));
    collapsedRowsInitialized = true;
    previousRowCount = rows.length;
    return;
  }

  Array.from(collapsedRows)
    .filter((rowIndex) => rowIndex >= rows.length)
    .forEach((rowIndex) => collapsedRows.delete(rowIndex));

  if (rows.length > previousRowCount) {
    rows.forEach((_row, rowIndex) => {
      if (rowIndex >= previousRowCount) {
        collapsedRows.add(rowIndex);
      }
    });
  }

  previousRowCount = rows.length;
}

function addKey(rowIndex: number) {
  emit("update-rows", addKeyToRow(props.rows, rowIndex));
}

function addGap(rowIndex: number) {
  emit("update-rows", addGapToRow(props.rows, rowIndex));
}

function appendRow() {
  emit("update-rows", addRow(props.rows));
}

function deleteRow(rowIndex: number) {
  emit("update-rows", removeRow(props.rows, rowIndex));
}

function shiftRow(rowIndex: number, offset: -1 | 1) {
  emit("update-rows", moveRow(props.rows, rowIndex, rowIndex + offset));
}

function toggleRow(rowIndex: number) {
  if (collapsedRows.has(rowIndex)) {
    collapsedRows.delete(rowIndex);
  } else {
    collapsedRows.add(rowIndex);
  }
}

function rowSummary(row: OverlayRow) {
  const keyCount = row.filter(isKeyBinding).length;
  const gapCount = row.length - keyCount;
  return t("layout.editorPanel.rowSummary", {
    keyCount,
    gapCount,
    itemCount: row.length,
  });
}

function itemSummary(item: OverlayRowItem) {
  return isKeyBinding(item)
    ? t("layout.editorPanel.itemSummary", {
      label: item.label,
      id: item.id,
      widthUnit: item.widthUnit,
    })
    : t("layout.editorPanel.gapSummary", { widthUnit: item.widthUnit });
}

function removeItem(rowIndex: number, itemIndex: number) {
  emit("update-rows", removeRowItem(props.rows, rowIndex, itemIndex));
}

function shiftItem(rowIndex: number, itemIndex: number, offset: -1 | 1) {
  emit("update-rows", moveRowItem(props.rows, rowIndex, itemIndex, itemIndex + offset));
}

function beginCapture(rowIndex: number, itemIndex: number, currentId: string) {
  captureTarget.value = { rowIndex, itemIndex, currentId };
}

function cancelCapture() {
  captureTarget.value = null;
}

function commitCapturedKey(capturedId: string) {
  const target = captureTarget.value;
  if (!target) {
    return;
  }

  const item = props.rows[target.rowIndex]?.[target.itemIndex];
  if (!item || !isKeyBinding(item)) {
    captureTarget.value = null;
    return;
  }

  const error = validateKeyId(capturedId, props.rows, target.currentId);
  if (error) {
    idErrors.set(widthDraftKey(target.rowIndex, target.itemIndex), error);
    captureTarget.value = null;
    return;
  }

  idErrors.delete(widthDraftKey(target.rowIndex, target.itemIndex));
  emit("update-rows", updateRowItem(props.rows, target.rowIndex, target.itemIndex, {
    ...item,
    id: capturedId,
  }));
  captureTarget.value = null;
}

onMounted(async () => {
  unlistenInputState = await listen<InputStatePayload>(
    INPUT_STATE_EVENT,
    (event) => {
      if (event.payload.pressed) {
        commitCapturedKey(event.payload.keyId);
      }
    },
  );
});

onUnmounted(() => {
  unlistenInputState?.();
});

function textDraftKey(
  rowIndex: number,
  itemIndex: number,
  field: "id" | "label",
) {
  return `${rowIndex}-${itemIndex}-${field}`;
}

function textDraft(
  rowIndex: number,
  itemIndex: number,
  field: "id" | "label",
  value: string,
) {
  return textDrafts.get(textDraftKey(rowIndex, itemIndex, field)) ?? value;
}

function updateTextDraft(
  rowIndex: number,
  itemIndex: number,
  field: "id" | "label",
  event: Event,
) {
  textDrafts.set(textDraftKey(rowIndex, itemIndex, field), (event.target as HTMLInputElement).value);
}

function commitKeyText(
  rowIndex: number,
  itemIndex: number,
  item: KeyBinding,
  field: "id" | "label",
) {
  const key = textDraftKey(rowIndex, itemIndex, field);
  const value = (textDrafts.get(key) ?? item[field]).trim();

  if (field === "id") {
    const error = validateKeyId(value, props.rows, item.id);
    if (error) {
      idErrors.set(widthDraftKey(rowIndex, itemIndex), error);
      return;
    }
    idErrors.delete(widthDraftKey(rowIndex, itemIndex));
  }

  textDrafts.delete(key);
  const nextItem = {
    ...item,
    [field]: value,
  };

  emit("update-rows", updateRowItem(props.rows, rowIndex, itemIndex, nextItem));
}

function widthDraftKey(rowIndex: number, itemIndex: number) {
  return `${rowIndex}-${itemIndex}`;
}

function widthDraft(rowIndex: number, itemIndex: number, value: number) {
  return widthDrafts.get(widthDraftKey(rowIndex, itemIndex)) ?? String(value);
}

function updateWidthDraft(rowIndex: number, itemIndex: number, event: Event) {
  widthDrafts.set(widthDraftKey(rowIndex, itemIndex), (event.target as HTMLInputElement).value);
}

function commitKeyWidth(rowIndex: number, itemIndex: number, item: KeyBinding) {
  const key = widthDraftKey(rowIndex, itemIndex);
  const widthUnit = Math.max(0.1, Number(widthDrafts.get(key) ?? item.widthUnit));
  widthDrafts.delete(key);
  emit("update-rows", updateRowItem(props.rows, rowIndex, itemIndex, {
    ...item,
    widthUnit,
  }));
}

function updateGapWidth(
  rowIndex: number,
  itemIndex: number,
  item: GapBinding,
) {
  const key = widthDraftKey(rowIndex, itemIndex);
  const widthUnit = Math.max(0.1, Number(widthDrafts.get(key) ?? item.widthUnit));
  widthDrafts.delete(key);
  emit("update-rows", updateRowItem(props.rows, rowIndex, itemIndex, {
    ...item,
    widthUnit,
  }));
}
</script>

<template>
  <div class="grid gap-3.5">
    <div class="flex justify-end">
      <BaseButton @click="appendRow">{{ t("layout.editor.addRow") }}</BaseButton>
    </div>
    <article
      v-for="(row, rowIndex) in rows"
      :key="rowIndex"
      class="layout-row-card"
    >
      <div class="layout-row-header">
        <button class="flex items-center gap-2 min-w-0 border-0 bg-transparent text-text-body cursor-pointer p-0 text-left hover:text-[#f4f7fb]" type="button" @click="toggleRow(rowIndex)">
          <span :class="['inline-grid place-items-center w-3.5 text-[#8f9baa] text-xs leading-none origin-center transition-[color,transform] duration-[160ms,180ms] ease', !collapsedRows.has(rowIndex) && 'text-text-body rotate-90']">▸</span>
          <strong class="text-sm transition-colors duration-[160ms] ease">{{ t("layout.editorPanel.row", { number: rowIndex + 1 }) }}</strong>
          <small class="overflow-hidden text-text-muted text-xs font-bold text-ellipsis whitespace-nowrap">{{ rowSummary(row) }}</small>
        </button>
        <div class="layout-row-actions">
          <BaseButton size="xs" :disabled="rowIndex === 0" @click="shiftRow(rowIndex, -1)">{{ t("layout.editor.up") }}</BaseButton>
          <BaseButton size="xs" :disabled="rowIndex === rows.length - 1" @click="shiftRow(rowIndex, 1)">{{ t("layout.editor.down") }}</BaseButton>
          <BaseButton size="xs" @click="addKey(rowIndex)">{{ t("layout.editor.addKey") }}</BaseButton>
          <BaseButton size="xs" @click="addGap(rowIndex)">{{ t("layout.editor.addGap") }}</BaseButton>
          <BaseButton size="xs" variant="danger" :disabled="rows.length <= 1" @click="deleteRow(rowIndex)">{{ t("layout.editor.deleteRow") }}</BaseButton>
        </div>
      </div>

      <Transition name="row-collapse">
        <div v-if="!collapsedRows.has(rowIndex)" class="overflow-hidden">
          <TransitionGroup name="row-item" tag="div" class="grid gap-2">
            <div
              v-for="(item, itemIndex) in row"
              :key="`${rowIndex}-${itemIndex}`"
              class="row-item-editor"
            >
              <div class="row-item-summary">{{ itemSummary(item) }}</div>
              <template v-if="isKeyBinding(item)">
                <label class="compact-field key-id-field">
                  <span>{{ t("layout.editor.id") }}</span>
                  <input
                    :value="textDraft(rowIndex, itemIndex, 'id', item.id)"
                    autocapitalize="off"
                    autocorrect="off"
                    spellcheck="false"
                    class="compact-input"
                    @blur="commitKeyText(rowIndex, itemIndex, item, 'id')"
                    @change="commitKeyText(rowIndex, itemIndex, item, 'id')"
                    @input="updateTextDraft(rowIndex, itemIndex, 'id', $event)"
                  />
                  <span v-if="idErrors.get(widthDraftKey(rowIndex, itemIndex))" class="text-danger text-[11px] font-bold">
                    {{ idErrors.get(widthDraftKey(rowIndex, itemIndex)) }}
                  </span>
                </label>
                <label class="compact-field display-name-field">
                  <span>{{ t("layout.editor.label") }}</span>
                  <input
                    :value="textDraft(rowIndex, itemIndex, 'label', item.label)"
                    autocapitalize="off"
                    autocorrect="off"
                    spellcheck="false"
                    class="compact-input"
                    @blur="commitKeyText(rowIndex, itemIndex, item, 'label')"
                    @change="commitKeyText(rowIndex, itemIndex, item, 'label')"
                    @input="updateTextDraft(rowIndex, itemIndex, 'label', $event)"
                  />
                </label>
                <label class="compact-field width-field">
                  <span>{{ t("layout.editor.width") }}</span>
                  <input
                    :value="widthDraft(rowIndex, itemIndex, item.widthUnit)"
                    min="0.1"
                    step="0.05"
                    type="number"
                    class="compact-input"
                    @blur="commitKeyWidth(rowIndex, itemIndex, item)"
                    @change="commitKeyWidth(rowIndex, itemIndex, item)"
                    @input="updateWidthDraft(rowIndex, itemIndex, $event)"
                  />
                </label>
                <div class="row-item-actions">
                  <BaseButton
                    size="xs"
                    @click="captureTarget?.rowIndex === rowIndex && captureTarget?.itemIndex === itemIndex ? cancelCapture() : beginCapture(rowIndex, itemIndex, item.id)"
                  >
                    {{ captureTarget?.rowIndex === rowIndex && captureTarget?.itemIndex === itemIndex ? t("layout.editor.pressKey") : t("layout.editor.captureKey") }}
                  </BaseButton>
                </div>
              </template>
              <template v-else>
                <div class="row-item-summary row-item-summary-gap">{{ t("layout.gapItem") }} · {{ item.widthUnit }}u</div>
                <label class="compact-field width-field">
                  <span>{{ t("layout.editor.width") }}</span>
                  <input
                    :value="widthDraft(rowIndex, itemIndex, item.widthUnit)"
                    min="0.1"
                    step="0.05"
                    type="number"
                    class="compact-input"
                    @blur="updateGapWidth(rowIndex, itemIndex, item)"
                    @change="updateGapWidth(rowIndex, itemIndex, item)"
                    @input="updateWidthDraft(rowIndex, itemIndex, $event)"
                  />
                </label>
              </template>
              <div class="row-item-actions row-item-actions-tail">
                <BaseButton size="xs" :disabled="itemIndex === 0" @click="shiftItem(rowIndex, itemIndex, -1)">
                  {{ t("layout.editor.left") }}
                </BaseButton>
                <BaseButton size="xs" :disabled="itemIndex === row.length - 1" @click="shiftItem(rowIndex, itemIndex, 1)">
                  {{ t("layout.editor.right") }}
                </BaseButton>
                <BaseButton size="xs" variant="danger" @click="removeItem(rowIndex, itemIndex)">
                  {{ t("layout.editor.delete") }}
                </BaseButton>
              </div>
            </div>
          </TransitionGroup>
        </div>
      </Transition>
    </article>
  </div>
</template>

<style scoped>
.layout-row-card {
  display: grid;
  gap: 10px;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-xl);
  background: color-mix(in srgb, var(--color-surface-control) 58%, transparent);
  padding: 12px;
  transition:
    border-color 160ms ease,
    background-color 160ms ease,
    box-shadow 160ms ease;
}

.layout-row-card:hover {
  border-color: var(--color-border-control);
  background: color-mix(in srgb, var(--color-surface-control-hover) 52%, transparent);
}

.layout-row-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.layout-row-actions,
.row-item-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.layout-row-actions {
  justify-content: flex-end;
}

.row-item-editor {
  position: relative;
  display: grid;
  grid-template-columns:
    minmax(clamp(96px, 11vw, 132px), 0.75fr)
    minmax(clamp(112px, 13vw, 150px), 1fr)
    minmax(clamp(96px, 11vw, 120px), 0.82fr)
    clamp(72px, 8vw, 82px)
    auto
    auto;
  align-items: center;
  gap: 8px;
  border: 1px solid var(--color-border-dim);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--color-surface-base) 54%, transparent);
  padding: 11px;
  transition:
    border-color 160ms ease,
    background-color 160ms ease,
    box-shadow 160ms ease,
    transform 160ms ease;
}

.row-item-editor:hover {
  border-color: var(--color-border-default);
  background: color-mix(in srgb, var(--color-surface-control) 48%, transparent);
  transform: translateY(-1px);
}

.row-item-summary {
  min-width: 0;
  overflow: hidden;
  border-right: 1px solid var(--color-border-dim);
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 900;
  letter-spacing: 0.02em;
  padding-right: 8px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-item-summary-gap {
  color: var(--color-text-muted);
}

.row-item-actions {
  align-self: end;
  justify-content: flex-end;
}

.row-item-actions-tail {
  justify-self: end;
}

.compact-field {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 7px;
  min-width: 0;
  margin: 0;
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 900;
  white-space: nowrap;
}

.compact-field > span {
  overflow: hidden;
  text-overflow: ellipsis;
}

.compact-input {
  min-width: 0;
  height: 30px;
  border: 1px solid var(--color-border-control);
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--color-surface-control) 84%, transparent);
  color: var(--color-text-body);
  font: inherit;
  font-size: 12px;
  font-weight: 800;
  padding: 0 8px;
}

.width-field {
  grid-template-columns: auto 48px;
}

.row-collapse-enter-active,
.row-collapse-leave-active {
  max-height: 1800px;
  opacity: 1;
  transition:
    max-height 220ms ease,
    opacity 180ms ease,
    transform 180ms ease;
}

.row-collapse-enter-from,
.row-collapse-leave-to {
  max-height: 0;
  opacity: 0;
  transform: translateY(-4px);
}

.field-reveal-enter-active,
.field-reveal-leave-active {
  max-height: 120px;
  opacity: 1;
  transition:
    max-height 180ms ease,
    opacity 150ms ease,
    transform 150ms ease;
}

.field-reveal-enter-from,
.field-reveal-leave-to {
  max-height: 0;
  opacity: 0;
  transform: translateY(-4px);
}

.row-item-enter-active,
.row-item-leave-active,
.row-item-move {
  transition:
    opacity 160ms ease,
    transform 180ms ease;
}

.row-item-enter-from,
.row-item-leave-to {
  opacity: 0;
  transform: translateY(-5px);
}

.row-item-leave-active {
  position: absolute;
  width: 100%;
}

@media (max-width: 920px) {
  .row-item-editor {
    grid-template-columns: 1fr;
    align-items: stretch;
  }

  .row-item-summary {
    border-right: 0;
    border-bottom: 1px solid var(--color-border-dim);
    padding-right: 0;
    padding-bottom: 8px;
  }

  .layout-row-header {
    align-items: stretch;
    flex-direction: column;
  }

  .layout-row-actions,
  .row-item-actions,
  .row-item-actions-tail {
    justify-content: flex-start;
  }
}

@media (prefers-reduced-motion: reduce) {
  .row-item-editor,
  .row-collapse-enter-active,
  .row-collapse-leave-active,
  .field-reveal-enter-active,
  .field-reveal-leave-active,
  .row-item-enter-active,
  .row-item-leave-active,
  .row-item-move {
    transition: none;
  }
}
</style>
