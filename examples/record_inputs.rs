//! Import an owned record, update a copy, and reuse the result as input.
use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::{parse_function_ref, parse_type},
};

fn main() -> Result<(), neoclr::Fault> {
    let source = assemble(include_str!("record_inputs.neoil"))?;
    let program = LoadedProgram::new(&source)?;
    program.verify()?;
    let point = Value::Object {
        ty: parse_type("[RecordInputs]Point")?,
        fields: vec![Value::Int32(20), Value::Int32(22)],
    };
    let sum = program.resolve_function(&parse_function_ref("Sum(Point)")?)?;
    let update = program.resolve_function(&parse_function_ref("WithX(Point,Int32)")?)?;
    let changed = update
        .invoke(vec![point.clone(), Value::Int32(40)], Limits::default())?
        .value;
    println!(
        "Original: {:?}",
        sum.invoke(vec![point], Limits::default())?.value
    );
    println!(
        "Updated: {:?}",
        sum.invoke(vec![changed], Limits::default())?.value
    );
    Ok(())
}
