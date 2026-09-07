use neoclr::{Limits, LoadedProgram, assemble, assembler::parse_function_ref};
fn main() -> Result<(), neoclr::Fault> {
    let module = assemble(include_str!("union_inputs.neoil"))?;
    let program = LoadedProgram::new(&module)?;
    let success = program
        .resolve_function(&parse_function_ref("Success()")?)?
        .invoke(vec![], Limits::default())?
        .value;
    let output = program
        .resolve_function(&parse_function_ref("Describe(System.Result<Void,Error>)")?)?
        .invoke(vec![success], Limits::default())?;
    println!("{:?}", output.value);
    Ok(())
}
