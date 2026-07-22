<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  buildRecordingTimelineMarkers,
  type RecordingTimelineMarker,
} from "../domain/recordingTimeline";
import type { RecordingInspection, RecordingInspectionEvent } from "../types/recording";
import BaseFieldRow from "./BaseFieldRow.vue";

const props = defineProps<{
  inspection: RecordingInspection;
}>();

const selectedTimelineMarker = ref<RecordingTimelineMarker | null>(null);

const keyIds = computed(() => props.inspection.keyIds ?? []);
const events = computed(() => props.inspection.events ?? []);
const frames = computed(() => props.inspection.frames ?? []);

const timelineMarkers = computed(() => {
  return buildRecordingTimelineMarkers({
    events: events.value,
    fps: props.inspection.fps,
    frameCount: frames.value.length,
  });
});

watch(
  timelineMarkers,
  (markers) => {
    selectedTimelineMarker.value = markers[0] ?? null;
  },
  { immediate: true },
);

function formatInspectionEvent(event: RecordingInspectionEvent) {
  if ("down" in event) {
    return `frame ${event.frame} down ${event.down}`;
  }

  if ("up" in event) {
    return `frame ${event.frame} up ${event.up}`;
  }

  return `frame ${event.frame} marker ${event.marker}`;
}

function markerEvents(events: RecordingInspectionEvent[]) {
  return events.filter((event): event is Extract<RecordingInspectionEvent, { marker: string }> =>
    "marker" in event
  );
}

function timelineMarkerPosition(percent: number) {
  return `calc(10px + ${percent / 100} * (100% - 20px))`;
}

/// 把 marker 帧号格式化成剪辑软件容易对齐的 HH:MM:SS:FF 形式。
function formatFrameTimecode(frame: number, fps: number) {
  const totalSeconds = Math.floor(frame / fps);
  const frameInSecond = frame % fps;
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${pad2(hours)}:${pad2(minutes)}:${pad2(seconds)}:${padFrame(frameInSecond, fps)} @ ${fps}fps`;
}

function pad2(value: number) {
  return String(value).padStart(2, "0");
}

function padFrame(frame: number, fps: number) {
  return String(frame).padStart(String(Math.max(fps - 1, 0)).length, "0");
}
</script>

<template>
  <div class="inspection-grid">
    <BaseFieldRow label="Version">{{ inspection.version }}</BaseFieldRow>
    <BaseFieldRow label="FPS">{{ inspection.fps }}</BaseFieldRow>
    <BaseFieldRow label="Keys">{{ keyIds.length }}</BaseFieldRow>
    <BaseFieldRow label="Events">{{ events.length }}</BaseFieldRow>
    <BaseFieldRow label="Frames">{{ frames.length }}</BaseFieldRow>
    <BaseFieldRow label="Markers">
      {{ events.filter((event) => "marker" in event).length }}
    </BaseFieldRow>
  </div>

  <div class="marker-timeline-panel">
    <div class="section-header">
      <h3>Marker timeline</h3>
      <span class="quiet">
        {{ frames.length }} frames
      </span>
    </div>
    <div v-if="timelineMarkers.length" class="marker-timeline" aria-label="Marker timeline">
      <button
        v-for="(marker, index) in timelineMarkers"
        :key="`${marker.frame}-${marker.name}-${index}`"
        type="button"
        class="timeline-marker"
        :class="{ selected: selectedTimelineMarker === marker }"
        :style="{ left: timelineMarkerPosition(marker.percent) }"
        :title="`${marker.name} · frame ${marker.frame} · ${marker.timecode}`"
        @click="selectedTimelineMarker = marker"
      >
        <span class="timeline-marker-dot" />
        <span class="timeline-marker-label">{{ marker.name }}</span>
      </button>
    </div>
    <p v-else class="quiet">No markers in this recording.</p>
    <div v-if="selectedTimelineMarker" class="timeline-marker-detail">
      <strong>{{ selectedTimelineMarker.name }}</strong>
      <span>frame {{ selectedTimelineMarker.frame }}</span>
      <span>{{ selectedTimelineMarker.timecode }}</span>
    </div>
  </div>

  <div class="inspection-lists">
    <div>
      <h4>Markers</h4>
      <div class="marker-metadata-list">
        <div
          v-for="(event, index) in markerEvents(events)"
          :key="`${event.frame}-${event.marker}-${index}`"
          class="marker-metadata"
        >
          <strong>marker {{ event.marker }}</strong>
          <span>frame {{ event.frame }}</span>
          <span>time {{ formatFrameTimecode(event.frame, inspection.fps) }}</span>
        </div>
      </div>
    </div>
    <div>
      <h4>Key table</h4>
      <p class="quiet">{{ keyIds.join(", ") || "None" }}</p>
    </div>
    <div>
      <h4>Events</h4>
      <ol>
        <li
          v-for="(event, index) in events.slice(0, 8)"
          :key="index"
        >
          {{ formatInspectionEvent(event) }}
        </li>
      </ol>
    </div>
    <div>
      <h4>Frames</h4>
      <ol>
        <li
          v-for="frame in frames.slice(0, 8)"
          :key="frame.frame"
        >
          frame {{ frame.frame }}: {{ frame.keys.join(", ") || "none" }}
        </li>
      </ol>
    </div>
  </div>
</template>

<style scoped>
.quiet {
  color: #9ca7b4;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.section-header h3,
.inspection-lists h4 {
  margin: 0;
  letter-spacing: 0;
}

.section-header h3 {
  font-size: 16px;
  line-height: 22px;
}

.inspection-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0 18px;
}

.marker-timeline-panel {
  display: grid;
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: #151a20;
  padding: 14px;
}

.marker-timeline {
  position: relative;
  height: 62px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background:
    linear-gradient(90deg, rgba(255, 255, 255, 0.08) 1px, transparent 1px)
      0 0 / 25% 100%,
    #10141a;
}

.marker-timeline::before {
  position: absolute;
  right: 10px;
  bottom: 13px;
  left: 10px;
  height: 2px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.18);
  content: "";
}

.timeline-marker {
  position: absolute;
  bottom: 9px;
  display: grid;
  justify-items: center;
  gap: 5px;
  min-width: 36px;
  max-width: 96px;
  border: 0;
  background: transparent;
  color: #dfe5ec;
  cursor: pointer;
  font: inherit;
  padding: 0;
  transform: translateX(-50%);
}

.timeline-marker-dot {
  width: 12px;
  height: 12px;
  border: 2px solid #10141a;
  border-radius: 999px;
  background: #9ff0b9;
  box-shadow: 0 0 0 1px rgba(159, 240, 185, 0.65);
}

.timeline-marker-label {
  max-width: 96px;
  overflow: hidden;
  color: #c9d1da;
  font-size: 11px;
  font-weight: 700;
  line-height: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.timeline-marker.selected .timeline-marker-dot,
.timeline-marker:focus-visible .timeline-marker-dot {
  background: #ffd166;
  box-shadow: 0 0 0 2px rgba(255, 209, 102, 0.32);
}

.timeline-marker.selected .timeline-marker-label,
.timeline-marker:focus-visible .timeline-marker-label {
  color: #fff2c2;
}

.timeline-marker-detail {
  display: grid;
  grid-template-columns: minmax(120px, 1fr) minmax(90px, auto) minmax(170px, auto);
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background: #10141a;
  color: #dfe5ec;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  padding: 8px 10px;
}

.timeline-marker-detail strong {
  color: #fff2c2;
}

.inspection-lists {
  display: grid;
  gap: 14px;
}

.marker-metadata-list {
  display: grid;
  gap: 8px;
}

.marker-metadata {
  display: grid;
  grid-template-columns: minmax(120px, 1.1fr) minmax(100px, 0.7fr) minmax(180px, 1.2fr);
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  background: #151a20;
  padding: 8px 10px;
  color: #dfe5ec;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.marker-metadata strong {
  color: #eafff0;
}

.inspection-lists h4 {
  margin-bottom: 6px;
  color: #c9d1da;
  font-size: 13px;
}

.inspection-lists ol {
  display: grid;
  gap: 5px;
  margin: 0;
  padding-left: 18px;
  color: #dfe5ec;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}
</style>
