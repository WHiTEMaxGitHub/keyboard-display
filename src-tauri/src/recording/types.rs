use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum RecordingEvent {
    KeyDown {
        frame: u64,
        #[serde(rename = "down")]
        key_id: String,
    },
    KeyUp {
        frame: u64,
        #[serde(rename = "up")]
        key_id: String,
    },
    Marker {
        frame: u64,
        #[serde(rename = "marker")]
        name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RecordingSnapshot {
    pub version: u8,
    pub fps: u16,
    pub timebase: &'static str,
    pub events: Vec<RecordingEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RecordingFrame {
    pub frame: u64,
    pub keys: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RecordingInspection {
    pub version: u8,
    pub fps: u16,
    pub key_ids: Vec<String>,
    pub events: Vec<RecordingEvent>,
    pub frames: Vec<RecordingFrame>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingFileSummary {
    pub file_name: String,
    pub size_bytes: u64,
    pub start_unix_ms: Option<u64>,
    pub end_unix_ms: Option<u64>,
    pub fps: u16,
    pub frame_count: u64,
    pub marker_count: usize,
    pub markers: Vec<RecordingMarkerSummary>,
    pub metadata: RecordingMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingMarkerSummary {
    pub frame: u64,
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingMetadata {
    pub display_name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub marker_notes: Vec<RecordingMarkerNote>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingMarkerNote {
    pub frame: u64,
    pub name: String,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingTreeNode {
    pub name: String,
    pub path: String,
    pub exists: bool,
    #[serde(rename = "type")]
    pub node_type: String,
    pub children: Vec<RecordingTreeNode>,
    pub summary: Option<RecordingFileSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StopRecordingResult {
    pub path: String,
}
