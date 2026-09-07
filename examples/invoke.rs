//! Resolve library functions once and invoke them with typed host arguments.
use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};

fn main() -> Result<(), neoclr::Fault> {
    let module = assemble(include_str!("invoke.neoil"))?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    let greet = program.resolve_function(&parse_function_ref("Greet(String)")?)?;
    for line in greet
        .invoke(
            vec![Value::String("Hello, world!".into())],
            Limits::default(),
        )?
        .output
    {
        println!("{line}");
    }
    let double = program.resolve_function(&parse_function_ref("Double(Int32)")?)?;
    for value in [21, 30] {
        println!(
            "{:?}",
            double
                .invoke(vec![Value::Int32(value)], Limits::default())?
                .value
        );
    }
    Ok(())
}
