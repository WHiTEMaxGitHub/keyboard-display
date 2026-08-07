use std::{
    fs::{create_dir_all, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub trait DebugLogWriter: Send + Sync {
    fn write_line(&self, line: &str) -> Result<(), String>;
    fn flush(&self) -> Result<(), String>;
    fn shutdown(&self) -> Result<(), String>;
}

pub struct SyncDebugLogWriter {
    path: PathBuf,
    lock: Mutex<()>,
}

impl SyncDebugLogWriter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            lock: Mutex::new(()),
        }
    }
}

impl DebugLogWriter for SyncDebugLogWriter {
    fn write_line(&self, line: &str) -> Result<(), String> {
        let _guard = self.lock.lock().map_err(|error| error.to_string())?;
        append_line_to_path(&self.path, line)
    }

    fn flush(&self) -> Result<(), String> {
        let _guard = self.lock.lock().map_err(|error| error.to_string())?;
        flush_path(&self.path)
    }

    fn shutdown(&self) -> Result<(), String> {
        self.flush()
    }
}

enum BufferedCommand {
    Write(String),
    Flush(mpsc::Sender<Result<(), String>>),
    Shutdown(mpsc::Sender<Result<(), String>>),
}

pub struct BufferedDebugLogWriter {
    sender: Mutex<Option<mpsc::Sender<BufferedCommand>>>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl BufferedDebugLogWriter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let (sender, receiver) = mpsc::channel::<BufferedCommand>();
        let worker = thread::spawn(move || run_buffered_writer(path, receiver));

        Self {
            sender: Mutex::new(Some(sender)),
            worker: Mutex::new(Some(worker)),
        }
    }

    fn send(&self, command: BufferedCommand) -> Result<(), String> {
        let sender = self.sender.lock().map_err(|error| error.to_string())?;
        let Some(sender) = sender.as_ref() else {
            return Err("debug log writer is shut down".to_string());
        };
        sender.send(command).map_err(|error| error.to_string())
    }
}

impl DebugLogWriter for BufferedDebugLogWriter {
    fn write_line(&self, line: &str) -> Result<(), String> {
        self.send(BufferedCommand::Write(line.to_string()))
    }

    fn flush(&self) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel();
        self.send(BufferedCommand::Flush(sender))?;
        receiver.recv().map_err(|error| error.to_string())?
    }

    fn shutdown(&self) -> Result<(), String> {
        let (sender, receiver) = mpsc::channel();
        self.send(BufferedCommand::Shutdown(sender))?;
        let result = receiver.recv().map_err(|error| error.to_string())?;

        {
            let mut sender = self.sender.lock().map_err(|error| error.to_string())?;
            sender.take();
        }

        if let Some(worker) = self.worker.lock().map_err(|error| error.to_string())?.take() {
            worker
                .join()
                .map_err(|_| "debug log worker thread panicked".to_string())?;
        }

        result
    }
}

impl Drop for BufferedDebugLogWriter {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn run_buffered_writer(path: PathBuf, receiver: mpsc::Receiver<BufferedCommand>) {
    let mut buffer = String::new();

    for command in receiver {
        match command {
            BufferedCommand::Write(line) => buffer.push_str(&line),
            BufferedCommand::Flush(response) => {
                let result = flush_buffer(&path, &mut buffer);
                let _ = response.send(result);
            }
            BufferedCommand::Shutdown(response) => {
                let result = flush_buffer(&path, &mut buffer);
                let _ = response.send(result);
                break;
            }
        }
    }
}

fn flush_buffer(path: &Path, buffer: &mut String) -> Result<(), String> {
    if buffer.is_empty() {
        return flush_path(path);
    }

    append_line_to_path(path, buffer)?;
    buffer.clear();
    flush_path(path)
}

fn append_line_to_path(path: &Path, line: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(line.as_bytes())
        .map_err(|error| error.to_string())
}

fn flush_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    OpenOptions::new()
        .append(true)
        .open(path)
        .and_then(|mut file| file.flush())
        .map_err(|error| error.to_string())
}

pub type SharedDebugLogWriter = Arc<dyn DebugLogWriter>;

#[cfg(test)]
mod tests {
    use super::{BufferedDebugLogWriter, DebugLogWriter, SyncDebugLogWriter};

    fn temp_log_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "keyboard-display-debug-log-{name}-{}.log",
            std::process::id()
        ))
    }

    #[test]
    fn sync_writer_appends_lines_and_flushes() {
        let path = temp_log_path("sync-append");
        let _ = std::fs::remove_file(&path);
        let writer = SyncDebugLogWriter::new(&path);

        writer.write_line("first\n").unwrap();
        writer.write_line("second\n").unwrap();
        writer.flush().unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "first\nsecond\n");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn sync_writer_creates_parent_directories() {
        let path = std::env::temp_dir()
            .join(format!("keyboard-display-debug-log-dir-{}", std::process::id()))
            .join("nested")
            .join("debug.log");
        let root = path.parent().unwrap().parent().unwrap();
        let _ = std::fs::remove_dir_all(root);
        let writer = SyncDebugLogWriter::new(&path);

        writer.write_line("hello\n").unwrap();
        writer.shutdown().unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello\n");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn buffered_writer_flushes_on_request() {
        let path = temp_log_path("buffered-flush");
        let _ = std::fs::remove_file(&path);
        let writer = BufferedDebugLogWriter::new(&path);

        writer.write_line("first\n").unwrap();
        writer.write_line("second\n").unwrap();
        writer.flush().unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "first\nsecond\n");
        writer.shutdown().unwrap();

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn buffered_writer_flushes_on_shutdown() {
        let path = temp_log_path("buffered-shutdown");
        let _ = std::fs::remove_file(&path);
        let writer = BufferedDebugLogWriter::new(&path);

        writer.write_line("pending\n").unwrap();
        writer.shutdown().unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "pending\n");

        let _ = std::fs::remove_file(path);
    }
}
