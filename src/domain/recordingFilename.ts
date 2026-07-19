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
  const expanded = template.replace(/\$\{(start|end|profileName|fps)\}/g, (_, key) => {
    const values = {
      start: String(input.start),
      end: String(input.end),
      profileName: input.profileName,
      fps: String(input.fps),
    };

    return values[key as keyof typeof values];
  });

  return `${sanitizeFileName(expanded)}.kbdrec`;
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
