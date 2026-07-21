use std::path::{Path, PathBuf};

use super::types::RecordingMetadata;

pub fn read_recording_metadata(path: PathBuf) -> Result<RecordingMetadata, String> {
    let sidecar_path = recording_metadata_path(&path);
    if !sidecar_path.exists() {
        return Ok(RecordingMetadata::default());
    }

    let contents = std::fs::read_to_string(sidecar_path).map_err(|error| error.to_string())?;
    parse_recording_metadata(&contents)
}

/// 写入 `.kbdrec.json` sidecar；这些字段只服务 UI，不改变 `.kbdrec` 二进制格式。
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

fn recording_metadata_path(path: &Path) -> PathBuf {
    let mut sidecar_path = path.as_os_str().to_os_string();
    sidecar_path.push(".json");
    PathBuf::from(sidecar_path)
}

pub fn is_kbdrec_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("kbdrec"))
        .unwrap_or(false)
}
