#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::AtomicUsize,
    sync::mpsc,
    sync::Mutex,
};
use tiny_skia::{Color, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

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
    pub render_threads: Option<u16>,
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

const BACKPLATE_PADDING: f32 = 10.0 * 2.0;
const OVERLAY_BLEED: f32 = 12.0 * 2.0;
const FLOAT_EPSILON: f32 = 0.000001;
const KEY_BORDER_RGBA: (u8, u8, u8, u8) = (255, 255, 255, 41);
const KEY_ACTIVE_BORDER_RGBA: (u8, u8, u8, u8) = (255, 255, 255, 128);
const TEXT_FONT_SIZE_FACTOR: f32 = 15.0;

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

    render_profile(&mut pixmap, profile, active_keys, marker_active, font.as_ref())?;

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
    export_overlay_video_with_progress(recording_path, output_path, ffmpeg_path, profile, |_| Ok(()))
}

/// 从 `.kbdrec` 帧状态流渲染透明 WebM overlay，并在渲染过程中上报帧进度。
/// 支持多线程并行渲染，通过 `profile.export.render_threads` 配置线程数（`None` = 自动检测 CPU 核心数）。
pub fn export_overlay_video_with_progress(
    recording_path: &Path,
    output_path: &Path,
    ffmpeg_path: &Path,
    profile: &ExportOverlayProfile,
    mut on_progress: impl FnMut(ExportOverlayProgress) -> Result<(), String>,
) -> Result<ExportOverlayVideoResult, String> {
    let bytes = std::fs::read(recording_path).map_err(|error| error.to_string())?;
    let decoded = crate::recording::decode_kbdrec(&bytes)?;
    let size = estimate_export_overlay_size(profile);
    let total_frames = decoded.frames.len() as u64;
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

    let available_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let num_threads = profile
        .export
        .render_threads
        .map(|n| (n as usize).max(1).min(available_threads * 4))
        .unwrap_or(available_threads);

    use std::sync::Arc;

    let next_frame_index = Arc::new(AtomicUsize::new(0));
    let frames = Arc::new(decoded.frames);
    let marker_ranges = Arc::new(marker_ranges);
    let frame_count = decoded.frame_count;
    let fps = decoded.fps;
    // Channel: workers send rendered frames to collector
    let (render_tx, render_rx) = mpsc::channel::<(usize, u64, Vec<String>, Vec<u8>)>();
    // Channel: collector sends progress updates to main thread
    let (progress_tx, progress_rx) = mpsc::channel::<ExportOverlayProgress>();

    // Collector thread: receives frames from channel, writes to ffmpeg stdin in order,
    // and sends progress updates back to the main thread.
    let collector = std::thread::spawn(move || {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "failed to open ffmpeg stdin".to_string())?;
        let mut pending: BTreeMap<usize, (u64, Vec<String>, Vec<u8>)> = BTreeMap::new();
        let mut next_write_index = 0usize;
        let mut rendered_frames = 0u64;

        loop {
            match render_rx.recv() {
                Ok((index, frame_number, active_keys, rgba)) => {
                    pending.insert(index, (frame_number, active_keys, rgba));
                    // Write all consecutive frames that are ready
                    while let Some((frame_number, mut active_keys, rgba)) = pending.remove(&next_write_index) {
                        stdin.write_all(&rgba).map_err(|error| error.to_string())?;
                        rendered_frames += 1;
                        active_keys.sort();
                        let _ = progress_tx.send(ExportOverlayProgress {
                            rendered_frames,
                            total_frames,
                            current_frame: frame_number,
                            active_key_ids: active_keys,
                        });
                        next_write_index += 1;
                    }
                }
                Err(mpsc::RecvError) => break, // All senders closed, no more frames incoming
            }
        }

        // Drain any remaining out-of-order frames
        for (_index, (frame_number, mut active_keys, rgba)) in pending.into_iter() {
            stdin.write_all(&rgba).map_err(|error| error.to_string())?;
            rendered_frames += 1;
            active_keys.sort();
            let _ = progress_tx.send(ExportOverlayProgress {
                rendered_frames,
                total_frames,
                current_frame: frame_number,
                active_key_ids: active_keys,
            });
        }

        let output = child
            .wait_with_output()
            .map_err(|error| format!("failed to finish ffmpeg export: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ffmpeg export failed: {}", stderr.trim()));
        }
        drop(progress_tx);
        Ok(())
    });

    // Render workers on a dedicated thread so the main thread can process
    // progress events in real-time (std::thread::scope blocks the caller).
    let profile_owned = profile.clone();
    let render_handle = std::thread::spawn(move || {
        let font = Arc::new(Mutex::new(load_font(profile_owned.export.font_path.as_deref())));
        // 帧缓存：按键状态 → RGBA 数据，避免连续相同状态重复渲染
        let frame_cache: Arc<Mutex<HashMap<(Vec<String>, bool), Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut worker_handles = Vec::new();

        for _ in 0..num_threads {
            let render_tx = render_tx.clone();
            let next_frame_index = next_frame_index.clone();
            let frames = frames.clone();
            let marker_ranges = marker_ranges.clone();
            let profile_owned = profile_owned.clone();
            let font = font.clone();
            let frame_cache = frame_cache.clone();

            worker_handles.push(std::thread::spawn(move || {
                loop {
                    let index = next_frame_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if index >= frames.len() {
                        break;
                    }

                    let frame = &frames[index];
                    let marker_active = marker_ranges
                        .iter()
                        .any(|(start, end)| frame.frame >= *start && frame.frame <= *end);
                    let mut sorted_keys = frame.keys.clone();
                    sorted_keys.sort();
                    let cache_key = (sorted_keys, marker_active);

                    // 查缓存，命中则 clone 已有数据
                    {
                        let cache = frame_cache.lock().unwrap();
                        if let Some(rgba) = cache.get(&cache_key) {
                            let _ = render_tx.send((index, frame.frame, frame.keys.clone(), rgba.clone()));
                            continue;
                        }
                    }

                    // 缓存未命中，渲染并写入缓存
                    let active_keys_set: HashSet<String> = frame.keys.iter().cloned().collect();
                    let font_guard = font.lock().unwrap();
                    let (_, rgba) = render_overlay_frame_with_font(
                        &profile_owned,
                        &active_keys_set,
                        marker_active,
                        font_guard.as_ref(),
                    )
                    .expect("frame rendering failed");
                    drop(font_guard);

                    let mut cache = frame_cache.lock().unwrap();
                    cache.insert(cache_key, rgba.clone());

                    let _ = render_tx.send((index, frame.frame, frame.keys.clone(), rgba));
                }
            }));
        }

        for handle in worker_handles {
            handle.join().expect("render worker panicked");
        }
    });

    // Main thread: process progress events in real-time while rendering happens
    // on the render thread. The collector drops progress_tx when done, which
    // terminates this loop.
    while let Ok(progress) = progress_rx.recv() {
        on_progress(progress)?;
    }

    // Wait for the render thread to finish
    render_handle.join().map_err(|_| "render thread panicked".to_string())?;

    // Wait for the collector to finish and propagate its result
    collector.join().map_err(|error| format!("collector thread panicked: {error:?}"))??;

    Ok(ExportOverlayVideoResult {
        output_path: output_path_string,
        frame_count,
        width: size.width,
        height: size.height,
        fps,
    })
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
                    let key_y = y + if active { 2.0 * profile.style.scale } else { 0.0 };
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

                    draw_key(pixmap, x, key_y, width, unit, color, active, profile.style.opacity)?;
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
    let duration_frames = ((duration_ms * u64::from(fps)).saturating_add(999) / 1000).max(1);

    marker_frames
        .into_iter()
        .map(|frame| (frame, frame + duration_frames.saturating_sub(1)))
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
    pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
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
            alpha_blend_pixel(&mut main_data[mi..mi + 4], temp_data[ti], temp_data[ti + 1], temp_data[ti + 2], src_alpha);
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

fn box_blur_alpha_horizontal(data: &mut [u8], temp: &mut [u8], width: usize, height: usize, radius: u32) {
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

fn box_blur_alpha_vertical(data: &mut [u8], temp: &mut [u8], width: usize, height: usize, radius: u32) {
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

    let rect = Rect::from_xywh(x + 0.5, y + 0.5, (width - 1.0).max(0.0), (height - 1.0).max(0.0))
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
    draw_rounded_stroke(pixmap, x, y, width, height, &glow_color, opacity, radius, 2.0)
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
    path.finish().unwrap_or_else(|| PathBuilder::from_rect(rect))
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
            let offset = ((target_y as usize * pixmap_width as usize + target_x as usize) * 4) as usize;
            alpha_blend_pixel(&mut data[offset..offset + 4], color.0, color.1, color.2, src_alpha);
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
        build_webm_ffmpeg_args, estimate_export_overlay_size, export_overlay_video,
        export_overlay_video_with_progress, render_overlay_frame, ExportOverlayItem,
        ExportOverlayLayout, ExportOverlayProfile, ExportOverlayStyle,
        ExportRecordingConfig, ExportVideoConfig,
    };
    use crate::recording::{encode_kbdrec, RecordingEvent, RecordingSnapshot};
    use std::{collections::HashSet, io::Write};

    #[test]
    fn estimates_export_overlay_size_like_frontend() {
        let profile = test_profile();

        assert_eq!(estimate_export_overlay_size(&profile).width, 154);
        assert_eq!(estimate_export_overlay_size(&profile).height, 74);
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

        export_overlay_video_with_progress(
            &recording_path,
            &output_path,
            &fake_ffmpeg_path,
            &profile,
            |progress| {
                progress_events.push(progress);
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(progress_events.len(), 3);
        assert_eq!(progress_events[0].rendered_frames, 1);
        assert_eq!(progress_events[0].total_frames, 3);
        assert_eq!(progress_events[2].rendered_frames, 3);
        assert_eq!(progress_events[2].total_frames, 3);
        assert_eq!(progress_events[2].active_key_ids, vec!["w".to_string()]);

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
