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
  recordingDirectory: string;
  defaultRecordingDirectory: string;
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

const recordingRoot = computed(() => props.recordingDirectory || props.defaultRecordingDirectory);

const emit = defineEmits<{
  "inspect-recording-file": [];
  "inspect-recording-path": [path: string];
  "clear-recording-inspection": [];
}>();

function inspectRecordingFile() {
  emit("inspect-recording-file");
}

async function refreshRecordingTree() {
  const root = recordingRoot.value;
  if (!root) {
    recordingTreeError.value = "Recording folder is not ready.";
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
    recordingTreeError.value = "Recording folder is not ready.";
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

function closeMetadataEditor() {
  emit("clear-recording-inspection");
}

</script>

<template>
  <BasePanel wide>
    <h2>Recordings</h2>
    <div class="section-header">
      <h3>Recording files</h3>
      <div class="header-actions">
        <BaseButton @click="showFolderEditor">
          New folder
        </BaseButton>
        <BaseButton :disabled="recordingTreeLoading" @click="refreshRecordingTree">
          {{ recordingTreeLoading ? "Loading..." : "Refresh" }}
        </BaseButton>
      </div>
    </div>
    <p class="quiet">
      {{ recordingDirectory || `Default app folder: ${defaultRecordingDirectory || "loading..."}` }}
    </p>
    <form v-if="folderEditorVisible" class="new-folder-form" @submit.prevent="createRecordingFolder">
      <label>
        <span>Folder name</span>
        <input
          v-model="folderNameDraft"
          type="text"
          placeholder="Match 01"
          :disabled="folderCreating"
        />
      </label>
      <div class="header-actions">
        <BaseButton type="submit" :disabled="folderCreating">
          {{ folderCreating ? "Creating..." : "Create" }}
        </BaseButton>
        <BaseButton :disabled="folderCreating" @click="cancelFolderEditor">
          Cancel
        </BaseButton>
      </div>
    </form>
    <p v-if="recordingTreeError" class="error-text">
      {{ recordingTreeError }}
    </p>
    <p v-else-if="recordingTree && !recordingTree.exists" class="notice-text">
      Recording folder does not exist yet. It may have been deleted; create a
      folder or start recording to initialize it again.
    </p>
    <div v-if="recordingTree" class="recording-tree">
      <RecordingTreeNodeView
        :node="recordingTree"
        @inspect="inspectRecordingPath"
      />
    </div>

    <div class="inspection-panel">
      <div class="section-header">
        <h3>Recording inspection</h3>
        <BaseButton @click="inspectRecordingFile">
          Inspect .kbdrec
        </BaseButton>
      </div>
      <p v-if="recordingInspectionError" class="error-text">
        {{ recordingInspectionError }}
      </p>
      <RecordingInspectionPanel
        v-if="recordingInspection"
        :inspection="recordingInspection"
      />
    </div>
    <RecordingMetadataEditor
      v-if="currentRecordingPath"
      :path="currentRecordingPath"
      @close="closeMetadataEditor"
      @saved="refreshRecordingTree"
    />
  </BasePanel>
</template>

<style scoped>
.quiet {
  color: #9ca7b4;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.header-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.section-header h3 {
  margin: 0;
  letter-spacing: 0;
  font-size: 16px;
  line-height: 22px;
}

.new-folder-form {
  display: grid;
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  padding: 12px;
}

.new-folder-form label {
  display: grid;
  gap: 6px;
  color: #c9d1da;
  font-size: 13px;
  font-weight: 700;
}

.new-folder-form input {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  background: #10141a;
  color: #dfe5ec;
  font: inherit;
  padding: 9px 10px;
}

.recording-tree {
  display: grid;
  gap: 6px;
  max-height: 360px;
  overflow: auto;
}

.inspection-panel {
  display: grid;
  gap: 14px;
  margin-top: 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding-top: 18px;
}

.error-text {
  margin: 0;
  color: #ff8f8f;
  font-size: 13px;
  font-weight: 700;
}

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
