import { describe, expect, it } from "vitest";
import {
  createDefaultVideoExporterConfig,
  describeVideoExporter,
  normalizeVideoExporterConfig,
  resolveVideoExporter,
  type VideoExporterStatus,
} from "./videoExporter";

describe("video exporter", () => {
  it("normalizes the optional user-selected path", () => {
    expect(createDefaultVideoExporterConfig()).toEqual({
      userSelectedPath: null,
    });
    expect(normalizeVideoExporterConfig({ userSelectedPath: "  /opt/ffmpeg  " })).toEqual({
      userSelectedPath: "/opt/ffmpeg",
    });
    expect(normalizeVideoExporterConfig({ userSelectedPath: "   " })).toEqual({
      userSelectedPath: null,
    });
  });

  it("prefers app-managed exporter over user-selected and PATH candidates", () => {
    const status: VideoExporterStatus = {
      resolved: null,
      appManaged: {
        source: "app-managed",
        path: "/app/exporter/ffmpeg",
        available: true,
      },
      userSelected: {
        source: "user-selected",
        path: "/user/ffmpeg",
        available: true,
      },
      path: {
        source: "path",
        path: "ffmpeg",
        available: true,
      },
    };

    expect(resolveVideoExporter(status)).toEqual(status.appManaged);
  });

  it("falls back to user-selected and then PATH candidates", () => {
    const status: VideoExporterStatus = {
      resolved: null,
      appManaged: {
        source: "app-managed",
        path: "/app/exporter/ffmpeg",
        available: false,
      },
      userSelected: {
        source: "user-selected",
        path: "/user/ffmpeg",
        available: true,
      },
      path: {
        source: "path",
        path: "ffmpeg",
        available: true,
      },
    };

    expect(resolveVideoExporter(status)).toEqual(status.userSelected);

    status.userSelected = null;
    expect(resolveVideoExporter(status)).toEqual(status.path);
  });

  it("describes the candidate source for UI labels", () => {
    expect(describeVideoExporter(null)).toBe("Not installed");
    expect(describeVideoExporter({ source: "app-managed", path: "/app/ffmpeg", available: true }))
      .toBe("App-managed exporter");
    expect(describeVideoExporter({ source: "user-selected", path: "/bin/ffmpeg", available: true }))
      .toBe("User-selected ffmpeg");
    expect(describeVideoExporter({ source: "path", path: "ffmpeg", available: true }))
      .toBe("System ffmpeg");
  });
});
