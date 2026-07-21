use std::collections::HashSet;

use super::types::{RecordingEvent, RecordingFrame, RecordingSnapshot};

pub struct RecordingSession {
    pub(crate) fps: u16,
    start_time_ms: u64,
    events: Vec<RecordingEvent>,
    active_keys: HashSet<String>,
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

/// 将按下/抬起事件流采样成逐帧状态，供二进制录制和未来视频导出复用。
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

pub fn ms_to_frame(ms: u64, fps: u16) -> u64 {
    (ms * u64::from(fps) + 500) / 1000
}

pub fn event_frame(event: &RecordingEvent) -> u64 {
    match event {
        RecordingEvent::KeyDown { frame, .. }
        | RecordingEvent::KeyUp { frame, .. }
        | RecordingEvent::Marker { frame, .. } => *frame,
    }
}
