<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  buildRecordingTimelineMarkers,
  type RecordingTimelineMarker,
} from "../domain/recordingTimeline";
import type { RecordingInspection, RecordingInspectionEvent } from "../types/recording";
import BaseFieldRow from "./BaseFieldRow.vue";

const props = defineProps<{
  inspection: RecordingInspection;
}>();

const { t } = useI18n();
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
    return t("recordingInspection.eventDown", { frame: event.frame, key: event.down });
  }

  if ("up" in event) {
    return t("recordingInspection.eventUp", { frame: event.frame, key: event.up });
  }

  return t("recordingInspection.eventMarker", { frame: event.frame, marker: event.marker });
}

function markerEvents(events: RecordingInspectionEvent[]) {
  return events.filter((event): event is Extract<RecordingInspectionEvent, { marker: string }> =>
    "marker" in event
  );
}

function timelineMarkerPosition(percent: number) {
  return `calc(10px + ${percent / 100} * (100% - 20px))`;
}

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
  <div class="grid grid-cols-2 gap-x-[18px] gap-y-0">
    <BaseFieldRow :label="t('recordingInspection.version')">{{ inspection.version }}</BaseFieldRow>
    <BaseFieldRow :label="t('recordingInspection.fps')">{{ inspection.fps }}</BaseFieldRow>
    <BaseFieldRow :label="t('recordingInspection.keys')">{{ keyIds.length }}</BaseFieldRow>
    <BaseFieldRow :label="t('recordingInspection.events')">{{ events.length }}</BaseFieldRow>
    <BaseFieldRow :label="t('recordingInspection.frames')">{{ frames.length }}</BaseFieldRow>
    <BaseFieldRow :label="t('recordingInspection.markers')">
      {{ events.filter((event) => "marker" in event).length }}
    </BaseFieldRow>
  </div>

  <div class="grid gap-2.5 border border-border-control rounded-lg bg-surface-control p-3.5">
    <div class="flex items-center justify-between gap-3">
      <h3 class="m-0 text-base leading-[22px] tracking-normal">{{ t("recordingInspection.markerTimeline") }}</h3>
      <span class="text-text-muted">
        {{ t("recordingInspection.framesCount", { count: frames.length }) }}
      </span>
    </div>
    <div v-if="timelineMarkers.length" class="marker-timeline" :aria-label="t('recordingInspection.timeline')">
      <button
        v-for="(marker, index) in timelineMarkers"
        :key="`${marker.frame}-${marker.name}-${index}`"
        type="button"
        :class="['timeline-marker', { selected: selectedTimelineMarker === marker }]"
        :style="{ left: timelineMarkerPosition(marker.percent) }"
        :title="t('recordingInspection.markerTitle', { name: marker.name, frame: marker.frame, timecode: marker.timecode })"
        @click="selectedTimelineMarker = marker"
      >
        <span class="timeline-marker-dot" />
        <span class="timeline-marker-label">{{ marker.name }}</span>
      </button>
    </div>
    <p v-else class="text-text-muted">{{ t("recordingInspection.noMarkers") }}</p>
    <div v-if="selectedTimelineMarker" class="grid grid-cols-[minmax(120px,1fr)_minmax(90px,auto)_minmax(170px,auto)] gap-2.5 border border-border-control rounded-md bg-surface-control text-text-body font-mono text-xs px-2.5 py-2">
      <strong class="text-[#fff2c2] font-mono font-extrabold">{{ selectedTimelineMarker.name }}</strong>
      <span>{{ t("recordingInspection.frame", { frame: selectedTimelineMarker.frame }) }}</span>
      <span>{{ selectedTimelineMarker.timecode }}</span>
    </div>
  </div>

  <div class="grid gap-3.5">
    <div>
      <h4 class="mb-1.5 text-text-secondary text-[13px] tracking-normal">{{ t("recordingInspection.markers") }}</h4>
      <div class="grid gap-2">
        <div
          v-for="(event, index) in markerEvents(events)"
          :key="`${event.frame}-${event.marker}-${index}`"
          class="grid grid-cols-[minmax(120px,1.1fr)_minmax(100px,0.7fr)_minmax(180px,1.2fr)] gap-2.5 border border-border-control rounded-md bg-surface-control text-text-body font-mono text-xs px-2.5 py-2"
        >
          <strong class="text-accent-text font-mono font-extrabold">{{ t("recordingInspection.marker", { name: event.marker }) }}</strong>
          <span>{{ t("recordingInspection.frame", { frame: event.frame }) }}</span>
          <span>{{ t("recordingInspection.time", { timecode: formatFrameTimecode(event.frame, inspection.fps) }) }}</span>
        </div>
      </div>
    </div>
    <div>
      <h4 class="mb-1.5 text-text-secondary text-[13px] tracking-normal">{{ t("recordingInspection.keyTable") }}</h4>
      <p class="text-text-muted">{{ keyIds.join(", ") || t("recordingInspection.none") }}</p>
    </div>
    <div>
      <h4 class="mb-1.5 text-text-secondary text-[13px] tracking-normal">{{ t("recordingInspection.events") }}</h4>
      <ol class="grid gap-1.5 m-0 pl-[18px] text-text-body font-mono text-xs">
        <li
          v-for="(event, index) in events.slice(0, 8)"
          :key="index"
        >
          {{ formatInspectionEvent(event) }}
        </li>
      </ol>
    </div>
    <div>
      <h4 class="mb-1.5 text-text-secondary text-[13px] tracking-normal">{{ t("recordingInspection.frames") }}</h4>
      <ol class="grid gap-1.5 m-0 pl-[18px] text-text-body font-mono text-xs">
        <li
          v-for="frame in frames.slice(0, 8)"
          :key="frame.frame"
        >
          {{ frame.keys.length
            ? t("recordingInspection.frameKeys", { frame: frame.frame, keys: frame.keys.join(", ") })
            : t("recordingInspection.frameNoKeys", { frame: frame.frame }) }}
        </li>
      </ol>
    </div>
  </div>
</template>

<style scoped>
.marker-timeline {
  position: relative;
  height: 62px;
  border: 1px solid rgba(120, 140, 170, 0.08);
  border-radius: 7px;
  background:
    linear-gradient(90deg, rgba(120, 140, 170, 0.06) 1px, transparent 1px)
      0 0 / 25% 100%,
    rgba(9, 14, 22, 0.18);
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
</style>
