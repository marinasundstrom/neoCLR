use neoclr::{
    Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref, metadata::Type,
};

fn main() -> Result<(), neoclr::Fault> {
    let module = assemble(include_str!("instance_invocation.neoil"))?;
    let program = LoadedProgram::new(&module)?;
    let get = program.resolve_function(&parse_function_ref("instance Box<Int32>::Get()")?)?;
    let update = program.resolve_function(&parse_function_ref(
        "instance Box<Int32>::WithValue(Int32)",
    )?)?;
    let original = Value::Object {
        ty: Type::Constructed {
            definition: "Box".into(),
            arguments: vec![Type::Int32],
        },
        fields: vec![Value::Int32(21)],
    };
    let updated = update
        .invoke_instance(original.clone(), vec![Value::Int32(42)], Limits::default())?
        .value;
    println!(
        "Original: {:?}",
        get.invoke_instance(original, vec![], Limits::default())?
            .value
    );
    println!(
        "Updated: {:?}",
        get.invoke_instance(updated, vec![], Limits::default())?
            .value
    );
    Ok(())
}
