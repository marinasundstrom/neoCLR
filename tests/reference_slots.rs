use neoclr::{Limits, LoadedProgram, Value, assemble, load};
fn program(extra: &str, body: &str, returns: &str) -> Result<LoadedProgram, neoclr::Fault> {
    let module = assemble(&format!(
        ".module App\n.entry Main\n{extra}\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))?;
    LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap())?)
}
#[test]
fn sample_passes_and_forwards_an_explicit_reference() {
    let p = LoadedProgram::new(
        &assemble(include_str!("../examples/reference_parameters.neoil")).unwrap(),
    )
    .unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.output, ["42"]);
    assert_eq!(result.memory.live_allocations(), 0);
}
#[test]
fn string_and_generic_record_slots_do_not_require_native_layout() {
    let extra = ".type Box<T>\n.field Value T\n.end\n.type Assign<T>\n.method static Set(T& destination,T value) -> Void\nldarg destination\nldarg value\nstobj T\nldvoid\nret\n.end\n.end";
    for (ty, before, after, result) in [
        (
            "String",
            "ldstr \"before\"",
            "ldstr \"after\"",
            Value::String("after".into()),
        ),
        (
            "Box<String>",
            "ldstr \"before\"\nnewobj Box<String>",
            "ldstr \"after\"\nnewobj Box<String>",
            Value::Object {
                ty: neoclr::assembler::parse_type("Box<String>").unwrap(),
                fields: vec![Value::String("after".into())],
            },
        ),
    ] {
        let body = format!(
            ".local {ty} value\n{before}\nstloc value\nldloca value\n{after}\ncall Assign<{ty}>::Set({ty}&,{ty})\npop\nldloc value"
        );
        let p = program(extra, &body, ty).unwrap();
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().value, result);
    }
}
#[test]
fn indirect_store_initializes_a_local_and_normalizes_small_values() {
    let p = program(
        "",
        ".local Byte value\nldloca value\nldc.i4 257\nstobj Byte\nldloc value",
        "Byte",
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Byte(1));
}
#[test]
fn writable_aliases_observe_writes_immediately() {
    let extra = ".function Change(Int32& left,Int32& right) -> Int32\nldarg left\nldc.i4 42\nstobj Int32\nldarg right\nldobj Int32\nret\n.end";
    let p = program(
        extra,
        ".local Int32 value\nldc.i4 0\nstloc value\nldloca value\ndup\ncall Change(Int32&,Int32&)",
        "Int32",
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn argument_address_targets_the_callees_value_copy() {
    let extra = ".function Change(Int32 value) -> Int32\nldarga value\nldc.i4 42\nstobj Int32\nldarg value\nret\n.end";
    let p = program(extra, "ldc.i4 7\ncall Change(Int32)", "Int32").unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn uninitialized_reference_arguments_fault_even_without_verification() {
    let extra = ".function Read(Int32& value) -> Int32\nldarg value\nldobj Int32\nret\n.end";
    let p = program(
        extra,
        ".local Int32 value\nldloca value\ncall Read(Int32&)",
        "Int32",
    )
    .unwrap();
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("uninitialized")
    );
}
#[test]
fn reference_escape_and_rebinding_are_rejected_in_metadata_or_execution() {
    for (extra, body, returns) in [
        ("", ".local Int32 value\nldloca value", "Int32&"),
        ("", ".local Int32& value\nldvoid", "Void"),
        (".type Bad\n.field Value Int32&\n.end", "ldvoid", "Void"),
        (
            ".type Box<T>\n.field Value T\n.end",
            ".local Box<Int32&> value\nldvoid",
            "Void",
        ),
        (
            "",
            ".local Int32 value\nldloca value\nvalue.pack Int32&",
            "System.Value",
        ),
        (
            ".function Bad(Int32& value) -> Void\nldarg value\nstarg value\nldvoid\nret\n.end",
            "ldvoid",
            "Void",
        ),
        (
            ".function Bad(Int32& value) -> Void\nldarga value\npop\nldvoid\nret\n.end",
            "ldvoid",
            "Void",
        ),
        (
            "",
            ".local Int32 value\nldloca value\nheap.new\npop\nldvoid",
            "Void",
        ),
    ] {
        assert!(
            program(extra, body, returns)
                .and_then(|p| p.run(Limits::default()))
                .is_err(),
            "accepted {body}"
        );
    }
}
