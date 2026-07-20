<script setup lang="ts">
defineProps<{
  overlayPosition: string;
  overlayVisible: boolean;
  alwaysOnTop: boolean;
  overlayAdjusting: boolean;
}>();

const emit = defineEmits<{
  "update-overlay-visible": [event: Event];
  "update-always-on-top": [event: Event];
  "start-overlay-adjust": [];
  "save-overlay-adjust": [];
  "cancel-overlay-adjust": [];
  "move-overlay": [
    position: "top-left" | "top-right" | "bottom-left" | "bottom-right" | "custom",
  ];
}>();
</script>

<template>
  <article class="panel wide-panel">
    <h2>Window</h2>
    <div class="field-row">
      <span>Position</span>
      <strong>{{ overlayPosition }}</strong>
    </div>
    <div class="adjust-control">
      <span>Visual adjust</span>
      <div class="adjust-actions">
        <button
          v-if="!overlayAdjusting"
          type="button"
          @click="emit('start-overlay-adjust')"
        >
          Adjust position
        </button>
        <template v-else>
          <button type="button" @click="emit('save-overlay-adjust')">Save position</button>
          <button type="button" @click="emit('cancel-overlay-adjust')">Cancel</button>
        </template>
      </div>
    </div>
    <label class="toggle-row">
      <input
        :checked="overlayVisible"
        type="checkbox"
        @change="emit('update-overlay-visible', $event)"
      />
      Show POV overlay
    </label>
    <label class="toggle-row">
      <input
        :checked="alwaysOnTop"
        type="checkbox"
        @change="emit('update-always-on-top', $event)"
      />
      Always on top
    </label>
    <div class="position-control">
      <span>Position</span>
      <div class="position-grid">
        <button type="button" @click="emit('move-overlay', 'top-left')">Top left</button>
        <button type="button" @click="emit('move-overlay', 'top-right')">Top right</button>
        <button type="button" @click="emit('move-overlay', 'bottom-left')">Bottom left</button>
        <button type="button" @click="emit('move-overlay', 'bottom-right')">Bottom right</button>
      </div>
    </div>
  </article>
</template>

<style scoped>
.panel {
  min-height: 190px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #171b22;
  padding: 18px;
}

.wide-panel {
  max-width: 760px;
}

.panel h2 {
  margin: 0 0 16px;
  letter-spacing: 0;
}

.field-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.field-row span {
  color: #9ca7b4;
}

.field-row strong {
  min-width: 0;
  overflow-wrap: anywhere;
  text-align: right;
}

.toggle-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
  color: #c9d1da;
  font-weight: 700;
}

.toggle-row input {
  width: 18px;
  height: 18px;
  accent-color: #25d366;
}

.adjust-control {
  display: grid;
  grid-template-columns: minmax(120px, 1fr) minmax(220px, 1.4fr);
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  color: #c9d1da;
  font-weight: 700;
}

.adjust-control > span,
.position-control > span {
  color: #9ca7b4;
  font-size: 13px;
  font-weight: 800;
}

.adjust-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.adjust-actions button,
.position-grid button {
  min-height: 34px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  cursor: pointer;
  font-weight: 700;
  padding: 0 10px;
}

.adjust-actions button:hover,
.position-grid button:hover {
  background: #29313d;
}

.position-control {
  display: grid;
  gap: 8px;
  margin-bottom: 16px;
  color: #c9d1da;
  font-weight: 700;
}

.position-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}
</style>
