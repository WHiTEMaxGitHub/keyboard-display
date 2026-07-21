export type RecordingInspectionEvent =
  | {
      frame: number;
      down: string;
    }
  | {
      frame: number;
      up: string;
    }
  | {
      frame: number;
      marker: string;
    };

export type RecordingInspectionFrame = {
  frame: number;
  keys: string[];
};

export type RecordingInspection = {
  version: number;
  fps: number;
  keyIds: string[];
  events: RecordingInspectionEvent[];
  frames: RecordingInspectionFrame[];
};

export type RecordingMarkerNote = {
  frame: number;
  name: string;
  note: string;
};

export type RecordingMetadata = {
  displayName: string;
  description: string;
  tags: string[];
  markerNotes: RecordingMarkerNote[];
};

export type RecordingFileSummary = {
  fileName: string;
  sizeBytes: number;
  startUnixMs: number | null;
  endUnixMs: number | null;
  fps: number;
  frameCount: number;
  markerCount: number;
  markers: Array<{ frame: number; name: string }>;
  metadata: RecordingMetadata;
};

export type RecordingTreeNode = {
  name: string;
  path: string;
  exists: boolean;
  type: "directory" | "file";
  children: RecordingTreeNode[];
  summary: RecordingFileSummary | null;
};
