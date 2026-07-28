export const DEFAULT_EXPORT_FILENAME_TEMPLATE = "${profileSlug}-${recordingName}-overlay";

export type ExportFilenameTemplateInput = {
  template: string;
  recordingPath: string;
  profileName: string;
  fps: number;
};

export function sanitizeExportFilenameTemplate(template: string) {
  return template.trim() || DEFAULT_EXPORT_FILENAME_TEMPLATE;
}

export function formatExportFileName(input: ExportFilenameTemplateInput) {
  const template = sanitizeExportFilenameTemplate(input.template);
  const recordingName = recordingNameFromPath(input.recordingPath);
  const expanded = template.replace(/\$\{(recordingName|profileName|profileSlug|fps)\}/g, (_, key) => {
    const values = {
      recordingName,
      profileName: input.profileName,
      profileSlug: slugify(input.profileName),
      fps: String(input.fps),
    };

    return values[key as keyof typeof values];
  });

  return `${sanitizeFileName(expanded)}.webm`;
}

function recordingNameFromPath(path: string) {
  const fileName = path.split(/[\\/]/).pop() ?? path;
  return fileName.replace(/\.kbdrec$/i, "") || "recording";
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

function sanitizeFileName(fileName: string) {
  return fileName
    .replace(/[\\/]+/g, "-")
    .replace(/[\u0000-\u001F\u007F]+/g, "-")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/-+/g, "-")
    || "keyboard-overlay";
}
