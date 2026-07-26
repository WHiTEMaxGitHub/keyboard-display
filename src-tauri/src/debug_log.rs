use std::{
    fs::{create_dir_all, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};

use chrono::Local;
use once_cell::sync::Lazy;

static LOG_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub fn write(source: &str, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("{timestamp} [{source}] {message}\n");

    if let Err(error) = append_line(&line) {
        eprintln!("failed to write debug log: {error}; {line}");
    }
}

fn append_line(line: &str) -> Result<(), String> {
    let _guard = LOG_LOCK.lock().map_err(|error| error.to_string())?;
    let path = log_path();
    if let Some(parent) = path.parent() {
        create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| error.to_string())?;
    file.write_all(line.as_bytes())
        .map_err(|error| error.to_string())
}

fn log_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("docs")
        .join("logs")
        .join("keyboard-display-debug.log")
}
