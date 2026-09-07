//! Pass bootstrap Option/Result values into IL and reuse owned results.
use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::parse_function_ref,
    metadata::{Case, Type},
};

fn main() -> Result<(), neoclr::Fault> {
    let module = assemble(include_str!("union_inputs.neoil"))?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    let describe =
        program.resolve_function(&parse_function_ref("Describe(Result<Void,Error>)")?)?;
    for (case, payload) in [
        (Case::Ok, Value::Void),
        (Case::Err, Value::Error("example failure".into())),
    ] {
        let value = Value::result(payload, Type::Void, Type::Error, case);
        println!(
            "{:?}",
            describe.invoke(vec![value], Limits::default())?.value
        );
    }
    let echo = program.resolve_function(&parse_function_ref("Echo(Option<String>)")?)?;
    let value = Value::Union {
        ty: Type::Option(Box::new(Type::String)),
        case: Case::Some,
        payload: Box::new(Value::String("Hello, world!".into())),
    };
    let result = echo.invoke(vec![value], Limits::default())?;
    println!(
        "{:?}",
        echo.invoke(vec![result.value], Limits::default())?.value
    );
    Ok(())
}
