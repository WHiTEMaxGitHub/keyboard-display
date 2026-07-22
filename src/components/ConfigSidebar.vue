<script setup lang="ts">
import {
  Clapperboard,
  Eye,
  Keyboard,
  MonitorCog,
  MonitorUp,
  Palette,
  Video,
} from "@lucide/vue";

type ConfigPage = "overview" | "layout" | "appearance" | "window" | "recording" | "export";
type RecordingSubPage = "control" | "files";

const navItems: Array<{
  id: ConfigPage;
  label: string;
  icon: typeof Eye;
}> = [
  { id: "overview", label: "Overview", icon: Eye },
  { id: "layout", label: "Layout", icon: MonitorUp },
  { id: "appearance", label: "Appearance", icon: Palette },
  { id: "window", label: "Window", icon: MonitorCog },
  { id: "recording", label: "Recording", icon: Clapperboard },
  { id: "export", label: "Export", icon: Video },
];

const recordingNavItems: Array<{ id: RecordingSubPage; label: string }> = [
  { id: "control", label: "Control" },
  { id: "files", label: "Files" },
];

defineProps<{
  activePage: ConfigPage;
  recordingSubPage: RecordingSubPage;
  collapsed: boolean;
}>();

const emit = defineEmits<{
  "toggle-collapse": [];
  "update-active-page": [page: ConfigPage];
  "update-recording-sub-page": [page: RecordingSubPage];
}>();

function selectPage(page: ConfigPage) {
  emit("update-active-page", page);
}

function selectRecordingSubPage(page: RecordingSubPage) {
  emit("update-recording-sub-page", page);
}
</script>

<template>
  <aside :class="['sidebar', { collapsed }]" aria-label="Workspace navigation">
    <div class="brand">
      <Keyboard :size="22" aria-hidden="true" />
      <div v-if="!collapsed">
        <strong>Keyboard Display</strong>
        <span>Desktop POV overlay</span>
      </div>
      <button
        class="collapse-button"
        type="button"
        :aria-label="collapsed ? 'Expand sidebar' : 'Collapse sidebar'"
        @click="emit('toggle-collapse')"
      >
        {{ collapsed ? "›" : "‹" }}
      </button>
    </div>

    <nav class="nav-list" aria-label="Configuration pages">
      <template v-for="item in navItems" :key="item.id">
        <button
          :class="{ active: activePage === item.id }"
          type="button"
          @pointerdown="selectPage(item.id)"
          @click="selectPage(item.id)"
        >
          <component :is="item.icon" :size="18" aria-hidden="true" />
          <span v-if="!collapsed">{{ item.label }}</span>
        </button>
        <div
          v-if="!collapsed && item.id === 'recording' && activePage === 'recording'"
          class="subnav-list"
        >
          <button
            v-for="child in recordingNavItems"
            :key="child.id"
            :class="{ active: recordingSubPage === child.id }"
            type="button"
            @pointerdown="selectRecordingSubPage(child.id)"
            @click="selectRecordingSubPage(child.id)"
          >
            {{ child.label }}
          </button>
        </div>
      </template>
    </nav>
  </aside>
</template>

<style scoped>
.sidebar {
  height: 100vh;
  overflow-y: auto;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  background: #171a1f;
  padding: 22px 18px;
}

.sidebar.collapsed {
  padding: 22px 14px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 28px;
}

.sidebar.collapsed .brand {
  justify-content: center;
}

.collapse-button {
  margin-left: auto;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  background: #202630;
  color: #c9d1da;
  cursor: pointer;
  font-size: 16px;
  font-weight: 800;
  line-height: 1;
  padding: 4px 8px;
}

.sidebar.collapsed .collapse-button {
  margin-left: 0;
}

.brand span {
  color: #9ca7b4;
}

.brand strong,
.brand span {
  display: block;
}

.brand span {
  margin-top: 2px;
  font-size: 12px;
}

.nav-list {
  display: grid;
  gap: 6px;
}

.nav-list button {
  display: flex;
  align-items: center;
  gap: 10px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: #c9d1da;
  cursor: pointer;
  padding: 10px 11px;
  text-align: left;
  font-weight: 700;
  transition:
    background-color 140ms ease,
    color 140ms ease,
    transform 140ms ease;
}

.sidebar.collapsed .nav-list button {
  justify-content: center;
  padding: 10px;
}

.nav-list button:hover,
.nav-list button.active {
  background: rgba(255, 255, 255, 0.06);
}

.nav-list button:hover {
  transform: translateX(2px);
}

.nav-list button.active {
  color: #eafff0;
  transform: translateX(2px);
}

.subnav-list {
  display: grid;
  gap: 4px;
  margin: -2px 0 4px 28px;
}

.subnav-list button {
  min-height: 30px;
  padding: 7px 10px;
  color: #9ca7b4;
  font-size: 13px;
}

.subnav-list button.active {
  color: #eafff0;
}

@media (max-width: 920px) {
  .sidebar {
    border-right: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding: 22px 14px;
  }

  .brand div,
  .nav-list button span,
  .subnav-list {
    display: none;
  }

  .brand,
  .nav-list button {
    justify-content: center;
  }

  .collapse-button {
    margin-left: 0;
  }
}
</style>
