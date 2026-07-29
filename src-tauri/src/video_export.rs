#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOverlayProgress {
    pub rendered_frames: u64,
    pub total_frames: u64,
}

const BACKPLATE_PADDING: f32 = 10.0 * 2.0;
const OVERLAY_BLEED: f32 = 12.0 * 2.0;
const FLOAT_EPSILON: f32 = 0.000001;

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

/// 渲染单帧 RGBA overlay。第一版先画透明背景、backplate、按键矩形和 marker 边框。
pub fn render_overlay_frame(
    profile: &ExportOverlayProfile,
    active_keys: &HashSet<String>,
    marker_active: bool,
) -> Result<(ExportOverlaySize, Vec<u8>), String> {
    let size = estimate_export_overlay_size(profile);
    let mut pixmap = Pixmap::new(size.width, size.height)
        .ok_or_else(|| "failed to allocate export frame".to_string())?;

    render_profile(&mut pixmap, profile, active_keys, marker_active)?;

    Ok((size, pixmap.take()))
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

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "failed to open ffmpeg stdin".to_string())?;

        for (index, frame) in decoded.frames.iter().enumerate() {
            let active_keys = frame.keys.iter().cloned().collect::<HashSet<_>>();
            let marker_active = marker_ranges
                .iter()
                .any(|(start, end)| frame.frame >= *start && frame.frame <= *end);
            let (_, rgba) = render_overlay_frame(profile, &active_keys, marker_active)?;
            stdin.write_all(&rgba).map_err(|error| error.to_string())?;
            on_progress(ExportOverlayProgress {
                rendered_frames: index as u64 + 1,
                total_frames,
            })?;
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to finish ffmpeg export: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg export failed: {}", stderr.trim()));
    }

    Ok(ExportOverlayVideoResult {
        output_path: output_path_string,
        frame_count: decoded.frame_count,
        width: size.width,
        height: size.height,
        fps: decoded.fps,
    })
}

fn render_profile(
    pixmap: &mut Pixmap,
    profile: &ExportOverlayProfile,
    active_keys: &HashSet<String>,
    marker_active: bool,
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
            if let ExportOverlayItem::Key { id, .. } = item {
                let active = active_keys.contains(id);
                if active || profile.style.idle_key_visibility != "hidden" {
                    let color = if active {
                        &profile.style.active_color
                    } else {
                        &profile.style.idle_color
                    };
                    draw_rect(pixmap, x, y, width, unit, color, profile.style.opacity)?;
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
) -> Result<(), String> {
    let rect = Rect::from_xywh(x, y, width.max(0.0), height.max(0.0))
        .ok_or_else(|| "invalid export rectangle".to_string())?;
    let mut paint = Paint::default();
    paint.set_color(parse_color(color, opacity)?);
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
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
) -> Result<(), String> {
    let thickness = 2.0;
    draw_rect(pixmap, x, y, width, thickness, color, opacity)?;
    draw_rect(
        pixmap,
        x,
        y + height - thickness,
        width,
        thickness,
        color,
        opacity,
    )?;
    draw_rect(pixmap, x, y, thickness, height, color, opacity)?;
    draw_rect(
        pixmap,
        x + width - thickness,
        y,
        thickness,
        height,
        color,
        opacity,
    )
}

fn parse_color(value: &str, opacity: f32) -> Result<Color, String> {
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
    Ok(Color::from_rgba8(r, g, b, (alpha * 255.0).round() as u8))
}

fn parse_hex_byte(value: &str) -> Result<u8, String> {
    u8::from_str_radix(value, 16).map_err(|error| error.to_string())
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
        ExportOverlayLayout, ExportOverlayProfile, ExportOverlayProgress, ExportOverlayStyle,
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
        let marker_pixel_offset = ((12 * size.width + 12) * 4) as usize;

        assert_eq!(
            &rgba[marker_pixel_offset..marker_pixel_offset + 4],
            &[255, 51, 102, 255],
        );
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
                "renderMarkers": true
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
        let marker_pixel_offset = ((12 * frame_size.width + 12) * 4) as usize;
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
        assert_eq!(
            &raw_video
                [bytes_per_frame + marker_pixel_offset..bytes_per_frame + marker_pixel_offset + 4],
            &[37, 211, 102, 255],
        );
        assert_eq!(
            &raw_video[5 * bytes_per_frame + marker_pixel_offset
                ..5 * bytes_per_frame + marker_pixel_offset + 4],
            &[37, 211, 102, 255],
        );
        assert_ne!(
            &raw_video[6 * bytes_per_frame + marker_pixel_offset
                ..6 * bytes_per_frame + marker_pixel_offset + 4],
            &[37, 211, 102, 255],
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

        assert_eq!(
            progress_events,
            vec![
                ExportOverlayProgress {
                    rendered_frames: 1,
                    total_frames: 3,
                },
                ExportOverlayProgress {
                    rendered_frames: 2,
                    total_frames: 3,
                },
                ExportOverlayProgress {
                    rendered_frames: 3,
                    total_frames: 3,
                },
            ],
        );

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
