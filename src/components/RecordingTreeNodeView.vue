<script setup lang="ts">
import { ref } from "vue";

type RecordingMetadata = {
  displayName: string;
  description: string;
  tags: string[];
  markerNotes: Array<{
    frame: number;
    name: string;
    note: string;
  }>;
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

function inspectAndToggleFile(path: string) {
  fileDetailsVisible.value = !fileDetailsVisible.value;
  inspect(path);
}

function toggleExpanded() {
  expanded.value = !expanded.value;
}

function collapseDuration(element: HTMLElement) {
  return `${Math.min(420, Math.max(180, element.scrollHeight * 1.25))}ms`;
}

function setCollapseDuration(element: HTMLElement) {
  element.style.setProperty("--collapse-duration", collapseDuration(element));
}

function beforeEnter(element: Element) {
  const el = element as HTMLElement;
  setCollapseDuration(el);
  el.style.height = "0";
}

function enter(element: Element) {
  const el = element as HTMLElement;
  el.style.height = `${el.scrollHeight}px`;
}

function afterEnter(element: Element) {
  const el = element as HTMLElement;
  el.style.height = "auto";
}

function beforeLeave(element: Element) {
  const el = element as HTMLElement;
  setCollapseDuration(el);
  el.style.height = `${el.scrollHeight}px`;
  void el.offsetHeight;
}

function leave(element: Element) {
  const el = element as HTMLElement;
  el.style.height = "0";
}

function afterLeave(element: Element) {
  const el = element as HTMLElement;
  el.style.height = "";
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
    summary.markerCount > 0
  );
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
      <Transition
        name="tree-collapse"
        @before-enter="beforeEnter"
        @enter="enter"
        @after-enter="afterEnter"
        @before-leave="beforeLeave"
        @leave="leave"
        @after-leave="afterLeave"
      >
        <div v-show="expanded && node.children.length > 0" class="tree-children-shell">
          <div class="tree-children">
            <RecordingTreeNodeView
              v-for="child in node.children"
              :key="child.path"
              :node="child"
              @inspect="inspect"
            />
          </div>
        </div>
      </Transition>
    </div>
    <div v-else class="file-branch">
      <button
        class="file-node"
        type="button"
        :aria-expanded="fileDetailsVisible"
        @click="inspectAndToggleFile(node.path)"
      >
        <span class="tree-prefix">{{ fileDetailsVisible ? "▾" : "▸" }}</span>
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
      </button>
      <Transition
        name="tree-collapse"
        @before-enter="beforeEnter"
        @enter="enter"
        @after-enter="afterEnter"
        @before-leave="beforeLeave"
        @leave="leave"
        @after-leave="afterLeave"
      >
        <div
          v-show="fileDetailsVisible && hasFileDetails(node.summary)"
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
            <div class="file-detail-block">
              <strong>Markers</strong>
              <span>{{ node.summary.markerCount }} markers</span>
            </div>
            <div
              v-for="markerNote in node.summary.metadata.markerNotes"
              :key="`${markerNote.frame}-${markerNote.name}-${markerNote.note}`"
              class="marker-note-row"
            >
              <strong>{{ markerNote.name || "marker" }}</strong>
              <span>frame {{ markerNote.frame }}</span>
              <span>{{ markerNote.note }}</span>
            </div>
          </div>
        </div>
      </Transition>
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

.directory-node,
.file-node {
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
}

.directory-node:hover {
  color: #eafff0;
}

.file-node {
  width: 100%;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background: #151a20;
  color: #dfe5ec;
  cursor: pointer;
  padding: 8px 10px;
  text-align: left;
}

.file-node:hover {
  background: #1e252e;
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
  clip-path: inset(0 0 0 0);
  height: auto;
  overflow: hidden;
  opacity: 1;
  transform: translateY(0) scaleY(1);
  transform-origin: top;
  will-change: height, opacity, transform, clip-path;
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
.marker-note-row {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.file-detail-block strong,
.marker-note-row strong {
  color: #c9d1da;
  font-size: 12px;
}

.file-detail-block span,
.marker-note-row span {
  overflow-wrap: anywhere;
  color: #9ca7b4;
  font-size: 12px;
}

.marker-note-row {
  grid-template-columns: minmax(80px, 0.7fr) minmax(74px, auto) minmax(120px, 1fr);
  align-items: start;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  padding-top: 8px;
}

.tree-collapse-enter-active,
.tree-collapse-leave-active {
  transition:
    height var(--collapse-duration, 240ms) cubic-bezier(0.16, 1, 0.3, 1),
    opacity calc(var(--collapse-duration, 240ms) * 0.75) ease,
    transform var(--collapse-duration, 240ms) cubic-bezier(0.16, 1, 0.3, 1),
    clip-path var(--collapse-duration, 240ms) cubic-bezier(0.16, 1, 0.3, 1);
}

.tree-collapse-enter-from,
.tree-collapse-leave-to {
  clip-path: inset(0 0 100% 0);
  opacity: 0;
  transform: translateY(-3px) scaleY(0.96);
}

.tree-collapse-enter-to,
.tree-collapse-leave-from {
  clip-path: inset(0 0 0 0);
  opacity: 1;
  transform: translateY(0) scaleY(1);
}
</style>
