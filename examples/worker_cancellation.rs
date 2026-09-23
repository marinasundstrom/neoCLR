//! Host cancellation during delivery of an already completed worker's output.
//! Run with `cargo run --example worker_cancellation`.
use neoclr::{CancellationToken, Console, ExecutionOptions, LoadedProgram, Value, assemble};
use std::io::Write;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct CancelAfterFirstLine {
    token: CancellationToken,
    delivered: Mutex<Vec<String>>,
}

impl Console for CancelAfterFirstLine {
    fn read_byte(&self) -> std::io::Result<Option<u8>> {
        Ok(None)
    }

    fn write_line(&self, text: &str) -> std::io::Result<()> {
        self.delivered.lock().unwrap().push(text.into());
        let mut output = std::io::stdout().lock();
        writeln!(output, "{text}")?;
        output.flush()?;
        self.token.cancel();
        Ok(())
    }
}

fn main() -> Result<(), neoclr::Fault> {
    let module = assemble(include_str!("worker_cancellation.neoil"))?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    let console = Arc::new(CancelAfterFirstLine::default());
    let fault = program
        .run(ExecutionOptions {
            cancellation: Some(console.token.clone()),
            console: Some(console.clone()),
            ..Default::default()
        })
        .unwrap_err();
    assert_eq!(fault.message, "execution cancelled");
    assert_eq!(*console.delivered.lock().unwrap(), ["First worker line"]);
    println!("Invocation: {}", fault.message);

    // Cancellation belongs to the invocation, not the reusable loaded program.
    let again = program.run(ExecutionOptions::default())?;
    assert_eq!(again.value, Value::String("completed result".into()));
    assert_eq!(again.output, ["First worker line", "Second worker line"]);
    println!("Fresh invocation: both lines and result delivered");
    Ok(())
}
