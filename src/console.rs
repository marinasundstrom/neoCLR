//! Minimal embedding boundary; not a guest stream or text-decoding API.
use std::io::{self, Read, Write};

/// Host console shared explicitly by executions. Implementations own synchronization.
/// Calls are synchronous; cancellation cannot interrupt an in-progress host call.
/// Errors must not panic. A write may have produced partial output before failing.
pub trait Console: std::fmt::Debug + Send + Sync {
    /// One raw byte, or None for EOF. No text decoding or newline conversion.
    fn read_byte(&self) -> io::Result<Option<u8>>;
    /// Write text followed by a line ending and make it visible before returning.
    fn write_line(&self, text: &str) -> io::Result<()>;
}

/// Explicit opt-in to process stdin/stdout. Uses UTF-8 bytes and LF output endings.
#[derive(Debug, Default)]
pub struct StdioConsole;

impl Console for StdioConsole {
    fn read_byte(&self) -> io::Result<Option<u8>> {
        let mut byte = [0u8];
        let mut input = io::stdin().lock();
        loop {
            match input.read(&mut byte) {
                Ok(0) => return Ok(None),
                Ok(_) => return Ok(Some(byte[0])),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
    }

    fn write_line(&self, text: &str) -> io::Result<()> {
        let mut output = io::stdout().lock();
        output.write_all(text.as_bytes())?;
        output.write_all(b"\n")?;
        output.flush()
    }
}
