//! Prepare and verify once, then execute the same program with fresh state twice.
use neoclr::{Limits, LoadedProgram, assemble};

fn main() -> Result<(), neoclr::Fault> {
    let source = assemble(include_str!("hello.neoil"))?;
    let program = LoadedProgram::new(&source)?;
    program.verify()?;
    drop(source);

    for _ in 0..2 {
        let execution = program.run(Limits::default())?;
        for line in execution.output {
            println!("{line}");
        }
    }
    Ok(())
}
