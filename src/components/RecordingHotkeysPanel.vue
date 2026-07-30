<script setup lang="ts">
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

function updateRecordingHotkeyMode(event: Event) {
  emit("update-recording-hotkey-mode", (event.target as HTMLSelectElement).value as RecordingHotkeyMode);
}

function beginHotkeyCapture(target: "start" | "stop" | "sync") {
  emit("begin-hotkey-capture", target);
}

function formatHotkey(keys: string[]) {
  return keys.length > 0 ? keys.join(" + ") : "Not set";
}
</script>

<template>
  <div class="grid gap-2.5 my-4">
    <label class="grid grid-cols-[minmax(112px,1fr)_minmax(180px,240px)] items-center gap-2.5 m-0 text-text-secondary font-bold">
      <span class="text-text-muted text-[13px] font-extrabold">Hotkey mode</span>
      <BaseSelect
        class="select-control justify-self-end w-[min(240px,100%)]"
        :model-value="recordingHotkeys.mode"
        @change="updateRecordingHotkeyMode"
      >
        <option value="disabled">Disabled</option>
        <option value="toggle">Toggle start/stop</option>
        <option value="separate">Separate start/stop</option>
      </BaseSelect>
    </label>
    <div v-if="recordingHotkeys.mode !== 'disabled'" class="grid grid-cols-[72px_minmax(0,1fr)_auto] items-center gap-2.5">
      <span class="text-text-muted text-[13px] font-extrabold">Start</span>
      <strong class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">{{ formatHotkey(recordingHotkeys.start) }}</strong>
      <BaseButton size="sm" @click="beginHotkeyCapture('start')">
        {{ hotkeyCaptureTarget === "start" ? "Press shortcut..." : "Set" }}
      </BaseButton>
    </div>
    <div v-if="recordingHotkeys.mode === 'separate'" class="grid grid-cols-[72px_minmax(0,1fr)_auto] items-center gap-2.5">
      <span class="text-text-muted text-[13px] font-extrabold">Stop</span>
      <strong class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">{{ formatHotkey(recordingHotkeys.stop) }}</strong>
      <BaseButton size="sm" @click="beginHotkeyCapture('stop')">
        {{ hotkeyCaptureTarget === "stop" ? "Press shortcut..." : "Set" }}
      </BaseButton>
    </div>
    <div class="grid grid-cols-[72px_minmax(0,1fr)_auto] items-center gap-2.5">
      <span class="text-text-muted text-[13px] font-extrabold">Sync</span>
      <strong class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">{{ formatHotkey(recordingHotkeys.sync) }}</strong>
      <BaseButton size="sm" @click="beginHotkeyCapture('sync')">
        {{ hotkeyCaptureTarget === "sync" ? "Press shortcut..." : "Set" }}
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