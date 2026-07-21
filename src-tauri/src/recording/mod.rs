mod binary;
mod browser;
mod clock;
mod filename;
mod inspection;
mod manager;
mod metadata;
mod session;
mod types;

pub use browser::{create_recording_folder, list_recording_files};
pub use clock::{monotonic_now_ms, unix_now_ms};
pub use inspection::inspect_kbdrec;
pub use manager::RecordingManager;
pub use metadata::{read_recording_metadata, save_recording_metadata};
pub use types::{RecordingInspection, RecordingMetadata, RecordingTreeNode, StopRecordingResult};

#[cfg(test)]
mod tests;
