//! NEOX native execution schemas 2/3: bounded definite-length CBOR object data.
use crate::{Fault, Module};
use std::collections::HashSet;

fn invalid(message: impl std::fmt::Display) -> Fault {
    Fault::new(format!("invalid binary native metadata: {message}"))
}

/// Serialize the native model directly, without a JSON intermediate.
pub(crate) fn encode_library(module: &Module) -> Result<Vec<u8>, Fault> {
    struct Bounded(Vec<u8>);
    impl std::io::Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > (8 * 1024 * 1024 - 32) - self.0.len() {
                return Err(std::io::Error::other(
                    "native payload exceeds library envelope limit",
                ));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    if module.format != 5 {
        return Err(invalid("unsupported semantic module format (expected 5)"));
    }
    if module.name.trim().is_empty() {
        return Err(invalid("native module name must not be empty"));
    }
    let mut output = Bounded(Vec::new());
    ciborium::into_writer(module, &mut output).map_err(invalid)?;
    validate_profile(&output.0, true)?;
    Ok(output.0)
}

pub(crate) fn decode(data: &[u8], library: bool) -> Result<Module, Fault> {
    validate_profile(data, library)?;
    // Deserialize into the runtime model, without a JSON string or serde_json::Value tree.
    let module: Module = ciborium::from_reader(data).map_err(invalid)?;
    if module.format != 5 {
        return Err(invalid("unsupported semantic module format (expected 5)"));
    }
    Ok(module)
}

#[cfg(test)]
fn validate(data: &[u8]) -> Result<(), Fault> {
    validate_profile(data, false)
}

fn validate_profile(data: &[u8], library: bool) -> Result<(), Fault> {
    if data.len()
        > (if library {
            8 * 1024 * 1024
        } else {
            1024 * 1024
        }) - 32
    {
        return Err(invalid("payload exceeds envelope limit"));
    }
    let mut reader = Reader {
        data,
        position: 0,
        nodes: 0,
        library,
    };
    reader.value(0)?;
    if reader.position != data.len() {
        return Err(invalid("trailing bytes"));
    }
    Ok(())
}
struct Reader<'a> {
    data: &'a [u8],
    position: usize,
    nodes: usize,
    library: bool,
}
impl<'a> Reader<'a> {
    fn take(&mut self, size: usize) -> Result<&'a [u8], Fault> {
        if size > self.data.len() - self.position {
            return Err(invalid("truncated payload"));
        }
        let bytes = &self.data[self.position..self.position + size];
        self.position += size;
        Ok(bytes)
    }
    fn head(&mut self) -> Result<(u8, u64), Fault> {
        let byte = self.take(1)?[0];
        let major = byte >> 5;
        let arg = byte & 31;
        if arg < 24 {
            return Ok((major, u64::from(arg)));
        }
        let size = match arg {
            24 => 1,
            25 => 2,
            26 => 4,
            27 => 8,
            _ => return Err(invalid("indefinite or reserved item")),
        };
        let mut value = 0u64;
        for byte in self.take(size)? {
            value = (value << 8) | u64::from(*byte);
        }
        let minimum = match size {
            1 => 24,
            2 => 256,
            4 => 65536,
            _ => 4294967296,
        };
        if value < minimum {
            return Err(invalid("nonminimal argument"));
        }
        Ok((major, value))
    }
    fn length(&self, size: u64) -> Result<usize, Fault> {
        if size > (self.data.len() - self.position) as u64 {
            return Err(invalid("length exceeds remaining bytes"));
        }
        Ok(size as usize)
    }
    fn text(&mut self, size: u64) -> Result<&'a str, Fault> {
        let size = self.length(size)?;
        std::str::from_utf8(self.take(size)?).map_err(invalid)
    }
    fn count(&mut self, depth: usize) -> Result<(), Fault> {
        self.nodes += 1;
        if depth > 64 || self.nodes > if self.library { 2097152 } else { 262144 } {
            return Err(invalid("depth/node limit exceeded"));
        }
        Ok(())
    }
    fn value(&mut self, depth: usize) -> Result<(), Fault> {
        self.count(depth)?;
        let (major, arg) = self.head()?;
        match major {
            0 if self.library => (),
            0 | 1 if arg <= i64::MAX as u64 => (),
            3 => {
                self.text(arg)?;
            }
            4 => {
                for _ in 0..self.length(arg)? {
                    self.value(depth + 1)?;
                }
            }
            5 => {
                let count = self.length(arg)?;
                if count > (self.data.len() - self.position) / 2 {
                    return Err(invalid("map count exceeds remaining bytes"));
                }
                let mut names = HashSet::new();
                for _ in 0..count {
                    self.count(depth + 1)?;
                    let (kind, size) = self.head()?;
                    if kind != 3 {
                        return Err(invalid("map key must be text"));
                    }
                    if !names.insert(self.text(size)?) {
                        return Err(invalid("duplicate map key"));
                    }
                    self.value(depth + 1)?;
                }
            }
            7 if matches!(arg, 20..=22) => (),
            _ => return Err(invalid("unsupported kind or integer outside Int64 range")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direct_writer_rejects_unsupported_format_and_payload_overflow() {
        let mut module: Module =
            serde_json::from_str(r#"{"format":5,"name":"Large","functions":[]}"#).unwrap();
        module.format = 4;
        assert!(encode_library(&module).is_err());
        module.format = 5;
        module.name.clear();
        assert!(encode_library(&module).is_err());
        module.name = "x".repeat(8 * 1024 * 1024);
        assert!(encode_library(&module).is_err());
    }

    #[test]
    fn library_profile_preserves_unsigned_bits_and_separate_budgets() {
        let unsigned = [0x1b, 255, 255, 255, 255, 255, 255, 255, 255];
        validate_profile(&unsigned, true).unwrap();
        assert!(validate(&unsigned).is_err());
        let mut negative = unsigned;
        negative[0] = 0x3b;
        assert!(validate_profile(&negative, true).is_err());
        let mut nodes = vec![0x9a, 0, 0x1f, 255, 255];
        nodes.resize(2097156, 0);
        validate_profile(&nodes, true).unwrap();
        assert!(validate(&nodes).is_err());
        nodes[2] = 0x20;
        nodes[3] = 0;
        nodes[4] = 0;
        nodes.push(0);
        assert!(validate_profile(&nodes, true).is_err());
        let mut largest = vec![b'x'; 8 * 1024 * 1024 - 32];
        largest[0] = 0x7a;
        let length = (largest.len() - 5) as u32;
        largest[1..5].copy_from_slice(&length.to_be_bytes());
        validate_profile(&largest, true).unwrap();
        assert!(validate_profile(&vec![0; 8 * 1024 * 1024 - 31], true).is_err());
        assert!(validate_profile(&[vec![0x81; 65], vec![0]].concat(), true).is_err());
        // Native Float64 operands use exact UInt64 bits, including negative zero and NaNs.
        for bits in [
            0x8000000000000000u64,
            0xbff0000000000000,
            0xfff8000000000001,
            u64::MAX,
        ] {
            let json = format!(
                r#"{{"format":5,"name":"Bits","functions":[{{"name":"Main","parameters":[],"returns":"Double","locals":[],"body":[{{"op":"ldc.r8","arg":{{"bits":{bits}}}}},{{"op":"ret"}}]}}]}}"#
            );
            let module: Module = serde_json::from_str(&json).unwrap();
            let mut binary = Vec::new();
            ciborium::into_writer(&module, &mut binary).unwrap();
            let decoded = decode(&binary, true).unwrap();
            assert_eq!(
                serde_json::to_value(module).unwrap(),
                serde_json::to_value(decoded).unwrap()
            );
            assert!(decode(&binary, false).is_err());
        }
    }

    #[test]
    fn profile_bounds_and_unsupported_encodings() {
        for bytes in [
            vec![0xbf, 0xff],
            vec![0x18, 0],
            vec![0xc0, 0],
            vec![0x40],
            vec![0x61, 0xff],
            vec![0xa2, 0x61, b'x', 0, 0x61, b'x', 1],
            vec![0xa1, 0, 0],
            vec![0x80, 0],
            vec![0xfb, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            vec![0x9a, 0xff, 0xff, 0xff, 0xff],
            [vec![0x81; 65], vec![0]].concat(),
        ] {
            assert!(validate(&bytes).is_err(), "{bytes:?}");
            if bytes != [0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff] {
                assert!(validate_profile(&bytes, true).is_err(), "{bytes:?}");
            }
        }
        let mut excessive_nodes = vec![0x9a, 0, 4, 0, 1];
        excessive_nodes.resize(262150, 0);
        assert!(validate(&excessive_nodes).is_err());
        for bytes in [
            vec![0xa1, 0x61, b'x', 0x18, 24],
            vec![0xf4],
            vec![0xf5],
            vec![0xf6],
            vec![0x80],
            vec![0xa0],
            vec![0x3b, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        ] {
            validate(&bytes).unwrap();
        }
    }
}
