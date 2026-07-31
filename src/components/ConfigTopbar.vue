<script setup lang="ts">
import { SlidersHorizontal } from "@lucide/vue";
import type { ThemeId } from "../domain/theme";
import { THEMES } from "../domain/theme";
import BaseButton from "./BaseButton.vue";

defineProps<{
  profileName: string;
  themeId: ThemeId;
}>();

const emit = defineEmits<{
  "load-config": [];
  "export-and-apply-config": [];
  "overwrite-and-apply-config": [];
  "set-theme": [id: ThemeId];
}>();

const themeOptions = Object.values(THEMES);
</script>

<template>
  <header class="sticky top-0 z-10 flex items-center justify-between gap-4 mb-5 px-6 py-4 mx-4 mt-4 rounded-[28px] bg-gradient-to-br from-[var(--glass-from)] to-[var(--glass-to)] backdrop-blur-2xl backdrop-saturate-[170%] border border-[var(--glass-border)] shadow-[var(--glass-shadow)] max-[920px]:flex-col max-[920px]:items-start">
    <div>
      <p class="m-0 mb-1 text-text-muted text-xs font-bold tracking-[0.08em] uppercase">Profile</p>
      <h1 class="m-0 text-[28px] leading-[34px] tracking-normal">{{ profileName }}</h1>
    </div>
    <div class="flex flex-wrap justify-end gap-2 items-center">
      <select
        :value="themeId"
        class="text-xs font-bold px-3 py-1.5 rounded-md border border-border-control bg-surface-control text-text-body cursor-pointer"
        @change="emit('set-theme', ($event.target as HTMLSelectElement).value as ThemeId)"
      >
        <option
          v-for="t in themeOptions"
          :key="t.id"
          :value="t.id"
        >
          {{ t.label }}
        </option>
      </select>
      <BaseButton @click="emit('load-config')">
        <SlidersHorizontal :size="15" aria-hidden="true" />
        Load config
      </BaseButton>
      <BaseButton variant="primary" @click="emit('export-and-apply-config')">
        Export & Apply
      </BaseButton>
      <BaseButton variant="primary" @click="emit('overwrite-and-apply-config')">
        Overwrite & Apply
      </BaseButton>
    </div>
  </header>
</template>