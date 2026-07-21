use std::path::PathBuf;

use super::{
    binary,
    filename::parse_recording_file_times,
    metadata::{is_kbdrec_file, read_recording_metadata},
    types::{RecordingFileSummary, RecordingMarkerSummary, RecordingTreeNode},
};

/// 递归扫描录制根目录，返回前端浏览器直接可渲染的目录树。
pub fn list_recording_files(root: PathBuf) -> Result<RecordingTreeNode, String> {
    if !root.exists() {
        return Ok(missing_directory_node(&root));
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
        exists: true,
        node_type: "directory".to_string(),
        children,
        summary: None,
    }
}

fn missing_directory_node(path: &PathBuf) -> RecordingTreeNode {
    RecordingTreeNode {
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string()),
        path: path.to_string_lossy().to_string(),
        exists: false,
        node_type: "directory".to_string(),
        children: Vec::new(),
        summary: None,
    }
}

fn file_node(path: &PathBuf) -> Result<RecordingTreeNode, String> {
    let summary = summarize_recording_file(path)?;

    Ok(RecordingTreeNode {
        name: summary.file_name.clone(),
        path: path.to_string_lossy().to_string(),
        exists: true,
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
        markers: decoded
            .markers
            .into_iter()
            .map(|marker| RecordingMarkerSummary {
                frame: marker.frame,
                name: marker.name,
            })
            .collect(),
        metadata: read_recording_metadata(path.clone())?,
    })
}
