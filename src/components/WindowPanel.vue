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
    <h2 class="m-0 mb-4 text-lg leading-6 tracking-normal">Window</h2>
    <BaseFieldRow label="Position">{{ overlayPosition }}</BaseFieldRow>
    <div class="grid grid-cols-[minmax(120px,1fr)_minmax(220px,1.4fr)] items-center gap-3 mb-4 text-text-secondary font-bold">
      <span class="text-text-muted text-[13px] font-extrabold">Visual adjust</span>
      <div class="flex flex-wrap justify-end gap-2">
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
    <div class="grid gap-2 mb-4 text-text-secondary font-bold">
      <span class="text-text-muted text-[13px] font-extrabold">Position</span>
      <div class="grid grid-cols-2 gap-2">
        <BaseButton block @click="emit('move-overlay', 'top-left')">Top left</BaseButton>
        <BaseButton block @click="emit('move-overlay', 'top-right')">Top right</BaseButton>
        <BaseButton block @click="emit('move-overlay', 'bottom-left')">Bottom left</BaseButton>
        <BaseButton block @click="emit('move-overlay', 'bottom-right')">Bottom right</BaseButton>
      </div>
    </div>
  </BasePanel>
</template>