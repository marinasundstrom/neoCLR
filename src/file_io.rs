//! Blocking, bounded UTF-8 file input for the initial platform library.
use std::io::{ErrorKind, Read};

use crate::{Fault, Value};

fn error(message: &str) -> Value {
    Value::Erased(Box::new(Value::Error(message.into())))
}

fn io_error(error_value: std::io::Error) -> Value {
    error(match error_value.kind() {
        ErrorKind::NotFound => "FileNotFound",
        ErrorKind::PermissionDenied => "AccessDenied",
        ErrorKind::InvalidInput => "InvalidPath",
        _ => "FileReadFailed",
    })
}

pub(crate) fn read_all_text(path: &str, max_bytes: i32) -> Result<Value, Fault> {
    let Ok(limit) = usize::try_from(max_bytes) else {
        return Ok(error("ArgumentOutOfRange"));
    };
    if path.is_empty() || path.contains('\0') {
        return Ok(error("InvalidPath"));
    }
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) => return Ok(io_error(e)),
    };
    match file.metadata() {
        Ok(metadata) if !metadata.is_file() => return Ok(error("NotRegularFile")),
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
            return Ok(error("FileTooLarge"));
        }
        bytes
            .try_reserve_exact(count)
            .map_err(|_| Fault::new("file input allocation failed"))?;
        bytes.extend_from_slice(&chunk[..count]);
    }
    Ok(match String::from_utf8(bytes) {
        Ok(text) => Value::Erased(Box::new(Value::String(text))),
        Err(_) => error("InvalidUtf8"),
    })
}
