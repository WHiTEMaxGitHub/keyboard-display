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
}>();

const emit = defineEmits<{
  "update-rows": [rows: OverlayRow[]];
}>();

const collapsedRows = reactive(new Set<number>());
const platformLabelEditors = reactive(new Set<string>());
const captureTarget = ref<{ rowIndex: number; itemIndex: number; currentId: string } | null>(null);
const widthDrafts = reactive(new Map<string, string>());
const textDrafts = reactive(new Map<string, string>());
const platformLabelDrafts = reactive(new Map<string, string>());
const idErrors = reactive(new Map<string, string>());
let unlistenInputState: UnlistenFn | undefined;

watch(
  () => props.rows,
  () => {
    widthDrafts.clear();
    textDrafts.clear();
    platformLabelDrafts.clear();
    platformLabelEditors.clear();
    idErrors.clear();
  },
);

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
  <div class="layout-editor">
    <div class="editor-toolbar">
      <BaseButton @click="appendRow">Add row</BaseButton>
    </div>
    <article
      v-for="(row, rowIndex) in rows"
      :key="rowIndex"
      class="row-editor"
    >
      <div class="row-editor-header">
        <button class="row-title-button" type="button" @click="toggleRow(rowIndex)">
          <span>{{ collapsedRows.has(rowIndex) ? "▸" : "▾" }}</span>
          <strong>Row {{ rowIndex + 1 }}</strong>
          <small>{{ rowSummary(row) }}</small>
        </button>
        <div class="row-actions">
          <BaseButton size="xs" :disabled="rowIndex === 0" @click="shiftRow(rowIndex, -1)">Up</BaseButton>
          <BaseButton size="xs" :disabled="rowIndex === rows.length - 1" @click="shiftRow(rowIndex, 1)">Down</BaseButton>
          <BaseButton size="xs" @click="addKey(rowIndex)">Add key</BaseButton>
          <BaseButton size="xs" @click="addGap(rowIndex)">Add gap</BaseButton>
          <BaseButton size="xs" variant="danger" :disabled="rows.length <= 1" @click="deleteRow(rowIndex)">Delete row</BaseButton>
        </div>
      </div>

      <div v-if="!collapsedRows.has(rowIndex)" class="row-item-list">
        <div
          v-for="(item, itemIndex) in row"
          :key="`${rowIndex}-${itemIndex}`"
          class="row-item-editor"
        >
          <div class="item-summary">{{ itemSummary(item) }}</div>
          <template v-if="isKeyBinding(item)">
            <label>
              ID
              <input
                :value="textDraft(rowIndex, itemIndex, 'id', item.id)"
                autocapitalize="off"
                autocorrect="off"
                spellcheck="false"
                @blur="commitKeyText(rowIndex, itemIndex, item, 'id')"
                @change="commitKeyText(rowIndex, itemIndex, item, 'id')"
                @input="updateTextDraft(rowIndex, itemIndex, 'id', $event)"
              />
              <span v-if="idErrors.get(widthDraftKey(rowIndex, itemIndex))" class="field-error">
                {{ idErrors.get(widthDraftKey(rowIndex, itemIndex)) }}
              </span>
            </label>
            <label>
              Label
              <input
                :value="textDraft(rowIndex, itemIndex, 'label', item.label)"
                autocapitalize="off"
                autocorrect="off"
                spellcheck="false"
                @blur="commitKeyText(rowIndex, itemIndex, item, 'label')"
                @change="commitKeyText(rowIndex, itemIndex, item, 'label')"
                @input="updateTextDraft(rowIndex, itemIndex, 'label', $event)"
              />
            </label>
            <label>
              Width
              <input
                :value="widthDraft(rowIndex, itemIndex, item.widthUnit)"
                min="0.1"
                step="0.05"
                type="number"
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
            <div
              v-if="isPlatformLabelEditorOpen(rowIndex, itemIndex)"
              class="platform-label-fields"
            >
              <label>
                macOS label
                <input
                  :value="platformLabelDraft(rowIndex, itemIndex, item, 'macos')"
                  autocapitalize="off"
                  autocorrect="off"
                  spellcheck="false"
                  @blur="commitPlatformLabel(rowIndex, itemIndex, item, 'macos')"
                  @change="commitPlatformLabel(rowIndex, itemIndex, item, 'macos')"
                  @input="updatePlatformLabelDraft(rowIndex, itemIndex, 'macos', $event)"
                />
              </label>
              <label>
                Windows label
                <input
                  :value="platformLabelDraft(rowIndex, itemIndex, item, 'windows')"
                  autocapitalize="off"
                  autocorrect="off"
                  spellcheck="false"
                  @blur="commitPlatformLabel(rowIndex, itemIndex, item, 'windows')"
                  @change="commitPlatformLabel(rowIndex, itemIndex, item, 'windows')"
                  @input="updatePlatformLabelDraft(rowIndex, itemIndex, 'windows', $event)"
                />
              </label>
            </div>
          </template>
          <template v-else>
            <div class="gap-label">{{ item.type }}</div>
            <label>
              Width
              <input
                :value="widthDraft(rowIndex, itemIndex, item.widthUnit)"
                min="0.1"
                step="0.05"
                type="number"
                @blur="updateGapWidth(rowIndex, itemIndex, item)"
                @change="updateGapWidth(rowIndex, itemIndex, item)"
                @input="updateWidthDraft(rowIndex, itemIndex, $event)"
              />
            </label>
          </template>
          <BaseButton size="xs" variant="danger" @click="removeItem(rowIndex, itemIndex)">
            Delete
          </BaseButton>
          <div class="item-move-actions">
            <BaseButton size="xs" :disabled="itemIndex === 0" @click="shiftItem(rowIndex, itemIndex, -1)">
              Left
            </BaseButton>
            <BaseButton size="xs" :disabled="itemIndex === row.length - 1" @click="shiftItem(rowIndex, itemIndex, 1)">
              Right
            </BaseButton>
          </div>
        </div>
      </div>
    </article>
  </div>
</template>

<style scoped>
.layout-editor {
  display: grid;
  gap: 14px;
}

.editor-toolbar {
  display: flex;
  justify-content: flex-end;
}

.row-editor {
  display: grid;
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  padding: 12px;
}

.row-editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.row-title-button {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
  border: 0;
  background: transparent;
  color: #dfe5ec;
  cursor: pointer;
  padding: 0;
  text-align: left;
}

.row-title-button strong {
  font-size: 14px;
}

.row-title-button small {
  overflow: hidden;
  color: #9ca7b4;
  font-size: 12px;
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.row-item-list {
  display: grid;
  gap: 8px;
}

.row-item-editor {
  display: grid;
  grid-template-columns: minmax(130px, 0.9fr) repeat(3, minmax(0, 1fr)) auto auto auto;
  align-items: end;
  gap: 8px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 7px;
  background: #10141a;
  padding: 10px;
}

.item-summary {
  align-self: center;
  min-width: 0;
  overflow: hidden;
  color: #dfe5ec;
  font-size: 12px;
  font-weight: 800;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-item-editor label {
  display: grid;
  gap: 5px;
  margin: 0;
  color: #9ca7b4;
  font-size: 12px;
  font-weight: 800;
}

.platform-label-fields {
  display: grid;
  grid-column: 1 / -1;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  padding-top: 8px;
}

.row-item-editor input,
.row-item-editor select {
  min-width: 0;
  height: 34px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  font: inherit;
  padding: 0 9px;
}

.row-item-editor select {
  line-height: 34px;
}

.gap-label {
  align-self: center;
  color: #9ca7b4;
  font-size: 12px;
  font-weight: 800;
  text-transform: uppercase;
}

.item-move-actions {
  display: flex;
  gap: 6px;
}

.field-error {
  color: #ff8f8f;
  font-size: 11px;
  font-weight: 700;
}

@media (max-width: 920px) {
  .row-item-editor {
    grid-template-columns: 1fr;
  }
}
</style>
