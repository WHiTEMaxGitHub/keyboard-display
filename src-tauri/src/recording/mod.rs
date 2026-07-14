#![allow(dead_code)]

use serde::Serialize;
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingFrame {
    pub t: u64,
    pub keys: Vec<String>,
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

    pub fn start(&self, fps: u16, now_ms: u64) -> Result<(), String> {
        let mut session = self.session.lock().map_err(|error| error.to_string())?;

        *session = Some(ActiveRecordingSession {
            start_unix_ms: now_ms,
            session: RecordingSession::new(fps, now_ms),
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
        let contents = serde_json::to_string_pretty(&active_session.session.snapshot())
            .map_err(|error| error.to_string())?;
        std::fs::write(&path, format!("{contents}\n")).map_err(|error| error.to_string())?;

        Ok(StopRecordingResult {
            path: path.to_string_lossy().to_string(),
        })
    }
}

pub fn now_ms() -> Result<u64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;

    Ok(duration.as_millis() as u64)
}

pub fn sample_frames(fps: u16, duration_ms: u64, events: &[RecordingEvent]) -> Vec<RecordingFrame> {
    let frame_duration_ms = 1000 / u64::from(fps);
    let mut sorted_events = events.to_vec();
    sorted_events.sort_by_key(event_time);

    let mut active_keys = HashSet::<String>::new();
    let mut frames = Vec::new();
    let mut event_index = 0;
    let mut t = 0;

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
        t += frame_duration_ms;
    }

    frames
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
    use super::{sample_frames, RecordingEvent, RecordingSession};

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
}
