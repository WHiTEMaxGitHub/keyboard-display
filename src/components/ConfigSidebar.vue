<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import {
  Clapperboard,
  Eye,
  Keyboard,
  MonitorCog,
  MonitorUp,
  Palette,
  Settings,
  Video,
} from "@lucide/vue";

type ConfigPage =
  | "overview"
  | "layout"
  | "appearance"
  | "window"
  | "recording"
  | "export"
  | "settings";
type RecordingSubPage = "control" | "files";

const navItems: Array<{
  id: ConfigPage;
  labelKey: string;
  icon: typeof Eye;
}> = [
  { id: "overview", labelKey: "nav.overview", icon: Eye },
  { id: "layout", labelKey: "nav.layout", icon: MonitorUp },
  { id: "appearance", labelKey: "nav.appearance", icon: Palette },
  { id: "window", labelKey: "nav.window", icon: MonitorCog },
  { id: "recording", labelKey: "nav.recording", icon: Clapperboard },
  { id: "export", labelKey: "nav.export", icon: Video },
  { id: "settings", labelKey: "nav.settings", icon: Settings },
];

const recordingNavItems: Array<{ id: RecordingSubPage; labelKey: string }> = [
  { id: "control", labelKey: "nav.recordingControl" },
  { id: "files", labelKey: "nav.recordingFiles" },
];

defineProps<{
  activePage: ConfigPage;
  recordingSubPage: RecordingSubPage;
}>();

const emit = defineEmits<{
  "update-active-page": [page: ConfigPage];
  "update-recording-sub-page": [page: RecordingSubPage];
}>();

const expanded = ref(false);
const { t } = useI18n();

function selectPage(page: ConfigPage) {
  emit("update-active-page", page);
}

function selectRecordingSubPage(page: RecordingSubPage) {
  emit("update-recording-sub-page", page);
}
</script>

<template>
  <aside
    class="sidebar fixed top-0 left-0 z-50 h-screen overflow-y-auto overflow-x-hidden rounded-r-[28px] border-r border-[var(--glass-border)] bg-gradient-to-br from-[var(--glass-from)] to-[var(--glass-to)] backdrop-blur-2xl backdrop-saturate-[170%] shadow-[var(--glass-shadow)] transition-[width] duration-[300ms] ease-out py-[22px] w-[48px] hover:w-[228px]"
    aria-label="Workspace navigation"
    @mouseenter="expanded = true"
    @mouseleave="expanded = false"
  >
    <div class="flex items-center gap-3 mb-7 px-3.5">
      <Keyboard :size="22" class="shrink-0" aria-hidden="true" />
      <div class="whitespace-nowrap opacity-0 transition-opacity duration-250 delay-100" :class="{ 'opacity-100': expanded }">
        <strong class="block">{{ t("app.name") }}</strong>
        <span class="block mt-0.5 text-xs text-text-muted">{{ t("app.tagline") }}</span>
      </div>
    </div>

    <nav class="grid gap-1.5" aria-label="Configuration pages">
      <template v-for="item in navItems" :key="item.id">
        <button
          :class="[
            'flex items-center gap-2.5 border-0 rounded-md bg-transparent cursor-pointer w-full text-left font-bold transition-[background-color,color,transform] duration-[140ms] ease px-3.5',
            'min-h-[38px]',
            'hover:bg-white/6 hover:translate-x-0.5',
            activePage === item.id ? 'bg-white/6 text-accent-text translate-x-0.5' : 'text-text-secondary',
          ]"
          type="button"
          @pointerdown="selectPage(item.id)"
          @click="selectPage(item.id)"
        >
          <component :is="item.icon" :size="18" class="shrink-0" aria-hidden="true" />
          <span class="whitespace-nowrap opacity-0 transition-opacity duration-250 delay-100" :class="{ 'opacity-100': expanded }">{{ t(item.labelKey) }}</span>
        </button>
        <div
          v-if="item.id === 'recording' && activePage === 'recording'"
          class="grid gap-1 -mt-0.5 mb-1 ml-10 opacity-0 transition-opacity duration-250 delay-100"
          :class="{ 'opacity-100': expanded }"
        >
          <button
            v-for="child in recordingNavItems"
            :key="child.id"
            :class="[
              'min-h-[30px] border-0 rounded-md bg-transparent text-text-muted cursor-pointer text-left text-[13px] font-bold px-2.5 py-[7px] transition-[background-color,color,transform] duration-[140ms] ease whitespace-nowrap',
              'hover:bg-white/6 hover:translate-x-0.5',
              recordingSubPage === child.id ? 'text-accent-text bg-white/6 translate-x-0.5' : '',
            ]"
            type="button"
            @pointerdown="selectRecordingSubPage(child.id)"
            @click="selectRecordingSubPage(child.id)"
          >
            {{ t(child.labelKey) }}
          </button>
        </div>
      </template>
    </nav>
  </aside>
</template>
