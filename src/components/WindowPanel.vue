<script setup lang="ts">
import BaseButton from "./BaseButton.vue";
import BaseFieldRow from "./BaseFieldRow.vue";
import BasePanel from "./BasePanel.vue";
import BaseToggleRow from "./BaseToggleRow.vue";

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
  <BasePanel wide>
    <h2>Window</h2>
    <BaseFieldRow label="Position">{{ overlayPosition }}</BaseFieldRow>
    <div class="adjust-control">
      <span>Visual adjust</span>
      <div class="adjust-actions">
        <BaseButton
          v-if="!overlayAdjusting"
          @click="emit('start-overlay-adjust')"
        >
          Adjust position
        </BaseButton>
        <template v-else>
          <BaseButton variant="primary" @click="emit('save-overlay-adjust')">Save position</BaseButton>
          <BaseButton @click="emit('cancel-overlay-adjust')">Cancel</BaseButton>
        </template>
      </div>
    </div>
    <BaseToggleRow :checked="overlayVisible" @change="emit('update-overlay-visible', $event)">
      Show POV overlay
    </BaseToggleRow>
    <BaseToggleRow :checked="alwaysOnTop" @change="emit('update-always-on-top', $event)">
      Always on top
    </BaseToggleRow>
    <div class="position-control">
      <span>Position</span>
      <div class="position-grid">
        <BaseButton block @click="emit('move-overlay', 'top-left')">Top left</BaseButton>
        <BaseButton block @click="emit('move-overlay', 'top-right')">Top right</BaseButton>
        <BaseButton block @click="emit('move-overlay', 'bottom-left')">Bottom left</BaseButton>
        <BaseButton block @click="emit('move-overlay', 'bottom-right')">Bottom right</BaseButton>
      </div>
    </div>
  </BasePanel>
</template>

<style scoped>
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
