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
  <div class="hotkey-panel">
    <label class="settings-row">
      <span>Hotkey mode</span>
      <BaseSelect
        class="select-control"
        :model-value="recordingHotkeys.mode"
        @change="updateRecordingHotkeyMode"
      >
        <option value="disabled">Disabled</option>
        <option value="toggle">Toggle start/stop</option>
        <option value="separate">Separate start/stop</option>
      </BaseSelect>
    </label>
    <div v-if="recordingHotkeys.mode !== 'disabled'" class="hotkey-row">
      <span>Start</span>
      <strong>{{ formatHotkey(recordingHotkeys.start) }}</strong>
      <BaseButton size="sm" @click="beginHotkeyCapture('start')">
        {{ hotkeyCaptureTarget === "start" ? "Press shortcut..." : "Set" }}
      </BaseButton>
    </div>
    <div v-if="recordingHotkeys.mode === 'separate'" class="hotkey-row">
      <span>Stop</span>
      <strong>{{ formatHotkey(recordingHotkeys.stop) }}</strong>
      <BaseButton size="sm" @click="beginHotkeyCapture('stop')">
        {{ hotkeyCaptureTarget === "stop" ? "Press shortcut..." : "Set" }}
      </BaseButton>
    </div>
    <div class="hotkey-row">
      <span>Sync</span>
      <strong>{{ formatHotkey(recordingHotkeys.sync) }}</strong>
      <BaseButton size="sm" @click="beginHotkeyCapture('sync')">
        {{ hotkeyCaptureTarget === "sync" ? "Press shortcut..." : "Set" }}
      </BaseButton>
    </div>
  </div>
</template>

<style scoped>
.hotkey-panel {
  display: grid;
  gap: 10px;
  margin: 16px 0;
}

.settings-row {
  display: grid;
  grid-template-columns: minmax(112px, 1fr) minmax(180px, 240px);
  align-items: center;
  gap: 10px;
  margin: 0;
}

.settings-row span,
.hotkey-row span {
  color: #9ca7b4;
  font-size: 13px;
  font-weight: 800;
}

.hotkey-row {
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
}

.hotkey-row strong {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.select-control {
  justify-self: end;
  width: min(240px, 100%);
}
</style>
