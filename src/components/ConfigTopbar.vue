<script setup lang="ts">
import { SlidersHorizontal } from "@lucide/vue";

const emit = defineEmits<{
  "load-config": [];
  "export-and-apply-config": [];
  "overwrite-and-apply-config": [];
}>();
</script>

<template>
  <header class="topbar">
    <div>
      <p>Configuration</p>
      <h1>POV overlay control panel</h1>
    </div>
    <div class="topbar-actions">
      <button class="load-config-button" type="button" @click="emit('load-config')">
        <SlidersHorizontal :size="15" aria-hidden="true" />
        Load config
      </button>
      <button class="save-apply-button" type="button" @click="emit('export-and-apply-config')">
        Export & Apply
      </button>
      <button class="save-apply-button" type="button" @click="emit('overwrite-and-apply-config')">
        Overwrite & Apply
      </button>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
}

.topbar-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.topbar p {
  margin: 0 0 4px;
  color: #9ca7b4;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h1 {
  margin: 0;
  font-size: 28px;
  line-height: 34px;
  letter-spacing: 0;
}

.load-config-button,
.save-apply-button {
  display: inline-flex;
  align-items: center;
  box-sizing: border-box;
  gap: 7px;
  height: 34px;
  margin: 0;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  appearance: none;
  background: #1d2229;
  color: #eef2f6;
  cursor: pointer;
  padding: 0 10px;
  font-size: 14px;
  font-weight: 750;
  line-height: 1;
  white-space: nowrap;
}

.save-apply-button {
  border-color: rgba(37, 211, 102, 0.24);
  background: rgba(37, 211, 102, 0.1);
  color: #eafff0;
}

@media (max-width: 920px) {
  .topbar {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
