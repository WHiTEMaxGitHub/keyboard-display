use serde::Serialize;
use std::{
    io::{Cursor, Read},
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};

static INSTALL_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoExporterStatus {
    pub resolved: Option<VideoExporterCandidate>,
    pub app_managed: VideoExporterCandidate,
    pub user_selected: Option<VideoExporterCandidate>,
    pub path: VideoExporterCandidate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoExporterCandidate {
    pub source: String,
    pub path: String,
    pub available: bool,
    pub version: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallVideoExporterResult {
    pub path: String,
}

/// 只检测导出器，不修改系统 PATH 或用户已有 ffmpeg 配置。
pub fn detect_video_exporter(
    app_data_dir: PathBuf,
    user_selected_path: Option<PathBuf>,
) -> VideoExporterStatus {
    let app_managed_path = app_managed_ffmpeg_path(app_data_dir);
    let app_managed = candidate_from_path("app-managed", app_managed_path);
    let user_selected = user_selected_path.map(|path| candidate_from_path("user-selected", path));
    let path = candidate_from_path("path", PathBuf::from("ffmpeg"));
    let mut candidates = vec![&app_managed];
    if let Some(user_selected) = &user_selected {
        candidates.push(user_selected);
    }
    candidates.push(&path);
    let resolved = candidates
        .into_iter()
        .find(|candidate| candidate.available)
        .cloned();

    VideoExporterStatus {
        resolved,
        app_managed,
        user_selected,
        path,
    }
}

pub fn app_managed_ffmpeg_path(app_data_dir: PathBuf) -> PathBuf {
    let file_name = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    app_data_dir.join("exporter").join(file_name)
}

/// 下载并安装应用隔离的 ffmpeg 到 app data 目录；不会写入系统路径。
pub fn install_app_managed_ffmpeg(
    app_data_dir: PathBuf,
) -> Result<InstallVideoExporterResult, String> {
    let _guard = InstallGuard::acquire()?;
    let package = exporter_package()?;
    let bytes = download_bytes(package.url)?;
    let target_path = app_managed_ffmpeg_path(app_data_dir);
    install_ffmpeg_from_zip(&bytes, &target_path, package.binary_suffix)?;

    Ok(InstallVideoExporterResult {
        path: target_path.to_string_lossy().to_string(),
    })
}

/// 删除应用自己托管的 ffmpeg；不会触碰 PATH 或用户手动选择的二进制。
pub fn uninstall_app_managed_ffmpeg(app_data_dir: PathBuf) -> Result<(), String> {
    let path = app_managed_ffmpeg_path(app_data_dir);
    if !path.exists() {
        return Ok(());
    }

    std::fs::remove_file(&path).map_err(|error| error.to_string())
}

struct ExporterPackage {
    url: &'static str,
    binary_suffix: &'static str,
}

fn exporter_package() -> Result<ExporterPackage, String> {
    if cfg!(target_os = "macos") {
        Ok(ExporterPackage {
            url: "https://evermeet.cx/ffmpeg/getrelease/zip",
            binary_suffix: "ffmpeg",
        })
    } else if cfg!(target_os = "windows") {
        Ok(ExporterPackage {
            url: "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip",
            binary_suffix: "bin/ffmpeg.exe",
        })
    } else {
        Err("app-managed ffmpeg install is only supported on macOS and Windows".to_string())
    }
}

fn download_bytes(url: &str) -> Result<Vec<u8>, String> {
    let response = ureq::get(url).call().map_err(|error| error.to_string())?;
    let mut reader = response.into_body().into_reader();
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn install_ffmpeg_from_zip(
    bytes: &[u8],
    target_path: &PathBuf,
    binary_suffix: &str,
) -> Result<(), String> {
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|error| error.to_string())?;
    let binary_index = find_binary_in_zip(&mut archive, binary_suffix)?;
    let mut binary = archive
        .by_index(binary_index)
        .map_err(|error| error.to_string())?;

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let temp_path = target_path.with_extension("download");
    {
        let mut output = std::fs::File::create(&temp_path).map_err(|error| error.to_string())?;
        std::io::copy(&mut binary, &mut output).map_err(|error| error.to_string())?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&temp_path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&temp_path, permissions).map_err(|error| error.to_string())?;
    }

    std::fs::rename(&temp_path, target_path).map_err(|error| error.to_string())
}

fn find_binary_in_zip<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    binary_suffix: &str,
) -> Result<usize, String> {
    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(|error| error.to_string())?;
        let name = file.name().replace('\\', "/");
        if !file.is_dir() && name.ends_with(binary_suffix) {
            return Ok(index);
        }
    }

    Err(format!(
        "ffmpeg binary not found in downloaded archive: {binary_suffix}"
    ))
}

fn candidate_from_path(source: &str, path: PathBuf) -> VideoExporterCandidate {
    let path_string = path.to_string_lossy().to_string();
    let version = ffmpeg_version(&path);

    VideoExporterCandidate {
        source: source.to_string(),
        path: path_string,
        available: version.is_some(),
        version,
    }
}

fn ffmpeg_version(path: &PathBuf) -> Option<String> {
    let output = Command::new(path).arg("-version").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

#[derive(Debug)]
struct InstallGuard;

impl InstallGuard {
    fn acquire() -> Result<Self, String> {
        INSTALL_IN_PROGRESS
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| "app-managed ffmpeg install is already running".to_string())?;

        Ok(Self)
    }
}

impl Drop for InstallGuard {
    fn drop(&mut self) {
        INSTALL_IN_PROGRESS.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        app_managed_ffmpeg_path, find_binary_in_zip, install_ffmpeg_from_zip,
        uninstall_app_managed_ffmpeg, InstallGuard,
    };

    #[test]
    fn builds_app_managed_ffmpeg_path_under_exporter_dir() {
        let path = app_managed_ffmpeg_path(std::path::PathBuf::from("/app/data"));

        assert!(path.ends_with(if cfg!(target_os = "windows") {
            "exporter/ffmpeg.exe"
        } else {
            "exporter/ffmpeg"
        }));
    }

    #[test]
    fn uninstall_missing_app_managed_ffmpeg_is_ok() {
        let root = std::env::temp_dir().join(format!(
            "keyboard-display-exporter-missing-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);

        uninstall_app_managed_ffmpeg(root.clone()).unwrap();

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn finds_nested_ffmpeg_binary_in_zip() {
        let bytes = test_zip_with_file("pkg/bin/ffmpeg", b"binary");
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();

        let index = find_binary_in_zip(&mut archive, "bin/ffmpeg").unwrap();

        assert_eq!(archive.by_index(index).unwrap().name(), "pkg/bin/ffmpeg");
    }

    #[test]
    fn installs_binary_from_zip_to_target_path() {
        let root = std::env::temp_dir().join(format!(
            "keyboard-display-exporter-install-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let target_path = root.join("exporter").join("ffmpeg");
        let bytes = test_zip_with_file("ffmpeg", b"binary");

        install_ffmpeg_from_zip(&bytes, &target_path, "ffmpeg").unwrap();

        assert_eq!(std::fs::read(&target_path).unwrap(), b"binary");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn install_guard_rejects_concurrent_installs() {
        let guard = InstallGuard::acquire().unwrap();

        let error = InstallGuard::acquire().unwrap_err();

        assert!(error.contains("already running"));
        drop(guard);
        InstallGuard::acquire().unwrap();
    }

    fn test_zip_with_file(name: &str, contents: &[u8]) -> Vec<u8> {
        let cursor = std::io::Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(cursor);
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut writer, contents).unwrap();
        writer.finish().unwrap().into_inner()
    }
}
