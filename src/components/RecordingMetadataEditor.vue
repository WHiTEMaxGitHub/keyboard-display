<script setup lang="ts">
import { ref, watch } from "vue";
import { tauriApi } from "../api/tauri";
import type { RecordingMetadata } from "../types/recording";

const props = defineProps<{
  path: string;
}>();

const emit = defineEmits<{
  saved: [];
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
  <div class="metadata-editor">
    <div class="section-header">
      <h3>Sidecar metadata</h3>
      <button type="button" :disabled="metadataSaving" @click="saveRecordingMetadata">
        {{ metadataSaving ? "Saving..." : "Save metadata" }}
      </button>
    </div>
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
  </div>
</template>

<style scoped>
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.section-header h3 {
  margin: 0;
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
  gap: 12px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  padding: 14px;
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
