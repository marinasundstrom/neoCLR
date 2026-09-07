//! Blocking, bounded UTF-8 file input for the initial platform library.
use std::io::{ErrorKind, Read};

use crate::{Fault, Value};

// Explicit internal Byte protocol consumed by System.IO.File's IL wrapper.
#[repr(u8)]
enum ReadStatus {
    InvalidLimit = 1,
    InvalidPath = 2,
    NotFound = 3,
    AccessDenied = 4,
    NotRegularFile = 5,
    ReadFailed = 6,
    TooLarge = 7,
    InvalidUtf8 = 8,
}

fn error(status: ReadStatus) -> Value {
    Value::Erased(Box::new(Value::Byte(status as u8)))
}

fn io_error(error_value: std::io::Error) -> Value {
    error(match error_value.kind() {
        ErrorKind::NotFound => ReadStatus::NotFound,
        ErrorKind::PermissionDenied => ReadStatus::AccessDenied,
        ErrorKind::InvalidInput => ReadStatus::InvalidPath,
        _ => ReadStatus::ReadFailed,
    })
}

pub(crate) fn read_all_text(path: &str, max_bytes: i32) -> Result<Value, Fault> {
    let Ok(limit) = usize::try_from(max_bytes) else {
        return Ok(error(ReadStatus::InvalidLimit));
    };
    if path.is_empty() || path.contains('\0') {
        return Ok(error(ReadStatus::InvalidPath));
    }
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) => return Ok(io_error(e)),
    };
    match file.metadata() {
        Ok(metadata) if !metadata.is_file() => return Ok(error(ReadStatus::NotRegularFile)),
        Ok(_) => {}
        Err(e) => return Ok(io_error(e)),
    }
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        // Probe one byte past the bound, even if metadata understates a growing file.
        let requested = chunk.len().min(limit - bytes.len() + 1);
        let count = match file.read(&mut chunk[..requested]) {
            Ok(0) => break,
            Ok(count) => count,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Ok(io_error(e)),
        };
        if count > limit - bytes.len() {
            return Ok(error(ReadStatus::TooLarge));
        }
        bytes
            .try_reserve_exact(count)
            .map_err(|_| Fault::new("file input allocation failed"))?;
        bytes.extend_from_slice(&chunk[..count]);
    }
    Ok(match String::from_utf8(bytes) {
        Ok(text) => Value::Erased(Box::new(Value::String(text))),
        Err(_) => error(ReadStatus::InvalidUtf8),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_io_kinds_have_explicit_cross_platform_statuses() {
        for (kind, status) in [
            (ErrorKind::NotFound, 3),
            (ErrorKind::PermissionDenied, 4),
            (ErrorKind::InvalidInput, 2),
            (ErrorKind::Other, 6),
        ] {
            assert_eq!(
                io_error(std::io::Error::from(kind)),
                Value::Erased(Box::new(Value::Byte(status)))
            );
        }
    }
}
