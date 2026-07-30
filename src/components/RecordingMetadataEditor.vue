<script setup lang="ts">
import { ref, watch } from "vue";
import { tauriApi } from "../api/tauri";
import type { RecordingMetadata } from "../types/recording";
import BaseButton from "./BaseButton.vue";

const props = defineProps<{
  path: string;
}>();

const emit = defineEmits<{
  saved: [];
  "saved-and-close": [];
  "discard-and-close": [];
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
    return false;
  }

  metadataSaving.value = true;
  metadataError.value = "";
  metadataStatus.value = "";

  try {
    const metadata = await tauriApi.saveRecordingMetadata(props.path, metadataFromDraft());
    setMetadataDraft(metadata);
    metadataStatus.value = "Metadata saved.";
    emit("saved");
    return true;
  } catch (error) {
    metadataError.value = String(error);
    return false;
  } finally {
    metadataSaving.value = false;
  }
}

async function saveRecordingMetadataAndClose() {
  if (await saveRecordingMetadata()) {
    emit("saved-and-close");
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
  <section class="grid gap-3 border border-border-default rounded-radius-lg bg-[#151a20] p-3.5">
    <div class="flex items-start justify-between gap-3 mb-1 border-b border-border-default pb-3 max-[520px]:flex-col max-[520px]:items-stretch">
      <div>
        <p class="m-0 text-text-muted text-[11px] font-extrabold tracking-[0.08em] uppercase">Recording metadata</p>
        <h3 class="m-0 mt-0.5 text-base leading-[22px] tracking-normal">Sidecar metadata</h3>
      </div>
      <div class="flex flex-wrap justify-end gap-2">
        <BaseButton :disabled="metadataSaving" @click="emit('discard-and-close')">
          Discard & Close
        </BaseButton>
        <BaseButton :disabled="metadataSaving" @click="saveRecordingMetadata">
          {{ metadataSaving ? "Saving..." : "Save" }}
        </BaseButton>
        <BaseButton variant="primary" :disabled="metadataSaving" @click="saveRecordingMetadataAndClose">
          {{ metadataSaving ? "Saving..." : "Save & Close" }}
        </BaseButton>
      </div>
    </div>
    <p class="m-0 overflow-wrap-anywhere text-text-muted font-mono text-xs">{{ path }}</p>
    <label class="grid gap-1.5 text-text-secondary text-[13px] font-bold">
      <span>Display name</span>
      <input v-model="metadataDraft.displayName" type="text" placeholder="Browser display name" class="w-full box-border border border-border-control rounded-radius-md bg-[#10141a] text-text-body font-inherit px-2.5 py-[9px]" />
    </label>
    <label class="grid gap-1.5 text-text-secondary text-[13px] font-bold">
      <span>Description</span>
      <textarea v-model="metadataDraft.description" rows="3" placeholder="Notes for this recording" class="w-full box-border border border-border-control rounded-radius-md bg-[#10141a] text-text-body font-inherit px-2.5 py-[9px] resize-y" />
    </label>
    <label class="grid gap-1.5 text-text-secondary text-[13px] font-bold">
      <span>Tags</span>
      <input v-model="metadataTagsDraft" type="text" placeholder="sync, ranked, aim" class="w-full box-border border border-border-control rounded-radius-md bg-[#10141a] text-text-body font-inherit px-2.5 py-[9px]" />
    </label>
    <p v-if="metadataStatus" class="m-0 text-[#9ff0b9] text-[13px] font-bold">{{ metadataStatus }}</p>
    <p v-if="metadataError" class="error">{{ metadataError }}</p>
  </section>
</template>