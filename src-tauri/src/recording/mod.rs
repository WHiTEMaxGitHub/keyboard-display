#![allow(dead_code)]

mod binary;

use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

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
    pub metadata: RecordingMetadata,
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
    #[serde(rename = "type")]
    pub node_type: String,
    pub children: Vec<RecordingTreeNode>,
    pub summary: Option<RecordingFileSummary>,
}

pub struct RecordingSession {
    fps: u16,
    start_time_ms: u64,
    events: Vec<RecordingEvent>,
    active_keys: HashSet<String>,
}

pub struct RecordingManager {
    session: Mutex<Option<ActiveRecordingSession>>,
}

struct ActiveRecordingSession {
    start_unix_ms: u64,
    session: RecordingSession,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StopRecordingResult {
    pub path: String,
}

impl RecordingSession {
    pub fn new(fps: u16, start_time_ms: u64) -> Self {
        Self {
            fps,
            start_time_ms,
            events: Vec::new(),
            active_keys: HashSet::new(),
        }
    }

    pub fn record_input(&mut self, now_ms: u64, key_id: impl Into<String>, pressed: bool) {
        let frame = self.frame_at(now_ms);
        let key_id = key_id.into();

        if pressed {
            if self.active_keys.contains(&key_id) {
                return;
            }

            self.active_keys.insert(key_id.clone());
            self.events.push(RecordingEvent::KeyDown { frame, key_id });
        } else {
            if !self.active_keys.remove(&key_id) {
                return;
            }

            self.events.push(RecordingEvent::KeyUp { frame, key_id });
        }
    }

    pub fn add_marker(&mut self, now_ms: u64, name: impl Into<String>) {
        self.events.push(RecordingEvent::Marker {
            frame: self.frame_at(now_ms),
            name: name.into(),
        });
    }

    pub fn suppress_recent_keys(&mut self, key_ids: &[String]) {
        let Some(suppress_since) = key_ids
            .iter()
            .filter_map(|key_id| self.last_key_down_frame(key_id))
            .min()
        else {
            return;
        };

        self.events.retain(|event| match event {
            RecordingEvent::KeyDown { frame, key_id } | RecordingEvent::KeyUp { frame, key_id } => {
                !key_ids.iter().any(|target| target == key_id) || *frame < suppress_since
            }
            RecordingEvent::Marker { .. } => true,
        });

        for key_id in key_ids {
            self.active_keys.remove(key_id);
        }
    }

    fn last_key_down_frame(&self, target_key_id: &str) -> Option<u64> {
        self.events.iter().rev().find_map(|event| match event {
            RecordingEvent::KeyDown { frame, key_id } if key_id == target_key_id => Some(*frame),
            _ => None,
        })
    }

    pub fn snapshot(&self) -> RecordingSnapshot {
        RecordingSnapshot {
            version: 1,
            fps: self.fps,
            timebase: "monotonic",
            events: self.events.clone(),
        }
    }

    fn frame_at(&self, now_ms: u64) -> u64 {
        ms_to_frame(now_ms.saturating_sub(self.start_time_ms), self.fps)
    }
}

impl RecordingManager {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(None),
        }
    }

    pub fn start(
        &self,
        fps: u16,
        start_unix_ms: u64,
        start_monotonic_ms: u64,
    ) -> Result<(), String> {
        let mut session = self.session.lock().map_err(|error| error.to_string())?;

        *session = Some(ActiveRecordingSession {
            start_unix_ms,
            session: RecordingSession::new(fps, start_monotonic_ms),
        });

        Ok(())
    }

    pub fn record_input(
        &self,
        now_ms: u64,
        key_id: impl Into<String>,
        pressed: bool,
    ) -> Result<(), String> {
        let mut session = self.session.lock().map_err(|error| error.to_string())?;

        if let Some(active_session) = session.as_mut() {
            active_session.session.record_input(now_ms, key_id, pressed);
        }

        Ok(())
    }

    pub fn add_marker(&self, now_ms: u64, name: impl Into<String>) -> Result<(), String> {
        let mut session = self.session.lock().map_err(|error| error.to_string())?;

        if let Some(active_session) = session.as_mut() {
            active_session.session.add_marker(now_ms, name);
        }

        Ok(())
    }

    pub fn suppress_recent_keys(&self, key_ids: Vec<String>) -> Result<(), String> {
        let mut session = self.session.lock().map_err(|error| error.to_string())?;

        if let Some(active_session) = session.as_mut() {
            active_session.session.suppress_recent_keys(&key_ids);
        }

        Ok(())
    }

    pub fn stop(&self, output_dir: PathBuf, now_ms: u64) -> Result<StopRecordingResult, String> {
        self.stop_with_filename_template(output_dir, now_ms, "${start}-${end}", "", 0)
    }

    pub fn stop_with_filename_template(
        &self,
        output_dir: PathBuf,
        now_ms: u64,
        filename_template: &str,
        profile_name: &str,
        fps: u16,
    ) -> Result<StopRecordingResult, String> {
        let mut session = self.session.lock().map_err(|error| error.to_string())?;
        let Some(active_session) = session.take() else {
            return Err("recording has not started".to_string());
        };

        std::fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;

        let file_name = format_recording_file_name(
            filename_template,
            active_session.start_unix_ms,
            now_ms,
            profile_name,
            if fps == 0 {
                active_session.session.fps
            } else {
                fps
            },
        );
        let path = output_dir.join(file_name);
        let contents = binary::encode_kbdrec(&active_session.session.snapshot())?;
        std::fs::write(&path, contents).map_err(|error| error.to_string())?;

        Ok(StopRecordingResult {
            path: path.to_string_lossy().to_string(),
        })
    }
}

pub fn format_recording_file_name(
    filename_template: &str,
    start_unix_ms: u64,
    end_unix_ms: u64,
    profile_name: &str,
    fps: u16,
) -> String {
    let template = if filename_template.trim().is_empty() {
        "${start}-${end}"
    } else {
        filename_template.trim()
    };
    let expanded = template
        .replace("${start}", &start_unix_ms.to_string())
        .replace("${end}", &end_unix_ms.to_string())
        .replace("${profileName}", profile_name)
        .replace("${fps}", &fps.to_string());
    let file_name = sanitize_file_name(&expanded);

    if file_name.to_ascii_lowercase().ends_with(".kbdrec") {
        file_name
    } else {
        format!("{file_name}.kbdrec")
    }
}

fn sanitize_file_name(file_name: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_dash = false;

    for character in file_name.chars() {
        let next = if character == '/' || character == '\\' || character.is_control() {
            '-'
        } else {
            character
        };

        if next == '-' {
            if previous_dash {
                continue;
            }
            previous_dash = true;
        } else {
            previous_dash = false;
        }

        sanitized.push(next);
    }

    let sanitized = sanitized.trim().to_string();
    if sanitized.is_empty() {
        "recording".to_string()
    } else {
        sanitized
    }
}

pub fn unix_now_ms() -> Result<u64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;

    Ok(duration.as_millis() as u64)
}

pub fn monotonic_now_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();

    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

pub fn sample_frames(
    _fps: u16,
    duration_frame: u64,
    events: &[RecordingEvent],
) -> Vec<RecordingFrame> {
    let mut sorted_events = events.to_vec();
    sorted_events.sort_by_key(event_frame);

    let mut active_keys = HashSet::<String>::new();
    let mut frames = Vec::new();
    let mut event_index = 0;
    let mut frame = 0;

    while frame <= duration_frame {
        while event_index < sorted_events.len() && event_frame(&sorted_events[event_index]) <= frame
        {
            match &sorted_events[event_index] {
                RecordingEvent::KeyDown { key_id, .. } => {
                    active_keys.insert(key_id.clone());
                }
                RecordingEvent::KeyUp { key_id, .. } => {
                    active_keys.remove(key_id);
                }
                RecordingEvent::Marker { .. } => {}
            }

            event_index += 1;
        }

        let mut keys = active_keys.iter().cloned().collect::<Vec<_>>();
        keys.sort();
        frames.push(RecordingFrame { frame, keys });
        frame += 1;
    }

    frames
}

fn ms_to_frame(ms: u64, fps: u16) -> u64 {
    (ms * u64::from(fps) + 500) / 1000
}

pub fn inspect_kbdrec(bytes: &[u8]) -> Result<RecordingInspection, String> {
    let decoded = binary::decode_kbdrec(bytes)?;
    let events = decoded
        .markers
        .iter()
        .map(|marker| RecordingEvent::Marker {
            frame: marker.frame,
            name: marker.name.clone(),
        })
        .collect::<Vec<_>>();

    Ok(RecordingInspection {
        version: 1,
        fps: decoded.fps,
        key_ids: decoded.key_ids,
        frames: decoded.frames,
        events,
    })
}

pub fn list_recording_files(root: PathBuf) -> Result<RecordingTreeNode, String> {
    if !root.exists() {
        return Ok(directory_node(&root, Vec::new()));
    }

    if !root.is_dir() {
        return Err(format!(
            "recording path is not a directory: {}",
            root.display()
        ));
    }

    scan_recording_directory(&root)
}

pub fn create_recording_folder(
    root: PathBuf,
    folder_name: String,
) -> Result<RecordingTreeNode, String> {
    let folder_name = normalize_folder_name(&folder_name)?;
    let folder_path = root.join(folder_name);

    std::fs::create_dir_all(&folder_path).map_err(|error| error.to_string())?;
    list_recording_files(root)
}

pub fn read_recording_metadata(path: PathBuf) -> Result<RecordingMetadata, String> {
    let sidecar_path = recording_metadata_path(&path);
    if !sidecar_path.exists() {
        return Ok(RecordingMetadata::default());
    }

    let contents = std::fs::read_to_string(sidecar_path).map_err(|error| error.to_string())?;
    parse_recording_metadata(&contents)
}

pub fn save_recording_metadata(
    path: PathBuf,
    metadata: RecordingMetadata,
) -> Result<RecordingMetadata, String> {
    if !is_kbdrec_file(&path) {
        return Err(format!(
            "recording metadata target is not .kbdrec: {}",
            path.display()
        ));
    }

    let metadata = normalize_recording_metadata(metadata);
    let sidecar_path = recording_metadata_path(&path);
    if let Some(parent) = sidecar_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let contents = serde_json::to_string_pretty(&metadata).map_err(|error| error.to_string())?;
    std::fs::write(sidecar_path, format!("{contents}\n")).map_err(|error| error.to_string())?;

    Ok(metadata)
}

fn scan_recording_directory(path: &PathBuf) -> Result<RecordingTreeNode, String> {
    let mut entries = std::fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    entries.sort_by_key(|entry| {
        let is_file = entry
            .file_type()
            .map(|kind| kind.is_file())
            .unwrap_or(false);
        (is_file, entry.file_name())
    });

    let mut children = Vec::new();

    for entry in entries {
        let entry_path = entry.path();
        let file_type = entry.file_type().map_err(|error| error.to_string())?;

        if file_type.is_dir() {
            children.push(scan_recording_directory(&entry_path)?);
        } else if is_kbdrec_file(&entry_path) {
            children.push(file_node(&entry_path)?);
        }
    }

    Ok(directory_node(path, children))
}

fn normalize_folder_name(folder_name: &str) -> Result<String, String> {
    let folder_name = folder_name.trim();

    if folder_name.is_empty() {
        return Err("folder name is required".to_string());
    }

    if folder_name.contains('/') || folder_name.contains('\\') {
        return Err("folder name must not contain path separators".to_string());
    }

    if folder_name == "." || folder_name == ".." {
        return Err("folder name must not be . or ..".to_string());
    }

    Ok(folder_name.to_string())
}

fn directory_node(path: &PathBuf, children: Vec<RecordingTreeNode>) -> RecordingTreeNode {
    RecordingTreeNode {
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string()),
        path: path.to_string_lossy().to_string(),
        node_type: "directory".to_string(),
        children,
        summary: None,
    }
}

fn file_node(path: &PathBuf) -> Result<RecordingTreeNode, String> {
    let summary = summarize_recording_file(path)?;

    Ok(RecordingTreeNode {
        name: summary.file_name.clone(),
        path: path.to_string_lossy().to_string(),
        node_type: "file".to_string(),
        children: Vec::new(),
        summary: Some(summary),
    })
}

fn summarize_recording_file(path: &PathBuf) -> Result<RecordingFileSummary, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let decoded = binary::decode_kbdrec(&bytes)?;
    let metadata = std::fs::metadata(path).map_err(|error| error.to_string())?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "recording file has no file name".to_string())?
        .to_string_lossy()
        .to_string();
    let (start_unix_ms, end_unix_ms) = parse_recording_file_times(&file_name);

    Ok(RecordingFileSummary {
        file_name,
        size_bytes: metadata.len(),
        start_unix_ms,
        end_unix_ms,
        fps: decoded.fps,
        frame_count: decoded.frame_count,
        marker_count: decoded.markers.len(),
        metadata: read_recording_metadata(path.clone())?,
    })
}

fn parse_recording_metadata(contents: &str) -> Result<RecordingMetadata, String> {
    let metadata = serde_json::from_str(contents).map_err(|error| error.to_string())?;
    Ok(normalize_recording_metadata(metadata))
}

fn normalize_recording_metadata(mut metadata: RecordingMetadata) -> RecordingMetadata {
    metadata.display_name = metadata.display_name.trim().to_string();
    metadata.description = metadata.description.trim().to_string();
    metadata.tags = metadata
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();
    metadata.marker_notes = metadata
        .marker_notes
        .into_iter()
        .map(|mut marker_note| {
            marker_note.name = marker_note.name.trim().to_string();
            marker_note.note = marker_note.note.trim().to_string();
            marker_note
        })
        .filter(|marker_note| !marker_note.name.is_empty() || !marker_note.note.is_empty())
        .collect();

    metadata
}

fn recording_metadata_path(path: &std::path::Path) -> PathBuf {
    let mut sidecar_path = path.as_os_str().to_os_string();
    sidecar_path.push(".json");
    PathBuf::from(sidecar_path)
}

fn is_kbdrec_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("kbdrec"))
        .unwrap_or(false)
}

fn parse_recording_file_times(file_name: &str) -> (Option<u64>, Option<u64>) {
    let Some(stem) = file_name.strip_suffix(".kbdrec") else {
        return (None, None);
    };
    let Some((start, end)) = stem.split_once('-') else {
        return (None, None);
    };

    (start.parse().ok(), end.parse().ok())
}

fn event_frame(event: &RecordingEvent) -> u64 {
    match event {
        RecordingEvent::KeyDown { frame, .. }
        | RecordingEvent::KeyUp { frame, .. }
        | RecordingEvent::Marker { frame, .. } => *frame,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        binary::{decode_kbdrec, encode_kbdrec},
        create_recording_folder, format_recording_file_name, inspect_kbdrec, list_recording_files,
        parse_recording_file_times, read_recording_metadata, sample_frames,
        save_recording_metadata, RecordingEvent, RecordingManager, RecordingMarkerNote,
        RecordingMetadata, RecordingSession, RecordingSnapshot,
    };

    #[test]
    fn stores_key_events_with_monotonic_relative_timestamps() {
        let mut session = RecordingSession::new(60, 1000);

        session.record_input(1120, "w", true);
        session.record_input(1190, "w", false);

        assert_eq!(
            session.snapshot().events,
            vec![
                RecordingEvent::KeyDown {
                    frame: 7,
                    key_id: "w".to_string(),
                },
                RecordingEvent::KeyUp {
                    frame: 11,
                    key_id: "w".to_string(),
                },
            ],
        );
    }

    #[test]
    fn ignores_duplicate_state_events() {
        let mut session = RecordingSession::new(60, 1000);

        session.record_input(1100, "tab", true);
        session.record_input(1110, "tab", true);
        session.record_input(1120, "tab", false);
        session.record_input(1130, "tab", false);

        assert_eq!(
            session.snapshot().events,
            vec![
                RecordingEvent::KeyDown {
                    frame: 6,
                    key_id: "tab".to_string(),
                },
                RecordingEvent::KeyUp {
                    frame: 7,
                    key_id: "tab".to_string(),
                },
            ],
        );
    }

    #[test]
    fn suppresses_only_recent_control_chord_events() {
        let mut session = RecordingSession::new(60, 1000);

        session.record_input(1010, "ctrl-left", true);
        session.record_input(1020, "ctrl-left", false);
        session.record_input(1100, "ctrl-left", true);
        session.record_input(1110, "shift-left", true);
        session.record_input(1120, "r", true);
        session.suppress_recent_keys(&[
            "ctrl-left".to_string(),
            "shift-left".to_string(),
            "r".to_string(),
        ]);

        assert_eq!(
            session.snapshot().events,
            vec![
                RecordingEvent::KeyDown {
                    frame: 1,
                    key_id: "ctrl-left".to_string(),
                },
                RecordingEvent::KeyUp {
                    frame: 1,
                    key_id: "ctrl-left".to_string(),
                },
            ],
        );
    }

    #[test]
    fn keeps_single_control_key_when_no_hotkey_is_suppressed() {
        let mut session = RecordingSession::new(60, 1000);

        session.record_input(1010, "ctrl-left", true);
        session.record_input(1020, "ctrl-left", false);

        assert_eq!(session.snapshot().events.len(), 2);
    }

    #[test]
    fn stores_sync_markers_in_event_stream() {
        let mut session = RecordingSession::new(60, 2000);

        session.add_marker(2500, "sync");

        assert_eq!(
            session.snapshot().events,
            vec![RecordingEvent::Marker {
                frame: 30,
                name: "sync".to_string(),
            }],
        );

        let serialized = serde_json::to_value(session.snapshot()).unwrap();
        assert_eq!(serialized["events"][0]["marker"], "sync");
        assert!(serialized["events"][0].get("type").is_none());
    }

    #[test]
    fn manager_adds_markers_to_active_session() {
        let manager = RecordingManager::new();

        manager.start(60, 1000, 1000).unwrap();
        manager.add_marker(1250, "hotkey-start").unwrap();

        let session = manager.session.lock().unwrap();
        let active_session = session.as_ref().unwrap();
        assert_eq!(
            active_session.session.snapshot().events,
            vec![RecordingEvent::Marker {
                frame: 15,
                name: "hotkey-start".to_string(),
            }],
        );
    }

    #[test]
    fn serializes_versioned_recording_envelope() {
        let mut session = RecordingSession::new(60, 3000);

        session.record_input(3016, "space", true);

        let serialized = serde_json::to_value(session.snapshot()).unwrap();

        assert_eq!(serialized["version"], 1);
        assert_eq!(serialized["fps"], 60);
        assert_eq!(serialized["timebase"], "monotonic");
        assert_eq!(serialized["events"][0]["frame"], 1);
        assert_eq!(serialized["events"][0]["down"], "space");
        assert!(serialized["events"][0].get("type").is_none());
        assert!(serialized["events"][0].get("pressed").is_none());
    }

    #[test]
    fn samples_held_keys_by_frame() {
        let frames = sample_frames(
            10,
            300,
            &[
                RecordingEvent::KeyDown {
                    frame: 1,
                    key_id: "w".to_string(),
                },
                RecordingEvent::KeyUp {
                    frame: 3,
                    key_id: "w".to_string(),
                },
            ],
        );

        assert_eq!(frames[0].frame, 0);
        assert!(frames[0].keys.is_empty());
        assert_eq!(frames[1].keys, vec!["w".to_string()]);
        assert_eq!(frames[2].keys, vec!["w".to_string()]);
        assert!(frames[3].keys.is_empty());
    }

    #[test]
    fn samples_frames_without_integer_millisecond_drift() {
        let frames = sample_frames(60, 6, &[]);

        assert_eq!(frames[0].frame, 0);
        assert_eq!(frames[3].frame, 3);
        assert_eq!(frames[6].frame, 6);
    }

    #[test]
    fn manager_writes_binary_kbdrec_file() {
        let output_dir = std::env::temp_dir().join(format!(
            "keyboard-display-recording-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&output_dir);
        let manager = RecordingManager::new();

        manager.start(60, 1000, 1000).unwrap();
        manager.record_input(1016, "w", true).unwrap();
        let result = manager.stop(output_dir.clone(), 1200).unwrap();
        let contents = std::fs::read(&result.path).unwrap();
        let decoded = decode_kbdrec(&contents).unwrap();

        assert_eq!(&contents[0..4], b"KBDR");
        assert_eq!(decoded.frames.len(), 2);
        assert!(decoded.frames[0].keys.is_empty());
        assert_eq!(decoded.frames[1].keys, vec!["w"]);

        let _ = std::fs::remove_dir_all(output_dir);
    }

    #[test]
    fn formats_recording_file_name_from_template() {
        assert_eq!(
            format_recording_file_name("${profileName}-${fps}-${start}", 1000, 2000, "Aim", 120),
            "Aim-120-1000.kbdrec",
        );
        assert_eq!(
            format_recording_file_name("../${profileName}\n${end}", 1000, 2000, "CS/POV", 60),
            "..-CS-POV-2000.kbdrec",
        );
    }

    #[test]
    fn manager_uses_filename_template_when_writing_recording() {
        let output_dir = std::env::temp_dir().join(format!(
            "keyboard-display-recording-template-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&output_dir);
        let manager = RecordingManager::new();

        manager.start(60, 1000, 1000).unwrap();
        manager.record_input(1016, "w", true).unwrap();
        let result = manager
            .stop_with_filename_template(
                output_dir.clone(),
                1200,
                "${profileName}-${fps}-${start}",
                "Aim",
                60,
            )
            .unwrap();

        assert!(result.path.ends_with("Aim-60-1000.kbdrec"));

        let _ = std::fs::remove_dir_all(output_dir);
    }

    #[test]
    fn binary_recording_roundtrips_frame_state_stream() {
        let snapshot = {
            let mut session = RecordingSession::new(60, 1000);
            session.record_input(1016, "w", true);
            session.record_input(1016, "shift-left", true);
            session.record_input(1083, "shift-left", false);
            session.add_marker(1200, "sync");
            session.snapshot()
        };

        let encoded = encode_kbdrec(&snapshot).unwrap();
        let decoded = decode_kbdrec(&encoded).unwrap();

        assert_eq!(decoded.fps, 60);
        assert_eq!(decoded.key_ids, vec!["w", "shift-left"]);
        assert_eq!(decoded.frames.len(), 13);
        assert!(decoded.frames[0].keys.is_empty());
        assert_eq!(decoded.frames[1].keys, vec!["w", "shift-left"]);
        assert_eq!(decoded.frames[6].keys, vec!["w"]);
        assert_eq!(decoded.markers[0].frame, 12);
        assert_eq!(decoded.markers[0].name, "sync");
        assert!(decoded.runs.len() < decoded.frames.len());
    }

    #[test]
    fn binary_recording_deduplicates_markers_by_frame_and_name() {
        let snapshot = RecordingSnapshot {
            version: 1,
            fps: 120,
            timebase: "monotonic",
            events: vec![
                RecordingEvent::Marker {
                    frame: 1361,
                    name: "sync".to_string(),
                },
                RecordingEvent::Marker {
                    frame: 1361,
                    name: "sync".to_string(),
                },
            ],
        };

        let decoded = decode_kbdrec(&encode_kbdrec(&snapshot).unwrap()).unwrap();

        assert_eq!(decoded.markers.len(), 1);
        assert_eq!(decoded.markers[0].frame, 1361);
    }

    #[test]
    fn binary_recording_excludes_virtual_void_keys() {
        let snapshot = RecordingSnapshot {
            version: 1,
            fps: 60,
            timebase: "monotonic",
            events: vec![RecordingEvent::KeyDown {
                frame: 0,
                key_id: "void".to_string(),
            }],
        };

        let decoded = decode_kbdrec(&encode_kbdrec(&snapshot).unwrap()).unwrap();

        assert!(decoded.key_ids.is_empty());
        assert!(decoded.frames.is_empty());
    }

    #[test]
    fn inspects_binary_kbdrec_as_human_readable_events_and_frames() {
        let snapshot = {
            let mut session = RecordingSession::new(10, 1000);
            session.record_input(1100, "w", true);
            session.record_input(1300, "w", false);
            session.add_marker(1200, "sync");
            session.snapshot()
        };

        let inspection = inspect_kbdrec(&encode_kbdrec(&snapshot).unwrap()).unwrap();

        assert_eq!(inspection.version, 1);
        assert_eq!(inspection.fps, 10);
        assert_eq!(inspection.key_ids, vec!["w"]);
        assert_eq!(
            inspection.events,
            vec![RecordingEvent::Marker {
                frame: 2,
                name: "sync".to_string(),
            },],
        );
        assert!(inspection.frames[0].keys.is_empty());
        assert_eq!(inspection.frames[1].keys, vec!["w".to_string()]);
        assert_eq!(inspection.frames[2].keys, vec!["w".to_string()]);
        assert!(inspection.frames[3].keys.is_empty());
    }

    #[test]
    fn parses_recording_file_times_from_fixed_name() {
        assert_eq!(
            parse_recording_file_times("1784311866993-1784311907364.kbdrec"),
            (Some(1784311866993), Some(1784311907364)),
        );
        assert_eq!(parse_recording_file_times("notes.kbdrec"), (None, None));
    }

    #[test]
    fn lists_recording_files_recursively() {
        let root = std::env::temp_dir().join(format!(
            "keyboard-display-recording-list-test-{}",
            std::process::id()
        ));
        let nested = root.join("nested");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&nested).unwrap();

        let snapshot = {
            let mut session = RecordingSession::new(120, 1000);
            session.record_input(1008, "w", true);
            session.add_marker(1016, "sync");
            session.snapshot()
        };
        std::fs::write(
            nested.join("1000-2000.kbdrec"),
            encode_kbdrec(&snapshot).unwrap(),
        )
        .unwrap();
        std::fs::write(root.join("ignore.txt"), "not a recording").unwrap();

        let tree = list_recording_files(root.clone()).unwrap();
        let nested_node = tree
            .children
            .iter()
            .find(|node| node.name == "nested")
            .unwrap();
        let file_node = nested_node.children.first().unwrap();
        let summary = file_node.summary.as_ref().unwrap();

        assert_eq!(file_node.node_type, "file");
        assert_eq!(summary.file_name, "1000-2000.kbdrec");
        assert_eq!(summary.start_unix_ms, Some(1000));
        assert_eq!(summary.end_unix_ms, Some(2000));
        assert_eq!(summary.fps, 120);
        assert_eq!(summary.marker_count, 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn lists_recording_sidecar_metadata_with_summary() {
        let root = std::env::temp_dir().join(format!(
            "keyboard-display-recording-metadata-list-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let snapshot = {
            let mut session = RecordingSession::new(120, 1000);
            session.record_input(1008, "w", true);
            session.snapshot()
        };
        let recording_path = root.join("1000-2000.kbdrec");
        std::fs::write(&recording_path, encode_kbdrec(&snapshot).unwrap()).unwrap();
        save_recording_metadata(
            recording_path.clone(),
            RecordingMetadata {
                display_name: "Aim warmup".to_string(),
                description: "first run".to_string(),
                tags: vec!["fps".to_string(), " aim ".to_string()],
                marker_notes: vec![RecordingMarkerNote {
                    frame: 12,
                    name: "sync".to_string(),
                    note: "first visible shot".to_string(),
                }],
            },
        )
        .unwrap();

        let tree = list_recording_files(root.clone()).unwrap();
        let summary = tree.children.first().unwrap().summary.as_ref().unwrap();

        assert_eq!(summary.metadata.display_name, "Aim warmup");
        assert_eq!(summary.metadata.description, "first run");
        assert_eq!(summary.metadata.tags, vec!["fps", "aim"]);
        assert_eq!(summary.metadata.marker_notes[0].frame, 12);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn creates_recording_folder_and_returns_updated_tree() {
        let root = std::env::temp_dir().join(format!(
            "keyboard-display-recording-folder-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);

        let tree = create_recording_folder(root.clone(), "Match 01".to_string()).unwrap();

        assert!(root.join("Match 01").is_dir());
        assert!(tree
            .children
            .iter()
            .any(|node| { node.name == "Match 01" && node.node_type == "directory" }));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_recording_folder_names_with_path_separators() {
        let root = std::env::temp_dir().join(format!(
            "keyboard-display-recording-folder-invalid-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);

        let error = create_recording_folder(root.clone(), "../escape".to_string()).unwrap_err();

        assert!(error.contains("folder name must not contain path separators"));
        assert!(!root.join("escape").exists());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn saves_and_reads_recording_sidecar_metadata() {
        let root = std::env::temp_dir().join(format!(
            "keyboard-display-recording-metadata-save-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let recording_path = root.join("1000-2000.kbdrec");

        save_recording_metadata(
            recording_path.clone(),
            RecordingMetadata {
                display_name: "  Session 1  ".to_string(),
                description: "  marker check  ".to_string(),
                tags: vec![" sync ".to_string(), "".to_string(), "ranked".to_string()],
                marker_notes: vec![RecordingMarkerNote {
                    frame: 24,
                    name: " sync ".to_string(),
                    note: " line up with reload ".to_string(),
                }],
            },
        )
        .unwrap();

        let metadata = read_recording_metadata(recording_path).unwrap();

        assert_eq!(metadata.display_name, "Session 1");
        assert_eq!(metadata.description, "marker check");
        assert_eq!(metadata.tags, vec!["sync", "ranked"]);
        assert_eq!(metadata.marker_notes[0].name, "sync");
        assert_eq!(metadata.marker_notes[0].note, "line up with reload");

        let _ = std::fs::remove_dir_all(root);
    }
}
