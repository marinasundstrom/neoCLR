//! Load a dependency pinned to an exact declared revision.
use neoclr::{Limits, LoadedProgram, assembler::assemble_modules, library};

fn main() -> Result<(), neoclr::Fault> {
    let modules = assemble_modules(&[
        include_str!("revisions/app.neoil"),
        include_str!("revisions/answers.neoil"),
    ])?;
    let program = LoadedProgram::with_modules(&modules[0], library::system()?, &modules[1..])?;
    program.verify()?;
    for line in program.run(Limits::default())?.output {
        println!("{line}");
    }
    Ok(())
}
