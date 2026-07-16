#![allow(dead_code)]

mod binary;

use serde::Serialize;
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
        t: u64,
        #[serde(rename = "down")]
        key_id: String,
    },
    KeyUp {
        t: u64,
        #[serde(rename = "up")]
        key_id: String,
    },
    Marker {
        t: u64,
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
    pub t: u64,
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
        let t = self.relative_time(now_ms);
        let key_id = key_id.into();

        if pressed {
            if self.active_keys.contains(&key_id) {
                return;
            }

            self.active_keys.insert(key_id.clone());
            self.events.push(RecordingEvent::KeyDown { t, key_id });
        } else {
            if !self.active_keys.remove(&key_id) {
                return;
            }

            self.events.push(RecordingEvent::KeyUp { t, key_id });
        }
    }

    pub fn add_marker(&mut self, now_ms: u64, name: impl Into<String>) {
        self.events.push(RecordingEvent::Marker {
            t: self.relative_time(now_ms),
            name: name.into(),
        });
    }

    pub fn suppress_recent_keys(&mut self, key_ids: &[String]) {
        let Some(suppress_since) = key_ids
            .iter()
            .filter_map(|key_id| self.last_key_down_time(key_id))
            .min()
        else {
            return;
        };

        self.events.retain(|event| match event {
            RecordingEvent::KeyDown { t, key_id } | RecordingEvent::KeyUp { t, key_id } => {
                !key_ids.iter().any(|target| target == key_id) || *t < suppress_since
            }
            RecordingEvent::Marker { .. } => true,
        });

        for key_id in key_ids {
            self.active_keys.remove(key_id);
        }
    }

    fn last_key_down_time(&self, target_key_id: &str) -> Option<u64> {
        self.events.iter().rev().find_map(|event| match event {
            RecordingEvent::KeyDown { t, key_id } if key_id == target_key_id => Some(*t),
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

    fn relative_time(&self, now_ms: u64) -> u64 {
        now_ms.saturating_sub(self.start_time_ms)
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
        let mut session = self.session.lock().map_err(|error| error.to_string())?;
        let Some(active_session) = session.take() else {
            return Err("recording has not started".to_string());
        };

        std::fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;

        let path = output_dir.join(format!(
            "{}-{}.kbdrec",
            active_session.start_unix_ms, now_ms
        ));
        let contents = binary::encode_kbdrec(&active_session.session.snapshot())?;
        std::fs::write(&path, contents).map_err(|error| error.to_string())?;

        Ok(StopRecordingResult {
            path: path.to_string_lossy().to_string(),
        })
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

pub fn sample_frames(fps: u16, duration_ms: u64, events: &[RecordingEvent]) -> Vec<RecordingFrame> {
    let mut sorted_events = events.to_vec();
    sorted_events.sort_by_key(event_time);

    let mut active_keys = HashSet::<String>::new();
    let mut frames = Vec::new();
    let mut event_index = 0;
    let mut frame_index = 0;

    let mut t = frame_time_ms(frame_index, fps);
    while t <= duration_ms {
        while event_index < sorted_events.len() && event_time(&sorted_events[event_index]) <= t {
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
        frames.push(RecordingFrame { t, keys });
        frame_index += 1;
        t = frame_time_ms(frame_index, fps);
    }

    frames
}

fn frame_time_ms(frame_index: u64, fps: u16) -> u64 {
    frame_index * 1000 / u64::from(fps)
}

pub fn inspect_kbdrec(bytes: &[u8]) -> Result<RecordingInspection, String> {
    let decoded = binary::decode_kbdrec(bytes)?;
    let events = decoded
        .markers
        .iter()
        .map(|marker| RecordingEvent::Marker {
            t: marker.t_ms,
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

fn event_time(event: &RecordingEvent) -> u64 {
    match event {
        RecordingEvent::KeyDown { t, .. }
        | RecordingEvent::KeyUp { t, .. }
        | RecordingEvent::Marker { t, .. } => *t,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        binary::{decode_kbdrec, encode_kbdrec},
        inspect_kbdrec, sample_frames, RecordingEvent, RecordingManager, RecordingSession,
        RecordingSnapshot,
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
                    t: 120,
                    key_id: "w".to_string(),
                },
                RecordingEvent::KeyUp {
                    t: 190,
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
                    t: 100,
                    key_id: "tab".to_string(),
                },
                RecordingEvent::KeyUp {
                    t: 120,
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
                    t: 10,
                    key_id: "ctrl-left".to_string(),
                },
                RecordingEvent::KeyUp {
                    t: 20,
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
                t: 500,
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
                t: 250,
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
        assert_eq!(serialized["events"][0]["t"], 16);
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
                    t: 50,
                    key_id: "w".to_string(),
                },
                RecordingEvent::KeyUp {
                    t: 250,
                    key_id: "w".to_string(),
                },
            ],
        );

        assert_eq!(frames[0].t, 0);
        assert!(frames[0].keys.is_empty());
        assert_eq!(frames[1].keys, vec!["w".to_string()]);
        assert_eq!(frames[2].keys, vec!["w".to_string()]);
        assert!(frames[3].keys.is_empty());
    }

    #[test]
    fn samples_frames_without_integer_millisecond_drift() {
        let frames = sample_frames(60, 120, &[]);

        assert_eq!(frames[0].t, 0);
        assert_eq!(frames[3].t, 50);
        assert_eq!(frames[6].t, 100);
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
        assert_eq!(decoded.markers[0].t_ms, 200);
        assert_eq!(decoded.markers[0].name, "sync");
        assert!(decoded.runs.len() < decoded.frames.len());
    }

    #[test]
    fn binary_recording_excludes_virtual_void_keys() {
        let snapshot = RecordingSnapshot {
            version: 1,
            fps: 60,
            timebase: "monotonic",
            events: vec![RecordingEvent::KeyDown {
                t: 0,
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
                t: 200,
                name: "sync".to_string(),
            },],
        );
        assert!(inspection.frames[0].keys.is_empty());
        assert_eq!(inspection.frames[1].keys, vec!["w".to_string()]);
        assert_eq!(inspection.frames[2].keys, vec!["w".to_string()]);
        assert!(inspection.frames[3].keys.is_empty());
    }
}
