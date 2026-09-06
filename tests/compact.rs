use neoclr::{Limits, Value, assemble, load, run};

fn body(source: &str) -> serde_json::Value {
    let module = assemble(&format!(".module Test\n.function F(Int32 a, Int32 b, Int32 c, Int32 d) -> Int32\n.local Int32 a\n.local Int32 b\n.local Int32 c\n.local Int32 d\n{source}\n.end")).unwrap();
    serde_json::to_value(&module.functions[0].body).unwrap()
}

#[test]
fn fixed_constants_and_slots_emit_canonical_instructions() {
    for (suffix, value) in [
        ("m1", -1),
        ("0", 0),
        ("1", 1),
        ("2", 2),
        ("3", 3),
        ("4", 4),
        ("5", 5),
        ("6", 6),
        ("7", 7),
        ("8", 8),
    ] {
        assert_eq!(
            body(&format!("ldc.i4.{suffix}\nret")),
            body(&format!("ldc.i4 {value}\nret"))
        );
    }
    for opcode in ["ldarg", "ldloc", "stloc"] {
        for index in 0..4 {
            assert_eq!(
                body(&format!("{opcode}.{index}\nldc.i4.0\nret")),
                body(&format!("{opcode} {index}\nldc.i4 0\nret"))
            );
        }
    }
}

#[test]
fn short_forms_support_names_and_enforce_encoded_ranges() {
    for opcode in ["ldarg", "starg", "ldloc", "stloc"] {
        assert_eq!(
            body(&format!("{opcode}.s d\nldc.i4 0\nret")),
            body(&format!("{opcode} 3\nldc.i4 0\nret"))
        );
    }
    for value in [-128, -1, 0, 127] {
        assert_eq!(
            body(&format!("ldc.i4.s {value}\nret")),
            body(&format!("ldc.i4 {value}\nret"))
        );
    }
    let parameters = (0..257)
        .map(|i| format!("Int32 p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    for (name, valid) in [
        ("255", true),
        ("p255", true),
        ("256", false),
        ("p256", false),
    ] {
        for opcode in ["ldarg", "starg"] {
            assert_eq!(assemble(&format!(".module Test\n.function F({parameters}) -> Void\nldc.i4.0\n{opcode}.s {name}\nldvoid\nret\n.end")).is_ok(), valid);
        }
    }
}

#[test]
fn malformed_aliases_and_invalid_slots_are_rejected() {
    for instruction in [
        "ldc.i4.s -129",
        "ldc.i4.s 128",
        "ldc.i4.s",
        "ldc.i4.9",
        "ldc.i4.01",
        "ldc.i4.1 2",
        "ldarg.4",
        "starg.0",
        "ldloc.0 extra",
        "ldarg.s missing",
        "stloc.s 256",
        "ldloc.s -1",
        "ldarg.0",
    ] {
        assert!(
            assemble(&format!(
                ".module Test\n.function F() -> Void\n{instruction}\nldvoid\nret\n.end"
            ))
            .is_err(),
            "{instruction}"
        );
    }
}

#[test]
fn compact_sample_roundtrips_and_executes() {
    let module = assemble(include_str!("../examples/compact.neoil")).unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let execution = run(&module, Limits::default()).unwrap();
    assert_eq!(execution.output, ["42"]);
    assert_eq!(execution.value, Value::Void);
}

#[test]
fn compact_instance_slots_and_labels_keep_canonical_indices() {
    let source = ".module Test\n.type Number\n.field Value Int32\n.method instance Add(Int32 amount) -> Int32\nbr Body\nldc.i4.m1\nret\nBody:\nldarg.s this\nldfld 0\nldarg.1\nadd\nstarg.s amount\nldarg.s amount\nret\n.end\n.end\n.entry Main\n.function Main() -> Int32\nldc.i4.s 40\nnewobj Number\nldc.i4.2\ncall instance Number::Add(Int32)\nret\n.end";
    let module = assemble(source).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
