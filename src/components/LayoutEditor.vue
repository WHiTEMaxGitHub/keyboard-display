<script setup lang="ts">
import { reactive, ref, watch } from "vue";
import {
  addRow,
  addGapToRow,
  addKeyToRow,
  moveRow,
  moveRowItem,
  removeRow,
  removeRowItem,
  updateRowItem,
} from "../domain/layoutEditor";
import {
  isKeyBinding,
  type GapBinding,
  type KeyBinding,
  type OverlayRow,
  type OverlayRowItem,
} from "../domain/defaultConfig";

const props = defineProps<{
  rows: OverlayRow[];
}>();

const emit = defineEmits<{
  "update-rows": [rows: OverlayRow[]];
}>();

const keyGroups: KeyBinding["group"][] = ["mouse", "movement", "modifier", "action"];
const collapsedRows = reactive(new Set<number>());
const draggingItem = ref<{ rowIndex: number; itemIndex: number } | null>(null);
const widthDrafts = reactive(new Map<string, string>());

watch(
  () => props.rows,
  () => {
    widthDrafts.clear();
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

function removeItem(rowIndex: number, itemIndex: number) {
  emit("update-rows", removeRowItem(props.rows, rowIndex, itemIndex));
}

function beginDrag(rowIndex: number, itemIndex: number) {
  draggingItem.value = { rowIndex, itemIndex };
}

function dropItem(rowIndex: number, itemIndex: number) {
  if (!draggingItem.value || draggingItem.value.rowIndex !== rowIndex) {
    draggingItem.value = null;
    return;
  }

  emit("update-rows", moveRowItem(props.rows, rowIndex, draggingItem.value.itemIndex, itemIndex));
  draggingItem.value = null;
}

function updateKeyField(
  rowIndex: number,
  itemIndex: number,
  item: KeyBinding,
  field: keyof Pick<KeyBinding, "id" | "label" | "group" | "widthUnit">,
  event: Event,
) {
  const target = event.target as HTMLInputElement | HTMLSelectElement;
  const value = field === "widthUnit" ? Number(target.value) : target.value;
  emit("update-rows", updateRowItem(props.rows, rowIndex, itemIndex, {
    ...item,
    [field]: value,
  } as OverlayRowItem));
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
          @dragover.prevent
          @drop="dropItem(rowIndex, itemIndex)"
        >
          <template v-if="isKeyBinding(item)">
            <label>
              ID
              <input :value="item.id" @input="updateKeyField(rowIndex, itemIndex, item, 'id', $event)" />
            </label>
            <label>
              Label
              <input :value="item.label" @input="updateKeyField(rowIndex, itemIndex, item, 'label', $event)" />
            </label>
            <label>
              Group
              <select :value="item.group" @change="updateKeyField(rowIndex, itemIndex, item, 'group', $event)">
                <option v-for="group in keyGroups" :key="group" :value="group">{{ group }}</option>
              </select>
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
          <span
            class="drag-hint"
            draggable="true"
            @dragstart="beginDrag(rowIndex, itemIndex)"
          >
            Drag
          </span>
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
.drag-hint {
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
.remove-button:hover {
  background: #29313d;
}

.row-actions button:disabled,
.remove-button:disabled {
  cursor: not-allowed;
  opacity: 0.42;
}

.row-item-list {
  display: grid;
  gap: 8px;
}

.row-item-editor {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr)) auto auto;
  align-items: end;
  gap: 8px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 7px;
  background: #10141a;
  padding: 10px;
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

.drag-hint {
  display: grid;
  place-items: center;
  color: #9ca7b4;
  cursor: grab;
  font-size: 12px;
}

@media (max-width: 920px) {
  .row-item-editor {
    grid-template-columns: 1fr;
  }
}
</style>
