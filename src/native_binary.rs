//! NEOX native execution schema 2: bounded definite-length CBOR object data.
use crate::{Fault, Module};
use std::collections::HashSet;

fn invalid(message: impl std::fmt::Display) -> Fault {
    Fault::new(format!("invalid binary native metadata: {message}"))
}

pub(crate) fn decode(data: &[u8]) -> Result<Module, Fault> {
    validate(data)?;
    // Deserialize into the runtime model, without a JSON string or serde_json::Value tree.
    let module: Module = ciborium::from_reader(data).map_err(invalid)?;
    if module.format != 5 {
        return Err(invalid("unsupported semantic module format (expected 5)"));
    }
    Ok(module)
}

fn validate(data: &[u8]) -> Result<(), Fault> {
    if data.len() > 1024 * 1024 - 32 {
        return Err(invalid("payload exceeds envelope limit"));
    }
    let mut reader = Reader {
        data,
        position: 0,
        nodes: 0,
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
        if depth > 64 || self.nodes > 262144 {
            return Err(invalid("depth/node limit exceeded"));
        }
        Ok(())
    }
    fn value(&mut self, depth: usize) -> Result<(), Fault> {
        self.count(depth)?;
        let (major, arg) = self.head()?;
        match major {
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
