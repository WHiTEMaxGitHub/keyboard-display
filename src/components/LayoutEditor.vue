<script setup lang="ts">
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

function removeItem(rowIndex: number, itemIndex: number) {
  emit("update-rows", removeRowItem(props.rows, rowIndex, itemIndex));
}

function shiftItem(rowIndex: number, itemIndex: number, offset: -1 | 1) {
  emit("update-rows", moveRowItem(props.rows, rowIndex, itemIndex, itemIndex + offset));
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

function updateGapWidth(
  rowIndex: number,
  itemIndex: number,
  item: GapBinding,
  event: Event,
) {
  emit("update-rows", updateRowItem(props.rows, rowIndex, itemIndex, {
    ...item,
    widthUnit: Number((event.target as HTMLInputElement).value),
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
        <h3>Row {{ rowIndex + 1 }}</h3>
        <div class="row-actions">
          <button type="button" :disabled="rowIndex === 0" @click="shiftRow(rowIndex, -1)">Up</button>
          <button type="button" :disabled="rowIndex === rows.length - 1" @click="shiftRow(rowIndex, 1)">Down</button>
          <button type="button" @click="addKey(rowIndex)">Add key</button>
          <button type="button" @click="addGap(rowIndex)">Add gap</button>
          <button type="button" :disabled="rows.length <= 1" @click="deleteRow(rowIndex)">Delete row</button>
        </div>
      </div>

      <div class="row-item-list">
        <div
          v-for="(item, itemIndex) in row"
          :key="`${rowIndex}-${itemIndex}`"
          class="row-item-editor"
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
                :value="item.widthUnit"
                min="0.1"
                step="0.05"
                type="number"
                @input="updateKeyField(rowIndex, itemIndex, item, 'widthUnit', $event)"
              />
            </label>
          </template>
          <template v-else>
            <div class="gap-label">{{ item.type }}</div>
            <label>
              Width
              <input
                :value="item.widthUnit"
                min="0.1"
                step="0.05"
                type="number"
                @input="updateGapWidth(rowIndex, itemIndex, item, $event)"
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

.row-editor-header h3 {
  margin: 0;
  color: #dfe5ec;
  font-size: 14px;
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

.item-move-actions {
  display: flex;
  gap: 6px;
}

@media (max-width: 920px) {
  .row-item-editor {
    grid-template-columns: 1fr;
  }
}
</style>
