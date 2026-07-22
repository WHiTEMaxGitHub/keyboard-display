#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOverlayProfile {
    pub layout: ExportOverlayLayout,
    pub rows: Vec<Vec<ExportOverlayItem>>,
    pub style: ExportOverlayStyle,
    pub export: ExportVideoConfig,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOverlayLayout {
    pub unit_px: f32,
    pub gap_unit: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOverlaySize {
    pub width: u32,
    pub height: u32,
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
        draw_marker_border(pixmap, bleed, bleed, cluster_width, cluster_height)?;
    }

    Ok(())
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
) -> Result<(), String> {
    let thickness = 2.0;
    draw_rect(pixmap, x, y, width, thickness, "#25d366", 0.9)?;
    draw_rect(
        pixmap,
        x,
        y + height - thickness,
        width,
        thickness,
        "#25d366",
        0.9,
    )?;
    draw_rect(pixmap, x, y, thickness, height, "#25d366", 0.9)?;
    draw_rect(
        pixmap,
        x + width - thickness,
        y,
        thickness,
        height,
        "#25d366",
        0.9,
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
        build_webm_ffmpeg_args, estimate_export_overlay_size, render_overlay_frame,
        ExportOverlayItem, ExportOverlayLayout, ExportOverlayProfile, ExportOverlayStyle,
        ExportVideoConfig,
    };
    use std::collections::HashSet;

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
        }
    }
}
