use neoclr::assembler::parse_type;
use neoclr::metadata::{Instruction, Type};
use neoclr::{Limits, LoadedProgram, Value, assemble};

#[test]
fn generic_function_method_packs_arguments_and_adapts_result() {
    let mut module = assemble(
        r#"
.module Projected
.entry Main
.type class abstract System.Object
.end
.function Forward<T>(arrayref<System.Object>) -> System.Object
.constraint T func
ldarg 0
ldc.i4 0
ldelem System.Object
ret
.end
.function Main() -> Int32
ldc.i4 42
call Forward<fn<Int32,Int32>>(arrayref<System.Object>)
ret
.end
"#,
    )
    .unwrap();
    module.format = 6;
    let forward = &mut module.functions[0];
    forward.parameter_names.clear();
    forward.function_type = Some(Type::MethodTypeParameter(0));
    forward.boxed_arguments = Some(Type::Named("System.Object".into()));
    if let Instruction::Call(target) = &mut module.functions[1].body[1] {
        target.parameters = vec![Type::Int32];
    } else {
        panic!("expected call");
    }
    let json = serde_json::to_string(&module).unwrap();
    let loaded = neoclr::load(&json).unwrap();
    let program = LoadedProgram::new(&loaded).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert_eq!(
        Type::function_return(parse_type("fn<Int32,Int32>").unwrap()).unwrap(),
        Type::Int32
    );
}

#[test]
fn dependent_signature_requires_function_constraint() {
    let mut module = assemble(
        r#"
.module Invalid
.function Forward<T>(arrayref<System.Object>) -> System.Object
ldarg 0
ldc.i4 0
ldelem System.Object
ret
.end
.type class abstract System.Object
.end
"#,
    )
    .unwrap();
    module.format = 6;
    module.functions[0].parameter_names.clear();
    module.functions[0].function_type = Some(Type::MethodTypeParameter(0));
    module.functions[0].boxed_arguments = Some(Type::Named("System.Object".into()));
    assert!(neoclr::load(&serde_json::to_string(&module).unwrap()).is_err());
}

#[test]
fn dependent_signature_rejects_conflicting_body_shape() {
    let mut module = assemble(
        r#"
.module Invalid
.function Forward<T>(Int32) -> Int32
.constraint T func
ldarg 0
ret
.end
.type class abstract System.Object
.end
"#,
    )
    .unwrap();
    module.format = 6;
    module.functions[0].parameter_names.clear();
    module.functions[0].function_type = Some(Type::MethodTypeParameter(0));
    module.functions[0].boxed_arguments = Some(Type::Named("System.Object".into()));
    assert!(neoclr::load(&serde_json::to_string(&module).unwrap()).is_err());
}
