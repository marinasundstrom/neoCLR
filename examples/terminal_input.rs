//! Low-level host transport diagnostic; not the future Raven key-input API.
//! Run interactively: cargo run --example terminal_input
use neoclr::{Console, StdioConsole, TerminalOptions, TerminalRead};
use std::alloc::{GlobalAlloc, Layout, System};
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn main() -> io::Result<()> {
    let console = StdioConsole;
    println!("{:?}", console.capabilities());
    let mut terminal = console.open_terminal(TerminalOptions {
        capture_interrupts: true,
    })?;
    println!("Type to inspect input bytes; Ctrl+C closes and restores the terminal.");
    let mut buffer = [0; 256];
    loop {
        let before = ALLOCATIONS.load(Ordering::Relaxed);
        let read = terminal.read(&mut buffer, None);
        let allocations = ALLOCATIONS.load(Ordering::Relaxed) - before;
        // This single-threaded diagnostic measures the read only, excluding setup/output.
        println!("read allocations: {allocations}");
        match read? {
            TerminalRead::Data(count) => {
                println!("{:02x?}", &buffer[..count]);
                if buffer[..count].contains(&3) {
                    break;
                }
            }
            TerminalRead::End => break,
            TerminalRead::Pending => unreachable!("unbounded blocking read timed out"),
        }
    }
    terminal.close()
}
