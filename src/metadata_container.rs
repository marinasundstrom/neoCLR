//! Experimental PE/#Neo native execution transport. CLI bodies are never executed.
//! The digest binds metadata streams for consistency; it is not authentication.
use crate::{Fault, Module, pe};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

fn invalid(message: &str) -> Fault {
    Fault::new(format!("native metadata container: {message}"))
}
fn bytes(data: &[u8], offset: usize, size: usize) -> Result<&[u8], Fault> {
    data.get(
        offset
            ..offset
                .checked_add(size)
                .ok_or_else(|| invalid("range overflow"))?,
    )
    .ok_or_else(|| invalid("truncated range"))
}
fn u16_at(data: &[u8], offset: usize) -> Result<u16, Fault> {
    Ok(u16::from_le_bytes(
        bytes(data, offset, 2)?.try_into().unwrap(),
    ))
}
fn u32_at(data: &[u8], offset: usize) -> Result<usize, Fault> {
    Ok(u32::from_le_bytes(bytes(data, offset, 4)?.try_into().unwrap()) as usize)
}

/// Extract a schema-1 JSON payload. Binary containers deliberately return an error;
/// use decode/load for runtime admission instead of converting binary data back to text.
pub fn native_json(image: &[u8]) -> Result<&str, Fault> {
    let (version, bytes) = payload(image)?;
    if version != 1 {
        return Err(invalid(
            "binary payload has no borrowed JSON representation",
        ));
    }
    std::str::from_utf8(bytes).map_err(|_| invalid("native payload must be UTF-8"))
}

fn payload(image: &[u8]) -> Result<(u16, &[u8]), Fault> {
    if image.len() > 16 * 1024 * 1024 {
        return Err(invalid("image exceeds 16 MiB library PE limit"));
    }
    let pe =
        pe::Image::parse(image, pe::Profile::StaticCallsV1).map_err(|e| invalid(&e.to_string()))?;
    validate_layout(image)?;
    let data = pe.metadata();
    let version_size = u32_at(data, 12)?;
    if version_size > 256 || version_size % 4 != 0 {
        return Err(invalid("unsupported metadata version length"));
    }
    let version = bytes(data, 16, version_size)?;
    let terminator = version
        .iter()
        .position(|v| *v == 0)
        .ok_or_else(|| invalid("unterminated marker"))?;
    if version[terminator..].iter().any(|v| *v != 0) {
        return Err(invalid("invalid marker padding"));
    }
    let claimed = version[..terminator]
        .strip_prefix(b"neoCLR.NEOX.0.1;sha256=")
        .ok_or_else(|| invalid("required recognition marker missing or unsupported"))?;
    if claimed.len() != 64
        || claimed
            .iter()
            .any(|v| !matches!(v, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(invalid("invalid binding digest"));
    }
    let count = u16_at(data, 18 + version_size)? as usize;
    if count > 16 {
        return Err(invalid("too many metadata streams"));
    }
    let mut position = 20 + version_size;
    let mut entries = Vec::new();
    let mut streams = BTreeMap::new();
    for _ in 0..count {
        let offset = u32_at(data, position)?;
        let size = u32_at(data, position + 4)?;
        position += 8;
        let start = position;
        while position - start < 32 && bytes(data, position, 1)?[0] != 0 {
            if data[position] > 127 {
                return Err(invalid("non-ASCII stream name"));
            }
            position += 1;
        }
        if position == start || position - start == 32 {
            return Err(invalid("invalid stream name"));
        }
        let name = &data[start..position];
        let end = start + ((position - start + 4) & !3);
        if bytes(data, position, end - position)?
            .iter()
            .any(|v| *v != 0)
        {
            return Err(invalid("invalid stream-name padding"));
        }
        position = end;
        let payload = bytes(data, offset, size)?;
        if streams.insert(name, payload).is_some() {
            return Err(invalid("duplicate stream name"));
        }
        entries.push((offset, size));
    }
    for (index, &(offset, size)) in entries.iter().enumerate() {
        if offset < position || offset % 4 != 0 {
            return Err(invalid("invalid stream offset"));
        }
        if entries[..index]
            .iter()
            .any(|&(other, length)| offset < other + length && other < offset + size)
        {
            return Err(invalid("overlapping streams"));
        }
    }
    let mut hash = Sha256::new();
    hash.update(b"neoCLR experimental metadata binding 0.1\0");
    for (name, payload) in &streams {
        hash.update((name.len() as u16).to_le_bytes());
        hash.update(name);
        hash.update((payload.len() as u32).to_le_bytes());
        hash.update(payload);
    }
    let actual = format!("{:x}", hash.finalize());
    if actual.as_bytes() != claimed {
        return Err(invalid("metadata binding mismatch"));
    }
    let envelope = streams
        .get(b"#Neo".as_slice())
        .ok_or_else(|| invalid("required #Neo stream missing"))?;
    let length = u32_at(envelope, 12)?;
    if !(16..=16 * 1024 * 1024).contains(&length)
        || length > envelope.len()
        || envelope.len() - length > 3
        || envelope[length..].iter().any(|v| *v != 0)
    {
        return Err(invalid("invalid envelope size/padding"));
    }
    let payload = execution_payload_profile(&envelope[..length], true)?;
    if payload.0 < 3 && image.len() > 4 * 1024 * 1024 {
        return Err(invalid("legacy image exceeds 4 MiB limit"));
    }
    Ok(payload)
}

// Keep runtime admission within the same unsigned, overlay-free PE32 bounds as the host writer.
fn validate_layout(image: &[u8]) -> Result<(), Fault> {
    let pe = u32_at(image, 0x3c)?;
    let count = u16_at(image, pe + 6)? as usize;
    let optional = pe + 24;
    if count > 16
        || u16_at(image, pe + 20)? != 224
        || u32_at(image, optional + 92)? != 16
        || u32_at(image, optional + 64)? != 0
        || bytes(image, optional + 128, 8)?.iter().any(|v| *v != 0)
    {
        return Err(invalid("unsupported PE layout, checksum or signature"));
    }
    let section_alignment = u32_at(image, optional + 32)?;
    let file_alignment = u32_at(image, optional + 36)?;
    if !(512..=65536).contains(&file_alignment)
        || !file_alignment.is_power_of_two()
        || !(file_alignment..=65536).contains(&section_alignment)
        || !section_alignment.is_power_of_two()
    {
        return Err(invalid("unsupported PE alignment"));
    }
    let headers = u32_at(image, optional + 60)?;
    let first_rva = (headers + section_alignment - 1) & !(section_alignment - 1);
    let mut file_end = 0;
    for index in 0..count {
        let row = optional + 224 + index * 40;
        let rva = u32_at(image, row + 12)?;
        let size = u32_at(image, row + 16)?;
        let raw = u32_at(image, row + 20)?;
        if raw < headers
            || raw % file_alignment != 0
            || size % file_alignment != 0
            || rva < first_rva
            || rva % section_alignment != 0
        {
            return Err(invalid("invalid section alignment/range"));
        }
        bytes(image, raw, size)?;
        file_end = file_end.max(raw + size);
    }
    if file_end != image.len() {
        return Err(invalid("PE overlays unsupported"));
    }
    Ok(())
}

#[cfg(test)]
fn execution_payload(envelope: &[u8]) -> Result<(u16, &[u8]), Fault> {
    execution_payload_profile(envelope, false)
}

fn execution_payload_profile(envelope: &[u8], library: bool) -> Result<(u16, &[u8]), Fault> {
    if bytes(envelope, 0, 4)? != b"NEOX" || u16_at(envelope, 4)? != 0 || u16_at(envelope, 6)? != 1 {
        return Err(invalid("unsupported envelope version"));
    }
    let count = u32_at(envelope, 8)?;
    if count > 64 || u32_at(envelope, 12)? != envelope.len() {
        return Err(invalid("invalid envelope directory"));
    }
    let mut end = 16 + count * 16;
    bytes(envelope, 0, end)?;
    let mut kinds = HashSet::new();
    let mut execution = None;
    for i in 0..count {
        let row = 16 + i * 16;
        let kind = u16_at(envelope, row)?;
        let version = u16_at(envelope, row + 2)?;
        let flags = u32_at(envelope, row + 4)?;
        let offset = u32_at(envelope, row + 8)?;
        let length = u32_at(envelope, row + 12)?;
        if kind == 0 || version == 0 || !kinds.insert(kind) || flags > 1 || offset != end {
            return Err(invalid("invalid section directory"));
        }
        let payload = bytes(envelope, offset, length)?;
        end += length;
        if flags == 1
            && (kind != 256 || !(matches!(version, 1 | 2) || library && matches!(version, 3 | 4)))
        {
            return Err(invalid("unsupported required section"));
        }
        if kind == 256 {
            if flags != 1 || !(matches!(version, 1 | 2) || library && matches!(version, 3 | 4)) {
                return Err(invalid("unsupported or optional native execution schema"));
            }
            execution = Some((version, payload));
        }
    }
    if end != envelope.len() {
        return Err(invalid("trailing envelope bytes"));
    }
    let (version, bytes) = execution.ok_or_else(|| invalid("native execution section missing"))?;
    if version == 3 && envelope.len() > 8 * 1024 * 1024 {
        return Err(invalid("schema-3 envelope exceeds 8 MiB limit"));
    }
    if version < 3 && envelope.len() > 1024 * 1024 {
        return Err(invalid("legacy envelope exceeds 1 MiB limit"));
    }
    if version == 1 {
        std::str::from_utf8(bytes).map_err(|_| invalid("native payload must be UTF-8"))?;
    }
    Ok((version, bytes))
}

/// Decode and apply the legacy load validation, including bundled System linking.
/// Use ModuleInput::MetadataPe for explicit dependency sets; LoadedProgram handles
/// their admission, preparation and typed verification.
pub fn load(image: &[u8]) -> Result<Module, Fault> {
    let module = decode(image)?;
    crate::vm::validate(&module)?;
    Ok(module)
}

/// Decode a recognized container into the native module model. Schema 2 uses direct
/// binary deserialization; schema 1 remains compatible. Does not link or verify bodies.
pub fn decode(image: &[u8]) -> Result<Module, Fault> {
    decode_payload(payload(image)?)
}

/// Encode a format-5 module as an owned standalone schema-3/4 NEOX assembly.
/// Uses direct binary serialization, preserves definition/reference identities and
/// selects schema 4 above 8 MiB and enforces the library profile's budgets. Does not link or verify method bodies.
pub fn write_module(module: &Module) -> Result<Vec<u8>, Fault> {
    let payload = crate::native_binary::encode_library(module)?;
    let mut image = vec![0u8; 32];
    image[..4].copy_from_slice(b"NEOX");
    image[6..8].copy_from_slice(&1u16.to_le_bytes());
    image[8..12].copy_from_slice(&1u32.to_le_bytes());
    image[12..16].copy_from_slice(&((32 + payload.len()) as u32).to_le_bytes());
    image[16..18].copy_from_slice(&256u16.to_le_bytes());
    let schema: u16 = if payload.len() <= 8 * 1024 * 1024 - 32 {
        3
    } else {
        4
    };
    image[18..20].copy_from_slice(&schema.to_le_bytes());
    image[20..24].copy_from_slice(&1u32.to_le_bytes());
    image[24..28].copy_from_slice(&32u32.to_le_bytes());
    image[28..32].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    image.extend_from_slice(&payload);
    Ok(image)
}

/// Decode a standalone NEOX native module without a CLI projection or PE binding.
/// The same runtime validation and dependency rules apply after decoding.
pub fn decode_envelope(image: &[u8]) -> Result<Module, Fault> {
    if image.len() > 16 * 1024 * 1024 {
        return Err(invalid("envelope exceeds 16 MiB limit"));
    }
    decode_payload(execution_payload_profile(image, true)?)
}

/// Decode a standalone native envelope and apply legacy bundled-System validation.
pub fn load_envelope(image: &[u8]) -> Result<Module, Fault> {
    let module = decode_envelope(image)?;
    crate::vm::validate(&module)?;
    Ok(module)
}

fn decode_payload((version, bytes): (u16, &[u8])) -> Result<Module, Fault> {
    if matches!(version, 2..=4) {
        crate::native_binary::decode(bytes, version >= 3, version == 4)
    } else {
        crate::decode_module(std::str::from_utf8(bytes).map_err(|_| invalid("invalid UTF-8"))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn envelope() -> Vec<u8> {
        let mut data = vec![0; 34];
        data[..4].copy_from_slice(b"NEOX");
        data[6] = 1;
        data[8] = 1;
        data[12] = 34;
        data[17] = 1;
        data[18] = 1;
        data[20] = 1;
        data[24] = 32;
        data[28] = 2;
        data[32..].copy_from_slice(b"{}");
        data
    }
    #[test]
    fn expanded_library_round_trip_and_downgrade_rejection() {
        let mut module: Module =
            serde_json::from_str(r#"{"format":5,"name":"Small","functions":[]}"#).unwrap();
        assert_eq!(write_module(&module).unwrap()[18], 3);
        module.name = "x".repeat(8 * 1024 * 1024);
        let encoded = write_module(&module).unwrap();
        assert_eq!(encoded[18], 4);
        assert_eq!(decode_envelope(&encoded).unwrap().name, module.name);
        let mut downgraded = encoded.clone();
        downgraded[18] = 3;
        assert!(
            decode_envelope(&downgraded)
                .unwrap_err()
                .to_string()
                .contains("8 MiB")
        );
        downgraded[18] = 5;
        assert!(decode_envelope(&downgraded).is_err());
        assert!(decode_envelope(&vec![0; 16 * 1024 * 1024 + 1]).is_err());
    }

    #[test]
    fn required_execution_schema_and_bounds() {
        let valid = envelope();
        assert_eq!(execution_payload(&valid).unwrap(), (1, b"{}".as_slice()));
        for (offset, value) in [
            (4, 1),
            (6, 2),
            (8, 65),
            (12, 33),
            (16, 1),
            (18, 3),
            (20, 0),
            (20, 2),
            (24, 31),
            (28, 3),
            (32, 255),
        ] {
            let mut bad = valid.clone();
            bad[offset] = value;
            assert!(execution_payload(&bad).is_err(), "offset {offset}");
        }
        for size in 0..valid.len() {
            assert!(execution_payload(&valid[..size]).is_err());
        }
    }
}
