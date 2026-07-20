<script setup lang="ts">
defineProps<{
  renderMarkers: boolean;
}>();

const emit = defineEmits<{
  "update-render-markers": [event: Event];
}>();
</script>

<template>
  <article class="panel wide-panel">
    <h2>Export</h2>
    <div class="field-row">
      <span>Transparent overlay</span>
      <strong>WebM</strong>
    </div>
    <div class="field-row">
      <span>Compatible video</span>
      <strong>MP4</strong>
    </div>
    <label class="toggle-row">
      <input
        :checked="renderMarkers"
        type="checkbox"
        @change="emit('update-render-markers', $event)"
      />
      Render sync markers
    </label>
    <p class="quiet">
      Video is generated from the input timeline, so size and format can be
      tuned after recording.
    </p>
  </article>
</template>

<style scoped>
.panel {
  min-height: 190px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #171b22;
  padding: 18px;
}

.wide-panel {
  max-width: 760px;
}

.panel h2 {
  margin: 0 0 16px;
  letter-spacing: 0;
}

.field-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.field-row span,
.quiet {
  color: #9ca7b4;
}

.field-row strong {
  min-width: 0;
  overflow-wrap: anywhere;
  text-align: right;
}

.toggle-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
  color: #c9d1da;
  font-weight: 700;
}

.toggle-row input {
  width: 18px;
  height: 18px;
  accent-color: #25d366;
}

.quiet {
  margin: 14px 0 0;
}
</style>
