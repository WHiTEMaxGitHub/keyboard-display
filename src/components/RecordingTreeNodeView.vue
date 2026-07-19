<script setup lang="ts">
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

function inspect(path: string) {
  emit("inspect", path);
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
</script>

<template>
  <div class="tree-node">
    <div v-if="node.type === 'directory'" class="directory-node">
      <span class="tree-prefix">▾</span>
      <strong>{{ node.name }}</strong>
    </div>
    <button
      v-else
      class="file-node"
      type="button"
      @click="inspect(node.path)"
    >
      <span class="tree-prefix">•</span>
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
    <div v-if="node.children.length > 0" class="tree-children">
      <RecordingTreeNodeView
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        @inspect="inspect"
      />
    </div>
  </div>
</template>

<style scoped>
.tree-node {
  display: grid;
  gap: 6px;
}

.directory-node,
.file-node {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  min-width: 0;
}

.directory-node {
  color: #c9d1da;
  font-size: 13px;
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

.tree-children {
  display: grid;
  gap: 6px;
  margin-left: 18px;
}
</style>
