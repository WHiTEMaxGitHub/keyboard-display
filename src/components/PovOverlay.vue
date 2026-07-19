<script setup lang="ts">
import { computed } from "vue";
import {
  isKeyBinding,
  type KeyBinding,
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
  activeKeys: Set<string>;
  overlayStyle: OverlayStyle;
  syncFeedbackActive?: boolean;
  adjusting?: boolean;
}>();

const platformKey = computed(() => detectPlatformKey());

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
</script>

<template>
  <section
    class="pov-shell"
    :class="[`idle-${overlayStyle.idleKeyVisibility}`]"
    :style="{
      '--overlay-scale': overlayStyle.scale,
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
    aria-label="POV key overlay"
  >
    <div
      :data-tauri-drag-region="adjusting ? true : undefined"
      :class="[
        'key-cluster',
        { 'backplate-visible': isBackplateVisible(overlayStyle) },
        { 'sync-feedback-active': syncFeedbackActive },
        { adjusting },
      ]"
    >
      <div class="backplate" aria-hidden="true"></div>

      <div class="row-layout" aria-label="Configured keyboard rows">
        <div v-for="(rowItems, rowIndex) in rows" :key="rowIndex" class="key-row">
          <template v-for="(item, itemIndex) in rowItems" :key="`${rowIndex}-${itemIndex}`">
            <span
              v-if="isKeyBinding(item)"
              class="key"
              :style="{ '--key-width-unit': normalizeUnit(item.widthUnit) }"
              :class="{ active: activeKeys.has(item.id), hidden: !isKeyVisible(item.id, activeKeys, overlayStyle) }"
            >
              {{ displayLabelForKey(item, platformKey) }}
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

.key-cluster {
  position: relative;
  width: max-content;
  border-radius: var(--overlay-bg-radius);
}

.key-cluster.adjusting {
  cursor: move;
}

.key-cluster.adjusting::before {
  position: absolute;
  inset: 0;
  z-index: 3;
  border: 2px solid rgba(37, 211, 102, 0.78);
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
  border-radius: 7px;
  background: var(--key-idle);
  color: var(--key-idle-text);
  box-shadow:
    inset 0 -3px 0 rgba(0, 0, 0, 0.35),
    0 6px 18px rgba(0, 0, 0, 0.24);
  font: 700 calc(15px * var(--overlay-scale)) / 1 Inter, system-ui, sans-serif;
  letter-spacing: 0;
  transition:
    background 90ms ease,
    border-color 90ms ease,
    box-shadow 90ms ease,
    transform 90ms ease;
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
  box-shadow:
    inset 0 -1px 0 rgba(0, 0, 0, 0.28),
    0 0 18px color-mix(in srgb, var(--key-active), transparent 34%);
  transform: translateY(2px);
}

.pov-shell.idle-faint .key:not(.active) {
  opacity: 0.32;
}

.pov-shell.idle-hidden .key {
  transition: none;
}

.key.hidden {
  visibility: hidden;
}

</style>
