<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, reactive, ref, watch } from "vue";
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
  keyIdLabels: Record<string, string>;
}>();

const emit = defineEmits<{
  "update-rows": [rows: OverlayRow[]];
  "update-key-id-labels": [labels: Record<string, string>];
}>();

const collapsedRows = reactive(new Set<number>());
const platformLabelEditors = reactive(new Set<string>());
const captureTarget = ref<{ rowIndex: number; itemIndex: number; currentId: string } | null>(null);
const widthDrafts = reactive(new Map<string, string>());
const textDrafts = reactive(new Map<string, string>());
const platformLabelDrafts = reactive(new Map<string, string>());
const registryLabelDrafts = reactive(new Map<string, string>());
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
    platformLabelDrafts.clear();
    registryLabelDrafts.clear();
    platformLabelEditors.clear();
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
  return `${keyCount} keys · ${gapCount} gaps · ${row.length} items`;
}

function itemSummary(item: OverlayRowItem) {
  return isKeyBinding(item)
    ? `${item.label} · ${item.id} · ${item.widthUnit}u`
    : `${item.type} · ${item.widthUnit}u`;
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

function platformLabelDraftKey(rowIndex: number, itemIndex: number, platform: "macos" | "windows") {
  return `${rowIndex}-${itemIndex}-platform-${platform}`;
}

function platformLabelEditorKey(rowIndex: number, itemIndex: number) {
  return `${rowIndex}-${itemIndex}`;
}

function registryLabelDraftKey(rowIndex: number, itemIndex: number) {
  return `${rowIndex}-${itemIndex}-registry-label`;
}

function isPlatformLabelEditorOpen(rowIndex: number, itemIndex: number) {
  return platformLabelEditors.has(platformLabelEditorKey(rowIndex, itemIndex));
}

function togglePlatformLabelEditor(rowIndex: number, itemIndex: number) {
  const key = platformLabelEditorKey(rowIndex, itemIndex);
  if (platformLabelEditors.has(key)) {
    platformLabelEditors.delete(key);
  } else {
    platformLabelEditors.add(key);
  }
}

function platformLabelDraft(
  rowIndex: number,
  itemIndex: number,
  item: KeyBinding,
  platform: "macos" | "windows",
) {
  return platformLabelDrafts.get(platformLabelDraftKey(rowIndex, itemIndex, platform)) ??
    item.platformLabels?.[platform] ??
    "";
}

function updatePlatformLabelDraft(
  rowIndex: number,
  itemIndex: number,
  platform: "macos" | "windows",
  event: Event,
) {
  platformLabelDrafts.set(
    platformLabelDraftKey(rowIndex, itemIndex, platform),
    (event.target as HTMLInputElement).value,
  );
}

function registryLabelDraft(rowIndex: number, itemIndex: number, item: KeyBinding) {
  return registryLabelDrafts.get(registryLabelDraftKey(rowIndex, itemIndex)) ??
    props.keyIdLabels[item.id] ??
    "";
}

function updateRegistryLabelDraft(rowIndex: number, itemIndex: number, event: Event) {
  registryLabelDrafts.set(
    registryLabelDraftKey(rowIndex, itemIndex),
    (event.target as HTMLInputElement).value,
  );
}

function commitRegistryLabel(rowIndex: number, itemIndex: number, item: KeyBinding) {
  const key = registryLabelDraftKey(rowIndex, itemIndex);
  const value = (registryLabelDrafts.get(key) ?? props.keyIdLabels[item.id] ?? "").trim();
  registryLabelDrafts.delete(key);

  const nextLabels = { ...props.keyIdLabels };
  if (value) {
    nextLabels[item.id] = value;
  } else {
    delete nextLabels[item.id];
  }

  emit("update-key-id-labels", nextLabels);
}

function commitPlatformLabel(
  rowIndex: number,
  itemIndex: number,
  item: KeyBinding,
  platform: "macos" | "windows",
) {
  const key = platformLabelDraftKey(rowIndex, itemIndex, platform);
  const value = (platformLabelDrafts.get(key) ?? item.platformLabels?.[platform] ?? "").trim();
  platformLabelDrafts.delete(key);

  emit("update-rows", updateRowItem(props.rows, rowIndex, itemIndex, {
    ...item,
    platformLabels: {
      ...(item.platformLabels ?? {}),
      [platform]: value || undefined,
    },
  }));
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
      <BaseButton @click="appendRow">Add row</BaseButton>
    </div>
    <article
      v-for="(row, rowIndex) in rows"
      :key="rowIndex"
      class="grid gap-2.5 border border-border-control rounded-lg bg-surface-control p-3 transition-[border-color,background-color,box-shadow,transform] duration-[160ms] ease hover:border-border-default hover:bg-surface-control-hover"
    >
      <div class="flex items-center justify-between gap-3">
        <button class="flex items-center gap-2 min-w-0 border-0 bg-transparent text-text-body cursor-pointer p-0 text-left hover:text-[#f4f7fb]" type="button" @click="toggleRow(rowIndex)">
          <span :class="['inline-grid place-items-center w-3.5 text-[#8f9baa] text-xs leading-none origin-center transition-[color,transform] duration-[160ms,180ms] ease', !collapsedRows.has(rowIndex) && 'text-text-body rotate-90']">▸</span>
          <strong class="text-sm transition-colors duration-[160ms] ease">Row {{ rowIndex + 1 }}</strong>
          <small class="overflow-hidden text-text-muted text-xs font-bold text-ellipsis whitespace-nowrap">{{ rowSummary(row) }}</small>
        </button>
        <div class="flex flex-wrap gap-2">
          <BaseButton size="xs" :disabled="rowIndex === 0" @click="shiftRow(rowIndex, -1)">Up</BaseButton>
          <BaseButton size="xs" :disabled="rowIndex === rows.length - 1" @click="shiftRow(rowIndex, 1)">Down</BaseButton>
          <BaseButton size="xs" @click="addKey(rowIndex)">Add key</BaseButton>
          <BaseButton size="xs" @click="addGap(rowIndex)">Add gap</BaseButton>
          <BaseButton size="xs" variant="danger" :disabled="rows.length <= 1" @click="deleteRow(rowIndex)">Delete row</BaseButton>
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
              <div class="self-center min-w-0 overflow-hidden text-text-body text-xs font-extrabold text-ellipsis whitespace-nowrap">{{ itemSummary(item) }}</div>
              <template v-if="isKeyBinding(item)">
                <label class="grid gap-[5px] m-0 text-text-muted text-xs font-extrabold">
                  ID
                  <input
                    :value="textDraft(rowIndex, itemIndex, 'id', item.id)"
                    autocapitalize="off"
                    autocorrect="off"
                    spellcheck="false"
                    class="min-w-0 h-[34px] border border-border-control rounded-md bg-surface-control text-text-body font-inherit px-[9px]"
                    @blur="commitKeyText(rowIndex, itemIndex, item, 'id')"
                    @change="commitKeyText(rowIndex, itemIndex, item, 'id')"
                    @input="updateTextDraft(rowIndex, itemIndex, 'id', $event)"
                  />
                  <span v-if="idErrors.get(widthDraftKey(rowIndex, itemIndex))" class="text-danger text-[11px] font-bold">
                    {{ idErrors.get(widthDraftKey(rowIndex, itemIndex)) }}
                  </span>
                </label>
                <label class="grid gap-[5px] m-0 text-text-muted text-xs font-extrabold">
                  Label
                  <input
                    :value="textDraft(rowIndex, itemIndex, 'label', item.label)"
                    autocapitalize="off"
                    autocorrect="off"
                    spellcheck="false"
                    class="min-w-0 h-[34px] border border-border-control rounded-md bg-surface-control text-text-body font-inherit px-[9px]"
                    @blur="commitKeyText(rowIndex, itemIndex, item, 'label')"
                    @change="commitKeyText(rowIndex, itemIndex, item, 'label')"
                    @input="updateTextDraft(rowIndex, itemIndex, 'label', $event)"
                  />
                </label>
                <label class="grid gap-[5px] m-0 text-text-muted text-xs font-extrabold">
                  Width
                  <input
                    :value="widthDraft(rowIndex, itemIndex, item.widthUnit)"
                    min="0.1"
                    step="0.05"
                    type="number"
                    class="min-w-0 h-[34px] border border-border-control rounded-md bg-surface-control text-text-body font-inherit px-[9px]"
                    @blur="commitKeyWidth(rowIndex, itemIndex, item)"
                    @change="commitKeyWidth(rowIndex, itemIndex, item)"
                    @input="updateWidthDraft(rowIndex, itemIndex, $event)"
                  />
                </label>
                <BaseButton
                  size="xs"
                  @click="captureTarget?.rowIndex === rowIndex && captureTarget?.itemIndex === itemIndex ? cancelCapture() : beginCapture(rowIndex, itemIndex, item.id)"
                >
                  {{ captureTarget?.rowIndex === rowIndex && captureTarget?.itemIndex === itemIndex ? "Press key..." : "Capture key" }}
                </BaseButton>
                <BaseButton
                  size="xs"
                  @click="togglePlatformLabelEditor(rowIndex, itemIndex)"
                >
                  {{ isPlatformLabelEditorOpen(rowIndex, itemIndex) ? "Hide platform labels" : "Platform labels" }}
                </BaseButton>
                <Transition name="field-reveal">
                  <div
                    v-if="isPlatformLabelEditorOpen(rowIndex, itemIndex)"
                    class="grid col-span-full grid-cols-[repeat(auto-fit,minmax(180px,1fr))] gap-2 border-t border-border-control pt-2"
                  >
                    <label class="grid gap-[5px] m-0 text-text-muted text-xs font-extrabold">
                      macOS label
                      <input
                        :value="platformLabelDraft(rowIndex, itemIndex, item, 'macos')"
                        autocapitalize="off"
                        autocorrect="off"
                        spellcheck="false"
                        class="min-w-0 h-[34px] border border-border-control rounded-md bg-surface-control text-text-body font-inherit px-[9px]"
                        @blur="commitPlatformLabel(rowIndex, itemIndex, item, 'macos')"
                        @change="commitPlatformLabel(rowIndex, itemIndex, item, 'macos')"
                        @input="updatePlatformLabelDraft(rowIndex, itemIndex, 'macos', $event)"
                      />
                    </label>
                    <label class="grid gap-[5px] m-0 text-text-muted text-xs font-extrabold">
                      Windows label
                      <input
                        :value="platformLabelDraft(rowIndex, itemIndex, item, 'windows')"
                        autocapitalize="off"
                        autocorrect="off"
                        spellcheck="false"
                        class="min-w-0 h-[34px] border border-border-control rounded-md bg-surface-control text-text-body font-inherit px-[9px]"
                        @blur="commitPlatformLabel(rowIndex, itemIndex, item, 'windows')"
                        @change="commitPlatformLabel(rowIndex, itemIndex, item, 'windows')"
                        @input="updatePlatformLabelDraft(rowIndex, itemIndex, 'windows', $event)"
                      />
                    </label>
                    <label class="grid gap-[5px] m-0 text-text-muted text-xs font-extrabold">
                      Registered label
                      <input
                        :value="registryLabelDraft(rowIndex, itemIndex, item)"
                        autocapitalize="off"
                        autocorrect="off"
                        spellcheck="false"
                        class="min-w-0 h-[34px] border border-border-control rounded-md bg-surface-control text-text-body font-inherit px-[9px]"
                        @blur="commitRegistryLabel(rowIndex, itemIndex, item)"
                        @change="commitRegistryLabel(rowIndex, itemIndex, item)"
                        @input="updateRegistryLabelDraft(rowIndex, itemIndex, $event)"
                      />
                    </label>
                  </div>
                </Transition>
              </template>
              <template v-else>
                <div class="self-center text-text-muted text-xs font-extrabold uppercase">gap</div>
                <label class="grid gap-[5px] m-0 text-text-muted text-xs font-extrabold">
                  Width
                  <input
                    :value="widthDraft(rowIndex, itemIndex, item.widthUnit)"
                    min="0.1"
                    step="0.05"
                    type="number"
                    class="min-w-0 h-[34px] border border-border-control rounded-md bg-surface-control text-text-body font-inherit px-[9px]"
                    @blur="updateGapWidth(rowIndex, itemIndex, item)"
                    @change="updateGapWidth(rowIndex, itemIndex, item)"
                    @input="updateWidthDraft(rowIndex, itemIndex, $event)"
                  />
                </label>
              </template>
              <BaseButton size="xs" variant="danger" @click="removeItem(rowIndex, itemIndex)">
                Delete
              </BaseButton>
              <div class="flex gap-1.5">
                <BaseButton size="xs" :disabled="itemIndex === 0" @click="shiftItem(rowIndex, itemIndex, -1)">
                  Left
                </BaseButton>
                <BaseButton size="xs" :disabled="itemIndex === row.length - 1" @click="shiftItem(rowIndex, itemIndex, 1)">
                  Right
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
.row-item-editor {
  position: relative;
  display: grid;
  grid-template-columns: minmax(130px, 0.9fr) repeat(3, minmax(0, 1fr)) auto auto auto;
  align-items: end;
  gap: 8px;
  border: 1px solid rgba(120, 140, 170, 0.08);
  border-radius: 7px;
  background: rgba(9, 14, 22, 0.18);
  padding: 10px;
  transition:
    border-color 160ms ease,
    background-color 160ms ease,
    box-shadow 160ms ease,
    transform 160ms ease;
}

.row-item-editor:hover {
  border-color: rgba(120, 140, 170, 0.12);
  background: rgba(12, 18, 28, 0.24);
  transform: translateY(-1px);
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