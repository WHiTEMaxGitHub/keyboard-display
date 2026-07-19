export const DEFAULT_RECORDING_FILENAME_TEMPLATE = "${start}-${end}";

export type RecordingFilenameTemplateInput = {
  template: string;
  start: number;
  end: number;
  profileName: string;
  fps: number;
};

export function sanitizeRecordingFilenameTemplate(template: string) {
  return template.trim() || DEFAULT_RECORDING_FILENAME_TEMPLATE;
}

export function formatRecordingFileName(input: RecordingFilenameTemplateInput) {
  const template = sanitizeRecordingFilenameTemplate(input.template);
  const expanded = template.replace(/\$\{(start|end|startDate|startTime|endTime|duration|profileName|profileSlug|fps)\}/g, (_, key) => {
    const values = {
      start: String(input.start),
      end: String(input.end),
      startDate: formatDate(input.start),
      startTime: formatTime(input.start),
      endTime: formatTime(input.end),
      duration: formatDuration(input.end - input.start),
      profileName: input.profileName,
      profileSlug: slugify(input.profileName),
      fps: String(input.fps),
    };

    return values[key as keyof typeof values];
  });

  return `${sanitizeFileName(expanded)}.kbdrec`;
}

function formatDate(unixMs: number) {
  const date = new Date(unixMs);
  return `${date.getFullYear()}-${pad2(date.getMonth() + 1)}-${pad2(date.getDate())}`;
}

function formatTime(unixMs: number) {
  const date = new Date(unixMs);
  return `${pad2(date.getHours())}-${pad2(date.getMinutes())}-${pad2(date.getSeconds())}`;
}

function formatDuration(durationMs: number) {
  const totalSeconds = Math.max(0, Math.floor(durationMs / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${pad2(hours)}-${pad2(minutes)}-${pad2(seconds)}`;
}

function slugify(value: string) {
  return (
    value
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "profile"
  );
}

function pad2(value: number) {
  return String(value).padStart(2, "0");
}

function sanitizeFileName(fileName: string) {
  return fileName
    .replace(/[\\/]+/g, "-")
    .replace(/[\u0000-\u001F\u007F]+/g, "-")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/-+/g, "-")
    || "recording";
}
