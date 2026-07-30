<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { tauriApi } from "../api/tauri";
import type {
  RecordingInspection,
  RecordingTreeNode,
} from "../types/recording";
import BasePanel from "./BasePanel.vue";
import BaseButton from "./BaseButton.vue";
import RecordingInspectionPanel from "./RecordingInspectionPanel.vue";
import RecordingMetadataEditor from "./RecordingMetadataEditor.vue";
import RecordingTreeNodeView from "./RecordingTreeNodeView.vue";

const props = defineProps<{
  recordingBrowserDirectory: string;
  currentRecordingPath: string;
  recordingInspection: RecordingInspection | null;
  recordingInspectionError: string;
}>();

const recordingTree = ref<RecordingTreeNode | null>(null);
const recordingTreeError = ref("");
const recordingTreeLoading = ref(false);
const folderNameDraft = ref("");
const folderCreating = ref(false);
const folderEditorVisible = ref(false);

const recordingRoot = computed(() => props.recordingBrowserDirectory);

const emit = defineEmits<{
  "inspect-recording-file": [];
  "inspect-recording-path": [path: string];
  "clear-recording-inspection": [];
  "choose-recording-browser-directory": [];
}>();

function inspectRecordingFile() {
  emit("inspect-recording-file");
}

async function refreshRecordingTree() {
  const root = recordingRoot.value;
  if (!root) {
    recordingTreeError.value = "Choose a recording files folder first.";
    return;
  }

  recordingTreeLoading.value = true;
  recordingTreeError.value = "";

  try {
    recordingTree.value = await tauriApi.listRecordingFiles(root);
  } catch (error) {
    recordingTreeError.value = String(error);
  } finally {
    recordingTreeLoading.value = false;
  }
}

onMounted(() => {
  if (recordingRoot.value) {
    void refreshRecordingTree();
  }
});

watch(recordingRoot, (root, previousRoot) => {
  if (root && root !== previousRoot) {
    void refreshRecordingTree();
  }
});

function showFolderEditor() {
  folderEditorVisible.value = true;
  recordingTreeError.value = "";
}

function cancelFolderEditor() {
  folderEditorVisible.value = false;
  folderNameDraft.value = "";
}

async function createRecordingFolder() {
  const root = recordingRoot.value;
  const folderName = folderNameDraft.value.trim();

  if (!root) {
    recordingTreeError.value = "Choose a recording files folder first.";
    return;
  }

  if (!folderName) {
    recordingTreeError.value = "Folder name is required.";
    return;
  }

  folderCreating.value = true;
  recordingTreeError.value = "";

  try {
    recordingTree.value = await tauriApi.createRecordingFolder(root, folderName);
    folderNameDraft.value = "";
    folderEditorVisible.value = false;
  } catch (error) {
    recordingTreeError.value = String(error);
  } finally {
    folderCreating.value = false;
  }
}

function inspectRecordingPath(path: string) {
  emit("inspect-recording-path", path);
}

function saveAndCloseRecordingMetadata() {
  void refreshRecordingTree();
  emit("clear-recording-inspection");
}

</script>

<template>
  <BasePanel wide>
    <h2 class="m-0 mb-4 text-lg leading-6 tracking-normal">Recordings</h2>
    <div class="flex items-center justify-between gap-3 mb-4">
      <h3 class="m-0 text-base leading-[22px] tracking-normal">Recording files</h3>
      <div class="flex flex-wrap justify-end gap-2">
        <BaseButton @click="emit('choose-recording-browser-directory')">
          Choose folder
        </BaseButton>
        <BaseButton @click="showFolderEditor">
          New folder
        </BaseButton>
        <BaseButton :disabled="recordingTreeLoading" @click="refreshRecordingTree">
          {{ recordingTreeLoading ? "Loading..." : "Refresh" }}
        </BaseButton>
      </div>
    </div>
    <p class="notice">
      {{ recordingBrowserDirectory || "Choose a folder to browse .kbdrec files." }}
    </p>
    <form v-if="folderEditorVisible" class="grid gap-2.5 border border-border-default rounded-radius-lg bg-[#151a20] p-3 my-4" @submit.prevent="createRecordingFolder">
      <label class="grid gap-1.5 text-text-secondary text-[13px] font-bold">
        <span>Folder name</span>
        <input
          v-model="folderNameDraft"
          type="text"
          placeholder="Match 01"
          :disabled="folderCreating"
          class="w-full box-border border border-border-control rounded-radius-md bg-[#10141a] text-text-body font-inherit px-2.5 py-[9px]"
        />
      </label>
      <div class="flex flex-wrap justify-end gap-2">
        <BaseButton type="submit" :disabled="folderCreating">
          {{ folderCreating ? "Creating..." : "Create" }}
        </BaseButton>
        <BaseButton :disabled="folderCreating" @click="cancelFolderEditor">
          Cancel
        </BaseButton>
      </div>
    </form>
    <p v-if="recordingTreeError" class="error">{{ recordingTreeError }}</p>
    <p v-else-if="recordingTree && !recordingTree.exists" class="notice-text">
      Recording folder does not exist yet. It may have been deleted; create a
      folder or start recording to initialize it again.
    </p>
    <div v-if="recordingTree" class="grid gap-1.5 max-h-[360px] overflow-auto mt-4">
      <RecordingTreeNodeView
        :node="recordingTree"
        @inspect="inspectRecordingPath"
      />
    </div>

    <div class="grid gap-3.5 mt-5 border-t border-border-default pt-4">
      <div class="flex items-center justify-between gap-3">
        <h3 class="m-0 text-base leading-[22px] tracking-normal">Recording inspection</h3>
        <BaseButton @click="inspectRecordingFile">
          Inspect .kbdrec
        </BaseButton>
      </div>
      <p v-if="recordingInspectionError" class="error">{{ recordingInspectionError }}</p>
      <RecordingMetadataEditor
        v-if="currentRecordingPath"
        :path="currentRecordingPath"
        @discard-and-close="emit('clear-recording-inspection')"
        @saved="refreshRecordingTree"
        @saved-and-close="saveAndCloseRecordingMetadata"
      />
      <RecordingInspectionPanel
        v-if="recordingInspection"
        :inspection="recordingInspection"
      />
    </div>
  </BasePanel>
</template>

<style scoped>
.notice-text {
  margin: 0;
  border: 1px solid rgba(255, 209, 102, 0.18);
  border-radius: 7px;
  background: rgba(255, 209, 102, 0.08);
  color: #e8cf88;
  font-size: 13px;
  font-weight: 700;
  padding: 9px 10px;
}
</style>