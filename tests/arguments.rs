use neoclr::{Limits, Value, assemble, load, metadata::Instruction, run};

#[test]
fn sample_roundtrips_and_preserves_callers_value() {
    let module = assemble(include_str!("../examples/arguments.neoil")).unwrap();
    let numeric =
        assemble(&include_str!("../examples/arguments.neoil").replace("starg count", "starg 0"))
            .unwrap();
    assert_eq!(
        serde_json::to_value(&module).unwrap(),
        serde_json::to_value(numeric).unwrap()
    );
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(run(&loaded, Limits::default()).unwrap().output, ["6", "3"]);
}

#[test]
fn stores_obey_declared_integer_and_floating_storage_types() {
    for (ty, initial, replacement, expected) in [
        ("Byte", "ldc.i4 0", "ldc.i4 257", Value::Byte(1)),
        ("SByte", "ldc.i4 0", "ldc.i4 255", Value::SByte(-1)),
        ("UInt64", "ldc.i8 0", "ldc.i8 -1", Value::UInt64(u64::MAX)),
        (
            "Single",
            "ldc.r4 0",
            "ldc.r8 16777217",
            Value::Single(16777216.0),
        ),
        ("Void", "ldvoid", "ldvoid", Value::Void),
    ] {
        let source = format!(
            ".module Test\n.entry Main\n.function Replace({ty} value) -> {ty}\n{replacement}\nstarg value\nldarg value\nret\n.end\n.function Main() -> {ty}\n{initial}\ncall Replace({ty})\nret\n.end"
        );
        assert_eq!(
            run(&assemble(&source).unwrap(), Limits::default())
                .unwrap()
                .value,
            expected
        );
    }
}

#[test]
fn instance_stores_resolve_this_and_named_parameter_indices() {
    let source = ".module Test\n.entry Main\n.type Point\n.field X Int32\n.method instance Replace(Int32 value) -> Point\nldc.i4 42\nstarg value\nldarg value\nnewobj Point\nstarg this\nldarg this\nret\n.end\n.end\n.function Main() -> Int32\n.local Point original\nldc.i4 7\nnewobj Point\nstloc original\nldloc original\nldc.i4 1\ncall instance Point::Replace(Int32)\nldfld 0\nldloc original\nldfld 0\nadd\nret\n.end";
    let module = assemble(source).unwrap();
    assert!(matches!(
        module.functions[0].body[1],
        Instruction::StoreArg(1)
    ));
    assert!(matches!(
        module.functions[0].body[4],
        Instruction::StoreArg(0)
    ));
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(49)
    );
}

#[test]
fn pointer_slot_replacement_does_not_change_callers_pointer() {
    let source = ".module Test\n.entry Main\n.function Clear(Int32* pointer) -> Void\nptr.null Int32\nstarg pointer\nldvoid\nret\n.end\n.function Main() -> Int32\n.local Int32* pointer\nldc.i4 4\nlocalloc\nptr.cast Int32\nstloc pointer\nldloc pointer\nldc.i4 42\nstobj Int32\nldloc pointer\ncall Clear(Int32*)\npop\nldloc pointer\nldobj Int32\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn invalid_indices_names_types_and_underflow_are_rejected() {
    for operand in ["1", "missing", "this", "-1"] {
        assert!(assemble(&format!(".module Test\n.function F(Int32 value) -> Void\nldc.i4 1\nstarg {operand}\nldvoid\nret\n.end")).is_err());
    }
    for body in ["ldstr \"bad\"\nstarg value", "starg value"] {
        let module = assemble(&format!(".module Test\n.entry Main\n.function F(Int32 value) -> Void\n{body}\nldvoid\nret\n.end\n.function Main() -> Void\nldc.i4 1\ncall F(Int32)\nret\n.end")).unwrap();
        let fault = run(&module, Limits::default()).unwrap_err();
        assert_eq!(fault.function.as_deref(), Some("F"));
        assert!(fault.message.contains("expected Int32") || fault.message.contains("underflow"));
    }
    let mut module = assemble(".module Test\n.function F() -> Void\nldvoid\nret\n.end").unwrap();
    module.functions[0].body.push(Instruction::StoreArg(0));
    assert!(load(&serde_json::to_string(&module).unwrap()).is_err());
}
