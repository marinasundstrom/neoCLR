//! Minimal embedding boundary; not a guest stream or text-decoding API.
use std::io::{self, Write};

mod terminal;
pub use terminal::{ConsoleCapabilities, TerminalInput, TerminalOptions, TerminalRead};

/// Host console shared explicitly by executions. Implementations own synchronization.
/// Calls are synchronous; cancellation cannot interrupt an in-progress host call.
/// Errors must not panic. A write may have produced partial output before failing.
pub trait Console: std::fmt::Debug + Send + Sync {
    /// Discover transport capabilities without acquiring input or changing terminal modes.
    fn capabilities(&self) -> ConsoleCapabilities {
        ConsoleCapabilities::default()
    }
    /// Acquire exclusive, nonblocking terminal input. Drop attempts mode restoration.
    /// Unsupported by default; redirected byte streams are not terminal input.
    fn open_terminal(&self, _options: TerminalOptions) -> io::Result<Box<dyn TerminalInput>> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "terminal input unavailable",
        ))
    }
    /// One raw byte, or None for EOF. No text decoding or newline conversion.
    fn read_byte(&self) -> io::Result<Option<u8>>;
    /// Write text followed by a line ending and make it visible before returning.
    fn write_line(&self, text: &str) -> io::Result<()>;
    /// Write raw stdout (false) or stderr (true) bytes. Hosts opt in explicitly.
    fn write_bytes(&self, _error: bool, _bytes: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "console byte output unavailable",
        ))
    }
    /// Flush the selected output channel without closing the host resource.
    fn flush(&self, _error: bool) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "console flush unavailable",
        ))
    }
}

/// Explicit opt-in to process stdin/stdout. Uses UTF-8 bytes and LF output endings.
#[derive(Debug, Default)]
pub struct StdioConsole;

impl Console for StdioConsole {
    fn capabilities(&self) -> ConsoleCapabilities {
        terminal::capabilities()
    }

    fn open_terminal(&self, options: TerminalOptions) -> io::Result<Box<dyn TerminalInput>> {
        terminal::open(options)
    }

    fn read_byte(&self) -> io::Result<Option<u8>> {
        // Coordinate all StdioConsole instances, including blocking legacy reads.
        let _input = terminal::acquire_standard_input()?;
        terminal::read_standard_byte()
    }

    fn write_bytes(&self, error: bool, bytes: &[u8]) -> io::Result<usize> {
        if error {
            io::stderr().lock().write(bytes)
        } else {
            io::stdout().lock().write(bytes)
        }
    }
    fn flush(&self, error: bool) -> io::Result<()> {
        if error {
            io::stderr().lock().flush()
        } else {
            io::stdout().lock().flush()
        }
    }
    fn write_line(&self, text: &str) -> io::Result<()> {
        let mut output = io::stdout().lock();
        output.write_all(text.as_bytes())?;
        output.write_all(b"\n")?;
        output.flush()
    }
}
