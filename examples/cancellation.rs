//! Cooperatively stop an interpreter invocation from another host thread.
use neoclr::{
    CancellationToken, ExecutionOptions, Limits, LoadedProgram, assemble,
    assembler::parse_function_ref,
};

fn main() -> Result<(), neoclr::Fault> {
    let module = assemble(
        ".module App\n.function Spin() -> Void\nLoop:\nbr Loop\n.end\n.function Ready() -> Void\nldvoid\nret\n.end",
    )?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    let spin = program.resolve_function(&parse_function_ref("Spin()")?)?;
    let token = CancellationToken::new();
    let options = ExecutionOptions {
        limits: Limits {
            instructions: usize::MAX,
            ..Limits::default()
        },
        cancellation: Some(token.clone()),
        ..ExecutionOptions::default()
    };
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| match spin.invoke(vec![], options) {
            Ok(_) => println!("Completed"),
            Err(fault) => println!("{fault}"),
        });
        std::thread::sleep(std::time::Duration::from_millis(20));
        token.cancel();
        worker.join().expect("host worker panicked");
    });
    let ready = program.resolve_function(&parse_function_ref("Ready()")?)?;
    println!(
        "Program remains usable: {:?}",
        ready.invoke(vec![], Limits::default())?.value
    );
    Ok(())
}
