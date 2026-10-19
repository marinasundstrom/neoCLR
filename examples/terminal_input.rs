//! Low-level host transport diagnostic; not the future Raven key-input API.
//! Run interactively: cargo run --example terminal_input
use neoclr::{Console, StdioConsole, TerminalOptions, TerminalRead};
use std::io;

fn main() -> io::Result<()> {
    let console = StdioConsole;
    println!("{:?}", console.capabilities());
    let mut terminal = console.open_terminal(TerminalOptions {
        capture_interrupts: true,
    })?;
    println!("Type to inspect input bytes; Ctrl+C closes and restores the terminal.");
    let mut buffer = [0; 256];
    loop {
        match terminal.try_read(&mut buffer)? {
            TerminalRead::Data(count) => {
                println!("{:02x?}", &buffer[..count]);
                if buffer[..count].contains(&3) {
                    break;
                }
            }
            TerminalRead::End => break,
            // Only this diagnostic polls. Production async integration must use
            // scheduler readiness, cancellation and invocation-owned cleanup.
            TerminalRead::Pending => std::thread::sleep(std::time::Duration::from_millis(10)),
        }
    }
    terminal.close()
}
