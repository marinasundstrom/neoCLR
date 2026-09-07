use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::{assemble_modules, parse_function_ref},
    library,
    metadata::{Case, Type},
};

#[test]
fn library_functions_resolve_by_signature_and_invoke_repeatedly() {
    let source = assemble(".module App\n.function Describe(Int32 value) -> String\nldstr \"integer\"\nret\n.end\n.function Describe(String value) -> String\nldarg value\nret\n.end").unwrap();
    let program = LoadedProgram::new(&source).unwrap();
    let integer = program
        .resolve_function(&parse_function_ref("Describe(Int32)").unwrap())
        .unwrap();
    let string = program
        .resolve_function(&parse_function_ref("Describe(String)").unwrap())
        .unwrap();
    assert_ne!(integer.definition(), string.definition());
    assert_eq!(integer.parameters(), [Type::Int32]);
    assert_eq!(integer.returns(), &Type::String);
    drop(source);
    for value in [1, 42] {
        assert_eq!(
            integer
                .invoke(vec![Value::Int32(value)], Limits::default())
                .unwrap()
                .value,
            Value::String("integer".into())
        );
    }
    assert_eq!(
        string
            .invoke(vec![Value::String("hello".into())], Limits::default())
            .unwrap()
            .value,
        Value::String("hello".into())
    );
}

#[test]
fn host_arguments_are_exact_storage_values_and_guest_loads_still_normalize() {
    let source = assemble(".module App\n.function Increment(Byte value) -> Byte\nldarg value\nldc.i4 1\nadd\nret\n.end\n.function Unit(Void value) -> Void\nldarg value\nret\n.end\n.function Float(Single value) -> Single\nldarg value\nret\n.end").unwrap();
    let program = LoadedProgram::new(&source).unwrap();
    let increment = program
        .resolve_function(&parse_function_ref("Increment(Byte)").unwrap())
        .unwrap();
    assert_eq!(
        increment
            .invoke(vec![Value::Byte(255)], Limits::default())
            .unwrap()
            .value,
        Value::Byte(0)
    );
    for args in [
        vec![],
        vec![Value::Byte(1), Value::Byte(2)],
        vec![Value::Int32(255)],
        vec![Value::Object {
            ty: Type::Byte,
            fields: vec![],
        }],
    ] {
        let fault = increment.invoke(args, Limits::default()).unwrap_err();
        assert_eq!(fault.function.as_deref(), Some("Increment"));
        assert!(fault.instruction.is_none());
    }
    let unit = program
        .resolve_function(&parse_function_ref("Unit(Void)").unwrap())
        .unwrap();
    assert_eq!(
        unit.invoke(vec![Value::Void], Limits::default())
            .unwrap()
            .value,
        Value::Void
    );
    assert!(unit.invoke(vec![], Limits::default()).is_err());
    let float = program
        .resolve_function(&parse_function_ref("Float(Single)").unwrap())
        .unwrap();
    assert_eq!(
        float
            .invoke(vec![Value::Single(1.25)], Limits::default())
            .unwrap()
            .value,
        Value::Single(1.25)
    );
    assert!(
        float
            .invoke(vec![Value::Double(1.25)], Limits::default())
            .is_err()
    );
}

#[test]
fn invocation_counts_only_guest_frames_and_instructions_and_recovers_after_faults() {
    let source = assemble(".module App\n.function Echo(Int32 value) -> Int32\nldarg value\nret\n.end\n.function Fail() -> Void\nfault \"failed\"\n.end").unwrap();
    let program = LoadedProgram::new(&source).unwrap();
    let echo = program
        .resolve_function(&parse_function_ref("Echo(Int32)").unwrap())
        .unwrap();
    let limits = Limits {
        frames: 1,
        instructions: 2,
        ..Limits::default()
    };
    assert_eq!(
        echo.invoke(vec![Value::Int32(42)], limits).unwrap().value,
        Value::Int32(42)
    );
    assert!(
        echo.invoke(
            vec![Value::Int32(1)],
            Limits {
                instructions: 1,
                ..Limits::default()
            }
        )
        .is_err()
    );
    let fail = program
        .resolve_function(&parse_function_ref("Fail()").unwrap())
        .unwrap();
    let fault = fail.invoke(vec![], Limits::default()).unwrap_err();
    assert_eq!(fault.function.as_deref(), Some("Fail"));
    assert_eq!(fault.instruction, Some(0));
    assert_eq!(
        echo.invoke(vec![Value::Int32(7)], Limits::default())
            .unwrap()
            .value,
        Value::Int32(7)
    );
}

#[test]
fn handles_respect_scopes_revisions_and_direct_references() {
    let modules = assemble_modules(&[
        ".module App\n.references (Helpers#r1)",
        ".module Helpers\n.revision r1\n.references ()\n.type Echo<T>\n.method static Value(T value) -> T\nldarg value\nret\n.end\n.end",
        ".module Hidden\n.function HiddenValue() -> Int32\nldc.i4 1\nret\n.end",
    ]).unwrap();
    let program =
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &modules[1..])
            .unwrap();
    let target = parse_function_ref("[Helpers]Echo<Int32>::Value(Int32) @ Helpers#r1:0").unwrap();
    let function = program.resolve_function(&target).unwrap();
    assert_eq!(function.definition().revision.as_deref(), Some("r1"));
    assert_eq!(
        function
            .invoke(vec![Value::Int32(42)], Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    let mut stale = target;
    stale.definition.as_mut().unwrap().revision = Some("r2".into());
    assert!(program.resolve_function(&stale).is_err());
    assert!(
        program
            .resolve_function(&parse_function_ref("HiddenValue()").unwrap())
            .unwrap_err()
            .message
            .contains("does not reference Hidden")
    );
    assert!(
        program
            .resolve_function(&parse_function_ref("[Helpers]Echo<!0>::Value(!0)").unwrap())
            .is_err()
    );
}

#[test]
fn unsupported_receivers_pointer_inputs_and_direct_native_targets_fail_at_resolution() {
    let module = assemble(".module App\n.type Point\n.method instance Get() -> Void\nldvoid\nret\n.end\n.end\n.function Record(Point value) -> Void\nldvoid\nret\n.end\n.function Pointer(Int32* value) -> Void\nldvoid\nret\n.end\n.function Native() -> Int32\n.pinvoke \"missing_invocation_fixture\" \"value\" cdecl\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    for target in [
        "instance Point::Get()",
        "Pointer(Int32*)",
        "Native()",
        "neoCLR.Runtime.WriteLine(String)",
    ] {
        assert!(
            program
                .resolve_function(&parse_function_ref(target).unwrap())
                .is_err(),
            "{target}"
        );
    }
}

#[test]
fn outputs_and_result_values_belong_to_each_invocation() {
    let module = assemble(".module App\n.function Report(Int32 value) -> Result<Int32,Error>\nldarg value\ncall System.Console::WriteLine(Int32)\npop\nldarg value\nok Error\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let report = program
        .resolve_function(&parse_function_ref("Report(Int32)").unwrap())
        .unwrap();
    for value in [1, 2] {
        let result = report
            .invoke(vec![Value::Int32(value)], Limits::default())
            .unwrap();
        assert_eq!(result.output, [value.to_string()]);
        assert_eq!(
            result.value,
            Value::Union {
                ty: Type::Result(Box::new(Type::Int32), Box::new(Type::Error)),
                case: Case::Ok,
                payload: Box::new(Value::Int32(value))
            }
        );
    }
}
