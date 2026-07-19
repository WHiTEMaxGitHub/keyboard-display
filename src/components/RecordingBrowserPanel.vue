<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, ref, watch } from "vue";
import {
  buildRecordingTimelineMarkers,
  type RecordingTimelineMarker,
} from "../domain/recordingTimeline";
import RecordingTreeNodeView from "./RecordingTreeNodeView.vue";

type RecordingInspectionEvent =
  | { frame: number; down: string }
  | { frame: number; up: string }
  | { frame: number; marker: string };

type RecordingInspectionFrame = {
  frame: number;
  keys: string[];
};

type RecordingInspection = {
  version: number;
  fps: number;
  keyIds: string[];
  events: RecordingInspectionEvent[];
  frames: RecordingInspectionFrame[];
};

type RecordingMarkerNote = {
  frame: number;
  name: string;
  note: string;
};

type RecordingMetadata = {
  displayName: string;
  description: string;
  tags: string[];
  markerNotes: RecordingMarkerNote[];
};

type RecordingFileSummary = {
  fileName: string;
  sizeBytes: number;
  startUnixMs: number | null;
  endUnixMs: number | null;
  fps: number;
  frameCount: number;
  markerCount: number;
  metadata: RecordingMetadata;
};

type RecordingTreeNode = {
  name: string;
  path: string;
  type: "directory" | "file";
  children: RecordingTreeNode[];
  summary: RecordingFileSummary | null;
};

const props = defineProps<{
  recordingDirectory: string;
  defaultRecordingDirectory: string;
  inspectedRecordingPath: string;
  recordingInspection: RecordingInspection | null;
  recordingInspectionError: string;
}>();

const recordingTree = ref<RecordingTreeNode | null>(null);
const recordingTreeError = ref("");
const recordingTreeLoading = ref(false);
const folderNameDraft = ref("");
const folderCreating = ref(false);
const folderEditorVisible = ref(false);
const metadataDraft = ref<RecordingMetadata>(createEmptyMetadata());
const metadataTagsDraft = ref("");
const metadataStatus = ref("");
const metadataError = ref("");
const metadataSaving = ref(false);
const selectedTimelineMarker = ref<RecordingTimelineMarker | null>(null);

const timelineMarkers = computed(() => {
  if (!props.recordingInspection) {
    return [];
  }

  return buildRecordingTimelineMarkers({
    events: props.recordingInspection.events,
    fps: props.recordingInspection.fps,
    frameCount: props.recordingInspection.frames.length,
  });
});

watch(
  timelineMarkers,
  (markers) => {
    selectedTimelineMarker.value = markers[0] ?? null;
  },
  { immediate: true },
);

const emit = defineEmits<{
  "inspect-recording-file": [];
  "inspect-recording-path": [path: string];
}>();

function inspectRecordingFile() {
  emit("inspect-recording-file");
}

async function refreshRecordingTree() {
  const root = props.recordingDirectory || props.defaultRecordingDirectory;
  if (!root) {
    recordingTreeError.value = "Recording folder is not ready.";
    return;
  }

  recordingTreeLoading.value = true;
  recordingTreeError.value = "";

  try {
    recordingTree.value = await invoke<RecordingTreeNode>("list_recording_files", { root });
  } catch (error) {
    recordingTreeError.value = String(error);
  } finally {
    recordingTreeLoading.value = false;
  }
}

function showFolderEditor() {
  folderEditorVisible.value = true;
  recordingTreeError.value = "";
}

function cancelFolderEditor() {
  folderEditorVisible.value = false;
  folderNameDraft.value = "";
}

async function createRecordingFolder() {
  const root = props.recordingDirectory || props.defaultRecordingDirectory;
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
    recordingTree.value = await invoke<RecordingTreeNode>("create_recording_folder", {
      root,
      folderName,
    });
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
  void loadRecordingMetadata(path);
}

async function loadRecordingMetadata(path: string) {
  metadataStatus.value = "";
  metadataError.value = "";

  try {
    const metadata = await invoke<RecordingMetadata>("read_recording_metadata", { path });
    setMetadataDraft(metadata);
  } catch (error) {
    metadataError.value = String(error);
    setMetadataDraft(createEmptyMetadata());
  }
}

async function saveRecordingMetadata() {
  if (!props.inspectedRecordingPath) {
    metadataError.value = "Choose a recording file first.";
    return;
  }

  metadataSaving.value = true;
  metadataError.value = "";
  metadataStatus.value = "";

  try {
    const metadata = await invoke<RecordingMetadata>("save_recording_metadata", {
      path: props.inspectedRecordingPath,
      metadata: metadataFromDraft(),
    });
    setMetadataDraft(metadata);
    metadataStatus.value = "Metadata saved.";
    await refreshRecordingTree();
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

function formatInspectionEvent(event: RecordingInspectionEvent) {
  if ("down" in event) {
    return `frame ${event.frame} down ${event.down}`;
  }

  if ("up" in event) {
    return `frame ${event.frame} up ${event.up}`;
  }

  return `frame ${event.frame} marker ${event.marker}`;
}

function markerEvents(events: RecordingInspectionEvent[]) {
  return events.filter((event): event is Extract<RecordingInspectionEvent, { marker: string }> =>
    "marker" in event
  );
}

function timelineMarkerPosition(percent: number) {
  return `calc(10px + ${percent / 100} * (100% - 20px))`;
}

function formatFrameTimecode(frame: number, fps: number) {
  const totalSeconds = Math.floor(frame / fps);
  const frameInSecond = frame % fps;
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${pad2(hours)}:${pad2(minutes)}:${pad2(seconds)}:${padFrame(frameInSecond, fps)} @ ${fps}fps`;
}

function pad2(value: number) {
  return String(value).padStart(2, "0");
}

function padFrame(frame: number, fps: number) {
  return String(frame).padStart(String(Math.max(fps - 1, 0)).length, "0");
}
</script>

<template>
  <article class="panel wide-panel">
    <h2>Recordings</h2>
    <div class="section-header">
      <h3>Recording files</h3>
      <div class="header-actions">
        <button type="button" @click="showFolderEditor">
          New folder
        </button>
        <button type="button" :disabled="recordingTreeLoading" @click="refreshRecordingTree">
          {{ recordingTreeLoading ? "Loading..." : "Refresh" }}
        </button>
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
        <button type="submit" :disabled="folderCreating">
          {{ folderCreating ? "Creating..." : "Create" }}
        </button>
        <button type="button" :disabled="folderCreating" @click="cancelFolderEditor">
          Cancel
        </button>
      </div>
    </form>
    <p v-if="recordingTreeError" class="error-text">
      {{ recordingTreeError }}
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
        <button type="button" @click="inspectRecordingFile">
          Inspect .kbdrec
        </button>
      </div>
      <p v-if="inspectedRecordingPath" class="quiet">
        File: {{ inspectedRecordingPath }}
      </p>
      <div v-if="inspectedRecordingPath" class="metadata-editor">
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
      <p v-if="recordingInspectionError" class="error-text">
        {{ recordingInspectionError }}
      </p>
      <div v-if="recordingInspection" class="inspection-grid">
        <div class="field-row">
          <span>Version</span>
          <strong>{{ recordingInspection.version }}</strong>
        </div>
        <div class="field-row">
          <span>FPS</span>
          <strong>{{ recordingInspection.fps }}</strong>
        </div>
        <div class="field-row">
          <span>Keys</span>
          <strong>{{ recordingInspection.keyIds.length }}</strong>
        </div>
        <div class="field-row">
          <span>Events</span>
          <strong>{{ recordingInspection.events.length }}</strong>
        </div>
        <div class="field-row">
          <span>Frames</span>
          <strong>{{ recordingInspection.frames.length }}</strong>
        </div>
        <div class="field-row">
          <span>Markers</span>
          <strong>
            {{ recordingInspection.events.filter((event) => "marker" in event).length }}
          </strong>
        </div>
      </div>
      <div v-if="recordingInspection" class="marker-timeline-panel">
        <div class="section-header">
          <h3>Marker timeline</h3>
          <span class="quiet">
            {{ recordingInspection.frames.length }} frames
          </span>
        </div>
        <div v-if="timelineMarkers.length" class="marker-timeline" aria-label="Marker timeline">
          <button
            v-for="(marker, index) in timelineMarkers"
            :key="`${marker.frame}-${marker.name}-${index}`"
            type="button"
            class="timeline-marker"
            :class="{ selected: selectedTimelineMarker === marker }"
            :style="{ left: timelineMarkerPosition(marker.percent) }"
            :title="`${marker.name} · frame ${marker.frame} · ${marker.timecode}`"
            @click="selectedTimelineMarker = marker"
          >
            <span class="timeline-marker-dot" />
            <span class="timeline-marker-label">{{ marker.name }}</span>
          </button>
        </div>
        <p v-else class="quiet">No markers in this recording.</p>
        <div v-if="selectedTimelineMarker" class="timeline-marker-detail">
          <strong>{{ selectedTimelineMarker.name }}</strong>
          <span>frame {{ selectedTimelineMarker.frame }}</span>
          <span>{{ selectedTimelineMarker.timecode }}</span>
        </div>
      </div>
      <div v-if="recordingInspection" class="inspection-lists">
        <div>
          <h4>Markers</h4>
          <div class="marker-metadata-list">
            <div
              v-for="(event, index) in markerEvents(recordingInspection.events)"
              :key="`${event.frame}-${event.marker}-${index}`"
              class="marker-metadata"
            >
              <strong>marker {{ event.marker }}</strong>
              <span>frame {{ event.frame }}</span>
              <span>time {{ formatFrameTimecode(event.frame, recordingInspection.fps) }}</span>
            </div>
          </div>
        </div>
        <div>
          <h4>Key table</h4>
          <p class="quiet">{{ recordingInspection.keyIds.join(", ") || "None" }}</p>
        </div>
        <div>
          <h4>Events</h4>
          <ol>
            <li
              v-for="(event, index) in recordingInspection.events.slice(0, 8)"
              :key="index"
            >
              {{ formatInspectionEvent(event) }}
            </li>
          </ol>
        </div>
        <div>
          <h4>Frames</h4>
          <ol>
            <li
              v-for="frame in recordingInspection.frames.slice(0, 8)"
              :key="frame.frame"
            >
              frame {{ frame.frame }}: {{ frame.keys.join(", ") || "none" }}
            </li>
          </ol>
        </div>
      </div>
    </div>
  </article>
</template>

<style scoped>
.panel {
  min-height: 190px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #171b22;
  padding: 18px;
}

.wide-panel {
  max-width: 860px;
}

.panel h2 {
  margin: 0 0 16px;
  letter-spacing: 0;
}

.field-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.field-row span,
.quiet {
  color: #9ca7b4;
}

.field-row strong {
  min-width: 0;
  overflow-wrap: anywhere;
  text-align: right;
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

.section-header h3,
.inspection-lists h4 {
  margin: 0;
  letter-spacing: 0;
}

.section-header h3 {
  font-size: 16px;
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

.inspection-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0 18px;
}

.marker-timeline-panel {
  display: grid;
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  padding: 14px;
}

.marker-timeline {
  position: relative;
  height: 62px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background:
    linear-gradient(90deg, rgba(255, 255, 255, 0.08) 1px, transparent 1px)
      0 0 / 25% 100%,
    #10141a;
}

.marker-timeline::before {
  position: absolute;
  right: 10px;
  bottom: 13px;
  left: 10px;
  height: 2px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.18);
  content: "";
}

.timeline-marker {
  position: absolute;
  bottom: 9px;
  display: grid;
  justify-items: center;
  gap: 5px;
  min-width: 36px;
  max-width: 96px;
  border: 0;
  background: transparent;
  color: #dfe5ec;
  cursor: pointer;
  font: inherit;
  padding: 0;
  transform: translateX(-50%);
}

.timeline-marker-dot {
  width: 12px;
  height: 12px;
  border: 2px solid #10141a;
  border-radius: 999px;
  background: #9ff0b9;
  box-shadow: 0 0 0 1px rgba(159, 240, 185, 0.65);
}

.timeline-marker-label {
  max-width: 96px;
  overflow: hidden;
  color: #c9d1da;
  font-size: 11px;
  font-weight: 700;
  line-height: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.timeline-marker.selected .timeline-marker-dot,
.timeline-marker:focus-visible .timeline-marker-dot {
  background: #ffd166;
  box-shadow: 0 0 0 2px rgba(255, 209, 102, 0.32);
}

.timeline-marker.selected .timeline-marker-label,
.timeline-marker:focus-visible .timeline-marker-label {
  color: #fff2c2;
}

.timeline-marker-detail {
  display: grid;
  grid-template-columns: minmax(120px, 1fr) minmax(90px, auto) minmax(170px, auto);
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background: #10141a;
  color: #dfe5ec;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  padding: 8px 10px;
}

.timeline-marker-detail strong {
  color: #fff2c2;
}

.inspection-lists {
  display: grid;
  gap: 14px;
}

.marker-metadata-list {
  display: grid;
  gap: 8px;
}

.marker-metadata {
  display: grid;
  grid-template-columns: minmax(120px, 1.1fr) minmax(100px, 0.7fr) minmax(180px, 1.2fr);
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background: #151a20;
  padding: 8px 10px;
  color: #dfe5ec;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.marker-metadata strong {
  color: #eafff0;
}

.inspection-lists h4 {
  margin-bottom: 6px;
  color: #c9d1da;
  font-size: 13px;
}

.inspection-lists ol {
  display: grid;
  gap: 5px;
  margin: 0;
  padding-left: 18px;
  color: #dfe5ec;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
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
