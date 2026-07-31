<script setup lang="ts">
import { ref } from "vue";
import type { RecordingFileSummary, RecordingTreeNode } from "../types/recording";
import BaseButton from "./BaseButton.vue";

defineProps<{
  node: RecordingTreeNode;
}>();

const emit = defineEmits<{
  inspect: [path: string];
}>();

const expanded = ref(true);
const fileDetailsVisible = ref(false);

function inspect(path: string) {
  emit("inspect", path);
}

function toggleFileDetails() {
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
  <div class="grid gap-1.5">
    <div v-if="node.type === 'directory'" class="grid gap-1.5 min-w-0">
      <button
        class="flex items-start gap-2 min-w-0 w-full border-0 bg-transparent text-text-secondary cursor-pointer text-[13px] p-0 text-left hover:text-accent-text hover:translate-x-0.5 transition-[color,transform] duration-[140ms] ease"
        type="button"
        :aria-expanded="expanded"
        @click="toggleExpanded"
      >
        <span class="flex-none text-text-muted font-mono">{{ expanded ? "▾" : "▸" }}</span>
        <strong>{{ node.name }}</strong>
      </button>
      <div v-if="expanded && node.children.length > 0" class="tree-children-shell expanded">
        <div class="grid gap-1.5 ml-[18px] min-h-0 min-w-0 overflow-hidden">
          <RecordingTreeNodeView
            v-for="child in node.children"
            :key="child.path"
            :node="child"
            @inspect="inspect"
          />
        </div>
      </div>
    </div>
    <div v-else class="grid gap-1.5 min-w-0">
      <div
        class="grid grid-cols-[auto_minmax(0,1fr)_auto] items-start gap-2 min-w-0 border border-border-control rounded-md bg-surface-control text-text-body cursor-pointer px-2.5 py-2 text-left transition-[background-color,border-color,transform] duration-[140ms] ease hover:border-border-default hover:bg-surface-control-hover focus-visible:border-accent-focus-border focus-visible:outline-2 focus-visible:outline-accent-focus-ring focus-visible:outline-offset-0"
        role="button"
        tabindex="0"
        :aria-expanded="fileDetailsVisible"
        @click="toggleFileDetails"
        @keydown.enter.prevent="toggleFileDetails"
        @keydown.space.prevent="toggleFileDetails"
      >
        <span class="flex-none text-text-muted font-mono">{{ fileDetailsVisible ? "▾" : "▸" }}</span>
        <span class="grid gap-[3px] min-w-0">
          <strong class="overflow-hidden text-ellipsis whitespace-nowrap">{{ displayTitle(node) }}</strong>
          <small v-if="node.summary?.metadata.displayName" class="text-text-muted text-xs">{{ node.name }}</small>
          <small v-if="node.summary" class="text-text-muted text-xs">
            {{ formatFileSize(node.summary.sizeBytes) }} · {{ node.summary.fps }}fps ·
            {{ node.summary.frameCount }} frames · {{ node.summary.markerCount }} markers
          </small>
          <small v-if="node.summary?.metadata.tags.length" class="text-text-muted text-xs">
            tags: {{ node.summary.metadata.tags.join(", ") }}
          </small>
          <small v-if="node.summary" class="text-text-muted text-xs">{{ formatFileTimes(node.summary) }}</small>
        </span>
        <BaseButton class="justify-self-end" size="sm" @click.stop="inspect(node.path)">
          Inspect / edit
        </BaseButton>
      </div>
      <div
        :class="[
          'tree-children-shell',
          { expanded: fileDetailsVisible && hasFileDetails(node.summary) },
        ]"
      >
        <div v-if="node.summary && hasFileDetails(node.summary)" class="grid gap-2 ml-[18px] min-h-0 overflow-hidden border border-border-control rounded-md bg-surface-control text-text-body p-2.5">
          <div v-if="node.summary.metadata.description" class="grid gap-[3px] min-w-0">
            <strong class="text-text-secondary text-xs">Description</strong>
            <span class="overflow-wrap-anywhere text-text-muted text-xs">{{ node.summary.metadata.description }}</span>
          </div>
          <div v-if="node.summary.metadata.tags.length" class="grid gap-[3px] min-w-0">
            <strong class="text-text-secondary text-xs">Tags</strong>
            <span class="overflow-wrap-anywhere text-text-muted text-xs">{{ node.summary.metadata.tags.join(", ") }}</span>
          </div>
          <div class="grid gap-2 mt-0.5">
            <div class="flex items-baseline justify-between gap-3">
              <strong class="text-text-body text-[13px]">Markers</strong>
              <span class="text-text-muted text-xs font-bold">{{ node.summary.markerCount }} total</span>
            </div>
            <div v-if="node.summary.markers.length" class="grid overflow-hidden border border-border-dim rounded-md">
              <div class="grid grid-cols-[minmax(110px,0.9fr)_minmax(90px,auto)_minmax(190px,auto)_minmax(120px,1fr)] gap-2.5 items-center px-2.5 py-2 bg-white/[0.035] text-text-subtle text-[11px] font-extrabold uppercase">
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
                <strong class="text-text-body font-mono font-extrabold text-xs">{{ marker.name || "marker" }}</strong>
                <span class="min-w-0 overflow-wrap-anywhere">frame {{ marker.frame }}</span>
                <span class="min-w-0 overflow-wrap-anywhere">{{ formatMarkerTime(marker.frame, node.summary.fps) }}</span>
                <span class="min-w-0 overflow-wrap-anywhere">{{ markerNoteFor(node.summary, marker)?.note || "-" }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tree-children-shell {
  display: grid;
  grid-template-rows: 0fr;
  min-width: 0;
  opacity: 0;
  transition:
    grid-template-rows 220ms cubic-bezier(0.2, 0.9, 0.2, 1),
    opacity 160ms ease,
    transform 220ms cubic-bezier(0.2, 0.9, 0.2, 1);
  transform: translateY(-3px);
}

.tree-children-shell.expanded {
  grid-template-rows: 1fr;
  opacity: 1;
  transform: translateY(0);
}

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
  color: #9ca7b4;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.marker-note-row + .marker-note-row {
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}
</style>