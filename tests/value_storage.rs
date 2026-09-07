use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
};

const CARRIER: &str = include_str!("../examples/ordinary_carrier.neoil");

#[test]
fn generic_erasure_substitutes_small_primitive_storage_types() {
    let src = ".module App\n.entry Main\n.type Convert<T>\n.method static Pack(T item) -> System.Value\nldarg item\nvalue.pack T\nret\n.end\n.method static Read(System.Value item) -> T\nldarg item\nvalue.unpack T\nret\n.end\n.end\n.function Main() -> Int32\nldc.i4 257\ncall Convert<Byte>::Pack(Byte)\ncall Convert<Byte>::Read(System.Value)\nret\n.end";
    let module = assemble(src).unwrap();
    let loaded = LoadedProgram::new(&module).unwrap();
    loaded.verify().unwrap();
    assert_eq!(
        loaded.run(Limits::default()).unwrap().value,
        Value::Int32(1)
    );
}

#[test]
fn erasing_a_pointer_does_not_retain_its_allocation() {
    let module = program(
        ".local System.Value saved\nldc.i4 1\nheap.alloc Int32\ndup\nvalue.pack Int32*\nstloc saved\nheap.free\npop\nldloc saved\nvalue.unpack Int32*\nldobj Int32\nret",
        "Int32",
    );
    let loaded = LoadedProgram::new(&module).unwrap();
    loaded.verify().unwrap();
    let fault = loaded.run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("use after free"), "{fault}");
}

fn program(body: &str, returns: &str) -> neoclr::Module {
    assemble(&format!(
        ".module App\n.entry Main\n.function Main() -> {returns}\n{body}\n.end"
    ))
    .unwrap()
}

#[test]
fn ordinary_generic_carrier_round_trips_and_preserves_equal_payload_type_variants() {
    let module = assemble(CARRIER).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let loaded = LoadedProgram::new(&module).unwrap();
    loaded.verify().unwrap();
    assert_eq!(
        loaded.run(Limits::default()).unwrap().output,
        ["Success", "42", "Failure", "7"]
    );
    let graph = loaded
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 32)
        .unwrap();
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::ValueStorage)
    );
    assert!(
        !graph
            .required_services()
            .contains(&RuntimeService::NativeAllocation)
    );
}

#[test]
fn erasure_preserves_storage_identity_then_normalizes_extraction_for_the_stack() {
    for (literal, ty, expected) in [
        ("ldc.i4 257", "Byte", Value::Int32(1)),
        ("ldc.i4 -1", "UInt32", Value::Int32(-1)),
        ("ldvoid", "Void", Value::Void),
        ("ldstr \"hello\"", "String", Value::String("hello".into())),
    ] {
        let returns = match ty {
            "Byte" | "UInt32" => "Int32",
            other => other,
        };
        let module = program(
            &format!(
                "{literal}\nvalue.pack {ty}\ndup\nvalue.is {ty}\nbrtrue Good\nfault \"lost identity\"\nGood:\nvalue.unpack {ty}\nret"
            ),
            returns,
        );
        let loaded = LoadedProgram::new(&module).unwrap();
        loaded.verify().unwrap();
        assert_eq!(loaded.run(Limits::default()).unwrap().value, expected);
    }
    let module = program("ldc.i4 1\nvalue.pack Byte\nvalue.is Int32\nret", "Boolean");
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Boolean(false)
    );
}

#[test]
fn a_failed_query_returns_false_and_a_wrong_extraction_is_a_fault() {
    let module = program("ldvoid\nvalue.pack Void\nvalue.is String\nret", "Boolean");
    let loaded = LoadedProgram::new(&module).unwrap();
    loaded.verify().unwrap();
    assert_eq!(
        loaded.run(Limits::default()).unwrap().value,
        Value::Boolean(false)
    );
    let module = program(
        "ldvoid\nvalue.pack Void\nvalue.unpack String\nret",
        "String",
    );
    let loaded = LoadedProgram::new(&module).unwrap();
    // A checked extraction may fail dynamically; no false static proof of its payload.
    loaded.verify().unwrap();
    let fault = loaded.run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("requested String"));
    assert_eq!(
        fault.stack_trace.unwrap().frames[0].location,
        neoclr::CodeLocation::IlInstruction(2)
    );
}

#[test]
fn erasure_is_explicit_and_operands_are_type_checked() {
    for (body, returns) in [
        ("ldc.i4 42\nret", "System.Value"),
        ("ldc.i4 42\nvalue.pack String\nret", "System.Value"),
        ("ldc.i4 42\nvalue.is Int32\nret", "Boolean"),
        ("ldc.i4 42\nvalue.unpack Int32\nret", "Int32"),
    ] {
        let module = program(body, returns);
        let loaded = LoadedProgram::new(&module).unwrap();
        assert!(loaded.verify().is_err());
        assert!(loaded.run(Limits::default()).is_err());
    }
    assert!(
        assemble(
            ".module App\n.function Main() -> Void\nldvoid\nvalue.pack Missing\npop\nret\n.end"
        )
        .is_err()
    );
}

#[test]
fn absent_empty_payload_and_nested_carriers_need_no_default_values() {
    let types = ".type Empty\n.end\n.type Present<T>\n.field Payload T\n.end";
    for (body, expected) in [
        (
            "newobj Empty\nvalue.pack Empty\nvalue.is Present<Void>\nret",
            false,
        ),
        (
            "ldvoid\nnewobj Present<Void>\nvalue.pack Present<Void>\nvalue.is Present<Void>\nret",
            true,
        ),
        (
            "ldvoid\nnewobj Present<Void>\nvalue.pack Present<Void>\nnewobj Present<System.Value>\nvalue.pack Present<System.Value>\nvalue.unpack Present<System.Value>\nldfld 0\nvalue.is Present<Void>\nret",
            true,
        ),
    ] {
        let src =
            format!(".module App\n.entry Main\n{types}\n.function Main() -> Boolean\n{body}\n.end");
        let module = assemble(&src).unwrap();
        let loaded = LoadedProgram::new(&module).unwrap();
        loaded.verify().unwrap();
        assert_eq!(
            loaded.run(Limits::default()).unwrap().value,
            Value::Boolean(expected)
        );
    }
}

#[test]
fn copied_erased_records_are_independent_values() {
    let src = ".module App\n.entry Main\n.type Cell\n.field Number Int32\n.end\n.function Main() -> Int32\n.local System.Value original\nldc.i4 7\nnewobj Cell\nvalue.pack Cell\nstloc original\nldloc original\nvalue.unpack Cell\nldc.i4 42\nstfld 0\nvalue.pack Cell\npop\nldloc original\nvalue.unpack Cell\nldfld 0\nret\n.end";
    let module = assemble(src).unwrap();
    let loaded = LoadedProgram::new(&module).unwrap();
    loaded.verify().unwrap();
    assert_eq!(
        loaded.run(Limits::default()).unwrap().value,
        Value::Int32(7)
    );
}

#[test]
fn erasure_does_not_grant_native_layout_or_implicit_host_conversion() {
    assert!(
        assemble(".module App\n.function Main() -> Int32\nsizeof System.Value\nret\n.end").is_err()
    );
    let module = assemble(
        ".module App\n.function Echo(System.Value input) -> System.Value\nldarg input\nret\n.end",
    )
    .unwrap();
    let loaded = LoadedProgram::new(&module).unwrap();
    let echo = loaded
        .resolve_function(&parse_function_ref("Echo(System.Value)").unwrap())
        .unwrap();
    assert!(
        echo.invoke(vec![Value::Int32(42)], Limits::default())
            .is_err()
    );
    let value = Value::Erased(Box::new(Value::Int32(42)));
    assert_eq!(
        echo.invoke(vec![value.clone()], Limits::default())
            .unwrap()
            .value,
        value
    );
}

#[test]
fn recursive_erasure_is_bounded_without_using_native_allocation_limits() {
    let mut body = "ldvoid\nvalue.pack Void\n".to_owned();
    body.push_str(&"value.pack System.Value\n".repeat(65));
    body.push_str("ret");
    let module = program(&body, "System.Value");
    let loaded = LoadedProgram::new(&module).unwrap();
    loaded.verify().unwrap();
    let fault = loaded.run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("depth or complexity limit"));
}
