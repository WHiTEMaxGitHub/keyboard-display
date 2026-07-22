export type VideoExporterSource = "app-managed" | "user-selected" | "path";

export type VideoExporterCandidate = {
  source: VideoExporterSource;
  path: string;
  available: boolean;
  version?: string;
};

export type VideoExporterConfig = {
  userSelectedPath: string | null;
};

export type VideoExporterStatus = {
  resolved: VideoExporterCandidate | null;
  appManaged: VideoExporterCandidate;
  userSelected: VideoExporterCandidate | null;
  path: VideoExporterCandidate;
};

export function createDefaultVideoExporterConfig(): VideoExporterConfig {
  return {
    userSelectedPath: null,
  };
}

export function normalizeVideoExporterConfig(
  config: Partial<VideoExporterConfig> | null | undefined,
): VideoExporterConfig {
  return {
    userSelectedPath: cleanPath(config?.userSelectedPath),
  };
}

/// 按隔离优先级选择导出器：应用托管版本优先，其次用户手动选择，最后 PATH。
export function resolveVideoExporter(status: VideoExporterStatus): VideoExporterCandidate | null {
  return [status.appManaged, status.userSelected, status.path].find(
    (candidate): candidate is VideoExporterCandidate => Boolean(candidate?.available),
  ) ?? null;
}

export function describeVideoExporter(candidate: VideoExporterCandidate | null): string {
  if (!candidate) {
    return "Not installed";
  }

  switch (candidate.source) {
    case "app-managed":
      return "App-managed exporter";
    case "user-selected":
      return "User-selected ffmpeg";
    case "path":
      return "System ffmpeg";
  }
}

function cleanPath(path: string | null | undefined): string | null {
  const trimmedPath = path?.trim() ?? "";
  return trimmedPath.length > 0 ? trimmedPath : null;
}
