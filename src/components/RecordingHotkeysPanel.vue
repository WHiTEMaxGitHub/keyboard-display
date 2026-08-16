<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { RecordingHotkeyConfig, RecordingHotkeyMode } from "../domain/recordingHotkeys";
import BaseButton from "./BaseButton.vue";
import BaseSelect from "./BaseSelect.vue";

defineProps<{
  recordingHotkeys: RecordingHotkeyConfig;
  hotkeyCaptureTarget: "start" | "stop" | "sync" | null;
}>();

const emit = defineEmits<{
  "update-recording-hotkey-mode": [mode: RecordingHotkeyMode];
  "begin-hotkey-capture": [target: "start" | "stop" | "sync"];
}>();

const { t } = useI18n();

const hotkeyModeOptions: Array<{ value: RecordingHotkeyMode; labelKey: string }> = [
  { value: "disabled", labelKey: "recording.hotkeys.disabled" },
  { value: "toggle", labelKey: "recording.hotkeys.toggle" },
  { value: "separate", labelKey: "recording.hotkeys.separate" },
];

function updateRecordingHotkeyMode(value: string) {
  emit("update-recording-hotkey-mode", value as RecordingHotkeyMode);
}

function beginHotkeyCapture(target: "start" | "stop" | "sync") {
  emit("begin-hotkey-capture", target);
}

function formatHotkey(keys: string[]) {
  return keys.length > 0 ? keys.join(" + ") : t("recording.hotkeys.notSet");
}
</script>

<template>
  <div class="grid gap-2.5 my-4">
    <label class="grid grid-cols-[minmax(112px,1fr)_minmax(180px,240px)] items-center gap-2.5 m-0 text-text-secondary font-bold">
      <span class="text-text-muted text-[13px] font-extrabold">{{ t("recording.hotkeys.mode") }}</span>
      <BaseSelect
        class="select-control justify-self-end w-[min(240px,100%)]"
        :model-value="recordingHotkeys.mode"
        :options="hotkeyModeOptions.map((option) => ({ value: option.value, label: t(option.labelKey) }))"
        @update:model-value="updateRecordingHotkeyMode"
      />
    </label>
    <div v-if="recordingHotkeys.mode !== 'disabled'" class="grid grid-cols-[72px_minmax(0,1fr)_auto] items-center gap-2.5">
      <span class="text-text-muted text-[13px] font-extrabold">{{ t("recording.hotkeys.start") }}</span>
      <strong class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">{{ formatHotkey(recordingHotkeys.start) }}</strong>
      <BaseButton size="sm" @click="beginHotkeyCapture('start')">
        {{ hotkeyCaptureTarget === "start" ? t("recording.hotkeys.pressShortcut") : t("recording.hotkeys.capture") }}
      </BaseButton>
    </div>
    <div v-if="recordingHotkeys.mode === 'separate'" class="grid grid-cols-[72px_minmax(0,1fr)_auto] items-center gap-2.5">
      <span class="text-text-muted text-[13px] font-extrabold">{{ t("recording.hotkeys.stop") }}</span>
      <strong class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">{{ formatHotkey(recordingHotkeys.stop) }}</strong>
      <BaseButton size="sm" @click="beginHotkeyCapture('stop')">
        {{ hotkeyCaptureTarget === "stop" ? t("recording.hotkeys.pressShortcut") : t("recording.hotkeys.capture") }}
      </BaseButton>
    </div>
    <div class="grid grid-cols-[72px_minmax(0,1fr)_auto] items-center gap-2.5">
      <span class="text-text-muted text-[13px] font-extrabold">{{ t("recording.hotkeys.sync") }}</span>
      <strong class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">{{ formatHotkey(recordingHotkeys.sync) }}</strong>
      <BaseButton size="sm" @click="beginHotkeyCapture('sync')">
        {{ hotkeyCaptureTarget === "sync" ? t("recording.hotkeys.pressShortcut") : t("recording.hotkeys.capture") }}
      </BaseButton>
    </div>
  </div>
</template>

<style scoped>
.select-control {
  justify-self: end;
  width: min(240px, 100%);
}
</style>
