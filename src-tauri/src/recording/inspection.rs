use super::{
    binary,
    types::{RecordingEvent, RecordingInspection},
};

/// 将 `.kbdrec` 二进制内容转换成 UI/调试更容易消费的结构。
pub fn inspect_kbdrec(bytes: &[u8]) -> Result<RecordingInspection, String> {
    let decoded = binary::decode_kbdrec(bytes)?;
    let events = decoded
        .markers
        .iter()
        .map(|marker| RecordingEvent::Marker {
            frame: marker.frame,
            name: marker.name.clone(),
        })
        .collect::<Vec<_>>();

    Ok(RecordingInspection {
        version: 1,
        fps: decoded.fps,
        key_ids: decoded.key_ids,
        frames: decoded.frames,
        events,
    })
}
