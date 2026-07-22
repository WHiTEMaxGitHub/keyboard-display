<script setup lang="ts">
import { SlidersHorizontal } from "@lucide/vue";
import BaseButton from "./BaseButton.vue";

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

@media (max-width: 920px) {
  .topbar {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
