#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::{mpsc, Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};
use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform,
};

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOverlayProfile {
    pub layout: ExportOverlayLayout,
    pub rows: Vec<Vec<ExportOverlayItem>>,
    pub style: ExportOverlayStyle,
    pub export: ExportVideoConfig,
    pub recording: ExportRecordingConfig,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOverlayLayout {
    pub unit_px: f32,
    pub gap_unit: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum ExportOverlayItem {
    Key {
        id: String,
        label: String,
        width_unit: f32,
    },
    Gap {
        width_unit: f32,
    },
    Void {
        width_unit: f32,
    },
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOverlayStyle {
    pub scale: f32,
    pub opacity: f32,
    pub background_color: String,
    pub background_opacity: f32,
    pub background_radius: f32,
    pub idle_key_visibility: String,
    pub idle_color: String,
    pub active_color: String,
    pub idle_text_color: String,
    pub active_text_color: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportVideoConfig {
    pub render_markers: bool,
    pub font_path: Option<String>,
    pub render_threads: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRecordingConfig {
    pub sync_feedback_duration_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOverlaySize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOverlayVideoResult {
    pub output_path: String,
    pub frame_count: u64,
    pub width: u32,
    pub height: u32,
    pub fps: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOverlayProgress {
    pub rendered_frames: u64,
    pub total_frames: u64,
    pub current_frame: u64,
    pub active_key_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFrameRange {
    pub start_frame: u64,
    pub end_frame_exclusive: u64,
}

const BACKPLATE_PADDING: f32 = 10.0 * 2.0;
const OVERLAY_BLEED: f32 = 12.0 * 2.0;
const FLOAT_EPSILON: f32 = 0.000001;
const KEY_BORDER_RGBA: (u8, u8, u8, u8) = (255, 255, 255, 41);
const KEY_ACTIVE_BORDER_RGBA: (u8, u8, u8, u8) = (255, 255, 255, 128);
const TEXT_FONT_SIZE_FACTOR: f32 = 15.0;
const FRAME_CACHE_BUDGET_BYTES: usize = 128 * 1024 * 1024;
const PREFETCH_BUDGET_BYTES: usize = 64 * 1024 * 1024;
const MAX_PREFETCH_FRAMES: usize = 16;
const MAX_FFMPEG_STDERR_BYTES: usize = 1024 * 1024;
const RENDER_RESULT_TIMEOUT: Duration = Duration::from_secs(120);
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);
static EXPORT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn estimate_export_overlay_size(profile: &ExportOverlayProfile) -> ExportOverlaySize {
    let unit = profile.layout.unit_px * profile.style.scale;
    let gap = unit * normalize_unit(profile.layout.gap_unit);
    let padding = if is_backplate_visible(&profile.style.background_color) {
        BACKPLATE_PADDING
    } else {
        0.0
    };
    let width_units = profile
        .rows
        .iter()
        .map(|row| row_width_units(row, profile.layout.gap_unit))
        .fold(1.0_f32, f32::max);
    let row_count = profile.rows.len().max(1) as f32;

    ExportOverlaySize {
        width: ceil_stable(width_units * unit + padding + OVERLAY_BLEED),
        height: ceil_stable(row_count * unit + (row_count - 1.0) * gap + padding + OVERLAY_BLEED),
    }
}

/// 渲染单帧 RGBA overlay。
pub fn render_overlay_frame(
    profile: &ExportOverlayProfile,
    active_keys: &HashSet<String>,
    marker_active: bool,
) -> Result<(ExportOverlaySize, Vec<u8>), String> {
    let size = estimate_export_overlay_size(profile);
    let mut pixmap = Pixmap::new(size.width, size.height)
        .ok_or_else(|| "failed to allocate export frame".to_string())?;
    let font = load_font(profile.export.font_path.as_deref());

    render_profile(
        &mut pixmap,
        profile,
        active_keys,
        marker_active,
        font.as_ref(),
    )?;

    Ok((size, premultiplied_to_straight_rgba(pixmap.take())))
}

pub fn render_overlay_frame_with_font(
    profile: &ExportOverlayProfile,
    active_keys: &HashSet<String>,
    marker_active: bool,
    font: Option<&fontdue::Font>,
) -> Result<(ExportOverlaySize, Vec<u8>), String> {
    let size = estimate_export_overlay_size(profile);
    let mut pixmap = Pixmap::new(size.width, size.height)
        .ok_or_else(|| "failed to allocate export frame".to_string())?;

    render_profile(&mut pixmap, profile, active_keys, marker_active, font)?;

    Ok((size, premultiplied_to_straight_rgba(pixmap.take())))
}

pub fn build_webm_ffmpeg_args(output_path: &str, size: ExportOverlaySize, fps: u16) -> Vec<String> {
    vec![
        "-y".to_string(),
        "-f".to_string(),
        "rawvideo".to_string(),
        "-pix_fmt".to_string(),
        "rgba".to_string(),
        "-s".to_string(),
        format!("{}x{}", size.width, size.height),
        "-r".to_string(),
        fps.to_string(),
        "-i".to_string(),
        "-".to_string(),
        "-an".to_string(),
        "-c:v".to_string(),
        "libvpx-vp9".to_string(),
        "-pix_fmt".to_string(),
        "yuva420p".to_string(),
        "-auto-alt-ref".to_string(),
        "0".to_string(),
        output_path.to_string(),
    ]
}

/// 从 `.kbdrec` 帧状态流渲染透明 WebM overlay，不修改用户 ffmpeg 安装。
pub fn export_overlay_video(
    recording_path: &Path,
    output_path: &Path,
    ffmpeg_path: &Path,
    profile: &ExportOverlayProfile,
) -> Result<ExportOverlayVideoResult, String> {
    export_overlay_video_with_progress(
        recording_path,
        output_path,
        ffmpeg_path,
        profile,
        |_| Ok(()),
    )
}

/// 从 `.kbdrec` 帧状态流渲染透明 WebM overlay，并在渲染过程中上报帧进度。
/// 支持多线程并行渲染，通过 `profile.export.render_threads` 配置线程数。
/// `None` 和 `0` 使用 CPU 核心数，`-1` 使用高并发上限。
pub fn export_overlay_video_with_progress(
    recording_path: &Path,
    output_path: &Path,
    ffmpeg_path: &Path,
    profile: &ExportOverlayProfile,
    mut on_progress: impl FnMut(ExportOverlayProgress) -> Result<(), String>,
) -> Result<ExportOverlayVideoResult, String> {
    export_overlay_video_with_progress_range(
        recording_path,
        output_path,
        ffmpeg_path,
        profile,
        None,
        &mut on_progress,
    )
}

pub fn export_overlay_video_with_progress_range(
    recording_path: &Path,
    output_path: &Path,
    ffmpeg_path: &Path,
    profile: &ExportOverlayProfile,
    range: Option<ExportFrameRange>,
    mut on_progress: impl FnMut(ExportOverlayProgress) -> Result<(), String>,
) -> Result<ExportOverlayVideoResult, String> {
    let _export_guard = EXPORT_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| "export lock poisoned".to_string())?;
    let bytes = std::fs::read(recording_path).map_err(|error| error.to_string())?;
    let decoded = crate::recording::decode_kbdrec_for_export(&bytes)?;
    let (start_frame, end_frame_exclusive) = validate_export_range(range, decoded.frame_count)?;
    let size = estimate_export_overlay_size(profile);
    let frame_bytes = frame_byte_len(size)?;
    let total_frames = end_frame_exclusive - start_frame;
    let marker_ranges = marker_frame_ranges(
        decoded.markers.iter().map(|marker| marker.frame),
        decoded.fps,
        profile.recording.sync_feedback_duration_ms,
    );
    let output_path_string = output_path.to_string_lossy().to_string();
    let args = build_webm_ffmpeg_args(&output_path_string, size, decoded.fps);
    let mut child = Command::new(ffmpeg_path)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start ffmpeg: {error}"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "failed to open ffmpeg stderr".to_string())?;
    let stderr_handle =
        std::thread::spawn(move || drain_reader_limited(stderr, MAX_FFMPEG_STDERR_BYTES));

    let available_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let num_threads = resolve_render_thread_count(profile.export.render_threads, available_threads);
    let fps = decoded.fps;
    let prefetch_capacity = resolve_prefetch_capacity(num_threads, frame_bytes);
    let worker_count = num_threads.min(prefetch_capacity).max(1);
    let (progress_tx, progress_rx) = mpsc::channel::<ExportOverlayProgress>();

    let profile_owned = profile.clone();
    let coordinator = std::thread::spawn(move || -> Result<(), String> {
        let pipeline_result = (|| -> Result<(), String> {
            let (job_tx, job_rx) = mpsc::sync_channel::<RenderJob>(prefetch_capacity);
            let job_rx = Arc::new(Mutex::new(job_rx));
            let (render_tx, render_rx) =
                mpsc::sync_channel::<Result<RenderedFrame, String>>(prefetch_capacity);
            let cache = Arc::new(Mutex::new(FrameCache::new(FRAME_CACHE_BUDGET_BYTES)));
            let font = load_font(profile_owned.export.font_path.as_deref()).map(Arc::new);
            let mut worker_handles = Vec::with_capacity(worker_count);

            for _ in 0..worker_count {
                let job_rx = Arc::clone(&job_rx);
                let render_tx = render_tx.clone();
                let cache = Arc::clone(&cache);
                let profile = profile_owned.clone();
                let font = font.clone();
                worker_handles.push(std::thread::spawn(move || {
                    render_worker(job_rx, render_tx, cache, profile, font)
                }));
            }
            drop(render_tx);

            let mut stdin = child
                .stdin
                .take()
                .ok_or_else(|| "failed to open ffmpeg stdin".to_string())?;
            let mut frames = decoded.frames_in_range(start_frame, end_frame_exclusive)?;
            let mut next_index = 0usize;
            let mut rendered_frames = 0u64;
            let mut last_progress = Instant::now();

            loop {
                let mut batch_len = 0usize;
                while batch_len < prefetch_capacity {
                    let Some((frame_number, keys)) = frames.next() else {
                        break;
                    };
                    let marker_active = marker_is_active(frame_number, &marker_ranges);
                    let active_key_ids = normalize_active_keys(keys);
                    job_tx
                        .send(RenderJob {
                            index: next_index + batch_len,
                            frame_number,
                            active_key_ids,
                            marker_active,
                        })
                        .map_err(|_| "render job channel disconnected".to_string())?;
                    batch_len += 1;
                }
                if batch_len == 0 {
                    break;
                }

                let mut pending =
                    collect_render_batch(&render_rx, batch_len, RENDER_RESULT_TIMEOUT)?;
                for expected_index in next_index..next_index + batch_len {
                    let rendered = pending
                        .remove(&expected_index)
                        .ok_or_else(|| format!("missing rendered frame index {expected_index}"))?;
                    stdin
                        .write_all(&rendered.rgba)
                        .map_err(|error| format!("failed to write ffmpeg stdin: {error}"))?;
                    rendered_frames += 1;
                    let is_final = rendered_frames == total_frames;
                    if is_final || last_progress.elapsed() >= PROGRESS_INTERVAL {
                        progress_tx
                            .send(ExportOverlayProgress {
                                rendered_frames,
                                total_frames,
                                current_frame: rendered.frame_number,
                                active_key_ids: rendered.active_key_ids,
                            })
                            .map_err(|_| "export progress receiver disconnected".to_string())?;
                        last_progress = Instant::now();
                    }
                }
                next_index += batch_len;
            }

            drop(job_tx);
            for handle in worker_handles {
                handle
                    .join()
                    .map_err(|_| "render worker panicked outside a job".to_string())??;
            }
            if rendered_frames != total_frames {
                return Err(format!(
                    "missing rendered frames: expected {total_frames}, wrote {rendered_frames}"
                ));
            }
            drop(stdin);
            Ok(())
        })();

        if pipeline_result.is_err() {
            let _ = child.kill();
        }
        let wait_result = child
            .wait()
            .map_err(|error| format!("failed to finish ffmpeg export: {error}"));
        let stderr = stderr_handle
            .join()
            .map_err(|_| "ffmpeg stderr reader panicked".to_string())??;

        pipeline_result.map_err(|error| append_ffmpeg_stderr(error, &stderr))?;
        let status = wait_result?;
        if !status.success() {
            return Err(append_ffmpeg_stderr(
                format!("ffmpeg export failed with status {status}"),
                &stderr,
            ));
        }
        Ok(())
    });

    let mut progress_error = None;
    while let Ok(progress) = progress_rx.recv() {
        if progress_error.is_none() {
            if let Err(error) = on_progress(progress) {
                progress_error = Some(error);
            }
        }
    }
    let coordinator_result = coordinator
        .join()
        .map_err(|_| "export coordinator panicked".to_string())?;
    coordinator_result?;
    if let Some(error) = progress_error {
        return Err(error);
    }

    Ok(ExportOverlayVideoResult {
        output_path: output_path_string,
        frame_count: total_frames,
        width: size.width,
        height: size.height,
        fps,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct VisualStateKey {
    active_key_ids: Vec<String>,
    marker_active: bool,
}

struct RenderJob {
    index: usize,
    frame_number: u64,
    active_key_ids: Vec<String>,
    marker_active: bool,
}

struct RenderedFrame {
    index: usize,
    frame_number: u64,
    active_key_ids: Vec<String>,
    rgba: Arc<Vec<u8>>,
}

struct FrameCache {
    max_bytes: usize,
    current_bytes: usize,
    entries: HashMap<VisualStateKey, Arc<Vec<u8>>>,
    order: VecDeque<VisualStateKey>,
}

impl FrameCache {
    fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            current_bytes: 0,
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn get(&self, key: &VisualStateKey) -> Option<Arc<Vec<u8>>> {
        self.entries.get(key).cloned()
    }

    fn insert(&mut self, key: VisualStateKey, rgba: Arc<Vec<u8>>) {
        let entry_bytes = rgba.len();
        if self.entries.contains_key(&key) || entry_bytes > self.max_bytes {
            return;
        }
        while self.current_bytes.saturating_add(entry_bytes) > self.max_bytes {
            if let Some(oldest) = self.order.pop_front() {
                if let Some(removed) = self.entries.remove(&oldest) {
                    self.current_bytes = self.current_bytes.saturating_sub(removed.len());
                }
            } else {
                break;
            }
        }
        self.order.push_back(key.clone());
        self.current_bytes = self.current_bytes.saturating_add(entry_bytes);
        self.entries.insert(key, rgba);
    }
}

fn render_worker(
    job_rx: Arc<Mutex<mpsc::Receiver<RenderJob>>>,
    render_tx: mpsc::SyncSender<Result<RenderedFrame, String>>,
    cache: Arc<Mutex<FrameCache>>,
    profile: ExportOverlayProfile,
    font: Option<Arc<fontdue::Font>>,
) -> Result<(), String> {
    loop {
        let job = {
            let receiver = job_rx
                .lock()
                .map_err(|_| "render job receiver lock poisoned".to_string())?;
            receiver.recv()
        };
        let job = match job {
            Ok(job) => job,
            Err(_) => return Ok(()),
        };
        let rendered = catch_job_panic(job.index, || {
            render_job(job, &profile, font.as_deref(), &cache)
        });
        let failed = rendered.is_err();
        render_tx
            .send(rendered)
            .map_err(|_| "render result receiver disconnected".to_string())?;
        if failed {
            return Ok(());
        }
    }
}

fn render_job(
    job: RenderJob,
    profile: &ExportOverlayProfile,
    font: Option<&fontdue::Font>,
    cache: &Mutex<FrameCache>,
) -> Result<RenderedFrame, String> {
    let cache_key = VisualStateKey {
        active_key_ids: job.active_key_ids.clone(),
        marker_active: job.marker_active,
    };
    let cached = cache
        .lock()
        .map_err(|_| "frame cache lock poisoned".to_string())?
        .get(&cache_key);
    let rgba = if let Some(rgba) = cached {
        rgba
    } else {
        let active_keys = job.active_key_ids.iter().cloned().collect::<HashSet<_>>();
        let (_, rgba) =
            render_overlay_frame_with_font(profile, &active_keys, job.marker_active, font)?;
        let rgba = Arc::new(rgba);
        cache
            .lock()
            .map_err(|_| "frame cache lock poisoned".to_string())?
            .insert(cache_key, Arc::clone(&rgba));
        rgba
    };
    Ok(RenderedFrame {
        index: job.index,
        frame_number: job.frame_number,
        active_key_ids: job.active_key_ids,
        rgba,
    })
}

fn catch_job_panic<T>(
    job_index: usize,
    work: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).unwrap_or_else(|payload| {
        let detail = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("unknown panic");
        Err(format!("render job {job_index} panicked: {detail}"))
    })
}

fn collect_render_batch(
    render_rx: &mpsc::Receiver<Result<RenderedFrame, String>>,
    batch_len: usize,
    timeout: Duration,
) -> Result<BTreeMap<usize, RenderedFrame>, String> {
    let deadline = Instant::now() + timeout;
    let mut pending = BTreeMap::new();
    for _ in 0..batch_len {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let rendered = render_rx
            .recv_timeout(remaining)
            .map_err(|error| match error {
                mpsc::RecvTimeoutError::Timeout => {
                    format!("timed out waiting for rendered frame after {timeout:?}")
                }
                mpsc::RecvTimeoutError::Disconnected => {
                    "render result channel disconnected before completing batch".to_string()
                }
            })??;
        if pending.insert(rendered.index, rendered).is_some() {
            return Err("render worker returned a duplicate frame".to_string());
        }
    }
    Ok(pending)
}

fn frame_byte_len(size: ExportOverlaySize) -> Result<usize, String> {
    usize::try_from(size.width)
        .ok()
        .and_then(|width| {
            usize::try_from(size.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| "export frame dimensions are too large".to_string())
}

fn resolve_prefetch_capacity(render_threads: usize, frame_bytes: usize) -> usize {
    let budget_frames = if frame_bytes == 0 {
        1
    } else {
        (PREFETCH_BUDGET_BYTES / frame_bytes).max(1)
    };
    render_threads
        .saturating_mul(2)
        .min(MAX_PREFETCH_FRAMES)
        .min(budget_frames)
        .max(1)
}

fn drain_reader_limited(mut reader: impl Read, max_bytes: usize) -> Result<String, String> {
    let mut retained = VecDeque::with_capacity(max_bytes.min(8192));
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| format!("failed to read ffmpeg stderr: {error}"))?;
        if read == 0 {
            break;
        }
        for byte in &buffer[..read] {
            if retained.len() == max_bytes {
                retained.pop_front();
                truncated = true;
            }
            if max_bytes > 0 {
                retained.push_back(*byte);
            }
        }
    }
    let bytes = retained.into_iter().collect::<Vec<_>>();
    let text = String::from_utf8_lossy(&bytes);
    if truncated {
        Ok(format!(
            "[ffmpeg stderr truncated to last {max_bytes} bytes]\n{text}"
        ))
    } else {
        Ok(text.into_owned())
    }
}

fn append_ffmpeg_stderr(error: String, stderr: &str) -> String {
    let stderr = stderr.trim();
    if stderr.is_empty() {
        error
    } else {
        format!("{error}: {stderr}")
    }
}

fn validate_export_range(
    range: Option<ExportFrameRange>,
    frame_count: u64,
) -> Result<(u64, u64), String> {
    let range = range.unwrap_or(ExportFrameRange {
        start_frame: 0,
        end_frame_exclusive: frame_count,
    });
    if range.start_frame >= range.end_frame_exclusive {
        return Err("export frame range must not be empty".to_string());
    }
    if range.end_frame_exclusive > frame_count {
        return Err(format!(
            "export frame range end {} exceeds frame count {frame_count}",
            range.end_frame_exclusive
        ));
    }
    Ok((range.start_frame, range.end_frame_exclusive))
}

fn normalize_active_keys(keys: &[String]) -> Vec<String> {
    let mut keys = keys.to_vec();
    keys.sort();
    keys.dedup();
    keys
}

fn marker_is_active(frame: u64, ranges: &[(u64, u64)]) -> bool {
    ranges
        .iter()
        .any(|(start, end)| frame >= *start && frame <= *end)
}

fn resolve_render_thread_count(configured_threads: Option<i32>, available_threads: usize) -> usize {
    let available_threads = available_threads.max(1);
    let max_render_threads = available_threads.saturating_mul(4).max(1);

    match configured_threads {
        None | Some(0) => available_threads,
        Some(-1) => max_render_threads,
        Some(value) if value > 0 => (value as usize).min(max_render_threads).max(1),
        Some(_) => max_render_threads,
    }
}

fn render_profile(
    pixmap: &mut Pixmap,
    profile: &ExportOverlayProfile,
    active_keys: &HashSet<String>,
    marker_active: bool,
    font: Option<&fontdue::Font>,
) -> Result<(), String> {
    let unit = profile.layout.unit_px * profile.style.scale;
    let gap = unit * normalize_unit(profile.layout.gap_unit);
    let padding = if is_backplate_visible(&profile.style.background_color) {
        10.0
    } else {
        0.0
    };
    let bleed = 12.0;
    let origin_x = bleed + padding;
    let origin_y = bleed + padding;
    let cluster_width = pixmap.width() as f32 - OVERLAY_BLEED;
    let cluster_height = pixmap.height() as f32 - OVERLAY_BLEED;

    if is_backplate_visible(&profile.style.background_color) {
        draw_rect(
            pixmap,
            bleed,
            bleed,
            cluster_width,
            cluster_height,
            &profile.style.background_color,
            backplate_opacity(profile),
            profile.style.background_radius,
        )?;
    }

    for (row_index, row) in profile.rows.iter().enumerate() {
        let mut x = origin_x;
        let y = origin_y + row_index as f32 * (unit + gap);

        for (item_index, item) in row.iter().enumerate() {
            if item_index > 0 && is_key(item) && is_key(&row[item_index - 1]) {
                x += gap;
            }

            let width = item.width_unit() * unit;
            if let ExportOverlayItem::Key { id, label, .. } = item {
                let active = active_keys.contains(id);
                if active || profile.style.idle_key_visibility != "hidden" {
                    let key_y = y + if active {
                        2.0 * profile.style.scale
                    } else {
                        0.0
                    };
                    let color = if active {
                        &profile.style.active_color
                    } else {
                        &profile.style.idle_color
                    };
                    let text_color = if active {
                        &profile.style.active_text_color
                    } else {
                        &profile.style.idle_text_color
                    };

                    draw_key(
                        pixmap,
                        x,
                        key_y,
                        width,
                        unit,
                        color,
                        active,
                        profile.style.opacity,
                    )?;
                    draw_key_label(
                        pixmap,
                        label,
                        x,
                        key_y,
                        width,
                        unit,
                        text_color,
                        profile.style.opacity,
                        profile.style.scale,
                        font,
                    )?;
                }
            }

            x += width;
        }
    }

    if marker_active && profile.export.render_markers {
        draw_marker_border(
            pixmap,
            bleed,
            bleed,
            cluster_width,
            cluster_height,
            &profile.style.active_color,
            profile.style.opacity,
            profile.style.background_radius,
        )?;
    }

    Ok(())
}

fn marker_frame_ranges(
    marker_frames: impl IntoIterator<Item = u64>,
    fps: u16,
    duration_ms: u64,
) -> Vec<(u64, u64)> {
    let duration_frames = (duration_ms
        .saturating_mul(u64::from(fps))
        .saturating_add(999)
        / 1000)
        .max(1);

    marker_frames
        .into_iter()
        .map(|frame| {
            (
                frame,
                frame.saturating_add(duration_frames.saturating_sub(1)),
            )
        })
        .collect()
}

fn draw_rect(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: &str,
    opacity: f32,
    radius: f32,
) -> Result<(), String> {
    let rect = Rect::from_xywh(x, y, width.max(0.0), height.max(0.0))
        .ok_or_else(|| "invalid export rectangle".to_string())?;
    let mut paint = Paint::default();
    paint.set_color(parse_color(color, opacity)?);
    let path = rounded_rect_path(rect, radius.max(0.0));
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
    Ok(())
}

fn draw_blurred_rect(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: &str,
    opacity: f32,
    radius: f32,
    blur_radius: f32,
) -> Result<(), String> {
    let blur_px = blur_radius.ceil() as u32;
    if blur_px == 0 {
        return draw_rect(pixmap, x, y, width, height, color, opacity, radius);
    }

    let spread = blur_px * 3 + 4;
    let pad = spread as f32 / 2.0;
    let buf_w = (width + spread as f32).ceil() as u32;
    let buf_h = (height + spread as f32).ceil() as u32;

    let mut temp = Pixmap::new(buf_w.max(1), buf_h.max(1))
        .ok_or_else(|| "failed to allocate blur buffer".to_string())?;

    draw_rect(&mut temp, pad, pad, width, height, color, opacity, radius)?;
    box_blur_alpha(temp.data_mut(), buf_w as usize, buf_h as usize, blur_px, 3);

    let dst_x = (x - pad).round() as i32;
    let dst_y = (y - pad).round() as i32;
    let main_w = pixmap.width() as i32;
    let main_h = pixmap.height() as i32;
    let temp_w = buf_w as i32;
    let temp_h = buf_h as i32;
    let temp_data = temp.take();
    let main_data = pixmap.data_mut();

    for sy in 0..temp_h {
        let ty = dst_y + sy;
        if ty < 0 || ty >= main_h {
            continue;
        }
        for sx in 0..temp_w {
            let tx = dst_x + sx;
            if tx < 0 || tx >= main_w {
                continue;
            }
            let ti = (sy * temp_w + sx) as usize * 4;
            let src_alpha = temp_data[ti + 3] as f32 / 255.0;
            if src_alpha <= 0.0 {
                continue;
            }
            let mi = (ty * main_w + tx) as usize * 4;
            alpha_blend_pixel(
                &mut main_data[mi..mi + 4],
                temp_data[ti],
                temp_data[ti + 1],
                temp_data[ti + 2],
                src_alpha,
            );
        }
    }

    Ok(())
}

fn box_blur_alpha(data: &mut [u8], width: usize, height: usize, radius: u32, passes: u32) {
    if radius == 0 || width < 2 || height < 2 {
        return;
    }
    let mut temp = vec![0u8; width * height * 4];
    for _ in 0..passes {
        box_blur_alpha_horizontal(data, &mut temp, width, height, radius);
        box_blur_alpha_vertical(data, &mut temp, width, height, radius);
    }
}

fn box_blur_alpha_horizontal(
    data: &mut [u8],
    temp: &mut [u8],
    width: usize,
    height: usize,
    radius: u32,
) {
    let r = radius as usize;
    let div = (r * 2 + 1) as f32;
    for y in 0..height {
        let src_row = y * width * 4;
        let dst_row = y * width * 4;
        let mut acc = [0u32; 4];
        for x in 0..r.min(width) {
            let i = src_row + x * 4;
            acc[0] += data[i] as u32;
            acc[1] += data[i + 1] as u32;
            acc[2] += data[i + 2] as u32;
            acc[3] += data[i + 3] as u32;
        }
        for x in 0..width {
            if x + r < width {
                let i = src_row + (x + r) * 4;
                acc[0] += data[i] as u32;
                acc[1] += data[i + 1] as u32;
                acc[2] += data[i + 2] as u32;
                acc[3] += data[i + 3] as u32;
            }
            let o = dst_row + x * 4;
            temp[o] = (acc[0] as f32 / div).round().min(255.0) as u8;
            temp[o + 1] = (acc[1] as f32 / div).round().min(255.0) as u8;
            temp[o + 2] = (acc[2] as f32 / div).round().min(255.0) as u8;
            temp[o + 3] = (acc[3] as f32 / div).round().min(255.0) as u8;
            if x >= r {
                let i = src_row + (x - r) * 4;
                acc[0] = acc[0].saturating_sub(data[i] as u32);
                acc[1] = acc[1].saturating_sub(data[i + 1] as u32);
                acc[2] = acc[2].saturating_sub(data[i + 2] as u32);
                acc[3] = acc[3].saturating_sub(data[i + 3] as u32);
            }
        }
    }
    data.copy_from_slice(temp);
}

fn box_blur_alpha_vertical(
    data: &mut [u8],
    temp: &mut [u8],
    width: usize,
    height: usize,
    radius: u32,
) {
    let r = radius as usize;
    let div = (r * 2 + 1) as f32;
    for x in 0..width {
        let mut acc = [0u32; 4];
        for y in 0..r.min(height) {
            let i = (y * width + x) * 4;
            acc[0] += data[i] as u32;
            acc[1] += data[i + 1] as u32;
            acc[2] += data[i + 2] as u32;
            acc[3] += data[i + 3] as u32;
        }
        for y in 0..height {
            if y + r < height {
                let i = ((y + r) * width + x) * 4;
                acc[0] += data[i] as u32;
                acc[1] += data[i + 1] as u32;
                acc[2] += data[i + 2] as u32;
                acc[3] += data[i + 3] as u32;
            }
            let o = (y * width + x) * 4;
            temp[o] = (acc[0] as f32 / div).round().min(255.0) as u8;
            temp[o + 1] = (acc[1] as f32 / div).round().min(255.0) as u8;
            temp[o + 2] = (acc[2] as f32 / div).round().min(255.0) as u8;
            temp[o + 3] = (acc[3] as f32 / div).round().min(255.0) as u8;
            if y >= r {
                let i = ((y - r) * width + x) * 4;
                acc[0] = acc[0].saturating_sub(data[i] as u32);
                acc[1] = acc[1].saturating_sub(data[i + 1] as u32);
                acc[2] = acc[2].saturating_sub(data[i + 2] as u32);
                acc[3] = acc[3].saturating_sub(data[i + 3] as u32);
            }
        }
    }
    data.copy_from_slice(temp);
}

fn draw_rounded_stroke(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: &str,
    opacity: f32,
    radius: f32,
    stroke_width: f32,
) -> Result<(), String> {
    let half = stroke_width / 2.0;
    let rect = Rect::from_xywh(
        x + half,
        y + half,
        (width - stroke_width).max(0.0),
        (height - stroke_width).max(0.0),
    )
    .ok_or_else(|| "invalid stroke rectangle".to_string())?;
    let mut paint = Paint::default();
    paint.set_color(parse_color(color, opacity)?);
    let mut stroke = Stroke::default();
    stroke.width = stroke_width;
    stroke.line_cap = LineCap::Butt;
    stroke.line_join = LineJoin::Round;
    pixmap.stroke_path(
        &rounded_rect_path(rect, (radius - half).max(0.0)),
        &paint,
        &stroke,
        Transform::identity(),
        None,
    );
    Ok(())
}

fn draw_key(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: &str,
    active: bool,
    opacity: f32,
) -> Result<(), String> {
    if active {
        draw_blurred_rect(
            pixmap,
            x - 4.0,
            y - 4.0,
            width + 8.0,
            height + 8.0,
            color,
            opacity * 0.18,
            10.0,
            9.0,
        )?;
    }
    draw_blurred_rect(
        pixmap,
        x,
        y + 6.0,
        width,
        height,
        "#000000",
        opacity * 0.24,
        8.0,
        9.0,
    )?;
    draw_rect(pixmap, x, y, width, height, color, opacity, 8.0)?;
    draw_rect(
        pixmap,
        x + 1.0,
        y + height - if active { 1.0 } else { 3.0 },
        (width - 2.0).max(0.0),
        if active { 1.0 } else { 3.0 },
        "#000000",
        opacity * if active { 0.28 } else { 0.35 },
        2.0,
    )?;

    let rect = Rect::from_xywh(
        x + 0.5,
        y + 0.5,
        (width - 1.0).max(0.0),
        (height - 1.0).max(0.0),
    )
    .ok_or_else(|| "invalid key border rectangle".to_string())?;
    let mut paint = Paint::default();
    let (r, g, b, a) = if active {
        KEY_ACTIVE_BORDER_RGBA
    } else {
        KEY_BORDER_RGBA
    };
    paint.set_color(Color::from_rgba8(r, g, b, scaled_alpha(a, opacity)));
    let mut stroke = Stroke::default();
    stroke.width = 1.0;
    stroke.line_cap = LineCap::Butt;
    stroke.line_join = LineJoin::Round;
    pixmap.stroke_path(
        &rounded_rect_path(rect, 8.0),
        &paint,
        &stroke,
        Transform::identity(),
        None,
    );

    Ok(())
}

fn draw_marker_border(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: &str,
    opacity: f32,
    radius: f32,
) -> Result<(), String> {
    let glow_color = brighten_color(color, 16.0);
    draw_blurred_rect(
        pixmap,
        x - 4.0,
        y - 4.0,
        width + 8.0,
        height + 8.0,
        &glow_color,
        opacity * 0.18,
        radius,
        11.0,
    )?;
    draw_rounded_stroke(
        pixmap,
        x,
        y,
        width,
        height,
        &glow_color,
        opacity,
        radius,
        2.0,
    )
}

fn rounded_rect_path(rect: Rect, radius: f32) -> tiny_skia::Path {
    let radius = radius.min(rect.width() / 2.0).min(rect.height() / 2.0);
    if radius <= 0.0 {
        return PathBuilder::from_rect(rect);
    }

    let x0 = rect.left();
    let y0 = rect.top();
    let x1 = rect.right();
    let y1 = rect.bottom();
    let c = radius * 0.55228475;
    let mut path = PathBuilder::new();
    path.move_to(x0 + radius, y0);
    path.line_to(x1 - radius, y0);
    path.cubic_to(x1 - radius + c, y0, x1, y0 + radius - c, x1, y0 + radius);
    path.line_to(x1, y1 - radius);
    path.cubic_to(x1, y1 - radius + c, x1 - radius + c, y1, x1 - radius, y1);
    path.line_to(x0 + radius, y1);
    path.cubic_to(x0 + radius - c, y1, x0, y1 - radius + c, x0, y1 - radius);
    path.line_to(x0, y0 + radius);
    path.cubic_to(x0, y0 + radius - c, x0 + radius - c, y0, x0 + radius, y0);
    path.close();
    path.finish()
        .unwrap_or_else(|| PathBuilder::from_rect(rect))
}

fn draw_key_label(
    pixmap: &mut Pixmap,
    label: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: &str,
    opacity: f32,
    scale: f32,
    font: Option<&fontdue::Font>,
) -> Result<(), String> {
    let Some(font) = font else {
        return Ok(());
    };
    let text = label.trim();
    if text.is_empty() {
        return Ok(());
    }

    let font_size = TEXT_FONT_SIZE_FACTOR * scale;
    let text_color = parse_color_rgba8(color, opacity)?;
    let glyphs = text
        .chars()
        .map(|character| {
            let (metrics, bitmap) = font.rasterize(character, font_size);
            (character, metrics, bitmap)
        })
        .collect::<Vec<_>>();
    let advance_width = glyphs
        .iter()
        .map(|(_, metrics, _)| metrics.advance_width)
        .sum::<f32>();
    let text_height = glyphs
        .iter()
        .map(|(_, metrics, _)| metrics.height as f32)
        .fold(0.0_f32, f32::max);
    let baseline_y = y + (height + text_height) / 2.0 - 1.0 * scale;
    let mut cursor_x = x + (width - advance_width).max(0.0) / 2.0;

    for (_, metrics, bitmap) in glyphs {
        let glyph_x = (cursor_x + metrics.xmin as f32).round() as i32;
        let glyph_y = (baseline_y - metrics.height as f32 - metrics.ymin as f32).round() as i32;
        blend_glyph_bitmap(
            pixmap,
            glyph_x,
            glyph_y,
            metrics.width,
            metrics.height,
            &bitmap,
            text_color,
        );
        cursor_x += metrics.advance_width;
    }

    Ok(())
}

fn load_font(custom_path: Option<&str>) -> Option<fontdue::Font> {
    if let Some(path) = custom_path {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(font) = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()) {
                return Some(font);
            }
        }
    }

    for path in DEFAULT_FONT_PATHS {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        if let Ok(font) = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()) {
            return Some(font);
        }
    }

    None
}

const DEFAULT_FONT_PATHS: &[&str] = &[
    // macOS: SF Pro
    "/Library/Fonts/SF-Pro-Text-Bold.otf",
    "/Library/Fonts/SF-Pro-Display-Bold.otf",
    "/System/Library/Fonts/SFNS.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    // Windows: Segoe UI
    "C:\\Windows\\Fonts\\segoeuib.ttf",
    "C:\\Windows\\Fonts\\segoeui.ttf",
];

fn blend_glyph_bitmap(
    pixmap: &mut Pixmap,
    x: i32,
    y: i32,
    width: usize,
    height: usize,
    bitmap: &[u8],
    color: (u8, u8, u8, u8),
) {
    let pixmap_width = pixmap.width() as i32;
    let pixmap_height = pixmap.height() as i32;
    let data = pixmap.data_mut();

    for glyph_y in 0..height {
        let target_y = y + glyph_y as i32;
        if target_y < 0 || target_y >= pixmap_height {
            continue;
        }

        for glyph_x in 0..width {
            let target_x = x + glyph_x as i32;
            if target_x < 0 || target_x >= pixmap_width {
                continue;
            }

            let coverage = bitmap[glyph_y * width + glyph_x] as f32 / 255.0;
            if coverage <= 0.0 {
                continue;
            }

            let src_alpha = (color.3 as f32 / 255.0) * coverage;
            let offset =
                ((target_y as usize * pixmap_width as usize + target_x as usize) * 4) as usize;
            alpha_blend_pixel(
                &mut data[offset..offset + 4],
                color.0,
                color.1,
                color.2,
                src_alpha,
            );
        }
    }
}

fn alpha_blend_pixel(pixel: &mut [u8], r: u8, g: u8, b: u8, src_alpha: f32) {
    let dst_alpha = pixel[3] as f32 / 255.0;
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);
    let blend_premultiplied_channel = |src: u8, dst: u8| -> u8 {
        let src_premultiplied = src as f32 * src_alpha;
        let dst_premultiplied = dst as f32;
        (src_premultiplied + dst_premultiplied * (1.0 - src_alpha))
            .round()
            .clamp(0.0, 255.0) as u8
    };

    pixel[0] = blend_premultiplied_channel(r, pixel[0]);
    pixel[1] = blend_premultiplied_channel(g, pixel[1]);
    pixel[2] = blend_premultiplied_channel(b, pixel[2]);
    pixel[3] = (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
}

fn premultiplied_to_straight_rgba(mut rgba: Vec<u8>) -> Vec<u8> {
    for pixel in rgba.chunks_exact_mut(4) {
        let alpha = pixel[3];
        if alpha == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
            continue;
        }

        let alpha_f = alpha as f32 / 255.0;
        pixel[0] = (pixel[0] as f32 / alpha_f).round().clamp(0.0, 255.0) as u8;
        pixel[1] = (pixel[1] as f32 / alpha_f).round().clamp(0.0, 255.0) as u8;
        pixel[2] = (pixel[2] as f32 / alpha_f).round().clamp(0.0, 255.0) as u8;
    }

    rgba
}

fn parse_color(value: &str, opacity: f32) -> Result<Color, String> {
    let (r, g, b, a) = parse_color_rgba8(value, opacity)?;
    Ok(Color::from_rgba8(r, g, b, a))
}

fn parse_color_rgba8(value: &str, opacity: f32) -> Result<(u8, u8, u8, u8), String> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_hex_byte(&hex[0..2])?,
            parse_hex_byte(&hex[2..4])?,
            parse_hex_byte(&hex[4..6])?,
            255,
        ),
        8 => (
            parse_hex_byte(&hex[0..2])?,
            parse_hex_byte(&hex[2..4])?,
            parse_hex_byte(&hex[4..6])?,
            parse_hex_byte(&hex[6..8])?,
        ),
        _ => return Err(format!("invalid color: {value}")),
    };

    let alpha = (a as f32 / 255.0) * opacity.clamp(0.0, 1.0);
    Ok((r, g, b, (alpha * 255.0).round() as u8))
}

fn parse_hex_byte(value: &str) -> Result<u8, String> {
    u8::from_str_radix(value, 16).map_err(|error| error.to_string())
}

fn scaled_alpha(alpha: u8, opacity: f32) -> u8 {
    ((alpha as f32 * opacity.clamp(0.0, 1.0)).round()).clamp(0.0, 255.0) as u8
}

fn brighten_color(hex: &str, percent: f32) -> String {
    let stripped = hex.strip_prefix('#').unwrap_or(hex);
    if stripped.len() < 6 {
        return hex.to_string();
    }
    let r = u8::from_str_radix(&stripped[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&stripped[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&stripped[4..6], 16).unwrap_or(0);
    let mix = percent / 100.0;
    let nr = ((r as f32 * (1.0 - mix) + 255.0 * mix).round().min(255.0)) as u8;
    let ng = ((g as f32 * (1.0 - mix) + 255.0 * mix).round().min(255.0)) as u8;
    let nb = ((b as f32 * (1.0 - mix) + 255.0 * mix).round().min(255.0)) as u8;
    format!("#{nr:02x}{ng:02x}{nb:02x}")
}

fn backplate_opacity(profile: &ExportOverlayProfile) -> f32 {
    if is_hex_alpha_color(&profile.style.background_color) {
        1.0
    } else {
        profile.style.background_opacity
    }
}

fn is_backplate_visible(background_color: &str) -> bool {
    !is_hex_alpha_color(background_color) || !background_color.ends_with("00")
}

fn is_hex_alpha_color(value: &str) -> bool {
    let hex = value.strip_prefix('#').unwrap_or(value);
    hex.len() == 8 && hex.chars().all(|char| char.is_ascii_hexdigit())
}

fn ceil_stable(value: f32) -> u32 {
    (value - FLOAT_EPSILON).ceil() as u32
}

fn row_width_units(row: &[ExportOverlayItem], gap_unit: f32) -> f32 {
    row.iter().enumerate().fold(0.0, |sum, (index, item)| {
        let default_gap = if index > 0 && is_key(item) && is_key(&row[index - 1]) {
            normalize_unit(gap_unit)
        } else {
            0.0
        };

        sum + normalize_unit(item.width_unit()) + default_gap
    })
}

fn is_key(item: &ExportOverlayItem) -> bool {
    matches!(item, ExportOverlayItem::Key { .. })
}

fn normalize_unit(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

impl ExportOverlayItem {
    fn width_unit(&self) -> f32 {
        match self {
            ExportOverlayItem::Key { width_unit, .. }
            | ExportOverlayItem::Gap { width_unit }
            | ExportOverlayItem::Void { width_unit } => normalize_unit(*width_unit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_webm_ffmpeg_args, catch_job_panic, collect_render_batch, drain_reader_limited,
        estimate_export_overlay_size, marker_frame_ranges, normalize_active_keys,
        render_overlay_frame, resolve_prefetch_capacity, resolve_render_thread_count,
        validate_export_range, ExportFrameRange, ExportOverlayItem, ExportOverlayLayout,
        ExportOverlayProfile, ExportOverlayStyle, ExportRecordingConfig, ExportVideoConfig,
        FrameCache, RenderedFrame, VisualStateKey, MAX_PREFETCH_FRAMES,
    };
    #[cfg(unix)]
    use super::{export_overlay_video, export_overlay_video_with_progress_range};
    #[cfg(unix)]
    use crate::recording::{encode_kbdrec, RecordingEvent, RecordingSnapshot};
    #[cfg(unix)]
    use std::io::Write;
    use std::{
        collections::HashSet,
        io::Cursor,
        sync::{mpsc, Arc},
        time::Duration,
    };

    #[test]
    fn estimates_export_overlay_size_like_frontend() {
        let profile = test_profile();

        assert_eq!(estimate_export_overlay_size(&profile).width, 154);
        assert_eq!(estimate_export_overlay_size(&profile).height, 74);
    }

    #[test]
    fn resolves_render_thread_count_modes() {
        assert_eq!(resolve_render_thread_count(None, 8), 8);
        assert_eq!(resolve_render_thread_count(None, 1), 1);
        assert_eq!(resolve_render_thread_count(Some(0), 8), 8);
        assert_eq!(resolve_render_thread_count(Some(-1), 8), 32);
        assert_eq!(resolve_render_thread_count(Some(-2), 8), 32);
        assert_eq!(resolve_render_thread_count(Some(12), 8), 12);
        assert_eq!(resolve_render_thread_count(Some(64), 8), 32);
        assert_eq!(resolve_render_thread_count(Some(-1), 64), 256);
        assert_eq!(resolve_render_thread_count(Some(512), 64), 256);
    }

    #[test]
    fn validates_half_open_export_ranges() {
        assert_eq!(
            validate_export_range(
                Some(ExportFrameRange {
                    start_frame: 4,
                    end_frame_exclusive: 9,
                }),
                10,
            )
            .unwrap(),
            (4, 9)
        );
        assert!(validate_export_range(
            Some(ExportFrameRange {
                start_frame: 4,
                end_frame_exclusive: 4,
            }),
            10,
        )
        .is_err());
        assert!(validate_export_range(None, 0).is_err());
        assert_eq!(
            normalize_active_keys(&["w".into(), "a".into(), "w".into()]),
            vec!["a".to_string(), "w".to_string()]
        );
    }

    #[test]
    fn catches_job_panics_and_missing_results_without_hanging() {
        let panic_result = catch_job_panic::<()>(7, || panic!("test panic"));
        assert!(panic_result.unwrap_err().contains("render job 7 panicked"));

        let (tx, rx) = mpsc::channel::<Result<RenderedFrame, String>>();
        drop(tx);
        let error = match collect_render_batch(&rx, 1, Duration::from_millis(1)) {
            Ok(_) => panic!("missing render result must fail"),
            Err(error) => error,
        };
        assert!(error.contains("disconnected"));
    }

    #[test]
    fn bounds_cache_and_prefetch_by_rgba_bytes() {
        let mut cache = FrameCache::new(7);
        let first_key = VisualStateKey {
            active_key_ids: vec!["a".into()],
            marker_active: false,
        };
        let second_key = VisualStateKey {
            active_key_ids: vec!["b".into()],
            marker_active: false,
        };
        cache.insert(first_key.clone(), Arc::new(vec![1; 4]));
        cache.insert(second_key.clone(), Arc::new(vec![2; 4]));
        assert!(cache.get(&first_key).is_none());
        assert!(cache.get(&second_key).is_some());
        assert!(cache.current_bytes <= cache.max_bytes);

        assert_eq!(resolve_prefetch_capacity(256, 1), MAX_PREFETCH_FRAMES);
        assert_eq!(resolve_prefetch_capacity(256, 128 * 1024 * 1024), 1);
    }

    #[test]
    fn drains_and_truncates_ffmpeg_stderr_while_preserving_the_tail() {
        let stderr = drain_reader_limited(Cursor::new(b"0123456789"), 4).unwrap();
        assert!(stderr.contains("truncated"));
        assert!(stderr.ends_with("6789"));
    }

    #[test]
    fn marker_ranges_saturate_at_the_maximum_frame() {
        assert_eq!(
            marker_frame_ranges([u64::MAX], 240, u64::MAX),
            vec![(u64::MAX, u64::MAX)]
        );
    }

    #[test]
    fn renders_active_key_pixels() {
        let profile = test_profile();
        let active_keys = HashSet::from(["w".to_string()]);

        let (size, rgba) = render_overlay_frame(&profile, &active_keys, false).unwrap();
        let active_pixel_offset = ((22 * size.width + 22) * 4) as usize;
        let idle_pixel_offset = ((22 * size.width + 82) * 4) as usize;

        assert_ne!(
            &rgba[active_pixel_offset..active_pixel_offset + 4],
            &[0, 0, 0, 0]
        );
        assert_ne!(
            &rgba[active_pixel_offset..active_pixel_offset + 4],
            &rgba[idle_pixel_offset..idle_pixel_offset + 4],
        );
    }

    #[test]
    fn renders_marker_border_with_active_key_color() {
        let mut profile = test_profile();
        profile.style.active_color = "#ff3366".to_string();

        let (size, rgba) = render_overlay_frame(&profile, &HashSet::new(), true).unwrap();
        // Brightened #ff3366 with 16% white: (255, 84, 126)
        // Sample a pixel on the flat top edge of the rounded stroke (x=50, y=13)
        let marker_pixel_offset = ((13 * size.width + 50) * 4) as usize;

        assert_eq!(
            &rgba[marker_pixel_offset..marker_pixel_offset + 4],
            &[255, 84, 126, 255],
        );
    }

    #[test]
    fn renders_backplate_with_configured_alpha_channel() {
        let mut profile = test_profile();
        profile.style.background_color = "#102030".to_string();
        profile.style.background_opacity = 0.5;
        profile.rows = vec![];

        let (size, rgba) = render_overlay_frame(&profile, &HashSet::new(), false).unwrap();
        let backplate_pixel_offset = ((30 * size.width + 80) * 4) as usize;

        assert_eq!(
            &rgba[backplate_pixel_offset..backplate_pixel_offset + 4],
            &[16, 32, 48, 128],
        );
    }

    #[test]
    fn renders_key_labels_when_system_font_is_available() {
        let profile = test_profile();
        let (size, rgba) = render_overlay_frame(&profile, &HashSet::new(), false).unwrap();
        let key_center_x = 12 + 25;
        let key_center_y = 12 + 25;
        let has_text_pixel = (key_center_y - 10..=key_center_y + 10).any(|y| {
            (key_center_x - 10..=key_center_x + 10).any(|x| {
                let offset = ((y * size.width + x) * 4) as usize;
                let pixel = &rgba[offset..offset + 4];
                pixel[3] > 0 && pixel != [18, 20, 23, 255]
            })
        });

        assert!(has_text_pixel);
    }

    #[test]
    fn deserializes_frontend_camel_case_overlay_items() {
        let profile = serde_json::from_value::<ExportOverlayProfile>(serde_json::json!({
            "layout": {
                "unitPx": 50.0,
                "gapUnit": 0.1
            },
            "rows": [[
                {
                    "type": "key",
                    "id": "w",
                    "label": "W",
                    "widthUnit": 1.0
                },
                {
                    "type": "gap",
                    "widthUnit": 0.5
                }
            ]],
            "style": {
                "scale": 1.0,
                "opacity": 1.0,
                "backgroundColor": "#00000000",
                "backgroundOpacity": 0.72,
                "backgroundRadius": 8.0,
                "idleKeyVisibility": "visible",
                "idleColor": "#121417",
                "activeColor": "#25d366",
                "idleTextColor": "#f5f7fa",
                "activeTextColor": "#ffffff"
            },
            "export": {
                "renderMarkers": true,
                "renderThreads": null
            },
            "recording": {
                "syncFeedbackDurationMs": 420
            }
        }))
        .unwrap();

        assert_eq!(profile.rows[0][0].width_unit(), 1.0);
        assert_eq!(profile.rows[0][1].width_unit(), 0.5);
    }

    #[test]
    fn builds_raw_rgba_webm_ffmpeg_arguments() {
        let args = build_webm_ffmpeg_args(
            "/tmp/out.webm",
            super::ExportOverlaySize {
                width: 320,
                height: 180,
            },
            60,
        );

        assert!(args.windows(2).any(|args| args == ["-pix_fmt", "rgba"]));
        assert!(args.windows(2).any(|args| args == ["-s", "320x180"]));
        assert!(args.windows(2).any(|args| args == ["-c:v", "libvpx-vp9"]));
        assert_eq!(args.last().unwrap(), "/tmp/out.webm");
    }

    #[test]
    #[cfg(unix)]
    fn exports_kbdrec_frames_to_ffmpeg_stdin() {
        let root = std::env::temp_dir().join(format!(
            "keyboard-display-video-export-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let recording_path = root.join("input.kbdrec");
        let output_path = root.join("out.webm");
        let raw_video_path = root.join("raw.rgba");
        let fake_ffmpeg_path = root.join("fake-ffmpeg.sh");
        let profile = test_profile();
        let frame_size = estimate_export_overlay_size(&profile);
        let recording = encode_kbdrec(&RecordingSnapshot {
            version: 1,
            fps: 60,
            timebase: "monotonic",
            events: vec![
                RecordingEvent::KeyDown {
                    frame: 0,
                    key_id: "w".to_string(),
                },
                RecordingEvent::Marker {
                    frame: 1,
                    name: "sync".to_string(),
                },
            ],
        })
        .unwrap();
        std::fs::write(&recording_path, recording).unwrap();
        write_fake_ffmpeg(&fake_ffmpeg_path, &raw_video_path);

        let result =
            export_overlay_video(&recording_path, &output_path, &fake_ffmpeg_path, &profile)
                .unwrap();

        assert_eq!(result.frame_count, 2);
        assert_eq!(result.width, frame_size.width);
        assert_eq!(result.height, frame_size.height);
        assert_eq!(
            std::fs::metadata(&raw_video_path).unwrap().len(),
            u64::from(frame_size.width) * u64::from(frame_size.height) * 4 * 2,
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[cfg(unix)]
    fn exports_marker_border_for_configured_duration() {
        let root = std::env::temp_dir().join(format!(
            "keyboard-display-marker-duration-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let recording_path = root.join("input.kbdrec");
        let output_path = root.join("out.webm");
        let raw_video_path = root.join("raw.rgba");
        let fake_ffmpeg_path = root.join("fake-ffmpeg.sh");
        let mut profile = test_profile();
        profile.recording.sync_feedback_duration_ms = 500;
        let frame_size = estimate_export_overlay_size(&profile);
        let bytes_per_frame =
            (u64::from(frame_size.width) * u64::from(frame_size.height) * 4) as usize;
        let marker_pixel_offset = ((13 * frame_size.width + 50) * 4) as usize;
        let recording = encode_kbdrec(&RecordingSnapshot {
            version: 1,
            fps: 10,
            timebase: "monotonic",
            events: vec![
                RecordingEvent::Marker {
                    frame: 1,
                    name: "sync".to_string(),
                },
                RecordingEvent::KeyDown {
                    frame: 6,
                    key_id: "unused-key".to_string(),
                },
            ],
        })
        .unwrap();
        std::fs::write(&recording_path, recording).unwrap();
        write_fake_ffmpeg(&fake_ffmpeg_path, &raw_video_path);

        export_overlay_video(&recording_path, &output_path, &fake_ffmpeg_path, &profile).unwrap();

        let raw_video = std::fs::read(&raw_video_path).unwrap();
        // #25d366 brightened with 16% white = (72, 218, 126)
        assert_eq!(
            &raw_video
                [bytes_per_frame + marker_pixel_offset..bytes_per_frame + marker_pixel_offset + 4],
            &[72, 218, 126, 255],
        );
        assert_eq!(
            &raw_video[5 * bytes_per_frame + marker_pixel_offset
                ..5 * bytes_per_frame + marker_pixel_offset + 4],
            &[72, 218, 126, 255],
        );
        assert_ne!(
            &raw_video[6 * bytes_per_frame + marker_pixel_offset
                ..6 * bytes_per_frame + marker_pixel_offset + 4],
            &[72, 218, 126, 255],
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[cfg(unix)]
    fn reports_rendered_frame_progress_during_export() {
        let root = std::env::temp_dir().join(format!(
            "keyboard-display-export-progress-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let recording_path = root.join("input.kbdrec");
        let output_path = root.join("out.webm");
        let raw_video_path = root.join("raw.rgba");
        let fake_ffmpeg_path = root.join("fake-ffmpeg.sh");
        let profile = test_profile();
        let recording = encode_kbdrec(&RecordingSnapshot {
            version: 1,
            fps: 60,
            timebase: "monotonic",
            events: vec![RecordingEvent::KeyDown {
                frame: 2,
                key_id: "w".to_string(),
            }],
        })
        .unwrap();
        std::fs::write(&recording_path, recording).unwrap();
        write_fake_ffmpeg(&fake_ffmpeg_path, &raw_video_path);
        let mut progress_events = Vec::new();

        export_overlay_video_with_progress_range(
            &recording_path,
            &output_path,
            &fake_ffmpeg_path,
            &profile,
            Some(ExportFrameRange {
                start_frame: 1,
                end_frame_exclusive: 3,
            }),
            |progress| {
                progress_events.push(progress);
                Ok(())
            },
        )
        .unwrap();

        assert!(!progress_events.is_empty());
        let final_progress = progress_events.last().unwrap();
        assert_eq!(final_progress.rendered_frames, 2);
        assert_eq!(final_progress.total_frames, 2);
        assert_eq!(final_progress.current_frame, 2);
        assert_eq!(final_progress.active_key_ids, vec!["w".to_string()]);

        let _ = std::fs::remove_dir_all(root);
    }

    fn test_profile() -> ExportOverlayProfile {
        ExportOverlayProfile {
            layout: ExportOverlayLayout {
                unit_px: 50.0,
                gap_unit: 0.1,
            },
            rows: vec![vec![
                ExportOverlayItem::Key {
                    id: "w".to_string(),
                    label: "W".to_string(),
                    width_unit: 1.0,
                },
                ExportOverlayItem::Key {
                    id: "a".to_string(),
                    label: "A".to_string(),
                    width_unit: 1.0,
                },
                ExportOverlayItem::Gap { width_unit: 0.5 },
            ]],
            style: ExportOverlayStyle {
                scale: 1.0,
                opacity: 1.0,
                background_color: "#00000000".to_string(),
                background_opacity: 0.72,
                background_radius: 8.0,
                idle_key_visibility: "visible".to_string(),
                idle_color: "#121417".to_string(),
                active_color: "#25d366".to_string(),
                idle_text_color: "#f5f7fa".to_string(),
                active_text_color: "#ffffff".to_string(),
            },
            export: ExportVideoConfig {
                render_markers: true,
                font_path: None,
                render_threads: None,
            },
            recording: ExportRecordingConfig {
                sync_feedback_duration_ms: 420,
            },
        }
    }

    #[cfg(unix)]
    fn write_fake_ffmpeg(path: &std::path::Path, raw_video_path: &std::path::Path) {
        let script = format!(
            "#!/bin/sh\ncat > '{}'\nfor arg in \"$@\"; do output=\"$arg\"; done\ntouch \"$output\"\n",
            raw_video_path.display()
        );
        std::fs::File::create(path)
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(path, permissions).unwrap();
        }
    }
}
