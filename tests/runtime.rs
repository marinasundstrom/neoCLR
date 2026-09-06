use neoclr::{
    Limits, Value, assemble, load,
    metadata::{Case, Type},
    run,
};

fn program(returns: &str, body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n.function Main -> {returns}\n{body}\n.end"
    ))
    .unwrap()
}
fn eval(returns: &str, body: &str) -> Value {
    run(&program(returns, body), Limits::default())
        .unwrap()
        .value
}
fn fault(returns: &str, body: &str, message: &str) {
    let error = run(&program(returns, body), Limits::default()).unwrap_err();
    assert!(error.message.contains(message), "{error}");
    assert_eq!(error.function.as_deref(), Some("Main"));
    assert!(error.instruction.is_some());
}

#[test]
fn hello_world() {
    let module = assemble(include_str!("../examples/hello.neoil")).unwrap();
    let execution = run(&module, Limits::default()).unwrap();
    assert_eq!(execution.output, ["Hello, world!"]);
    assert_eq!(execution.value, Value::Void);
    assert!(execution.heap.is_empty());
}

#[test]
fn feature_tour() {
    let module = assemble(include_str!("../examples/features.neoil")).unwrap();
    let execution = run(&module, Limits::default()).unwrap();
    assert_eq!(
        execution.output,
        ["neoCLR feature tour", "All feature checks passed."]
    );
    assert_eq!(
        execution.value,
        Value::result(Value::Void, Type::Void, Type::Error, Case::Ok)
    );
    assert_eq!(execution.heap.len(), 1);
    assert_eq!(
        execution.heap[0],
        Value::Object {
            name: "Point".into(),
            fields: vec![Value::Int32(42), Value::Int32(20)]
        }
    );
}

#[test]
fn void_is_a_value_and_a_generic_argument() {
    assert_eq!(eval("Void", "ldvoid\nret"), Value::Void);
    let some = eval("Option<Void>", "ldvoid\nsome\nret");
    let none = eval("Option<Void>", "none Void\nret");
    assert_eq!(some.ty(), none.ty());
    assert_ne!(some, none);
    assert_eq!(
        eval("Result<Void,Void>", "ldvoid\nok Void\nret").ty(),
        Type::Result(Box::new(Type::Void), Box::new(Type::Void))
    );
    assert_eq!(
        eval("Option<Result<Void,Error>>", "ldvoid\nok Error\nsome\nret").ty(),
        Type::Option(Box::new(Type::Result(
            Box::new(Type::Void),
            Box::new(Type::Error)
        )))
    );
}

#[test]
fn arithmetic_failures_are_errors_when_requested() {
    for (body, expected) in [
        (
            "ldc.i4 1\nldc.i4 0\ncall System.Int32.Divide(int32, int32)\nldcase Err\nret",
            "DivisionByZero",
        ),
        (
            "ldc.i4 -2147483648\nldc.i4 -1\ncall System.Int32.Divide(int32, int32)\nldcase Err\nret",
            "Overflow",
        ),
        (
            "ldstr \"2147483648\"\ncall System.Int32.Parse(string)\nldcase Err\nret",
            "InvalidInt32",
        ),
    ] {
        assert_eq!(eval("Error", body), Value::Error(expected.into()));
    }
    assert_eq!(
        eval(
            "Int32",
            "ldstr \"42\"\ncall System.Int32.Parse(string)\nldcase Ok\nret"
        ),
        Value::Int32(42)
    );
}

#[test]
fn invalid_execution_faults_with_location() {
    for (returns, body, message) in [
        ("Void", "ret", "underflow"),
        ("Void", "ldvoid\nldvoid\nret", "exactly one"),
        ("Void", "ldc.i4 0\nret", "expected Void"),
        ("Void", ".local Void\nldloc 0\nret", "uninitialized"),
        (
            "Void",
            ".local Void\nldc.i4 1\nstloc 0\nldvoid\nret",
            "expected Void",
        ),
        (
            "Int32",
            "ldc.i4 2147483647\nldc.i4 1\nadd.ovf\nret",
            "overflow",
        ),
        (
            "Int32",
            "ldc.i4 -2147483648\nldc.i4 1\nsub.ovf\nret",
            "overflow",
        ),
        (
            "Int32",
            "ldc.i4 2147483647\nldc.i4 2\nmul.ovf\nret",
            "overflow",
        ),
        (
            "Void",
            "ldc.i4 1\nldvoid\nadd\nret",
            "matching integer types",
        ),
        ("Void", "none Void\nldcase Some\nret", "case mismatch"),
        ("Void", "none Void\nis.case Ok\nret", "does not belong"),
        ("Void", "ldvoid\nheap.load\nret", "requires Ref"),
        (
            "Void",
            "ldc.i4 1\nheap.new\nldvoid\nheap.store\nret",
            "expected Int32",
        ),
        (
            "Void",
            "ldvoid\ncall System.Int32.Parse(string)\nret",
            "expected String",
        ),
        ("Void", "fault \"stop\"", "stop"),
        ("Void", "ldvoid", "fell through"),
        (
            "Void",
            "ldc.i4 1\nbrtrue End\nEnd:\nldvoid\nret",
            "requires Boolean",
        ),
        ("Boolean", "ldvoid\nldc.i4 1\nceq\nret", "expected Void"),
    ] {
        fault(returns, body, message);
    }
}

#[test]
fn calls_copy_frame_owned_records_across_return() {
    let module = assemble(".module Copy\n.entry Main\n.type Box\n.field Value Int32\n.end\n.function Make -> Box\nldc.i4 7\nnewobj Box\nret\n.end\n.function Main -> Int32\ncall Make()\nldfld 0\nret\n.end").unwrap();
    let result = run(&module, Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(7));
    assert!(result.heap.is_empty());
}

#[test]
fn reference_identity_and_explicit_allocation_apply_to_scalars() {
    assert_eq!(
        eval("Boolean", "ldvoid\nheap.new\ndup\nceq\nret"),
        Value::Boolean(true)
    );
    assert_eq!(
        eval("Boolean", "ldvoid\nheap.new\nldvoid\nheap.new\nceq\nret"),
        Value::Boolean(false)
    );
    assert_eq!(
        eval(
            "Int32",
            ".local Ref<Int32>\nldc.i4 1\nheap.new\nstloc 0\nldloc 0\nldc.i4 2\nheap.store\npop\nldloc 0\nheap.load\nret"
        ),
        Value::Int32(2)
    );
}

#[test]
fn limits_stop_loops_recursion_stack_and_heap_growth() {
    let looping = program("Void", "Again:\nbr Again");
    assert!(
        run(
            &looping,
            Limits {
                instructions: 10,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .message
        .contains("instruction limit")
    );
    let recursive = program("Void", "call Main()\nret");
    assert!(
        run(
            &recursive,
            Limits {
                frames: 4,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .message
        .contains("frame limit")
    );
    let stack = program("Void", "ldvoid\nldvoid\npop\nret");
    assert!(
        run(
            &stack,
            Limits {
                stack: 1,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .message
        .contains("stack limit")
    );
    let heap = program("Ref<Void>", "ldvoid\nheap.new\nret");
    assert!(
        run(
            &heap,
            Limits {
                heap_objects: 0,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .message
        .contains("heap object limit")
    );
}

#[test]
fn field_access_is_typed_and_bounds_checked() {
    for body in [
        "ldvoid\nnewobj Box\nret",
        "ldc.i4 0\nnewobj Box\nldfld 1\nret",
        "ldc.i4 0\nnewobj Box\nldvoid\nstfld 0\nret",
    ] {
        let module = assemble(&format!(".module Fields\n.entry Main\n.type Box\n.field Value Int32\n.end\n.function Main -> Void\n{body}\n.end")).unwrap();
        assert!(run(&module, Limits::default()).is_err());
    }
}

#[test]
fn all_implemented_opcodes_have_a_sample() {
    // Keep every advertised opcode represented in the sample sources.
    let samples = [
        include_str!("../examples/hello.neoil"),
        include_str!("../examples/features.neoil"),
        include_str!("../examples/fault.neoil"),
        include_str!("../examples/pointers.neoil"),
        include_str!("../examples/native-integers.neoil"),
    ];
    let mut covered = std::collections::HashSet::new();
    for source in samples {
        for f in assemble(source).unwrap().functions {
            for op in f.body {
                covered.insert(
                    serde_json::to_value(op).unwrap()["op"]
                        .as_str()
                        .unwrap()
                        .to_owned(),
                );
            }
        }
    }
    for op in [
        "ldc.i4",
        "ldc.bool",
        "ldstr",
        "ldvoid",
        "ldarg",
        "ldloc",
        "stloc",
        "dup",
        "pop",
        "add",
        "sub",
        "mul",
        "div",
        "add.ovf",
        "add.ovf.un",
        "sub.ovf.un",
        "mul.ovf.un",
        "div.un",
        "clt.un",
        "conv.i",
        "conv.u",
        "conv.i4",
        "ptr.fromint",
        "sub.ovf",
        "mul.ovf",
        "ceq",
        "clt",
        "br",
        "brtrue",
        "call",
        "ret",
        "newobj",
        "ldfld",
        "stfld",
        "sizeof",
        "alignof",
        "heap.alloc",
        "heap.free",
        "ptr.null",
        "ptr.cast",
        "ptr.add",
        "ldflda",
        "ldobj",
        "stobj",
        "ldind.i4",
        "stind.i4",
        "heap.new",
        "heap.load",
        "heap.store",
        "some",
        "none",
        "ok",
        "err",
        "is.case",
        "ldcase",
        "error",
        "fault",
    ] {
        assert!(covered.contains(op), "sample missing {op}");
    }
}

#[test]
fn assembled_module_round_trips_through_loader() {
    let module = assemble(include_str!("../examples/features.neoil")).unwrap();
    let text = serde_json::to_string_pretty(&module).unwrap();
    let loaded = load(&text).unwrap();
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().value,
        run(&module, Limits::default()).unwrap().value
    );
}

#[test]
fn ordinary_arithmetic_preserves_cil_wrapping_behavior() {
    assert_eq!(
        eval("Int32", "ldc.i4 2147483647\nldc.i4 1\nadd\nret"),
        Value::Int32(i32::MIN)
    );
    assert_eq!(
        eval("Int32", "ldc.i4 -2147483648\nldc.i4 1\nsub\nret"),
        Value::Int32(i32::MAX)
    );
    assert_eq!(
        eval("Int32", "ldc.i4 2147483647\nldc.i4 2\nmul\nret"),
        Value::Int32(-2)
    );
}
