use neoclr::{
    Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref, metadata::Type,
};

const ECHO: &str = ".module App\n.type Cell\n.field Number Int32\n.end\n.type Pair\n.field Left System.Value\n.field Right System.Value\n.end\n.function Echo(System.Value item) -> System.Value\nldarg item\nret\n.end";
fn erased(value: Value) -> Value {
    Value::Erased(Box::new(value))
}
fn cell(ty: Type, fields: Vec<Value>) -> Value {
    Value::Object { ty, fields }
}

#[test]
fn guest_constructed_ordinary_result_can_cross_the_host_boundary_again() {
    let module = assemble(include_str!("../examples/ordinary_union_inputs.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let create = program
        .resolve_function(&parse_function_ref("Create(Int32)").unwrap())
        .unwrap();
    let read = program
        .resolve_function(&parse_function_ref("Read(System.Result<Int32,String>)").unwrap())
        .unwrap();
    let original = create
        .invoke(vec![Value::Int32(42)], Limits::default())
        .unwrap()
        .value;
    for _ in 0..2 {
        assert_eq!(
            read.invoke(vec![original.clone()], Limits::default())
                .unwrap()
                .value,
            Value::Int32(42)
        );
    }
}

#[test]
fn nested_erasure_normalizes_scoped_record_tags_without_converting_primitives() {
    let module = assemble(ECHO).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let echo = program
        .resolve_function(&parse_function_ref("Echo(System.Value)").unwrap())
        .unwrap();
    let input = erased(erased(cell(
        Type::Scoped {
            module: "App".into(),
            name: "Cell".into(),
            arguments: vec![],
        },
        vec![Value::Int32(42)],
    )));
    assert_eq!(
        echo.invoke(vec![input], Limits::default()).unwrap().value,
        erased(erased(cell(
            Type::Named("Cell".into()),
            vec![Value::Int32(42)]
        )))
    );
    let byte = erased(Value::Byte(255));
    assert_eq!(
        echo.invoke(vec![byte.clone()], Limits::default())
            .unwrap()
            .value,
        byte
    );
}

#[test]
fn malformed_erased_trees_are_rejected_before_guest_execution() {
    let module = assemble(ECHO).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let echo = program
        .resolve_function(&parse_function_ref("Echo(System.Value)").unwrap())
        .unwrap();
    for input in [
        Value::Int32(1),
        erased(cell(Type::Named("Missing".into()), vec![])),
        erased(cell(Type::TypeParameter(0), vec![])),
        erased(cell(Type::String, vec![])),
        erased(cell(Type::Value, vec![])),
        erased(cell(Type::Named("Cell".into()), vec![])),
        erased(cell(
            Type::Named("Cell".into()),
            vec![Value::String("wrong".into())],
        )),
        erased(cell(
            Type::Scoped {
                module: "Wrong".into(),
                name: "Cell".into(),
                arguments: vec![],
            },
            vec![Value::Int32(1)],
        )),
        erased(erased(
            neoclr::run(
                &assemble(
                    ".module Heap
.entry Main
.function Main() -> Int32&
ldc.i4 7
heap.new
ret
.end",
                )
                .unwrap(),
                Limits::default(),
            )
            .unwrap()
            .value,
        )),
    ] {
        let fault = echo.invoke(vec![input], Limits::default()).unwrap_err();
        assert!(fault.message.contains("invocation argument 0"), "{fault}");
        assert!(fault.stack_trace.is_none());
    }
}

#[test]
fn pointer_payloads_remain_unsupported_even_when_returned_by_guest_code() {
    let module = assemble(&format!(
        "{ECHO}\n.function Pointer() -> Int32*\nptr.null Int32\nret\n.end"
    ))
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let pointer = program
        .resolve_function(&parse_function_ref("Pointer()").unwrap())
        .unwrap()
        .invoke(vec![], Limits::default())
        .unwrap()
        .value;
    let echo = program
        .resolve_function(&parse_function_ref("Echo(System.Value)").unwrap())
        .unwrap();
    assert!(
        echo.invoke(vec![erased(pointer)], Limits::default())
            .unwrap_err()
            .message
            .contains("pointer and managed reference")
    );
}

#[test]
fn import_depth_and_complexity_budgets_are_shared_across_erased_payloads() {
    let module = assemble(ECHO).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let echo = program
        .resolve_function(&parse_function_ref("Echo(System.Value)").unwrap())
        .unwrap();
    let deep = (0..100).fold(Value::Void, |value, _| erased(value));
    let fault = echo.invoke(vec![deep], Limits::default()).unwrap_err();
    assert!(fault.message.contains("depth or complexity"));
    fn tree(depth: usize) -> Value {
        if depth == 0 {
            erased(Value::Void)
        } else {
            erased(cell(
                Type::Named("Pair".into()),
                vec![tree(depth - 1), tree(depth - 1)],
            ))
        }
    }
    let fault = echo.invoke(vec![tree(14)], Limits::default()).unwrap_err();
    assert!(fault.message.contains("complexity"));
    assert!(fault.stack_trace.is_none());
}

#[test]
fn import_validates_shape_without_certifying_a_union_convention() {
    let module = assemble(include_str!("../examples/ordinary_union_inputs.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let read = program
        .resolve_function(&parse_function_ref("Read(System.Result<Int32,String>)").unwrap())
        .unwrap();
    // A trusted host can supply private fields; shape checking is not constructor provenance.
    let invalid_carrier = cell(
        Type::Constructed {
            definition: "System.Result".into(),
            arguments: vec![Type::Int32, Type::String],
        },
        vec![erased(Value::Void)],
    );
    assert_eq!(
        read.invoke(vec![invalid_carrier], Limits::default())
            .unwrap()
            .value,
        Value::Int32(-1)
    );
}
