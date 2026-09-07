//! Format a preserved Fault snapshot after dropping its loaded program.
use neoclr::{Limits, LoadedProgram, assemble};

fn main() -> Result<(), neoclr::Fault> {
    let fault = {
        let module = assemble(include_str!("stack_trace.neoil"))?;
        let program = LoadedProgram::new(&module)?;
        program.verify()?;
        program
            .run(Limits::default())
            .expect_err("sample must fault")
    };
    println!("{fault}");
    Ok(())
}
