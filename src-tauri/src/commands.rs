use tauri::{Emitter, Manager};

use crate::{
    debug_log,
    exporter::{self, InstallVideoExporterResult, VideoExporterStatus},
    recording::{self, RecordingManager},
    video_export::{self, ExportOverlayProfile, ExportOverlayProgress, ExportOverlayVideoResult},
};

#[tauri::command]
pub fn save_config_file(path: std::path::PathBuf, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents).map_err(|error| {
        log_error(
            "profile-config",
            &format!("save path={}", path.display()),
            error,
        )
    })
}

#[tauri::command]
pub fn write_debug_log(source: String, message: String) {
    debug_log::write(&source, &message);
}

#[tauri::command]
pub fn read_config_file(path: std::path::PathBuf) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|error| {
        log_error(
            "profile-config",
            &format!("read path={}", path.display()),
            error,
        )
    })
}

#[tauri::command]
pub fn load_app_config(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = app_config_path(&app)?;

    if !path.exists() {
        debug_log::warn(
            "app-config",
            &format!("load-missing path={}", path.display()),
        );
        return Ok(None);
    }

    std::fs::read_to_string(&path).map(Some).map_err(|error| {
        log_error(
            "app-config",
            &format!("load path={}", path.display()),
            error,
        )
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeAppConfigResult {
    pub path: String,
    pub initialized: bool,
}

#[tauri::command]
pub fn initialize_app_config(
    app: tauri::AppHandle,
    contents: String,
) -> Result<InitializeAppConfigResult, String> {
    let path = app_config_path(&app)?;

    #[cfg(debug_assertions)]
    {
        let _ = contents;
        return Ok(InitializeAppConfigResult {
            path: path.to_string_lossy().to_string(),
            initialized: false,
        });
    }

    #[cfg(not(debug_assertions))]
    {
        let state = app_config_json_state(&path);
        if matches!(state, AppConfigJsonState::Valid) {
            return Ok(InitializeAppConfigResult {
                path: path.to_string_lossy().to_string(),
                initialized: false,
            });
        }

        let reason = state.reason();
        debug_log::warn(
            "app-config",
            &format!(
                "initialize-regenerate reason={reason} path={}",
                path.display()
            ),
        );
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                log_error(
                    "app-config",
                    &format!("create-parent path={}", parent.display()),
                    error,
                )
            })?;
        }
        std::fs::write(&path, contents).map_err(|error| {
            log_error(
                "app-config",
                &format!("initialize path={}", path.display()),
                error,
            )
        })?;

        Ok(InitializeAppConfigResult {
            path: path.to_string_lossy().to_string(),
            initialized: true,
        })
    }
}

#[cfg(not(debug_assertions))]
enum AppConfigJsonState {
    Missing,
    Valid,
    Invalid,
}

#[cfg(not(debug_assertions))]
impl AppConfigJsonState {
    fn reason(&self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Valid => "valid",
            Self::Invalid => "invalid-json",
        }
    }
}

#[cfg(not(debug_assertions))]
fn app_config_json_state(path: &std::path::Path) -> AppConfigJsonState {
    if !path.exists() {
        return AppConfigJsonState::Missing;
    }

    match std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str::<serde_json::Value>(&contents).ok())
    {
        Some(_) => AppConfigJsonState::Valid,
        None => AppConfigJsonState::Invalid,
    }
}

#[tauri::command]
pub fn app_config_path_string(app: tauri::AppHandle) -> Result<String, String> {
    app_config_path(&app).map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_app_config(app: tauri::AppHandle, contents: String) -> Result<(), String> {
    let path = app_config_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            log_error(
                "app-config",
                &format!("create-parent path={}", parent.display()),
                error,
            )
        })?;
    }
    std::fs::write(&path, contents).map_err(|error| {
        log_error(
            "app-config",
            &format!("save path={}", path.display()),
            error,
        )
    })
}

fn app_config_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app_config_dir(app).map(|dir| dir.join("app-config.json"))
}

fn app_config_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        return app
            .path()
            .app_config_dir()
            .map_err(|error| error.to_string());
    }

    #[cfg(not(debug_assertions))]
    {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        executable
            .parent()
            .map(|path| path.to_path_buf())
            .ok_or_else(|| "failed to resolve executable directory".to_string())
    }
}

#[allow(dead_code)]
fn legacy_app_config_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join("app-config.json"))
}

#[tauri::command]
pub fn default_recording_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app_config_dir(&app)?.join("recording-files");

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn default_export_video_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app_config_dir(&app)?.join("export-videos");

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn start_recording(state: tauri::State<'_, RecordingManager>, fps: u16) -> Result<(), String> {
    debug_log::write("recording", &format!("start fps={fps}"));
    state
        .start(
            fps,
            recording::unix_now_ms()?,
            recording::monotonic_now_ms(),
        )
        .map_err(|error| log_error("recording", "start", error))
}

#[tauri::command]
pub fn record_input_event(
    state: tauri::State<'_, RecordingManager>,
    key_id: String,
    pressed: bool,
) -> Result<(), String> {
    state.record_input(recording::monotonic_now_ms(), key_id, pressed)
}

#[tauri::command]
pub fn add_recording_marker(
    state: tauri::State<'_, RecordingManager>,
    name: String,
) -> Result<(), String> {
    state
        .add_marker(recording::monotonic_now_ms(), name)
        .map_err(|error| log_error("recording", "marker", error))
}

#[tauri::command]
pub fn suppress_recording_keys(
    state: tauri::State<'_, RecordingManager>,
    key_ids: Vec<String>,
) -> Result<(), String> {
    state
        .suppress_recent_keys(key_ids)
        .map_err(|error| log_error("recording", "suppress-keys", error))
}

#[tauri::command]
pub fn stop_recording(
    state: tauri::State<'_, RecordingManager>,
    output_dir: std::path::PathBuf,
    filename_template: String,
    profile_name: String,
    fps: u16,
) -> Result<recording::StopRecordingResult, String> {
    debug_log::write(
        "recording",
        &format!(
            "stop output_dir={} fps={} profile={}",
            output_dir.display(),
            fps,
            profile_name
        ),
    );
    let result = state
        .stop_with_filename_template(
            output_dir,
            recording::unix_now_ms()?,
            &filename_template,
            &profile_name,
            fps,
        )
        .map_err(|error| log_error("recording", "stop", error))?;
    debug_log::write("recording", &format!("saved path={}", result.path));
    Ok(result)
}

#[tauri::command]
pub fn inspect_recording_file(
    path: std::path::PathBuf,
) -> Result<recording::RecordingInspection, String> {
    let bytes = std::fs::read(&path).map_err(|error| {
        log_error(
            "recording",
            &format!("inspect-read path={}", path.display()),
            error,
        )
    })?;
    recording::inspect_kbdrec(&bytes).map_err(|error| {
        log_error(
            "recording",
            &format!("inspect-parse path={}", path.display()),
            error,
        )
    })
}

#[tauri::command]
pub fn list_recording_files(
    root: std::path::PathBuf,
) -> Result<recording::RecordingTreeNode, String> {
    recording::list_recording_files(root.clone())
        .map_err(|error| log_error("recording", &format!("list root={}", root.display()), error))
}

#[tauri::command]
pub fn create_recording_folder(
    root: std::path::PathBuf,
    folder_name: String,
) -> Result<recording::RecordingTreeNode, String> {
    recording::create_recording_folder(root.clone(), folder_name.clone()).map_err(|error| {
        log_error(
            "recording",
            &format!("create-folder root={} name={folder_name}", root.display()),
            error,
        )
    })
}

#[tauri::command]
pub fn read_recording_metadata(
    path: std::path::PathBuf,
) -> Result<recording::RecordingMetadata, String> {
    recording::read_recording_metadata(path.clone()).map_err(|error| {
        log_error(
            "recording-metadata",
            &format!("read path={}", path.display()),
            error,
        )
    })
}

#[tauri::command]
pub fn save_recording_metadata(
    path: std::path::PathBuf,
    metadata: recording::RecordingMetadata,
) -> Result<recording::RecordingMetadata, String> {
    recording::save_recording_metadata(path.clone(), metadata).map_err(|error| {
        log_error(
            "recording-metadata",
            &format!("save path={}", path.display()),
            error,
        )
    })
}

#[tauri::command]
pub fn detect_video_exporter(
    app: tauri::AppHandle,
    user_selected_path: Option<std::path::PathBuf>,
) -> Result<VideoExporterStatus, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| log_error("exporter", "resolve-app-data-dir", error))?;

    Ok(exporter::detect_video_exporter(
        app_data_dir,
        user_selected_path,
    ))
}

#[tauri::command]
pub async fn install_app_managed_video_exporter(
    app: tauri::AppHandle,
) -> Result<InstallVideoExporterResult, String> {
    debug_log::write("exporter", "install-start");
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| log_error("exporter", "resolve-app-data-dir", error))?;

    let result = tauri::async_runtime::spawn_blocking(move || {
        exporter::install_app_managed_ffmpeg(app_data_dir)
    })
    .await
    .map_err(|error| log_error("exporter", "install-join", error))?
    .map_err(|error| log_error("exporter", "install", error))?;
    debug_log::write(
        "exporter",
        &format!("install-complete path={}", result.path),
    );
    Ok(result)
}

#[tauri::command]
pub async fn uninstall_app_managed_video_exporter(app: tauri::AppHandle) -> Result<(), String> {
    debug_log::write("exporter", "uninstall-start");
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| log_error("exporter", "resolve-app-data-dir", error))?;

    tauri::async_runtime::spawn_blocking(move || {
        exporter::uninstall_app_managed_ffmpeg(app_data_dir)
    })
    .await
    .map_err(|error| log_error("exporter", "uninstall-join", error))?
    .map_err(|error| log_error("exporter", "uninstall", error))?;
    debug_log::write("exporter", "uninstall-complete");
    Ok(())
}

#[tauri::command]
pub async fn export_overlay_video(
    app: tauri::AppHandle,
    recording_path: std::path::PathBuf,
    output_path: std::path::PathBuf,
    ffmpeg_path: std::path::PathBuf,
    profile: ExportOverlayProfile,
) -> Result<ExportOverlayVideoResult, String> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            log_error(
                "export",
                &format!("create-output-parent path={}", parent.display()),
                error,
            )
        })?;
    }

    debug_log::write(
        "export",
        &format!(
            "overlay-start recording={} output={} ffmpeg={} render_threads={:?}",
            recording_path.display(),
            output_path.display(),
            ffmpeg_path.display(),
            profile.export.render_threads
        ),
    );
    tauri::async_runtime::spawn_blocking(move || {
        video_export::export_overlay_video_with_progress(
            &recording_path,
            &output_path,
            &ffmpeg_path,
            &profile,
            |progress: ExportOverlayProgress| {
                app.emit("export-progress", progress)
                    .map_err(|error| error.to_string())
            },
        )
    })
    .await
    .map_err(|error| log_error("export", "overlay-join", error))?
    .map_err(|error| log_error("export", "overlay", error))
    .map(|result| {
        debug_log::write(
            "export",
            &format!(
                "overlay-complete output={} frames={} size={}x{} fps={}",
                result.output_path, result.frame_count, result.width, result.height, result.fps
            ),
        );
        result
    })
}

#[tauri::command]
pub fn copy_font_file(
    app: tauri::AppHandle,
    source_path: std::path::PathBuf,
) -> Result<String, String> {
    let fonts_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| log_error("font", "resolve-app-data-dir", error))?
        .join("fonts");
    std::fs::create_dir_all(&fonts_dir).map_err(|error| {
        log_error(
            "font",
            &format!("create-font-dir path={}", fonts_dir.display()),
            error,
        )
    })?;

    let file_name = source_path.file_name().ok_or_else(|| {
        let error = "invalid font file path".to_string();
        debug_log::error(
            "font",
            &format!("copy error={error} path={}", source_path.display()),
        );
        error
    })?;
    let dest = fonts_dir.join(file_name);
    std::fs::copy(&source_path, &dest).map_err(|error| {
        log_error(
            "font",
            &format!(
                "copy source={} dest={}",
                source_path.display(),
                dest.display()
            ),
            error,
        )
    })?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_directory(path: String) -> Result<(), String> {
    let path = std::path::Path::new(&path);
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|error| {
            log_error(
                "filesystem",
                &format!("open-directory-create path={}", path.display()),
                error,
            )
        })?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|error| {
                log_error(
                    "filesystem",
                    &format!("open-directory path={}", path.display()),
                    error,
                )
            })?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|error| {
                log_error(
                    "filesystem",
                    &format!("open-directory path={}", path.display()),
                    error,
                )
            })?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|error| {
                log_error(
                    "filesystem",
                    &format!("open-directory path={}", path.display()),
                    error,
                )
            })?;
    }
    Ok(())
}

fn log_error(source: &str, action: &str, error: impl std::fmt::Display) -> String {
    let message = error.to_string();
    debug_log::error(source, &format!("{action} error={message}"));
    message
}
