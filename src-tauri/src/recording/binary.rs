use super::{
    session::{event_frame, sample_frames},
    types::{RecordingEvent, RecordingFrame, RecordingSnapshot},
};
use serde::Serialize;

const MAGIC: &[u8; 4] = b"KBDR";
const VERSION: u8 = 1;
const FLAGS: u8 = 0;
const MAX_KEY_COUNT: u64 = 65_536;
const MAX_RUN_COUNT: u64 = 1_000_000;
const MAX_MARKER_COUNT: u64 = 1_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedKbdrec {
    pub fps: u16,
    pub key_ids: Vec<String>,
    pub frame_count: u64,
    pub runs: Vec<DecodedFrameRun>,
    pub frames: Vec<RecordingFrame>,
    pub markers: Vec<DecodedMarker>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedFrameRun {
    pub run_len: u64,
    pub keys: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedMarker {
    pub frame: u64,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExportDecodedKbdrec {
    pub fps: u16,
    pub key_ids: Vec<String>,
    pub frame_count: u64,
    pub runs: Vec<DecodedFrameRun>,
    pub markers: Vec<DecodedMarker>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingExportInfo {
    pub fps: u16,
    pub frame_count: u64,
}

impl ExportDecodedKbdrec {
    pub fn frames_in_range(
        &self,
        start: u64,
        end: u64,
    ) -> Result<ExportFrameRangeIter<'_>, String> {
        if start > end {
            return Err(format!("frame range start {start} exceeds end {end}"));
        }
        if end > self.frame_count {
            return Err(format!(
                "frame range end {end} exceeds frame count {}",
                self.frame_count
            ));
        }

        Ok(ExportFrameRangeIter {
            runs: &self.runs,
            range_start: start,
            range_end: end,
            run_index: 0,
            run_start: 0,
            next_frame: start,
        })
    }
}

pub(crate) struct ExportFrameRangeIter<'a> {
    runs: &'a [DecodedFrameRun],
    range_start: u64,
    range_end: u64,
    run_index: usize,
    run_start: u64,
    next_frame: u64,
}

impl<'a> Iterator for ExportFrameRangeIter<'a> {
    type Item = (u64, &'a [String]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_frame >= self.range_end {
            return None;
        }

        while let Some(run) = self.runs.get(self.run_index) {
            let run_end = self.run_start + run.run_len;
            if run_end <= self.range_start || self.next_frame >= run_end {
                self.run_start = run_end;
                self.run_index += 1;
                continue;
            }

            let frame = self.next_frame.max(self.run_start);
            self.next_frame = frame + 1;
            return Some((frame, &run.keys));
        }

        None
    }
}

pub fn encode_kbdrec(snapshot: &RecordingSnapshot) -> Result<Vec<u8>, String> {
    if snapshot.fps == 0 {
        return Err("fps must be greater than zero".to_string());
    }

    let mut events = Vec::new();
    let mut markers = Vec::new();
    let mut key_ids = Vec::new();

    for event in &snapshot.events {
        match event {
            RecordingEvent::KeyDown { key_id, .. } | RecordingEvent::KeyUp { key_id, .. }
                if is_recordable_key(key_id) =>
            {
                push_unique(&mut key_ids, key_id);
                events.push(event.clone());
            }
            RecordingEvent::Marker { frame, name } => {
                push_marker_unique(
                    &mut markers,
                    DecodedMarker {
                        frame: *frame,
                        name: name.clone(),
                    },
                );
            }
            RecordingEvent::KeyDown { .. } | RecordingEvent::KeyUp { .. } => {}
        }
    }

    let input_duration_frame = events.iter().map(event_frame).max().unwrap_or(0);
    let marker_duration_frame = markers.iter().map(|marker| marker.frame).max().unwrap_or(0);
    let duration_frame = input_duration_frame.max(marker_duration_frame);
    let frames = if key_ids.is_empty() {
        Vec::new()
    } else {
        sample_frames(snapshot.fps, duration_frame, &events)
    };
    let bitset_len = key_ids.len().div_ceil(8);
    let key_index = key_ids
        .iter()
        .enumerate()
        .map(|(index, key_id)| (key_id.as_str(), index))
        .collect::<std::collections::BTreeMap<_, _>>();
    let frame_bitsets = frames
        .iter()
        .map(|frame| encode_frame_bits(frame, bitset_len, &key_index))
        .collect::<Result<Vec<_>, _>>()?;
    let runs = rle_bitsets(&frame_bitsets);

    let mut bytes = Vec::with_capacity(estimated_buffer_size(
        key_ids.len(),
        bitset_len,
        runs.len(),
        markers.len(),
    ));
    bytes.extend_from_slice(MAGIC);
    bytes.push(VERSION);
    bytes.push(FLAGS);
    bytes.extend_from_slice(&snapshot.fps.to_le_bytes());
    write_varint(key_ids.len() as u64, &mut bytes);
    write_varint(frames.len() as u64, &mut bytes);
    write_varint(runs.len() as u64, &mut bytes);
    write_varint(markers.len() as u64, &mut bytes);

    for key_id in &key_ids {
        write_string(key_id, &mut bytes);
    }

    for (run_len, bits) in &runs {
        write_varint(*run_len, &mut bytes);
        bytes.extend_from_slice(bits);
    }

    for marker in markers {
        write_varint(marker.frame, &mut bytes);
        write_string(&marker.name, &mut bytes);
    }

    Ok(bytes)
}

pub fn decode_kbdrec(bytes: &[u8]) -> Result<DecodedKbdrec, String> {
    let export = decode_kbdrec_for_export(bytes)?;
    let mut frames = Vec::with_capacity(
        usize::try_from(export.frame_count).map_err(|_| "frame count is too large".to_string())?,
    );
    let mut frame_index = 0_u64;
    for run in &export.runs {
        for _ in 0..run.run_len {
            frames.push(RecordingFrame {
                frame: frame_index,
                keys: run.keys.clone(),
            });
            frame_index += 1;
        }
    }

    Ok(DecodedKbdrec {
        fps: export.fps,
        key_ids: export.key_ids,
        frame_count: export.frame_count,
        runs: export.runs,
        frames,
        markers: export.markers,
    })
}

pub(crate) fn decode_kbdrec_for_export(bytes: &[u8]) -> Result<ExportDecodedKbdrec, String> {
    let mut reader = Reader::new(bytes);
    reader.expect_bytes(MAGIC)?;
    let version = reader.read_u8()?;
    if version != VERSION {
        return Err(format!("unsupported kbdrec version: {version}"));
    }
    let _flags = reader.read_u8()?;
    let fps = reader.read_u16_le()?;
    if fps == 0 {
        return Err("kbdrec fps must be greater than zero".to_string());
    }
    let key_count_value = reader.read_varint()?;
    let frame_count = reader.read_varint()?;
    let run_count_value = reader.read_varint()?;
    let marker_count_value = reader.read_varint()?;
    let key_count = validate_collection_count(
        "key",
        key_count_value,
        MAX_KEY_COUNT,
        reader.remaining().len(),
        1,
    )?;

    let mut key_ids = Vec::with_capacity(key_count);
    for _ in 0..key_count {
        key_ids.push(reader.read_string()?);
    }

    let bitset_len = key_ids.len().div_ceil(8);
    let minimum_run_bytes = bitset_len
        .checked_add(1)
        .ok_or_else(|| "kbdrec run size overflow".to_string())?;
    let run_count = validate_collection_count(
        "run",
        run_count_value,
        MAX_RUN_COUNT,
        reader.remaining().len(),
        minimum_run_bytes,
    )?;
    let mut runs = Vec::with_capacity(run_count);
    let mut decoded_frame_count = 0_u64;

    for _ in 0..run_count {
        let run_len = reader.read_varint()?;
        if run_len == 0 {
            return Err("kbdrec frame run length must be greater than zero".to_string());
        }
        let bits = reader.read_bytes(bitset_len)?;
        let keys = decode_frame_bits(&bits, &key_ids)?;
        decoded_frame_count = decoded_frame_count
            .checked_add(run_len)
            .ok_or_else(|| "kbdrec frame run count overflow".to_string())?;
        runs.push(DecodedFrameRun { run_len, keys });
    }
    if decoded_frame_count != frame_count {
        return Err(format!(
            "kbdrec frame count mismatch: header={frame_count}, runs={decoded_frame_count}"
        ));
    }

    let marker_count = validate_collection_count(
        "marker",
        marker_count_value,
        MAX_MARKER_COUNT,
        reader.remaining().len(),
        2,
    )?;
    let mut markers = Vec::with_capacity(marker_count);
    for _ in 0..marker_count {
        let frame = reader.read_varint()?;
        markers.push(DecodedMarker {
            frame,
            name: reader.read_string()?,
        });
    }

    reader.expect_end()?;

    Ok(ExportDecodedKbdrec {
        fps,
        key_ids,
        frame_count,
        runs,
        markers,
    })
}

fn validate_collection_count(
    name: &str,
    value: u64,
    hard_limit: u64,
    remaining_bytes: usize,
    minimum_encoded_bytes: usize,
) -> Result<usize, String> {
    if value > hard_limit {
        return Err(format!(
            "kbdrec {name} count {value} exceeds safety limit {hard_limit}"
        ));
    }
    let count = usize::try_from(value).map_err(|_| format!("kbdrec {name} count is too large"))?;
    let byte_bound = remaining_bytes / minimum_encoded_bytes.max(1);
    if count > byte_bound {
        return Err(format!(
            "kbdrec {name} count {value} exceeds remaining payload capacity"
        ));
    }
    Ok(count)
}

pub fn inspect_kbdrec_export_info(bytes: &[u8]) -> Result<RecordingExportInfo, String> {
    let decoded = decode_kbdrec_for_export(bytes)?;
    Ok(RecordingExportInfo {
        fps: decoded.fps,
        frame_count: decoded.frame_count,
    })
}

fn is_recordable_key(key_id: &str) -> bool {
    key_id != "void"
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

fn push_marker_unique(markers: &mut Vec<DecodedMarker>, marker: DecodedMarker) {
    if !markers
        .iter()
        .any(|existing| existing.frame == marker.frame && existing.name == marker.name)
    {
        markers.push(marker);
    }
}

fn encode_frame_bits(
    frame: &RecordingFrame,
    bitset_len: usize,
    key_index: &std::collections::BTreeMap<&str, usize>,
) -> Result<Vec<u8>, String> {
    let mut bits = vec![0_u8; bitset_len];

    for key_id in &frame.keys {
        let index = key_index
            .get(key_id.as_str())
            .ok_or_else(|| format!("missing key table entry: {key_id}"))?;
        bits[*index / 8] |= 1 << (*index % 8);
    }

    Ok(bits)
}

fn decode_frame_bits(bits: &[u8], key_ids: &[String]) -> Result<Vec<String>, String> {
    let mut keys = Vec::new();

    for (index, key_id) in key_ids.iter().enumerate() {
        if bits[index / 8] & (1 << (index % 8)) != 0 {
            keys.push(key_id.clone());
        }
    }

    Ok(keys)
}

fn rle_bitsets(bitsets: &[Vec<u8>]) -> Vec<(u64, Vec<u8>)> {
    let mut runs = Vec::new();

    for bits in bitsets {
        if let Some((run_len, previous_bits)) = runs.last_mut() {
            if previous_bits == bits {
                *run_len += 1;
                continue;
            }
        }

        runs.push((1, bits.clone()));
    }

    runs
}

fn estimated_buffer_size(
    key_count: usize,
    bitset_len: usize,
    run_count: usize,
    marker_count: usize,
) -> usize {
    const HEADER_BYTES: usize = 32;
    const AVERAGE_KEY_ID_BYTES: usize = 12;
    const AVERAGE_MARKER_BYTES: usize = 24;
    const MAX_VARINT_BYTES: usize = 10;

    HEADER_BYTES
        + key_count * (AVERAGE_KEY_ID_BYTES + MAX_VARINT_BYTES)
        + run_count * (MAX_VARINT_BYTES + bitset_len)
        + marker_count * AVERAGE_MARKER_BYTES
}

fn write_string(value: &str, bytes: &mut Vec<u8>) {
    write_varint(value.len() as u64, bytes);
    bytes.extend_from_slice(value.as_bytes());
}

fn write_varint(mut value: u64, bytes: &mut Vec<u8>) {
    while value >= 0x80 {
        bytes.push((value as u8) | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), String> {
        if self.remaining().len() < expected.len() {
            return Err("unexpected end of file".to_string());
        }

        let actual = &self.bytes[self.cursor..self.cursor + expected.len()];
        if actual != expected {
            return Err("invalid kbdrec magic".to_string());
        }

        self.cursor += expected.len();
        Ok(())
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        let byte = *self
            .remaining()
            .first()
            .ok_or_else(|| "unexpected end of file".to_string())?;
        self.cursor += 1;
        Ok(byte)
    }

    fn read_u16_le(&mut self) -> Result<u16, String> {
        if self.remaining().len() < 2 {
            return Err("unexpected end of file".to_string());
        }

        let value = u16::from_le_bytes([self.bytes[self.cursor], self.bytes[self.cursor + 1]]);
        self.cursor += 2;
        Ok(value)
    }

    fn read_varint(&mut self) -> Result<u64, String> {
        let mut result = 0_u64;
        let mut shift = 0;

        loop {
            let byte = self.read_u8()?;
            result |= u64::from(byte & 0x7f) << shift;

            if byte & 0x80 == 0 {
                return Ok(result);
            }

            shift += 7;
            if shift >= 64 {
                return Err("varint is too large".to_string());
            }
        }
    }

    fn read_string(&mut self) -> Result<String, String> {
        let len = usize::try_from(self.read_varint()?)
            .map_err(|_| "string length is too large".to_string())?;
        if self.remaining().len() < len {
            return Err("unexpected end of file".to_string());
        }

        let value = std::str::from_utf8(&self.bytes[self.cursor..self.cursor + len])
            .map_err(|error| error.to_string())?
            .to_string();
        self.cursor += len;
        Ok(value)
    }

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, String> {
        if self.remaining().len() < len {
            return Err("unexpected end of file".to_string());
        }

        let value = self.bytes[self.cursor..self.cursor + len].to_vec();
        self.cursor += len;
        Ok(value)
    }

    fn expect_end(&self) -> Result<(), String> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err("trailing bytes after kbdrec payload".to_string())
        }
    }

    fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.cursor..]
    }
}
