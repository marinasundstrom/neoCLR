//! Assemble separate sources, then execute one explicit module set.
use neoclr::{Limits, LoadedProgram, assembler::assemble_modules, library};

fn main() -> Result<(), neoclr::Fault> {
    let modules = assemble_modules(&[
        include_str!("modules/app.neoil"),
        include_str!("modules/operations.neoil"),
        include_str!("modules/models.neoil"),
    ])?;
    let program = LoadedProgram::with_modules(&modules[0], library::system()?, &modules[1..])?;
    program.verify()?;
    for line in program.run(Limits::default())?.output {
        println!("{line}");
    }
    Ok(())
}
