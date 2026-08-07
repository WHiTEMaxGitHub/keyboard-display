<script setup lang="ts">
import { useI18n } from "vue-i18n";
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

const { t } = useI18n();
</script>

<template>
  <BasePanel wide>
    <h2 class="m-0 mb-4 text-lg leading-6 tracking-normal">{{ t("window.title") }}</h2>
    <BaseFieldRow :label="t('window.positionLabel')">{{ overlayPosition }}</BaseFieldRow>
    <div class="grid grid-cols-[minmax(120px,1fr)_minmax(220px,1.4fr)] items-center gap-3 mb-4 text-text-secondary font-bold">
      <span class="text-text-muted text-[13px] font-extrabold">{{ t("window.visualAdjust") }}</span>
      <div class="flex flex-wrap justify-end gap-2">
        <BaseButton
          v-if="!overlayAdjusting"
          @click="emit('start-overlay-adjust')"
        >
          {{ t("window.adjust.start") }}
        </BaseButton>
        <template v-else>
          <BaseButton variant="primary" @click="emit('save-overlay-adjust')">{{ t("window.adjust.savePosition") }}</BaseButton>
          <BaseButton @click="emit('cancel-overlay-adjust')">{{ t("window.adjust.cancel") }}</BaseButton>
        </template>
      </div>
    </div>
    <BaseToggleRow :checked="overlayVisible" @change="emit('update-overlay-visible', $event)">
      {{ t("window.showOverlay") }}
    </BaseToggleRow>
    <BaseToggleRow :checked="alwaysOnTop" @change="emit('update-always-on-top', $event)">
      {{ t("window.alwaysOnTop") }}
    </BaseToggleRow>
    <div class="grid gap-2 mb-4 text-text-secondary font-bold">
      <span class="text-text-muted text-[13px] font-extrabold">{{ t("window.positionLabel") }}</span>
      <div class="grid grid-cols-2 gap-2">
        <BaseButton block @click="emit('move-overlay', 'top-left')">{{ t("window.position.topLeft") }}</BaseButton>
        <BaseButton block @click="emit('move-overlay', 'top-right')">{{ t("window.position.topRight") }}</BaseButton>
        <BaseButton block @click="emit('move-overlay', 'bottom-left')">{{ t("window.position.bottomLeft") }}</BaseButton>
        <BaseButton block @click="emit('move-overlay', 'bottom-right')">{{ t("window.position.bottomRight") }}</BaseButton>
      </div>
    </div>
  </BasePanel>
</template>
