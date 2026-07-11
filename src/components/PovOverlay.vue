<script setup lang="ts">
import { computed } from "vue";
import type { KeyBinding, OverlayLayout, OverlayStyle } from "../domain/defaultConfig";
import { detectPlatformKey, displayLabelForKey } from "../domain/keyLabels";
import { normalizeUnit } from "../domain/layoutUnits";

const props = defineProps<{
  layout: OverlayLayout;
  keys: KeyBinding[];
  activeKeys: Set<string>;
  overlayStyle: OverlayStyle;
}>();

const usesRowLayout = computed(() => props.keys.some((key) => key.row !== undefined));
const platformKey = computed(() => detectPlatformKey());

const rowLayoutKeys = computed(() => {
  const rowMap = new Map<number, KeyBinding[]>();

  for (const key of props.keys) {
    const row = key.row ?? 0;
    rowMap.set(row, [...(rowMap.get(row) ?? []), key]);
  }

  return [...rowMap.entries()]
    .sort(([left], [right]) => left - right)
    .map(([, keys]) => keys);
});

function isKeyVisible(keyId: string, activeKeys: Set<string>, overlayStyle: OverlayStyle) {
  return overlayStyle.idleKeyVisibility !== "hidden" || activeKeys.has(keyId);
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
      '--overlay-bg-opacity': overlayStyle.backgroundOpacity,
      '--overlay-bg-radius': `${overlayStyle.backgroundRadius}px`,
      '--key-idle': overlayStyle.idleColor,
      '--key-active': overlayStyle.activeColor,
      '--key-idle-text': overlayStyle.idleTextColor,
      '--key-active-text': overlayStyle.activeTextColor,
    }"
    aria-label="POV key overlay"
  >
    <div
      :class="[
        'key-cluster',
        `background-${overlayStyle.backgroundMode}`,
      ]"
    >
      <div class="backplate" aria-hidden="true"></div>

      <div v-if="usesRowLayout" class="row-layout" aria-label="Configured keyboard rows">
        <div v-for="(rowKeys, index) in rowLayoutKeys" :key="index" class="key-row">
          <span
            v-for="key in rowKeys"
            :key="key.id"
            class="key"
            :style="{ '--key-width-unit': normalizeUnit(key.widthUnit) }"
            :class="{ active: activeKeys.has(key.id), hidden: !isKeyVisible(key.id, activeKeys, overlayStyle) }"
          >
            {{ displayLabelForKey(key, platformKey) }}
          </span>
        </div>
      </div>

      <div v-else>
        <div class="mouse-row" aria-label="Mouse buttons">
          <span
            v-for="key in keys.filter((item) => item.group === 'mouse')"
            :key="key.id"
            class="key mouse-key"
            :style="{ '--key-width-unit': normalizeUnit(key.widthUnit) }"
            :class="{ active: activeKeys.has(key.id), hidden: !isKeyVisible(key.id, activeKeys, overlayStyle) }"
          >
            {{ displayLabelForKey(key, platformKey) }}
          </span>
        </div>

        <div class="keyboard-grid" aria-label="Keyboard keys">
          <span
            v-for="key in keys.filter((item) => item.group !== 'mouse')"
            :key="key.id"
            class="key"
            :style="{ '--key-width-unit': normalizeUnit(key.widthUnit) }"
            :class="[
              key.gridArea ? `area-${key.gridArea}` : '',
              { active: activeKeys.has(key.id), hidden: !isKeyVisible(key.id, activeKeys, overlayStyle) },
            ]"
          >
            {{ displayLabelForKey(key, platformKey) }}
          </span>
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
}

.key-cluster.background-panel {
  padding: 18px;
  border: 0;
  border-radius: var(--overlay-bg-radius);
  background: transparent;
}

.backplate {
  display: none;
}

.key-cluster.background-panel .backplate {
  position: absolute;
  inset: 0;
  display: block;
  z-index: 0;
  border-radius: inherit;
  background: var(--overlay-bg);
  opacity: var(--overlay-bg-opacity);
}

.key-cluster.background-panel .mouse-row,
.key-cluster.background-panel .keyboard-grid {
  position: relative;
  z-index: 1;
  opacity: 1;
}

.key-cluster.background-transparent {
  padding: 0;
  border: 0;
  background: transparent;
}

.mouse-row {
  display: grid;
  grid-template-columns: repeat(2, calc(var(--unit) * 1.35));
  gap: var(--gap);
  margin-bottom: var(--gap);
}

.row-layout {
  display: grid;
  gap: var(--gap);
}

.key-row {
  display: flex;
  gap: var(--gap);
}

.keyboard-grid {
  display: grid;
  grid-template-areas:
    ". w . r"
    "a s d q"
    "shift ctrl e ."
    "space space space space";
  grid-template-columns: repeat(4, var(--unit));
  grid-auto-rows: var(--unit);
  gap: var(--gap);
}

.key {
  display: grid;
  place-items: center;
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

.key.hidden {
  visibility: hidden;
}

.mouse-key {
  width: calc(var(--unit) * var(--key-width-unit));
}

.area-w {
  grid-area: w;
}

.area-a {
  grid-area: a;
}

.area-s {
  grid-area: s;
}

.area-d {
  grid-area: d;
}

.area-shift {
  grid-area: shift;
}

.area-ctrl {
  grid-area: ctrl;
}

.area-space {
  grid-area: space;
  width: calc(var(--unit) * var(--key-width-unit));
}

.area-r {
  grid-area: r;
}

.area-q {
  grid-area: q;
}

.area-e {
  grid-area: e;
}
</style>
