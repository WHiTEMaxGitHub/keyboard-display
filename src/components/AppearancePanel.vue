<script setup lang="ts">
import { normalizeHexColor } from "../domain/colorPicker";
import type { AppConfig, OverlayStyle } from "../domain/defaultConfig";
import BaseButton from "./BaseButton.vue";
import BaseSelect from "./BaseSelect.vue";
import ColorPicker from "./ColorPicker.vue";

const props = defineProps<{
  config: AppConfig;
  recentColors: string[];
}>();

const emit = defineEmits<{
  "preview-overlay-style": [style: OverlayStyle];
  "update-overlay-style": [style: OverlayStyle];
  "remember-color": [color: string];
  "refresh-pov": [];
}>();

function updateScale(event: Event) {
  const scale = Number((event.target as HTMLInputElement).value);
  emit("update-overlay-style", { ...props.config.style, scale });
}

function formatScale(scale: number) {
  return `${scale.toFixed(2)}x`;
}

function effectiveUnitPx() {
  return Math.round(props.config.layout.unitPx * props.config.style.scale);
}

function updateOpacity(event: Event) {
  const opacity = Number((event.target as HTMLInputElement).value);
  emit("update-overlay-style", { ...props.config.style, opacity });
}

function updateBackgroundRadius(event: Event) {
  const backgroundRadius = Number((event.target as HTMLInputElement).value);
  emit("update-overlay-style", {
    ...props.config.style,
    backgroundRadius,
  });
}

function updateIdleKeyVisibility(event: Event) {
  const idleKeyVisibility = (event.target as HTMLSelectElement)
    .value as OverlayStyle["idleKeyVisibility"];
  emit("update-overlay-style", { ...props.config.style, idleKeyVisibility });
}

function updateBackplateTransparent(event: Event) {
  const transparent = (event.target as HTMLInputElement).checked;
  emit("update-overlay-style", {
    ...props.config.style,
    backgroundColor: setHexAlpha(props.config.style.backgroundColor, transparent ? 0 : 255),
  });
}

function updateStyleColor(
  field:
    | "idleColor"
    | "activeColor"
    | "idleTextColor"
    | "activeTextColor"
    | "backgroundColor",
  color: string,
) {
  const nextColor = normalizeHexColor(color, props.config.style[field]);
  emit("update-overlay-style", {
    ...props.config.style,
    [field]: nextColor,
  });
}

function previewStyleColor(
  field:
    | "idleColor"
    | "activeColor"
    | "idleTextColor"
    | "activeTextColor"
    | "backgroundColor",
  color: string,
) {
  const nextColor = normalizeHexColor(color, props.config.style[field]);
  emit("preview-overlay-style", {
    ...props.config.style,
    [field]: nextColor,
  });
}

function isBackplateTransparent() {
  const normalizedColor = normalizeHexColor(props.config.style.backgroundColor);
  return normalizedColor.length === 9 && normalizedColor.endsWith("00");
}

function setHexAlpha(color: string, alpha: number) {
  const normalizedColor = normalizeHexColor(color);
  const rgb = normalizedColor.slice(0, 7);
  return `${rgb}${Math.min(255, Math.max(0, Math.round(alpha)))
    .toString(16)
    .padStart(2, "0")}`;
}
</script>

<template>
  <article class="box-border min-h-[190px] border border-border-default rounded-radius-lg bg-surface-panel p-[18px] w-full">
    <div class="flex items-center justify-between gap-3 mb-4">
      <h2 class="m-0 text-lg leading-6 tracking-normal">Appearance</h2>
      <BaseButton @click="emit('refresh-pov')">
        Refresh POV
      </BaseButton>
    </div>
    <label class="grid gap-2 mb-4 text-text-secondary font-bold">
      <span class="flex items-baseline justify-between gap-3">
        <span>Scale</span>
        <strong class="text-text-muted text-xs font-extrabold">{{ formatScale(config.style.scale) }} · {{ effectiveUnitPx() }}px unit</strong>
      </span>
      <input
        :value="config.style.scale"
        min="0.75"
        max="1.5"
        step="0.05"
        type="range"
        class="w-full"
        style="accent-color: var(--color-accent)"
        @input="updateScale"
      />
    </label>
    <label class="grid gap-2 mb-4 text-text-secondary font-bold">
      Overlay transparency
      <input
        :value="config.style.opacity"
        min="0.35"
        max="1"
        step="0.01"
        type="range"
        class="w-full"
        style="accent-color: var(--color-accent)"
        @input="updateOpacity"
      />
      <span class="text-text-subtle text-xs font-medium">Controls the whole POV overlay opacity.</span>
    </label>
    <div class="grid grid-cols-2 gap-3.5 mb-4">
      <label class="grid gap-2 mb-4 text-text-secondary font-bold">
        Backplate radius
        <input
          :value="config.style.backgroundRadius"
          min="0"
          max="24"
          step="1"
          type="range"
          class="w-full"
          style="accent-color: var(--color-accent)"
          @input="updateBackgroundRadius"
        />
      </label>
    </div>
    <label class="grid grid-cols-[minmax(120px,1fr)_minmax(180px,240px)] items-center gap-3 mb-4 text-text-secondary font-bold">
      <span class="min-w-0">Idle keys</span>
      <BaseSelect
        class="select-control justify-self-end w-[min(240px,100%)]"
        compact
        :model-value="config.style.idleKeyVisibility"
        @change="updateIdleKeyVisibility"
      >
        <option value="visible">Visible</option>
        <option value="hidden">Hidden until pressed</option>
      </BaseSelect>
    </label>
    <div class="grid grid-cols-2 gap-2.5" aria-label="Overlay colors">
      <ColorPicker
        label="Idle key"
        :value="config.style.idleColor"
        :recent-colors="recentColors"
        alpha-enabled
        @preview:value="previewStyleColor('idleColor', $event)"
        @update:value="updateStyleColor('idleColor', $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <ColorPicker
        label="Pressed key"
        :value="config.style.activeColor"
        :recent-colors="recentColors"
        alpha-enabled
        @preview:value="previewStyleColor('activeColor', $event)"
        @update:value="updateStyleColor('activeColor', $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <ColorPicker
        label="Idle text"
        :value="config.style.idleTextColor"
        :recent-colors="recentColors"
        alpha-enabled
        @preview:value="previewStyleColor('idleTextColor', $event)"
        @update:value="updateStyleColor('idleTextColor', $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <ColorPicker
        label="Pressed text"
        :value="config.style.activeTextColor"
        :recent-colors="recentColors"
        alpha-enabled
        @preview:value="previewStyleColor('activeTextColor', $event)"
        @update:value="updateStyleColor('activeTextColor', $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <ColorPicker
        label="Backplate"
        :value="config.style.backgroundColor"
        :recent-colors="recentColors"
        alpha-enabled
        @preview:value="previewStyleColor('backgroundColor', $event)"
        @update:value="updateStyleColor('backgroundColor', $event)"
        @remember-color="emit('remember-color', $event)"
      />
      <label class="flex items-center gap-2.5 min-h-[38px] m-0 text-text-secondary text-[13px] font-bold">
        <input
          :checked="isBackplateTransparent()"
          type="checkbox"
          class="w-[18px] h-[18px]"
          style="accent-color: var(--color-accent)"
          @change="updateBackplateTransparent"
        />
        Transparent backplate
      </label>
    </div>
  </article>
</template>

<style scoped>
.select-control {
  justify-self: end;
  width: min(240px, 100%);
}
</style>