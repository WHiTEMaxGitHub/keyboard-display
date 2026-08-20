mod binary;
mod browser;
mod clock;
mod filename;
mod hotkeys;
mod inspection;
mod manager;
mod metadata;
mod session;
mod types;

pub(crate) use binary::decode_kbdrec_for_export;
#[cfg(all(test, unix))]
pub(crate) use binary::encode_kbdrec;
pub use binary::{inspect_kbdrec_export_info, RecordingExportInfo};
pub use browser::{create_recording_folder, list_recording_files};
pub use clock::{monotonic_now_ms, unix_now_ms};
pub use hotkeys::{evaluate_hotkeys, RecordingRuntimeSettings};
pub use inspection::inspect_kbdrec;
pub use manager::RecordingManager;
pub use metadata::{read_recording_metadata, save_recording_metadata};
#[cfg(test)]
pub(crate) use types::RecordingEvent;
#[cfg(all(test, unix))]
pub(crate) use types::RecordingSnapshot;
pub use types::{RecordingInspection, RecordingMetadata, RecordingTreeNode, StopRecordingResult};

#[cfg(test)]
mod tests;
