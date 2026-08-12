use std::{path::PathBuf, sync::OnceLock};

use chrono::Local;
use keyboard_display_debug_log::{BufferedDebugLogWriter, DebugLogWriter};
use tauri::Manager;

static LOG_WRITER: OnceLock<BufferedDebugLogWriter> = OnceLock::new();

pub fn init(app: &tauri::AppHandle) {
    let path = log_path(app);
    let _ = LOG_WRITER.set(BufferedDebugLogWriter::new(path));
}

pub fn write(source: &str, message: &str) {
    write_with_level("INFO", source, message);
}

pub fn warn(source: &str, message: &str) {
    write_with_level("WARN", source, message);
}

pub fn error(source: &str, message: &str) {
    write_with_level("ERROR", source, message);
}

fn write_with_level(level: &str, source: &str, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("{timestamp} {level} [{source}] {message}\n");

    if let Err(error) = writer().write_line(&line) {
        eprintln!("failed to write debug log: {error}; {line}");
    }
}

pub fn flush() -> Result<(), String> {
    writer().flush()
}

fn writer() -> &'static BufferedDebugLogWriter {
    LOG_WRITER.get_or_init(|| BufferedDebugLogWriter::new(fallback_log_path()))
}

fn log_path(app: &tauri::AppHandle) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d");
    app_runtime_dir(app)
        .join("log")
        .join(format!("keyboard-display-{date}.log"))
}

fn app_runtime_dir(app: &tauri::AppHandle) -> PathBuf {
    #[cfg(debug_assertions)]
    {
        return app
            .path()
            .app_config_dir()
            .unwrap_or_else(|_| fallback_runtime_dir());
    }

    #[cfg(not(debug_assertions))]
    {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
            .unwrap_or_else(fallback_runtime_dir)
    }
}

fn fallback_log_path() -> PathBuf {
    let date = Local::now().format("%Y-%m-%d");
    fallback_runtime_dir()
        .join("log")
        .join(format!("keyboard-display-{date}.log"))
}

fn fallback_runtime_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("keyboard-display-runtime")
}
