<script setup lang="ts">
import { ref, watch } from "vue";
import { tauriApi } from "../api/tauri";
import type { RecordingMetadata } from "../types/recording";

const props = defineProps<{
  path: string;
}>();

const emit = defineEmits<{
  saved: [];
  close: [];
}>();

const metadataDraft = ref<RecordingMetadata>(createEmptyMetadata());
const metadataTagsDraft = ref("");
const metadataStatus = ref("");
const metadataError = ref("");
const metadataSaving = ref(false);

watch(
  () => props.path,
  (path) => {
    if (path) {
      void loadRecordingMetadata(path);
    } else {
      setMetadataDraft(createEmptyMetadata());
    }
  },
  { immediate: true },
);

async function loadRecordingMetadata(path: string) {
  metadataStatus.value = "";
  metadataError.value = "";

  try {
    const metadata = await tauriApi.readRecordingMetadata(path);
    setMetadataDraft(metadata);
  } catch (error) {
    metadataError.value = String(error);
    setMetadataDraft(createEmptyMetadata());
  }
}

async function saveRecordingMetadata() {
  if (!props.path) {
    metadataError.value = "Choose a recording file first.";
    return;
  }

  metadataSaving.value = true;
  metadataError.value = "";
  metadataStatus.value = "";

  try {
    const metadata = await tauriApi.saveRecordingMetadata(props.path, metadataFromDraft());
    setMetadataDraft(metadata);
    metadataStatus.value = "Metadata saved.";
    emit("saved");
  } catch (error) {
    metadataError.value = String(error);
  } finally {
    metadataSaving.value = false;
  }
}

function setMetadataDraft(metadata: RecordingMetadata) {
  metadataDraft.value = {
    displayName: metadata.displayName,
    description: metadata.description,
    tags: [...metadata.tags],
    markerNotes: metadata.markerNotes.map((markerNote) => ({ ...markerNote })),
  };
  metadataTagsDraft.value = metadata.tags.join(", ");
}

/// 将表单草稿转换成 sidecar 存储结构，并清理空 tag。
function metadataFromDraft(): RecordingMetadata {
  return {
    displayName: metadataDraft.value.displayName,
    description: metadataDraft.value.description,
    tags: metadataTagsDraft.value
      .split(",")
      .map((tag) => tag.trim())
      .filter(Boolean),
    markerNotes: metadataDraft.value.markerNotes.map((markerNote) => ({ ...markerNote })),
  };
}

function createEmptyMetadata(): RecordingMetadata {
  return {
    displayName: "",
    description: "",
    tags: [],
    markerNotes: [],
  };
}
</script>

<template>
  <div class="metadata-modal" role="dialog" aria-modal="true" @click.self="emit('close')">
    <section class="metadata-editor" @click.stop>
      <div class="section-header">
        <div>
          <p>Recording metadata</p>
          <h3>Sidecar metadata</h3>
        </div>
        <div class="header-actions">
          <button type="button" :disabled="metadataSaving" @click="emit('close')">
            Close
          </button>
          <button type="button" :disabled="metadataSaving" @click="saveRecordingMetadata">
            {{ metadataSaving ? "Saving..." : "Save metadata" }}
          </button>
        </div>
      </div>
      <p class="file-path">{{ path }}</p>
      <label>
        <span>Display name</span>
        <input v-model="metadataDraft.displayName" type="text" placeholder="Browser display name" />
      </label>
      <label>
        <span>Description</span>
        <textarea v-model="metadataDraft.description" rows="3" placeholder="Notes for this recording" />
      </label>
      <label>
        <span>Tags</span>
        <input v-model="metadataTagsDraft" type="text" placeholder="sync, ranked, aim" />
      </label>
      <p v-if="metadataStatus" class="status-text">{{ metadataStatus }}</p>
      <p v-if="metadataError" class="error-text">{{ metadataError }}</p>
    </section>
  </div>
</template>

<style scoped>
.metadata-modal {
  position: fixed;
  inset: 0;
  z-index: 18;
  display: grid;
  align-items: start;
  justify-items: end;
  overflow: auto;
  background: rgba(0, 0, 0, 0.38);
  padding: 24px;
}

.section-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  position: sticky;
  top: 0;
  z-index: 1;
  margin: -14px -14px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: #151a20;
  padding: 14px;
}

.header-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.section-header p,
.section-header h3 {
  margin: 0;
}

.section-header p {
  color: #9ca7b4;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.section-header h3 {
  margin-top: 2px;
  font-size: 16px;
  letter-spacing: 0;
  line-height: 22px;
}

.section-header button {
  min-height: 34px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #202630;
  color: #dfe5ec;
  cursor: pointer;
  font-weight: 700;
  padding: 0 10px;
}

.section-header button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.metadata-editor {
  display: grid;
  width: min(520px, calc(100vw - 48px));
  gap: 12px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  box-shadow: 0 18px 48px rgba(0, 0, 0, 0.34);
  padding: 14px;
}

.file-path {
  margin: 0;
  overflow-wrap: anywhere;
  color: #9ca7b4;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.metadata-editor label {
  display: grid;
  gap: 6px;
  color: #c9d1da;
  font-size: 13px;
  font-weight: 700;
}

.metadata-editor input,
.metadata-editor textarea {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #10141a;
  color: #dfe5ec;
  font: inherit;
  padding: 9px 10px;
}

.metadata-editor textarea {
  resize: vertical;
}

.error-text {
  margin: 0;
  color: #ff8f8f;
  font-size: 13px;
  font-weight: 700;
}

.status-text {
  margin: 0;
  color: #9ff0b9;
  font-size: 13px;
  font-weight: 700;
}
</style>
