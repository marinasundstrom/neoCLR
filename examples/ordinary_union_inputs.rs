use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};

fn main() -> Result<(), neoclr::Fault> {
    let module = assemble(include_str!("ordinary_union_inputs.neoil"))?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    let create = program.resolve_function(&parse_function_ref("Create(Int32)")?)?;
    let read =
        program.resolve_function(&parse_function_ref("Read(System.Result<Int32,String>)")?)?;
    let result = create
        .invoke(vec![Value::Int32(42)], Limits::default())?
        .value;
    let answer = read.invoke(vec![result], Limits::default())?.value;
    assert_eq!(answer, Value::Int32(42));
    println!("{answer:?}");
    Ok(())
}
