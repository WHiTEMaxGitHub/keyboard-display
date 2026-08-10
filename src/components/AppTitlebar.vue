<script setup lang="ts">
import { computed } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "vue-i18n";

type Platform = "macos" | "windows";

const { t } = useI18n();

const platform = computed<Platform>(() => {
  const platformName = navigator.platform.toLowerCase();
  return platformName.includes("mac") ? "macos" : "windows";
});

const controls = computed(() => {
  const items = [
    { id: "close", label: t("windowControls.close"), action: closeWindow },
    { id: "minimize", label: t("windowControls.minimize"), action: minimizeWindow },
    { id: "maximize", label: t("windowControls.maximize"), action: toggleMaximize },
  ];

  return platform.value === "macos" ? items : [items[1], items[2], items[0]];
});

function currentWindow() {
  return getCurrentWindow();
}

async function closeWindow() {
  document.documentElement.classList.add("app-window-closing");
  await waitForCloseAnimation();
  await currentWindow().close();
}

async function minimizeWindow() {
  await currentWindow().minimize();
}

async function toggleMaximize() {
  await currentWindow().toggleMaximize();
}

async function startDrag(event: MouseEvent) {
  if (event.detail > 1) {
    await toggleMaximize();
    return;
  }

  await currentWindow().startDragging();
}

function waitForCloseAnimation() {
  return new Promise<void>((resolve) => {
    window.setTimeout(resolve, 150);
  });
}
</script>

<template>
  <header
    :class="['app-titlebar', `app-titlebar-${platform}`]"
    data-tauri-drag-region
    @mousedown="startDrag"
  >
    <div :class="['window-controls', `window-controls-${platform}`]">
      <button
        v-for="control in controls"
        :key="control.id"
        :class="['window-control', `window-control-${control.id}`]"
        type="button"
        :aria-label="control.label"
        :title="control.label"
        @mousedown.stop
        @click.stop="control.action"
      >
        <span aria-hidden="true" />
      </button>
    </div>

    <div class="titlebar-title" data-tauri-drag-region>
      {{ t("app.name") }}
    </div>
  </header>
</template>

<style scoped>
.app-titlebar {
  --titlebar-height: var(--app-titlebar-height, 32px);

  position: relative;
  display: grid;
  grid-template-columns: 96px minmax(0, 1fr) 96px;
  align-items: center;
  height: var(--titlebar-height);
  border-bottom: 1px solid var(--color-border-dim);
  border-radius: 16px 16px 0 0;
  background: color-mix(in srgb, var(--color-surface-base) 88%, black 12%);
  color: var(--color-text-secondary);
  user-select: none;
  -webkit-user-select: none;
}

.app-titlebar-windows {
  grid-template-columns: 96px minmax(0, 1fr) 138px;
}

.titlebar-title {
  grid-column: 2;
  min-width: 0;
  overflow: hidden;
  font-size: 12px;
  font-weight: 800;
  text-align: center;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.window-controls {
  display: flex;
  align-items: center;
  height: 100%;
}

.window-controls-macos {
  grid-column: 1;
  gap: 8px;
  padding-left: 13px;
}

.window-controls-windows {
  grid-column: 3;
  justify-self: end;
}

.window-control {
  display: grid;
  place-items: center;
  border: 0;
  background: transparent;
  color: var(--color-text-muted);
  cursor: default;
  font: inherit;
  padding: 0;
  transition:
    background-color 120ms ease,
    color 120ms ease,
    filter 120ms ease,
    transform 120ms ease;
}

.window-controls-macos .window-control {
  width: 12px;
  height: 12px;
  border-radius: 999px;
}

.window-controls-windows .window-control {
  width: 46px;
  height: var(--titlebar-height);
}

.window-controls-windows .window-control:hover {
  background: var(--color-surface-control-hover);
  color: var(--color-text-primary);
}

.window-controls-windows .window-control:active {
  background: color-mix(in srgb, var(--color-surface-control-hover) 72%, black 28%);
}

.window-controls-windows .window-control-close:hover {
  background: #c42b1c;
  color: white;
}

.window-controls-macos .window-control-close {
  background: #ff5f57;
}

.window-controls-macos .window-control-minimize {
  background: #febc2e;
}

.window-controls-macos .window-control-maximize {
  background: #28c840;
}

.window-controls-macos .window-control:hover {
  filter: brightness(1.08);
  transform: scale(1.04);
}

.window-controls-macos:hover .window-control span {
  opacity: 1;
}

.window-control span {
  position: relative;
}

.window-controls-macos .window-control span {
  width: 7px;
  height: 7px;
  color: rgba(30, 30, 30, 0.72);
  opacity: 0;
  transition: opacity 120ms ease;
}

.window-controls-macos .window-control span::before,
.window-controls-macos .window-control span::after {
  content: "";
  position: absolute;
  border-radius: 999px;
  background: currentColor;
}

.window-controls-macos .window-control-close span::before,
.window-controls-macos .window-control-close span::after {
  left: 0;
  right: 0;
  top: 3px;
  height: 1.4px;
}

.window-controls-macos .window-control-close span::before {
  transform: rotate(45deg);
}

.window-controls-macos .window-control-close span::after {
  transform: rotate(-45deg);
}

.window-controls-macos .window-control-minimize span::before {
  left: 0;
  right: 0;
  top: 3px;
  height: 1.4px;
}

.window-controls-macos .window-control-maximize span::before {
  inset: 1px;
  border-radius: 1px;
  background: transparent;
  border: 1.4px solid currentColor;
}

.window-controls-windows .window-control span {
  width: 12px;
  height: 12px;
}

.window-controls-windows .window-control-minimize span::before,
.window-controls-windows .window-control-close span::before,
.window-controls-windows .window-control-close span::after {
  content: "";
  position: absolute;
  left: 1px;
  right: 1px;
  top: 50%;
  height: 1.5px;
  border-radius: 999px;
  background: currentColor;
}

.window-controls-windows .window-control-maximize span {
  border: 1.5px solid currentColor;
  border-radius: 2px;
}

.window-controls-windows .window-control-close span::before {
  transform: rotate(45deg);
}

.window-controls-windows .window-control-close span::after {
  transform: rotate(-45deg);
}
</style>
