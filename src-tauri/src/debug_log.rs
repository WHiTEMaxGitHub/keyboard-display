use std::{path::PathBuf, sync::LazyLock};

use chrono::Local;
use keyboard_display_debug_log::{BufferedDebugLogWriter, DebugLogWriter};

static LOG_WRITER: LazyLock<BufferedDebugLogWriter> =
    LazyLock::new(|| BufferedDebugLogWriter::new(log_path()));

pub fn write(source: &str, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("{timestamp} [{source}] {message}\n");

    if let Err(error) = LOG_WRITER.write_line(&line) {
        eprintln!("failed to write debug log: {error}; {line}");
    }
}

pub fn flush() -> Result<(), String> {
    LOG_WRITER.flush()
}

fn log_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("docs")
        .join("logs")
        .join("keyboard-display-debug.log")
}
