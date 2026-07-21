use std::{path::PathBuf, sync::Mutex};

use super::{
    binary, filename::format_recording_file_name, session::RecordingSession,
    types::StopRecordingResult,
};

pub struct RecordingManager {
    pub(crate) session: Mutex<Option<ActiveRecordingSession>>,
}

pub(crate) struct ActiveRecordingSession {
    pub(crate) start_unix_ms: u64,
    pub(crate) session: RecordingSession,
}

impl RecordingManager {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(None),
        }
    }

    /// Tauri State 持有的线程安全入口；具体按键事件去重由 RecordingSession 负责。
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

    #[cfg(test)]
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
