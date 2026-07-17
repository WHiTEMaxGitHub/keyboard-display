use super::{sample_frames, RecordingEvent, RecordingFrame, RecordingSnapshot};

const MAGIC: &[u8; 4] = b"KBDR";
const VERSION: u8 = 1;
const FLAGS: u8 = 0;

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
    pub t_ms: u64,
    pub name: String,
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
            RecordingEvent::Marker { t, name } => {
                push_marker_unique(
                    &mut markers,
                    DecodedMarker {
                        frame: ms_to_frame(*t, snapshot.fps),
                        t_ms: *t,
                        name: name.clone(),
                    },
                );
            }
            RecordingEvent::KeyDown { .. } | RecordingEvent::KeyUp { .. } => {}
        }
    }

    let input_duration_ms = events.iter().map(event_time).max().unwrap_or(0);
    let marker_duration_ms = markers.iter().map(|marker| marker.t_ms).max().unwrap_or(0);
    let duration_ms = input_duration_ms.max(marker_duration_ms);
    let frames = if key_ids.is_empty() {
        Vec::new()
    } else {
        sample_frames(snapshot.fps, duration_ms, &events)
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
    let mut reader = Reader::new(bytes);
    reader.expect_bytes(MAGIC)?;
    let version = reader.read_u8()?;
    if version != VERSION {
        return Err(format!("unsupported kbdrec version: {version}"));
    }
    let _flags = reader.read_u8()?;
    let fps = reader.read_u16_le()?;
    let key_count = reader.read_varint()? as usize;
    let frame_count = reader.read_varint()?;
    let run_count = reader.read_varint()? as usize;
    let marker_count = reader.read_varint()? as usize;

    let mut key_ids = Vec::with_capacity(key_count);
    for _ in 0..key_count {
        key_ids.push(reader.read_string()?);
    }

    let bitset_len = key_ids.len().div_ceil(8);
    let mut runs = Vec::with_capacity(run_count);
    let mut frames = Vec::new();
    let mut frame_index = 0_u64;

    for _ in 0..run_count {
        let run_len = reader.read_varint()?;
        let bits = reader.read_bytes(bitset_len)?;
        let keys = decode_frame_bits(&bits, &key_ids)?;

        for _ in 0..run_len {
            frames.push(RecordingFrame {
                t: frame_to_ms(frame_index, fps),
                keys: keys.clone(),
            });
            frame_index += 1;
        }

        runs.push(DecodedFrameRun { run_len, keys });
    }

    let mut markers = Vec::with_capacity(marker_count);
    for _ in 0..marker_count {
        let frame = reader.read_varint()?;
        markers.push(DecodedMarker {
            frame,
            t_ms: frame_to_ms(frame, fps),
            name: reader.read_string()?,
        });
    }

    reader.expect_end()?;

    Ok(DecodedKbdrec {
        fps,
        key_ids,
        frame_count,
        runs,
        frames,
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

fn push_marker_unique(markers: &mut Vec<DecodedMarker>, marker: DecodedMarker) {
    if !markers
        .iter()
        .any(|existing| existing.frame == marker.frame && existing.name == marker.name)
    {
        markers.push(marker);
    }
}

fn ms_to_frame(t_ms: u64, fps: u16) -> u64 {
    (t_ms * u64::from(fps) + 500) / 1000
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

fn frame_to_ms(frame: u64, fps: u16) -> u64 {
    frame * 1000 / u64::from(fps)
}

fn event_time(event: &RecordingEvent) -> u64 {
    match event {
        RecordingEvent::KeyDown { t, .. }
        | RecordingEvent::KeyUp { t, .. }
        | RecordingEvent::Marker { t, .. } => *t,
    }
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
