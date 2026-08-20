<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  isKeyBinding,
  type KeyBinding,
  type KeyIdLabelRegistry,
  type OverlayLayout,
  type OverlayRow,
  type OverlayStyle,
} from "../domain/defaultConfig";
import { detectPlatformKey, displayLabelForKey } from "../domain/keyLabels";
import { normalizeUnit } from "../domain/layoutUnits";

const props = defineProps<{
  layout: OverlayLayout;
  rows: OverlayRow[];
  keys: KeyBinding[];
  keyIdLabels?: KeyIdLabelRegistry;
  activeKeys: Set<string>;
  overlayStyle: OverlayStyle;
  syncFeedbackActive?: boolean;
  adjusting?: boolean;
  fitToContainer?: boolean;
}>();

const emit = defineEmits<{
  "start-drag": [];
}>();

const { t } = useI18n();
const platformKey = computed(() => detectPlatformKey());
const shellRef = ref<HTMLElement | null>(null);
const clusterRef = ref<HTMLElement | null>(null);
const fitScale = ref(1);
const fitHeight = ref(160);
let resizeObserver: ResizeObserver | undefined;

function isKeyVisible(keyId: string, activeKeys: Set<string>, overlayStyle: OverlayStyle) {
  return overlayStyle.idleKeyVisibility !== "hidden" || activeKeys.has(keyId);
}

function backplateOpacity(overlayStyle: OverlayStyle) {
  return /^#[0-9a-fA-F]{8}$/.test(overlayStyle.backgroundColor)
    ? 1
    : overlayStyle.backgroundOpacity;
}

function isBackplateVisible(overlayStyle: OverlayStyle) {
  return !/^#[0-9a-fA-F]{8}$/.test(overlayStyle.backgroundColor) ||
    !overlayStyle.backgroundColor.endsWith("00");
}

function updateFitScale() {
  if (!props.fitToContainer) {
    fitScale.value = 1;
    return;
  }

  const shell = shellRef.value;
  const cluster = clusterRef.value;
  if (!shell || !cluster) {
    return;
  }

  const style = getComputedStyle(shell);
  const declaredWidth = Number.parseFloat(
    style.getPropertyValue("--preview-available-width"),
  );
  const parent = shell.parentElement;
  const availableWidth = Math.max(
    1,
    Number.isFinite(declaredWidth) && declaredWidth > 0
      ? declaredWidth
      : parent?.clientWidth ?? shell.clientWidth,
  );
  const contentWidth = Math.max(1, cluster.scrollWidth);
  fitScale.value = Math.min(1, availableWidth / contentWidth);
  fitHeight.value = Math.ceil(cluster.scrollHeight * fitScale.value);
}

onMounted(() => {
  updateFitScale();
  resizeObserver = new ResizeObserver(updateFitScale);
  if (shellRef.value) resizeObserver.observe(shellRef.value);
});

onUnmounted(() => {
  resizeObserver?.disconnect();
});

watch(
  () => [props.layout, props.rows, props.overlayStyle, props.fitToContainer],
  () => requestAnimationFrame(updateFitScale),
  { deep: true },
);
</script>

<template>
  <section
    ref="shellRef"
    class="pov-shell"
    :class="[
      `idle-${overlayStyle.idleKeyVisibility}`,
      {
        'fit-to-container': fitToContainer,
        'enhanced-key-style': overlayStyle.enhancedKeyStyle,
      },
    ]"
    :style="{
      '--overlay-scale': overlayStyle.scale,
      '--preview-fit-scale': fitScale,
      '--preview-fit-height': `${fitHeight}px`,
      '--unit-px': `${layout.unitPx}px`,
      '--gap-unit': normalizeUnit(layout.gapUnit),
      '--overlay-opacity': overlayStyle.opacity,
      '--overlay-bg': overlayStyle.backgroundColor,
      '--overlay-bg-opacity': backplateOpacity(overlayStyle),
      '--overlay-bg-radius': `${overlayStyle.backgroundRadius}px`,
      '--key-idle': overlayStyle.idleColor,
      '--key-active': overlayStyle.activeColor,
      '--key-idle-text': overlayStyle.idleTextColor,
      '--key-active-text': overlayStyle.activeTextColor,
    }"
    :aria-label="t('overlay.keyOverlay')"
  >
    <div
      ref="clusterRef"
      :data-tauri-drag-region="adjusting ? true : undefined"
      :class="[
        'key-cluster',
        { 'backplate-visible': isBackplateVisible(overlayStyle) },
        { 'sync-feedback-active': syncFeedbackActive },
        { adjusting },
      ]"
      @mousedown="adjusting && emit('start-drag')"
    >
      <div class="backplate" aria-hidden="true"></div>

      <div class="row-layout" :aria-label="t('overlay.configuredRows')">
        <div v-for="(rowItems, rowIndex) in rows" :key="rowIndex" class="key-row">
          <template v-for="(item, itemIndex) in rowItems" :key="`${rowIndex}-${itemIndex}`">
            <span
              v-if="isKeyBinding(item)"
              class="key"
              :style="{ '--key-width-unit': normalizeUnit(item.widthUnit) }"
              :class="{ active: activeKeys.has(item.id), hidden: !isKeyVisible(item.id, activeKeys, overlayStyle) }"
            >
              {{ displayLabelForKey(item, platformKey, keyIdLabels) }}
            </span>
            <span
              v-else
              class="key-gap"
              :style="{ '--key-width-unit': normalizeUnit(item.widthUnit) }"
              aria-hidden="true"
            ></span>
          </template>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.pov-shell {
  --unit: calc(var(--unit-px) * var(--overlay-scale));
  --gap: calc(var(--unit) * var(--gap-unit));

  width: max-content;
  opacity: var(--overlay-opacity);
  user-select: none;
}

.pov-shell.fit-to-container {
  display: grid;
  place-items: center;
  width: 100%;
  min-width: 0;
  height: var(--preview-fit-height);
  overflow: hidden;
}

.key-cluster {
  position: relative;
  width: max-content;
  border-radius: var(--overlay-bg-radius);
  transform-origin: left center;
}

.pov-shell.fit-to-container .key-cluster {
  transform-origin: center center;
  transform: scale(var(--preview-fit-scale));
}

.key-cluster.adjusting {
  cursor: move;
}

.key-cluster.adjusting::before {
  position: absolute;
  inset: 0;
  z-index: 3;
  border: 2px solid var(--color-accent-focus-border);
  border-radius: inherit;
  content: "";
  pointer-events: none;
}

.key-cluster.backplate-visible {
  padding: 10px;
  border: 0;
  border-radius: var(--overlay-bg-radius);
  background: transparent;
}

.backplate {
  display: none;
}

.key-cluster.backplate-visible .backplate {
  position: absolute;
  inset: 0;
  display: block;
  z-index: 0;
  border-radius: inherit;
  background: var(--overlay-bg);
  opacity: var(--overlay-bg-opacity);
}

.key-cluster.backplate-visible .row-layout {
  position: relative;
  z-index: 1;
  opacity: 1;
}

.key-cluster:not(.backplate-visible) {
  padding: 0;
  border: 0;
  background: transparent;
}

.key-cluster.sync-feedback-active::after {
  position: absolute;
  inset: 0;
  z-index: 2;
  border: 2px solid color-mix(in srgb, var(--key-active), white 16%);
  border-radius: var(--overlay-bg-radius);
  box-shadow:
    0 0 0 2px color-mix(in srgb, var(--key-active), transparent 48%),
    0 0 22px color-mix(in srgb, var(--key-active), transparent 18%);
  content: "";
  pointer-events: none;
}

.row-layout {
  display: grid;
  gap: var(--gap);
}

.key-row {
  display: flex;
}

.key-row > .key + .key {
  margin-left: var(--gap);
}

.key {
  display: grid;
  place-items: center;
  box-sizing: border-box;
  min-width: 0;
  width: calc(var(--unit) * var(--key-width-unit, 1));
  height: var(--unit);
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: var(--radius-md);
  background: var(--key-idle);
  color: var(--key-idle-text);
  box-shadow: none;
  font: 700 calc(15px * var(--overlay-scale)) / 1 Inter, system-ui, sans-serif;
  letter-spacing: 0;
  transition: none;
}

.key-gap {
  display: block;
  width: calc(var(--unit) * var(--key-width-unit, 1));
  height: var(--unit);
  flex: 0 0 auto;
}

.key.active {
  border-color: rgba(255, 255, 255, 0.5);
  background: var(--key-active);
  color: var(--key-active-text);
}

.pov-shell.enhanced-key-style .key {
  box-shadow:
    inset 0 -3px 0 rgba(0, 0, 0, 0.35),
    0 6px 18px rgba(0, 0, 0, 0.24);
  transition:
    background-color 55ms ease-out,
    border-color 55ms ease-out,
    transform 55ms ease-out;
}

.pov-shell.enhanced-key-style .key.active {
  box-shadow:
    inset 0 -1px 0 rgba(0, 0, 0, 0.28),
    0 0 18px color-mix(in srgb, var(--key-active), transparent 34%);
  transform: translateY(2px);
  transition-duration: 0ms;
}

.pov-shell.idle-hidden .key {
  transition: none;
}

.key.hidden {
  visibility: hidden;
}
</style>
