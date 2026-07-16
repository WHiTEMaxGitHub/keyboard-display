use super::{RecordingEvent, RecordingSnapshot};
use std::collections::BTreeMap;

const MAGIC: &[u8; 4] = b"KBDR";
const VERSION: u8 = 1;
const FLAGS: u8 = 0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedKbdrec {
    pub fps: u16,
    pub key_ids: Vec<String>,
    pub events: Vec<DecodedFrameEvent>,
    pub markers: Vec<DecodedMarker>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedFrameEvent {
    pub frame_delta: u64,
    pub down: Vec<String>,
    pub up: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedMarker {
    pub t_ms: u64,
    pub name: String,
}

#[derive(Default)]
struct FrameChanges {
    down: Vec<String>,
    up: Vec<String>,
}

pub fn encode_kbdrec(snapshot: &RecordingSnapshot) -> Result<Vec<u8>, String> {
    if snapshot.fps == 0 {
        return Err("fps must be greater than zero".to_string());
    }

    let mut frames = BTreeMap::<u64, FrameChanges>::new();
    let mut key_ids = Vec::<String>::new();
    let mut markers = Vec::<DecodedMarker>::new();

    for event in &snapshot.events {
        match event {
            RecordingEvent::KeyDown { t, key_id } if is_recordable_key(key_id) => {
                push_unique(&mut key_ids, key_id);
                push_unique(
                    &mut frames
                        .entry(ms_to_frame(*t, snapshot.fps))
                        .or_default()
                        .down,
                    key_id,
                );
            }
            RecordingEvent::KeyUp { t, key_id } if is_recordable_key(key_id) => {
                push_unique(&mut key_ids, key_id);
                push_unique(
                    &mut frames.entry(ms_to_frame(*t, snapshot.fps)).or_default().up,
                    key_id,
                );
            }
            RecordingEvent::Marker { t, name } => {
                markers.push(DecodedMarker {
                    t_ms: *t,
                    name: name.clone(),
                });
            }
            RecordingEvent::KeyDown { .. } | RecordingEvent::KeyUp { .. } => {}
        }
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.push(VERSION);
    bytes.push(FLAGS);
    bytes.extend_from_slice(&snapshot.fps.to_le_bytes());
    write_varint(key_ids.len() as u64, &mut bytes);
    write_varint(frames.len() as u64, &mut bytes);
    write_varint(markers.len() as u64, &mut bytes);

    for key_id in &key_ids {
        write_string(key_id, &mut bytes);
    }

    let key_index = key_ids
        .iter()
        .enumerate()
        .map(|(index, key_id)| (key_id.as_str(), index as u64))
        .collect::<BTreeMap<_, _>>();
    let mut previous_frame = 0;

    for (frame, changes) in frames {
        write_varint(frame.saturating_sub(previous_frame), &mut bytes);
        previous_frame = frame;
        write_key_ids(&changes.down, &key_index, &mut bytes)?;
        write_key_ids(&changes.up, &key_index, &mut bytes)?;
    }

    for marker in markers {
        write_varint(marker.t_ms, &mut bytes);
        write_string(&marker.name, &mut bytes);
    }

    Ok(bytes)
}

pub fn decode_kbdrec(bytes: &[u8]) -> Result<DecodedKbdrec, String> {
    let mut reader = Reader::new(bytes);
    reader.expect_bytes(MAGIC)?;
    let version = reader.read_u8()?;
    if version != VERSION {
        return Err(format!("unsupported kbdrec version: {version}"));
    }
    let _flags = reader.read_u8()?;
    let fps = reader.read_u16_le()?;
    let key_count = reader.read_varint()? as usize;
    let event_count = reader.read_varint()? as usize;
    let marker_count = reader.read_varint()? as usize;

    let mut key_ids = Vec::with_capacity(key_count);
    for _ in 0..key_count {
        key_ids.push(reader.read_string()?);
    }

    let mut events = Vec::with_capacity(event_count);
    for _ in 0..event_count {
        let frame_delta = reader.read_varint()?;
        let down = reader.read_key_ids(&key_ids)?;
        let up = reader.read_key_ids(&key_ids)?;
        events.push(DecodedFrameEvent {
            frame_delta,
            down,
            up,
        });
    }

    let mut markers = Vec::with_capacity(marker_count);
    for _ in 0..marker_count {
        markers.push(DecodedMarker {
            t_ms: reader.read_varint()?,
            name: reader.read_string()?,
        });
    }

    reader.expect_end()?;

    Ok(DecodedKbdrec {
        fps,
        key_ids,
        events,
        markers,
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

fn ms_to_frame(t_ms: u64, fps: u16) -> u64 {
    (t_ms * u64::from(fps) + 500) / 1000
}

fn write_key_ids(
    key_ids: &[String],
    key_index: &BTreeMap<&str, u64>,
    bytes: &mut Vec<u8>,
) -> Result<(), String> {
    write_varint(key_ids.len() as u64, bytes);

    for key_id in key_ids {
        let index = key_index
            .get(key_id.as_str())
            .ok_or_else(|| format!("missing key table entry: {key_id}"))?;
        write_varint(*index, bytes);
    }

    Ok(())
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
        let len = self.read_varint()? as usize;
        if self.remaining().len() < len {
            return Err("unexpected end of file".to_string());
        }

        let value = std::str::from_utf8(&self.bytes[self.cursor..self.cursor + len])
            .map_err(|error| error.to_string())?
            .to_string();
        self.cursor += len;
        Ok(value)
    }

    fn read_key_ids(&mut self, key_table: &[String]) -> Result<Vec<String>, String> {
        let count = self.read_varint()? as usize;
        let mut key_ids = Vec::with_capacity(count);

        for _ in 0..count {
            let index = self.read_varint()? as usize;
            let key_id = key_table
                .get(index)
                .ok_or_else(|| format!("key index out of range: {index}"))?;
            key_ids.push(key_id.clone());
        }

        Ok(key_ids)
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
