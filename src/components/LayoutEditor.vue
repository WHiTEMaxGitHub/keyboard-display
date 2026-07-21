<script setup lang="ts">
import { reactive, watch } from "vue";
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
import { detectPlatformKey } from "../domain/keyLabels";

const props = defineProps<{
  rows: OverlayRow[];
}>();

const emit = defineEmits<{
  "update-rows": [rows: OverlayRow[]];
}>();

const collapsedRows = reactive(new Set<number>());
const widthDrafts = reactive(new Map<string, string>());
const textDrafts = reactive(new Map<string, string>());
const idErrors = reactive(new Map<string, string>());
const platformKey = detectPlatformKey();

watch(
  () => props.rows,
  () => {
    widthDrafts.clear();
    textDrafts.clear();
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
  if (field === "label" && platformKey !== "default") {
    nextItem.platformLabels = {
      ...(item.platformLabels ?? {}),
      [platformKey]: value,
    };
  }

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
      <button type="button" @click="appendRow">Add row</button>
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
          <button type="button" :disabled="rowIndex === 0" @click="shiftRow(rowIndex, -1)">Up</button>
          <button type="button" :disabled="rowIndex === rows.length - 1" @click="shiftRow(rowIndex, 1)">Down</button>
          <button type="button" @click="addKey(rowIndex)">Add key</button>
          <button type="button" @click="addGap(rowIndex)">Add gap</button>
          <button type="button" :disabled="rows.length <= 1" @click="deleteRow(rowIndex)">Delete row</button>
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
          <button class="remove-button" type="button" @click="removeItem(rowIndex, itemIndex)">
            Delete
          </button>
          <div class="item-move-actions">
            <button type="button" :disabled="itemIndex === 0" @click="shiftItem(rowIndex, itemIndex, -1)">
              Left
            </button>
            <button type="button" :disabled="itemIndex === row.length - 1" @click="shiftItem(rowIndex, itemIndex, 1)">
              Right
            </button>
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

.editor-toolbar button {
  min-height: 32px;
  border: 1px solid rgba(37, 211, 102, 0.3);
  border-radius: 7px;
  background: rgba(37, 211, 102, 0.12);
  color: #eafff0;
  cursor: pointer;
  font-weight: 800;
  padding: 0 10px;
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

.row-actions button,
.remove-button,
.item-move-actions button {
  min-height: 30px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  cursor: pointer;
  font-weight: 700;
  padding: 0 9px;
}

.row-actions button:hover,
.remove-button:hover,
.item-move-actions button:hover {
  background: #29313d;
}

.row-actions button:disabled,
.remove-button:disabled,
.item-move-actions button:disabled {
  cursor: not-allowed;
  opacity: 0.42;
}

.row-item-list {
  display: grid;
  gap: 8px;
}

.row-item-editor {
  display: grid;
  grid-template-columns: minmax(130px, 0.9fr) repeat(3, minmax(0, 1fr)) auto auto;
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

.remove-button {
  color: #ffb3b3;
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
