use chrono::{Local, TimeZone};

/// 展开录制文件名模板，并保证最终产物是可写入磁盘的 `.kbdrec` 文件名。
pub fn format_recording_file_name(
    filename_template: &str,
    start_unix_ms: u64,
    end_unix_ms: u64,
    profile_name: &str,
    fps: u16,
) -> String {
    let template = if filename_template.trim().is_empty() {
        "${start}-${end}"
    } else {
        filename_template.trim()
    };
    let expanded = template
        .replace("${start}", &start_unix_ms.to_string())
        .replace("${end}", &end_unix_ms.to_string())
        .replace("${startDate}", &format_local_date(start_unix_ms))
        .replace("${startTime}", &format_local_time(start_unix_ms))
        .replace("${endTime}", &format_local_time(end_unix_ms))
        .replace(
            "${duration}",
            &format_duration(end_unix_ms.saturating_sub(start_unix_ms)),
        )
        .replace("${profileName}", profile_name)
        .replace("${profileSlug}", &slugify(profile_name))
        .replace("${fps}", &fps.to_string());
    let file_name = sanitize_file_name(&expanded);

    if file_name.to_ascii_lowercase().ends_with(".kbdrec") {
        file_name
    } else {
        format!("{file_name}.kbdrec")
    }
}

fn format_local_date(unix_ms: u64) -> String {
    format_local_datetime(unix_ms, "%Y-%m-%d")
}

fn format_local_time(unix_ms: u64) -> String {
    format_local_datetime(unix_ms, "%H-%M-%S")
}

fn format_local_datetime(unix_ms: u64, format: &str) -> String {
    Local
        .timestamp_millis_opt(unix_ms as i64)
        .single()
        .unwrap_or_else(|| {
            Local
                .timestamp_millis_opt(0)
                .single()
                .expect("unix epoch must be representable in local time")
        })
        .format(format)
        .to_string()
}

fn format_duration(duration_ms: u64) -> String {
    let total_seconds = duration_ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{hours:02}-{minutes:02}-{seconds:02}")
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for character in value.trim().to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "profile".to_string()
    } else {
        slug
    }
}

fn sanitize_file_name(file_name: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_dash = false;

    for character in file_name.chars() {
        let next = if character == '/' || character == '\\' || character.is_control() {
            '-'
        } else {
            character
        };

        if next == '-' {
            if previous_dash {
                continue;
            }
            previous_dash = true;
        } else {
            previous_dash = false;
        }

        sanitized.push(next);
    }

    let sanitized = sanitized.trim().to_string();
    if sanitized.is_empty() {
        "recording".to_string()
    } else {
        sanitized
    }
}

pub fn parse_recording_file_times(file_name: &str) -> (Option<u64>, Option<u64>) {
    let Some(stem) = file_name.strip_suffix(".kbdrec") else {
        return (None, None);
    };
    let Some((start, end)) = stem.split_once('-') else {
        return (None, None);
    };

    (start.parse().ok(), end.parse().ok())
}
