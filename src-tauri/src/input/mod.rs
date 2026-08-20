pub mod mapping;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;

use serde::Serialize;
use std::{
    collections::{BTreeSet, VecDeque},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
};
use tauri::{AppHandle, Emitter, Manager};

use crate::debug_log;

const INPUT_STATE_EVENT: &str = "input-state";
const OVERLAY_ACTIVE_KEYS_EVENT: &str = "overlay-active-keys";
const CONFIG_QUEUE_CAPACITY: usize = 512;
const CONFIG_EMIT_BATCH: usize = 32;
const POV_QUEUE_CAPACITY: usize = 64;
const POV_EMIT_BATCH: usize = 8;
const POV_STALE_MS: u64 = 100;

/// Parsed capture edge. Capture stamps `t_mono` and returns; ingest applies
/// the edge in order (recording is 1:1, never coalesced).
struct IngestEvent {
    capture_seq: u64,
    key_id: String,
    pressed: bool,
    t_mono: u64,
}

#[derive(Clone, Debug)]
struct PovEmitJob {
    seq: u64,
    capture_seq: u64,
    t_capture: u64,
    key_ids: Vec<String>,
}

#[derive(Clone, Debug)]
struct ConfigEmitJob {
    seq: u64,
    capture_seq: u64,
    t_capture: u64,
    key_ids: Vec<String>,
    key_id: String,
    pressed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputStatePayload {
    pub seq: u64,
    pub capture_seq: u64,
    pub t_capture: u64,
    pub key_ids: Vec<String>,
    pub key_id: String,
    pub pressed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayActiveKeysPayload {
    pub seq: u64,
    pub capture_seq: u64,
    pub t_capture: u64,
    pub key_ids: Vec<String>,
    pub debug: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum ApplyResult {
    Changed(Vec<String>),
    Unchanged(Vec<String>),
}

impl ApplyResult {
    fn snapshot(&self) -> &[String] {
        match self {
            Self::Changed(keys) | Self::Unchanged(keys) => keys,
        }
    }
}

#[derive(Default)]
struct PovQueueState {
    jobs: VecDeque<PovEmitJob>,
    dropped: u64,
}

#[derive(Default)]
struct PovQueue {
    state: Mutex<PovQueueState>,
    ready: Condvar,
}

impl PovQueue {
    fn publish(&self, job: PovEmitJob) -> u64 {
        let Ok(mut state) = self.state.lock() else {
            return 0;
        };
        if state.jobs.len() >= POV_QUEUE_CAPACITY {
            state.jobs.pop_front();
            state.dropped = state.dropped.saturating_add(1);
        }
        state.jobs.push_back(job);
        let dropped = state.dropped;
        self.ready.notify_one();
        dropped
    }

    fn wait_take_batch(&self) -> Option<(VecDeque<PovEmitJob>, u64)> {
        let mut state = self.state.lock().ok()?;
        while state.jobs.is_empty() {
            state = self.ready.wait(state).ok()?;
        }
        let jobs = take_pov_batch(&mut state);
        let dropped = std::mem::take(&mut state.dropped);
        Some((jobs, dropped))
    }

    fn take_latest_now(&self) -> Option<(PovEmitJob, u64)> {
        let mut state = self.state.lock().ok()?;
        let job = state.jobs.pop_back()?;
        let skipped = state.jobs.len() as u64;
        state.jobs.clear();
        state.dropped = state.dropped.saturating_add(skipped);
        let dropped = std::mem::take(&mut state.dropped);
        Some((job, dropped))
    }

    #[cfg(test)]
    fn take_batch(&self) -> Option<(VecDeque<PovEmitJob>, u64)> {
        let mut state = self.state.lock().ok()?;
        if state.jobs.is_empty() {
            return None;
        }
        let jobs = take_pov_batch(&mut state);
        let dropped = std::mem::take(&mut state.dropped);
        Some((jobs, dropped))
    }
}

fn take_pov_batch(state: &mut PovQueueState) -> VecDeque<PovEmitJob> {
    if state.jobs.len() > POV_EMIT_BATCH {
        let latest = state.jobs.pop_back().expect("non-empty POV queue");
        state.dropped = state.dropped.saturating_add(state.jobs.len() as u64);
        state.jobs.clear();
        return VecDeque::from([latest]);
    }
    state.jobs.drain(..).collect()
}

#[derive(Default)]
struct ConfigQueue {
    jobs: Mutex<VecDeque<ConfigEmitJob>>,
    ready: Condvar,
}

impl ConfigQueue {
    /// The lock only protects an in-memory queue. The WebView emit happens
    /// after it is released, so ingest can never wait on WebView IPC.
    fn publish(&self, job: ConfigEmitJob) -> u64 {
        let Ok(mut jobs) = self.jobs.lock() else {
            return 0;
        };
        let mut dropped = 0;
        if jobs.len() >= CONFIG_QUEUE_CAPACITY {
            // Drop strictly oldest without biasing Down versus Up. Every job
            // carries a complete key snapshot so a seq gap can restore config
            // preview state and safely close a missed binding release.
            jobs.pop_front();
            dropped = 1;
        }
        jobs.push_back(job);
        self.ready.notify_one();
        dropped
    }

    fn wait_take_batch(&self) -> Option<Vec<ConfigEmitJob>> {
        let mut jobs = self.jobs.lock().ok()?;
        while jobs.is_empty() {
            jobs = self.ready.wait(jobs).ok()?;
        }
        Some(take_bounded_batch(&mut jobs, CONFIG_EMIT_BATCH))
    }

    #[cfg(test)]
    fn take_batch(&self, limit: usize) -> Vec<ConfigEmitJob> {
        self.jobs
            .lock()
            .map(|mut jobs| take_bounded_batch(&mut jobs, limit))
            .unwrap_or_default()
    }
}

fn take_bounded_batch<T>(queue: &mut VecDeque<T>, limit: usize) -> Vec<T> {
    let count = queue.len().min(limit);
    queue.drain(..count).collect()
}

pub struct InputStateBridge {
    active_keys: Mutex<BTreeSet<String>>,
    next_capture_seq: AtomicU64,
    next_display_seq: AtomicU64,
    ingest_tx: Sender<IngestEvent>,
    ingest_rx: Mutex<Option<Receiver<IngestEvent>>>,
    pov_queue: Arc<PovQueue>,
    config_queue: Arc<ConfigQueue>,
}

impl InputStateBridge {
    pub fn new() -> Self {
        let (ingest_tx, ingest_rx) = mpsc::channel();
        Self {
            active_keys: Mutex::new(BTreeSet::new()),
            next_capture_seq: AtomicU64::new(1),
            next_display_seq: AtomicU64::new(1),
            ingest_tx,
            ingest_rx: Mutex::new(Some(ingest_rx)),
            pov_queue: Arc::new(PovQueue::default()),
            config_queue: Arc::new(ConfigQueue::default()),
        }
    }

    fn start_ingest_worker(&self, app_handle: AppHandle) {
        let Ok(mut ingest_rx) = self.ingest_rx.lock() else {
            debug_log::error("input", "failed to lock ingest receiver");
            return;
        };
        let Some(rx) = ingest_rx.take() else {
            return;
        };
        if let Err(error) = std::thread::Builder::new()
            .name("input-ingest".into())
            .spawn(move || ingest_loop(app_handle, rx))
        {
            debug_log::error("input", &format!("failed to start ingest worker: {error}"));
        }
    }

    fn start_pov_worker(&self, app_handle: AppHandle) {
        let queue = Arc::clone(&self.pov_queue);
        if let Err(error) = std::thread::Builder::new()
            .name("pov-emit".into())
            .spawn(move || pov_emit_loop(app_handle, queue))
        {
            debug_log::error("input", &format!("failed to start POV worker: {error}"));
        }
    }

    fn start_config_worker(&self, app_handle: AppHandle) {
        let queue = Arc::clone(&self.config_queue);
        if let Err(error) = std::thread::Builder::new()
            .name("config-input-emit".into())
            .spawn(move || config_emit_loop(app_handle, queue))
        {
            debug_log::error("input", &format!("failed to start config worker: {error}"));
        }
    }

    /// Capture-thread enqueue: unbounded send, then return. No apply, record,
    /// hotkeys, overlay emit, or file I/O.
    fn enqueue_ingest(&self, key_id: String, pressed: bool, t_mono: u64) -> u64 {
        let capture_seq = self.next_capture_seq.fetch_add(1, Ordering::Relaxed);
        if self
            .ingest_tx
            .send(IngestEvent {
                capture_seq,
                key_id,
                pressed,
                t_mono,
            })
            .is_err()
        {
            debug_log::error("input", "ingest worker disconnected");
        }
        capture_seq
    }

    fn ingest_edge(&self, app_handle: &AppHandle, event: IngestEvent) -> Result<(), String> {
        let recording = app_handle.try_state::<crate::recording::RecordingManager>();
        let applied = self.apply_key_and_record(
            recording.as_deref(),
            event.t_mono,
            &event.key_id,
            event.pressed,
        )?;
        let ingest_age = crate::recording::monotonic_now_ms().saturating_sub(event.t_mono);
        let changed = matches!(applied, ApplyResult::Changed(_));
        if !changed {
            log_ingest_debug(&event, None, ingest_age, false, applied.snapshot());
            return Ok(());
        }
        let display_seq = self.next_display_seq.fetch_add(1, Ordering::Relaxed);
        log_ingest_debug(
            &event,
            Some(display_seq),
            ingest_age,
            true,
            applied.snapshot(),
        );
        let key_ids = applied.snapshot().to_vec();

        // Keyboard Raw Input (not mouse) evaluates recording hotkeys so
        // F8 / Ctrl+Shift+R work while a game is focused. One ingest-queue
        // hop of delay is OK; do not add polling.
        if !event.key_id.starts_with("mouse-") {
            crate::recording::evaluate_hotkeys(app_handle, &key_ids);
        }

        let dropped = self.pov_queue.publish(PovEmitJob {
            seq: display_seq,
            capture_seq: event.capture_seq,
            t_capture: event.t_mono,
            key_ids: key_ids.clone(),
        });
        let config_dropped = self.config_queue.publish(ConfigEmitJob {
            seq: display_seq,
            capture_seq: event.capture_seq,
            t_capture: event.t_mono,
            key_ids,
            key_id: event.key_id,
            pressed: event.pressed,
        });
        if debug_log::input_debug_enabled() && (dropped > 0 || config_dropped > 0) {
            debug_log::debug(
                "input-ingest",
                &format!(
                    "capture_seq={} display_seq={display_seq} pov_pending_dropped={} config_dropped={config_dropped}",
                    event.capture_seq, dropped
                ),
            );
        }
        Ok(())
    }

    /// Overlay snapshot and recording events share this mutation so a held
    /// key is written at the same monotonic timestamp the overlay sees.
    fn apply_key_and_record(
        &self,
        recording: Option<&crate::recording::RecordingManager>,
        now_ms: u64,
        key_id: &str,
        pressed: bool,
    ) -> Result<ApplyResult, String> {
        let applied = self.apply_key_change(key_id, pressed)?;
        if matches!(applied, ApplyResult::Changed(_)) {
            if let Some(recording) = recording {
                recording.record_input(now_ms, key_id, pressed)?;
            }
        }
        Ok(applied)
    }

    /// Authoritative active-set mutation. Repeated keyboard Down and unmatched
    /// Up are filtered here before recording, hotkeys, POV, or config jobs.
    fn apply_key_change(&self, key_id: &str, pressed: bool) -> Result<ApplyResult, String> {
        let mut active_keys = self.active_keys.lock().map_err(|error| error.to_string())?;
        let changed = if pressed {
            active_keys.insert(key_id.to_string())
        } else {
            active_keys.remove(key_id)
        };
        let snapshot = active_keys.iter().cloned().collect();
        Ok(if changed {
            ApplyResult::Changed(snapshot)
        } else {
            ApplyResult::Unchanged(snapshot)
        })
    }

    #[cfg(test)]
    fn apply_key(&self, key_id: &str, pressed: bool) -> Result<Vec<String>, String> {
        Ok(self.apply_key_change(key_id, pressed)?.snapshot().to_vec())
    }

    #[cfg(test)]
    pub(super) fn is_active(&self, key_id: &str) -> bool {
        self.active_keys
            .lock()
            .map(|keys| keys.contains(key_id))
            .unwrap_or(false)
    }

    #[cfg(test)]
    fn snapshot(&self) -> Vec<String> {
        self.active_keys
            .lock()
            .map(|keys| keys.iter().cloned().collect())
            .unwrap_or_default()
    }

    #[cfg(test)]
    fn drain_queued_ingest(&self) -> Vec<IngestEvent> {
        let Ok(rx_guard) = self.ingest_rx.lock() else {
            return Vec::new();
        };
        let Some(rx) = rx_guard.as_ref() else {
            return Vec::new();
        };
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }
        events
    }

    /// Test stand-in for the ingest worker: apply + record (memory) + overlay
    /// enqueue. Hotkeys need AppHandle and are covered by the live worker.
    #[cfg(test)]
    fn process_queued_ingest(
        &self,
        recording: Option<&crate::recording::RecordingManager>,
    ) -> Result<(), String> {
        for event in self.drain_queued_ingest() {
            let applied =
                self.apply_key_and_record(recording, event.t_mono, &event.key_id, event.pressed)?;
            if let ApplyResult::Changed(key_ids) = applied {
                let display_seq = self.next_display_seq.fetch_add(1, Ordering::Relaxed);
                self.pov_queue.publish(PovEmitJob {
                    seq: display_seq,
                    capture_seq: event.capture_seq,
                    t_capture: event.t_mono,
                    key_ids: key_ids.clone(),
                });
                self.config_queue.publish(ConfigEmitJob {
                    seq: display_seq,
                    capture_seq: event.capture_seq,
                    t_capture: event.t_mono,
                    key_ids,
                    key_id: event.key_id,
                    pressed: event.pressed,
                });
            }
        }
        Ok(())
    }
}

pub fn start_native_input_backend(app_handle: AppHandle) {
    if let Some(bridge) = app_handle.try_state::<InputStateBridge>() {
        bridge.start_ingest_worker(app_handle.clone());
        bridge.start_pov_worker(app_handle.clone());
        bridge.start_config_worker(app_handle.clone());
    }

    #[cfg(target_os = "macos")]
    macos::start(app_handle);

    #[cfg(target_os = "windows")]
    windows::start(app_handle);

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    unsupported::start(app_handle);
}

/// Capture-thread entry: stamp monotonic time, enqueue the parsed edge, return.
/// Must not apply, record, evaluate hotkeys, emit, or wait on disk / WebView.
fn emit_input_state(app_handle: &AppHandle, key_id: impl Into<String>, pressed: bool) {
    let key_id = key_id.into();
    let t_mono = crate::recording::monotonic_now_ms();

    if let Some(state) = app_handle.try_state::<InputStateBridge>() {
        state.enqueue_ingest(key_id, pressed, t_mono);
        return;
    }

    debug_log::error(
        "input",
        "InputStateBridge missing; dropped capture event before ingest",
    );
}

fn ingest_loop(app_handle: AppHandle, rx: Receiver<IngestEvent>) {
    while let Ok(event) = rx.recv() {
        let Some(bridge) = app_handle.try_state::<InputStateBridge>() else {
            debug_log::error("input", "InputStateBridge missing on ingest worker");
            continue;
        };
        if let Err(error) = bridge.ingest_edge(&app_handle, event) {
            debug_log::error("input", &format!("failed to ingest capture edge: {error}"));
        }
    }
}

fn pov_emit_loop(app_handle: AppHandle, queue: Arc<PovQueue>) {
    while let Some((mut batch, mut dropped)) = queue.wait_take_batch() {
        let batch_size = batch.len() as u64;
        while let Some(mut job) = batch.pop_front() {
            let mut age = crate::recording::monotonic_now_ms().saturating_sub(job.t_capture);
            let mut stale = u64::from(age > POV_STALE_MS);

            if stale > 0 {
                if let Some(latest_in_batch) = batch.pop_back() {
                    dropped = dropped.saturating_add(batch.len() as u64 + 1);
                    batch.clear();
                    job = latest_in_batch;
                }
                if let Some((latest_queued, queued_dropped)) = queue.take_latest_now() {
                    dropped = dropped.saturating_add(queued_dropped + 1);
                    job = latest_queued;
                }
                age = crate::recording::monotonic_now_ms().saturating_sub(job.t_capture);
                stale = u64::from(age > POV_STALE_MS);
            }

            let overlay = OverlayActiveKeysPayload {
                seq: job.seq,
                capture_seq: job.capture_seq,
                t_capture: job.t_capture,
                key_ids: job.key_ids,
                debug: debug_log::input_debug_enabled(),
            };
            if let Err(error) = app_handle.emit_to("pov", OVERLAY_ACTIVE_KEYS_EVENT, overlay) {
                debug_log::error(
                    "input",
                    &format!("failed to emit overlay active keys: {error}"),
                );
            }
            log_emit_debug(
                "pov",
                job.capture_seq,
                job.seq,
                age,
                batch_size,
                dropped,
                stale,
            );
            dropped = 0;
        }
    }
}

fn config_emit_loop(app_handle: AppHandle, queue: Arc<ConfigQueue>) {
    while let Some(batch) = queue.wait_take_batch() {
        let batch_size = batch.len() as u64;
        for job in batch {
            let age = crate::recording::monotonic_now_ms().saturating_sub(job.t_capture);
            let payload = InputStatePayload {
                seq: job.seq,
                capture_seq: job.capture_seq,
                t_capture: job.t_capture,
                key_ids: job.key_ids,
                key_id: job.key_id,
                pressed: job.pressed,
            };
            if let Err(error) = app_handle.emit(INPUT_STATE_EVENT, payload) {
                debug_log::error(
                    "input",
                    &format!("failed to emit input state to config: {error}"),
                );
            }
            log_emit_debug("config", job.capture_seq, job.seq, age, batch_size, 0, 0);
        }
    }
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn emit_backend_log(
    _app_handle: &AppHandle,
    message: impl Into<String>,
    details: impl IntoIterator<Item = (impl Into<String>, String)>,
) {
    let message = message.into();
    let details = details
        .into_iter()
        .map(|(key, value)| (key.into(), value))
        .collect::<std::collections::BTreeMap<String, String>>();

    debug_log::warn("input-backend", &format!("{message} {details:?}"));
}

fn log_ingest_debug(
    event: &IngestEvent,
    display_seq: Option<u64>,
    age: u64,
    changed: bool,
    active_keys: &[String],
) {
    if !debug_log::input_debug_enabled() {
        return;
    }
    debug_log::debug(
        "input-ingest",
        &format!(
            "capture_seq={} display_seq={} t_capture={} age_ms={age} key_id={} pressed={} result={} repeat={} active_keys={active_keys:?}",
            event.capture_seq,
            display_seq.map_or_else(|| "-".to_string(), |seq| seq.to_string()),
            event.t_mono,
            event.key_id,
            event.pressed,
            if changed { "changed" } else { "unchanged" },
            u8::from(!changed),
        ),
    );
}

fn log_emit_debug(
    source: &str,
    capture_seq: u64,
    display_seq: u64,
    age: u64,
    batch: u64,
    dropped: u64,
    stale: u64,
) {
    if debug_log::input_debug_enabled() {
        debug_log::debug(
            source,
            &format!(
                "capture_seq={capture_seq} display_seq={display_seq} capture_to_emit_age_ms={age} batch={batch} dropped={dropped} stale={stale}"
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        take_bounded_batch, ApplyResult, ConfigEmitJob, InputStateBridge, PovEmitJob,
        CONFIG_EMIT_BATCH, CONFIG_QUEUE_CAPACITY,
    };
    use crate::recording::{RecordingEvent, RecordingManager};
    use std::{collections::VecDeque, thread, time::Duration};

    fn keys(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_string()).collect()
    }

    #[test]
    fn jump_throw_sequence_clears_mouse_left_and_space() {
        let bridge = InputStateBridge::new();

        assert_eq!(
            bridge.apply_key("mouse-left", true).unwrap(),
            keys(&["mouse-left"])
        );
        assert_eq!(
            bridge.apply_key("space", true).unwrap(),
            keys(&["mouse-left", "space"])
        );
        assert_eq!(
            bridge.apply_key("space", false).unwrap(),
            keys(&["mouse-left"])
        );

        thread::sleep(Duration::from_millis(80));

        assert_eq!(bridge.apply_key("mouse-left", false).unwrap(), keys(&[]));
        assert!(bridge.snapshot().is_empty());
        assert!(!bridge.is_active("mouse-left"));
        assert!(!bridge.is_active("space"));
    }

    #[test]
    fn apply_key_records_keydown_and_keyup_into_session() {
        let bridge = InputStateBridge::new();
        let manager = RecordingManager::new();
        manager.start(60, 1000, 1000).unwrap();

        assert!(matches!(
            bridge
                .apply_key_and_record(Some(&manager), 1016, "w", true)
                .unwrap(),
            ApplyResult::Changed(_)
        ));
        assert!(matches!(
            bridge
                .apply_key_and_record(Some(&manager), 1083, "w", false)
                .unwrap(),
            ApplyResult::Changed(_)
        ));

        let session = manager.session.lock().unwrap();
        assert_eq!(
            session.as_ref().unwrap().session.snapshot().events,
            vec![
                RecordingEvent::KeyDown {
                    frame: 1,
                    key_id: "w".to_string(),
                },
                RecordingEvent::KeyUp {
                    frame: 5,
                    key_id: "w".to_string(),
                },
            ]
        );
    }

    #[test]
    fn recording_keeps_every_real_edge_when_pov_is_not_consumed() {
        let bridge = InputStateBridge::new();
        let manager = RecordingManager::new();
        manager.start(60, 1000, 1000).unwrap();

        let edges = [
            (1010_u64, "mouse-left", true),
            (1016, "space", true),
            (1036, "space", false),
            (1048, "mouse-left", false),
        ];
        for (now_ms, key_id, pressed) in edges {
            bridge.enqueue_ingest(key_id.to_string(), pressed, now_ms);
        }
        bridge.process_queued_ingest(Some(&manager)).unwrap();

        let session = manager.session.lock().unwrap();
        assert_eq!(
            session.as_ref().unwrap().session.snapshot().events,
            vec![
                RecordingEvent::KeyDown {
                    frame: 1,
                    key_id: "mouse-left".to_string(),
                },
                RecordingEvent::KeyDown {
                    frame: 1,
                    key_id: "space".to_string(),
                },
                RecordingEvent::KeyUp {
                    frame: 2,
                    key_id: "space".to_string(),
                },
                RecordingEvent::KeyUp {
                    frame: 3,
                    key_id: "mouse-left".to_string(),
                },
            ]
        );

        drop(session);
        let (batch, dropped) = bridge.pov_queue.take_batch().unwrap();
        assert_eq!(batch.len(), 4);
        assert!(batch.back().unwrap().key_ids.is_empty());
        assert_eq!(dropped, 0);
        assert_eq!(bridge.config_queue.take_batch(16).len(), 4);
    }

    #[test]
    fn bounded_drain_never_consumes_unlimited_history() {
        let mut queue = (0..100).collect::<VecDeque<_>>();
        let batch = take_bounded_batch(&mut queue, CONFIG_EMIT_BATCH);
        assert_eq!(batch.len(), CONFIG_EMIT_BATCH);
        assert_eq!(queue.len(), 100 - CONFIG_EMIT_BATCH);
    }

    #[test]
    fn pov_backlog_drops_old_and_keeps_latest_snapshot() {
        let bridge = InputStateBridge::new();
        for seq in 1..=256 {
            bridge.pov_queue.publish(PovEmitJob {
                seq,
                capture_seq: seq,
                t_capture: seq,
                key_ids: vec![format!("key-{seq}")],
            });
        }
        let (batch, dropped) = bridge.pov_queue.take_batch().unwrap();
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].seq, 256);
        assert_eq!(batch[0].key_ids, keys(&["key-256"]));
        assert_eq!(dropped, 255);
    }

    #[test]
    fn pov_and_config_backpressure_are_independent() {
        let bridge = InputStateBridge::new();
        for seq in 1..=64 {
            bridge.pov_queue.publish(PovEmitJob {
                seq,
                capture_seq: seq,
                t_capture: seq,
                key_ids: vec![seq.to_string()],
            });
        }
        bridge.config_queue.publish(ConfigEmitJob {
            seq: 65,
            capture_seq: 65,
            t_capture: 65,
            key_ids: keys(&["w"]),
            key_id: "w".into(),
            pressed: true,
        });

        assert_eq!(bridge.config_queue.take_batch(1)[0].seq, 65);
        assert_eq!(bridge.pov_queue.take_batch().unwrap().0[0].seq, 64);
    }

    #[test]
    fn repeated_states_do_not_record_or_enqueue_display_jobs() {
        let bridge = InputStateBridge::new();
        let manager = RecordingManager::new();
        manager.start(60, 1000, 1000).unwrap();
        for (time, pressed) in [(1010, true), (1011, true), (1020, false), (1021, false)] {
            bridge.enqueue_ingest("w".into(), pressed, time);
        }
        bridge.process_queued_ingest(Some(&manager)).unwrap();

        let events = manager
            .session
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .session
            .snapshot()
            .events;
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], RecordingEvent::KeyDown { .. }));
        assert!(matches!(events[1], RecordingEvent::KeyUp { .. }));
        let config_jobs = bridge.config_queue.take_batch(16);
        assert_eq!(config_jobs.len(), 2);
        assert_eq!(
            config_jobs.iter().map(|job| job.seq).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            config_jobs
                .iter()
                .map(|job| job.capture_seq)
                .collect::<Vec<_>>(),
            vec![1, 3]
        );
        let (batch, dropped) = bridge.pov_queue.take_batch().unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(
            batch.iter().map(|job| job.seq).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            batch.iter().map(|job| job.capture_seq).collect::<Vec<_>>(),
            vec![1, 3]
        );
        assert!(batch.back().unwrap().key_ids.is_empty());
        assert_eq!(dropped, 0);
    }

    #[test]
    fn display_seq_follows_single_ingest_order_when_capture_seq_arrives_out_of_order() {
        let bridge = InputStateBridge::new();
        bridge
            .ingest_tx
            .send(super::IngestEvent {
                capture_seq: 2,
                key_id: "w".into(),
                pressed: true,
                t_mono: 1010,
            })
            .unwrap();
        bridge
            .ingest_tx
            .send(super::IngestEvent {
                capture_seq: 1,
                key_id: "space".into(),
                pressed: true,
                t_mono: 1011,
            })
            .unwrap();
        bridge.process_queued_ingest(None).unwrap();

        let jobs = bridge.config_queue.take_batch(4);
        assert_eq!(
            jobs.iter().map(|job| job.seq).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            jobs.iter().map(|job| job.capture_seq).collect::<Vec<_>>(),
            vec![2, 1]
        );
    }

    #[test]
    fn config_overflow_drops_fifo_and_latest_snapshot_recovers_state() {
        let bridge = InputStateBridge::new();
        for seq in 1..=CONFIG_QUEUE_CAPACITY as u64 {
            bridge.config_queue.publish(ConfigEmitJob {
                seq,
                capture_seq: seq,
                t_capture: seq,
                key_ids: keys(&["mouse-left"]),
                key_id: "mouse-left".into(),
                pressed: seq % 2 == 1,
            });
        }
        bridge.config_queue.publish(ConfigEmitJob {
            seq: CONFIG_QUEUE_CAPACITY as u64 + 1,
            capture_seq: CONFIG_QUEUE_CAPACITY as u64 + 1,
            t_capture: 999,
            key_ids: Vec::new(),
            key_id: "mouse-left".into(),
            pressed: false,
        });

        let jobs = bridge.config_queue.take_batch(CONFIG_QUEUE_CAPACITY + 1);
        assert_eq!(jobs.len(), CONFIG_QUEUE_CAPACITY);
        assert_eq!(jobs[0].seq, 2);
        assert!(jobs.last().unwrap().key_ids.is_empty());
        assert!(!jobs.last().unwrap().pressed);
    }

    #[test]
    fn capture_enqueue_does_not_apply_or_record() {
        let bridge = InputStateBridge::new();
        let manager = RecordingManager::new();
        manager.start(60, 1000, 1000).unwrap();

        bridge.enqueue_ingest("w".to_string(), true, 1016);

        assert!(
            bridge.snapshot().is_empty(),
            "capture must return before overlay apply"
        );
        assert!(
            manager
                .session
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .session
                .snapshot()
                .events
                .is_empty(),
            "capture must return before recording commit"
        );
        assert!(bridge.pov_queue.take_batch().is_none());
        assert!(bridge.config_queue.take_batch(1).is_empty());
        assert_eq!(bridge.drain_queued_ingest().len(), 1);
    }

    #[test]
    fn ingest_commits_recording_without_waiting_for_overlay() {
        let bridge = InputStateBridge::new();
        let manager = RecordingManager::new();
        manager.start(60, 1000, 1000).unwrap();

        let edges = [
            (1010_u64, "mouse-left", true),
            (1016, "space", true),
            (1036, "space", false),
            (1048, "mouse-left", false),
        ];
        for (t_mono, key_id, pressed) in edges {
            bridge.enqueue_ingest(key_id.to_string(), pressed, t_mono);
        }

        bridge.process_queued_ingest(Some(&manager)).unwrap();

        let session = manager.session.lock().unwrap();
        assert_eq!(
            session.as_ref().unwrap().session.snapshot().events,
            vec![
                RecordingEvent::KeyDown {
                    frame: 1,
                    key_id: "mouse-left".to_string(),
                },
                RecordingEvent::KeyDown {
                    frame: 1,
                    key_id: "space".to_string(),
                },
                RecordingEvent::KeyUp {
                    frame: 2,
                    key_id: "space".to_string(),
                },
                RecordingEvent::KeyUp {
                    frame: 3,
                    key_id: "mouse-left".to_string(),
                },
            ]
        );
        drop(session);

        let (batch, dropped) = bridge.pov_queue.take_batch().unwrap();
        assert_eq!(batch.len(), 4);
        assert!(batch.back().unwrap().key_ids.is_empty());
        assert_eq!(dropped, 0);
        assert_eq!(bridge.config_queue.take_batch(16).len(), 4);
        assert!(bridge.snapshot().is_empty());
    }

    #[test]
    fn interleaved_jump_throw_records_every_changed_edge_with_contiguous_display_seq() {
        let bridge = InputStateBridge::new();
        let manager = RecordingManager::new();
        manager.start(60, 1000, 1000).unwrap();
        let edges = [
            (1010, "mouse-left", true),
            (1011, "mouse-left", true),
            (1014, "w", true),
            (1016, "space", true),
            (1020, "space", false),
            (1022, "w", false),
            (1028, "mouse-left", false),
        ];
        for (time, key, pressed) in edges {
            bridge.enqueue_ingest(key.into(), pressed, time);
        }
        bridge.process_queued_ingest(Some(&manager)).unwrap();

        let events = manager
            .session
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .session
            .snapshot()
            .events;
        assert_eq!(events.len(), 6);

        let (jobs, dropped) = bridge.pov_queue.take_batch().unwrap();
        assert_eq!(dropped, 0);
        assert_eq!(
            jobs.iter().map(|job| job.seq).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6]
        );
        assert_eq!(
            jobs.iter().map(|job| job.capture_seq).collect::<Vec<_>>(),
            vec![1, 3, 4, 5, 6, 7]
        );
        assert!(jobs.back().unwrap().key_ids.is_empty());
    }
}
