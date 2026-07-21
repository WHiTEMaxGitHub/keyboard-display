use std::{
    sync::OnceLock,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

pub fn unix_now_ms() -> Result<u64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;

    Ok(duration.as_millis() as u64)
}

/// 进程内单调时钟，用于录制帧对齐，避免系统时间调整影响事件时间。
pub fn monotonic_now_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();

    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}
