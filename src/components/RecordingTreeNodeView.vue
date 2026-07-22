<script setup lang="ts">
import { ref } from "vue";
import type { RecordingFileSummary, RecordingTreeNode } from "../types/recording";
import BaseButton from "./BaseButton.vue";

defineProps<{
  node: RecordingTreeNode;
}>();

const emit = defineEmits<{
  select: [path: string];
  inspect: [path: string];
}>();

const expanded = ref(true);
const fileDetailsVisible = ref(false);

function inspect(path: string) {
  emit("inspect", path);
}

function toggleFileDetails(path: string) {
  emit("select", path);
  fileDetailsVisible.value = !fileDetailsVisible.value;
}

function toggleExpanded() {
  expanded.value = !expanded.value;
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }

  return `${(bytes / 1024).toFixed(1)} KiB`;
}

function formatFileTimes(summary: RecordingFileSummary) {
  if (!summary.startUnixMs || !summary.endUnixMs) {
    return "unknown time";
  }

  return `${new Date(summary.startUnixMs).toLocaleString()} - ${new Date(summary.endUnixMs).toLocaleTimeString()}`;
}

function displayTitle(node: RecordingTreeNode) {
  return node.summary?.metadata.displayName || node.name;
}

function hasFileDetails(summary: RecordingFileSummary | null) {
  if (!summary) {
    return false;
  }

  return (
    Boolean(summary.metadata.displayName) ||
    Boolean(summary.metadata.description) ||
    summary.metadata.tags.length > 0 ||
    summary.metadata.markerNotes.length > 0 ||
    summary.markers.length > 0 ||
    summary.markerCount > 0
  );
}

function markerNoteFor(summary: RecordingFileSummary, marker: { frame: number; name: string }) {
  return summary.metadata.markerNotes.find(
    (markerNote) => markerNote.frame === marker.frame && markerNote.name === marker.name,
  );
}

function formatMarkerTime(frame: number, fps: number) {
  const safeFps = Math.max(Math.floor(fps), 1);
  const totalSeconds = Math.floor(frame / safeFps);
  const frameInSecond = frame % safeFps;
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${pad2(hours)}:${pad2(minutes)}:${pad2(seconds)}:${padFrame(frameInSecond, safeFps)} @ ${safeFps}fps`;
}

function pad2(value: number) {
  return String(value).padStart(2, "0");
}

function padFrame(frame: number, fps: number) {
  return String(frame).padStart(String(Math.max(fps - 1, 0)).length, "0");
}
</script>

<template>
  <div class="tree-node">
    <div v-if="node.type === 'directory'" class="directory-branch">
      <button
        class="directory-node"
        type="button"
        :aria-expanded="expanded"
        @click="toggleExpanded"
      >
        <span class="tree-prefix">{{ expanded ? "▾" : "▸" }}</span>
        <strong>{{ node.name }}</strong>
      </button>
      <div v-if="expanded && node.children.length > 0" class="tree-children-shell">
        <div class="tree-children">
          <RecordingTreeNodeView
            v-for="child in node.children"
            :key="child.path"
            :node="child"
            @select="emit('select', $event)"
            @inspect="inspect"
          />
        </div>
      </div>
    </div>
    <div v-else class="file-branch">
      <div class="file-node">
        <button
          class="file-toggle-button"
          type="button"
          :aria-expanded="fileDetailsVisible"
          @click="toggleFileDetails(node.path)"
        >
          <span class="tree-prefix">{{ fileDetailsVisible ? "▾" : "▸" }}</span>
        </button>
        <span class="file-main">
          <strong>{{ displayTitle(node) }}</strong>
          <small v-if="node.summary?.metadata.displayName">{{ node.name }}</small>
          <small v-if="node.summary">
            {{ formatFileSize(node.summary.sizeBytes) }} · {{ node.summary.fps }}fps ·
            {{ node.summary.frameCount }} frames · {{ node.summary.markerCount }} markers
          </small>
          <small v-if="node.summary?.metadata.tags.length">
            tags: {{ node.summary.metadata.tags.join(", ") }}
          </small>
          <small v-if="node.summary">{{ formatFileTimes(node.summary) }}</small>
        </span>
        <BaseButton class="inspect-file-button" size="sm" @click.stop="inspect(node.path)">
          Inspect / edit
        </BaseButton>
      </div>
      <div
        v-if="fileDetailsVisible && hasFileDetails(node.summary)"
        class="tree-children-shell"
      >
        <div v-if="node.summary" class="file-details">
          <div v-if="node.summary.metadata.description" class="file-detail-block">
            <strong>Description</strong>
            <span>{{ node.summary.metadata.description }}</span>
          </div>
          <div v-if="node.summary.metadata.tags.length" class="file-detail-block">
            <strong>Tags</strong>
            <span>{{ node.summary.metadata.tags.join(", ") }}</span>
          </div>
          <div class="marker-detail-section">
            <div class="marker-detail-header">
              <strong>Markers</strong>
              <span>{{ node.summary.markerCount }} total</span>
            </div>
            <div v-if="node.summary.markers.length" class="marker-table">
              <div class="marker-table-head" aria-hidden="true">
                <span>Name</span>
                <span>Frame</span>
                <span>Timecode</span>
                <span>Note</span>
              </div>
              <div
                v-for="marker in node.summary.markers"
                :key="`${marker.frame}-${marker.name}`"
                class="marker-note-row"
              >
                <strong>{{ marker.name || "marker" }}</strong>
                <span>frame {{ marker.frame }}</span>
                <span>{{ formatMarkerTime(marker.frame, node.summary.fps) }}</span>
                <span>{{ markerNoteFor(node.summary, marker)?.note || "-" }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tree-node {
  display: grid;
  gap: 6px;
}

.directory-branch {
  display: grid;
  gap: 6px;
  min-width: 0;
}

.file-branch {
  display: grid;
  gap: 6px;
  min-width: 0;
}

.directory-node {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  min-width: 0;
}

.directory-node {
  width: 100%;
  border: 0;
  background: transparent;
  color: #c9d1da;
  cursor: pointer;
  font-size: 13px;
  padding: 0;
  text-align: left;
  transition:
    color 140ms ease,
    transform 140ms ease;
}

.directory-node:hover {
  color: #eafff0;
  transform: translateX(2px);
}

.file-node {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: start;
  gap: 8px;
  min-width: 0;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background: #151a20;
  color: #dfe5ec;
  padding: 8px 10px;
  text-align: left;
  transition:
    background-color 140ms ease,
    border-color 140ms ease,
    transform 140ms ease;
}

.file-node:hover {
  border-color: rgba(255, 255, 255, 0.14);
  background: #1e252e;
}

.file-toggle-button {
  display: grid;
  width: 24px;
  height: 24px;
  place-items: center;
  border: 0;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  padding: 0;
}

.file-toggle-button:hover {
  background: rgba(255, 255, 255, 0.08);
}

.tree-prefix {
  flex: 0 0 auto;
  color: #9ca7b4;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.file-main {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.file-main strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-main small {
  color: #9ca7b4;
  font-size: 12px;
}

.tree-children-shell {
  min-width: 0;
}

.tree-children {
  display: grid;
  gap: 6px;
  margin-left: 18px;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}

.file-details {
  display: grid;
  gap: 8px;
  margin-left: 18px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background: #11161d;
  color: #dfe5ec;
  padding: 10px;
}

.file-detail-block,
.marker-detail-section {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.file-detail-block strong {
  color: #c9d1da;
  font-size: 12px;
}

.file-detail-block span {
  overflow-wrap: anywhere;
  color: #9ca7b4;
  font-size: 12px;
}

.marker-detail-section {
  gap: 8px;
  margin-top: 2px;
}

.marker-detail-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
}

.marker-detail-header strong {
  color: #dfe5ec;
  font-size: 13px;
}

.marker-detail-header span {
  color: #9ca7b4;
  font-size: 12px;
  font-weight: 700;
}

.inspect-file-button {
  justify-self: end;
}

.marker-table {
  display: grid;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 7px;
}

.marker-table-head,
.marker-note-row {
  display: grid;
  grid-template-columns:
    minmax(110px, 0.9fr)
    minmax(90px, auto)
    minmax(190px, auto)
    minmax(120px, 1fr);
  gap: 10px;
  align-items: center;
  padding: 8px 10px;
}

.marker-table-head {
  background: rgba(255, 255, 255, 0.035);
  color: #7f8b99;
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.marker-note-row {
  color: #9ca7b4;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.marker-note-row + .marker-note-row {
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.marker-note-row strong {
  color: #dfe5ec;
  font: inherit;
  font-weight: 800;
}

.marker-note-row span {
  min-width: 0;
  overflow-wrap: anywhere;
}

</style>
