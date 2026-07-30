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
  <aside :class="['sidebar h-screen overflow-y-auto border-r border-border-default bg-surface-sidebar', collapsed ? 'px-3.5 py-[22px]' : 'px-[18px] py-[22px]']" aria-label="Workspace navigation">
    <div :class="['flex items-center gap-3 mb-7', collapsed && 'justify-center']">
      <Keyboard :size="22" aria-hidden="true" />
      <div v-if="!collapsed">
        <strong class="block">Keyboard Display</strong>
        <span class="block mt-0.5 text-xs text-text-muted">Desktop POV overlay</span>
      </div>
      <button
        :class="['border border-border-control rounded-radius-sm bg-surface-control text-text-secondary cursor-pointer text-base font-extrabold leading-none px-2 py-1', collapsed ? 'ml-0' : 'ml-auto']"
        type="button"
        :aria-label="collapsed ? 'Expand sidebar' : 'Collapse sidebar'"
        @click="emit('toggle-collapse')"
      >
        {{ collapsed ? "›" : "‹" }}
      </button>
    </div>

    <nav class="grid gap-1.5" aria-label="Configuration pages">
      <template v-for="item in navItems" :key="item.id">
        <button
          :class="[
            'flex items-center gap-2.5 border-0 rounded-radius-md bg-transparent text-text-secondary cursor-pointer px-[11px] py-2.5 text-left font-bold transition-[background-color,color,transform] duration-[140ms] ease',
            collapsed && 'justify-center px-2.5',
            'hover:bg-white/6 hover:translate-x-0.5',
            activePage === item.id ? 'bg-white/6 text-accent-text translate-x-0.5' : '',
          ]"
          type="button"
          @pointerdown="selectPage(item.id)"
          @click="selectPage(item.id)"
        >
          <component :is="item.icon" :size="18" aria-hidden="true" />
          <span v-if="!collapsed">{{ item.label }}</span>
        </button>
        <div
          v-if="!collapsed && item.id === 'recording' && activePage === 'recording'"
          class="grid gap-1 -mt-0.5 mb-1 ml-7"
        >
          <button
            v-for="child in recordingNavItems"
            :key="child.id"
            :class="[
              'min-h-[30px] border-0 rounded-radius-md bg-transparent text-text-muted cursor-pointer text-left text-[13px] font-bold px-2.5 py-[7px] transition-[background-color,color,transform] duration-[140ms] ease',
              'hover:bg-white/6 hover:translate-x-0.5',
              recordingSubPage === child.id ? 'text-accent-text bg-white/6 translate-x-0.5' : '',
            ]"
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
@media (max-width: 920px) {
  .sidebar {
    border-right: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding: 22px 14px;
  }

  .sidebar div:first-child > div,
  .sidebar button span,
  .sidebar .grid.gap-1 {
    display: none;
  }

  .sidebar > div:first-child,
  .sidebar button {
    justify-content: center;
  }

  .sidebar button.ml-auto {
    margin-left: 0;
  }
}
</style>