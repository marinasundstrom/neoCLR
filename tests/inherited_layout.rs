use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_type, frontend};

#[test]
fn source_frame_heap_and_base_reflection_roundtrip() {
    let m = frontend::compile(include_str!("../examples/source/inherited-layout.neo")).unwrap();
    let restored = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    let p = LoadedProgram::new(&restored).unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 1);
}

fn program(definitions: &str, body: &str) -> Result<LoadedProgram, neoclr::Fault> {
    LoadedProgram::new(&assemble(&format!(
        ".module Test\n.entry Main\n{definitions}\n.function Main() -> Int32\n{body}\nret\n.end"
    ))?)
}

#[test]
fn generic_base_fields_are_substituted_and_prefix_ordered() {
    let defs = ".type Base<T>\n.field First T\n.end\n.type Mid<T>\n.extends Base<T>\n.field Second Int32\n.end\n.type Child\n.extends Mid<Int32>\n.field Third String\n.end";
    let m = assemble(&format!(".module Test\n{defs}")).unwrap();
    let fields = m
        .instantiated_fields(&parse_type("Child").unwrap())
        .unwrap();
    assert_eq!(
        fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
        ["First", "Second", "Third"]
    );
    assert_eq!(fields[0].ty, parse_type("Int32").unwrap());
    let p = program(
        defs,
        "ldc.i4 42\nldc.i4 2\nldstr \"ok\"\nnewobj Child\nldfld Child::First",
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn default_initialization_includes_inherited_fields() {
    let p = program(
        ".type Base\n.field First Int32\n.end\n.type Child\n.extends Base\n.end",
        ".local Child x\nldloca x\ninitobj Child\nldloc x\nldfld Child::First",
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(0));
}

#[test]
fn inherited_managed_fields_are_traced_and_copies_keep_value_semantics() {
    let source = "record Base(Data: int[]&)\nrecord Child(Count: int): Base\nfunc Discard() -> () { let a = new int[1] }\nfunc Main() -> int { let data = new int[1]; data[0] = 40; var first = Child(data, 1); var copy = first; copy.Count = 2; let heap = new Child(data, copy.Count); Discard(); Discard(); return heap.Data[0] + heap.Count + first.Count - 1 }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert_eq!(
        p.run(Limits {
            heap_objects: 3,
            ..Limits::default()
        })
        .unwrap()
        .value,
        Value::Int32(42)
    );
}

#[test]
fn inherited_private_field_access_is_not_granted_to_derived_owner() {
    let defs = ".type Base\n.field private Secret Int32\n.end\n.type Child\n.extends Base\n.method static Read(Child x) -> Int32\nldarg x\nldfld Child::Secret\nret\n.end\n.end";
    let p = program(
        defs,
        ".local Child x\nldloca x\ninitobj Child\nldloc x\ncall Child::Read(Child)",
    );
    assert!(p.and_then(|p| p.run(Limits::default())).is_err());
}

#[test]
fn invalid_or_unsupported_bases_fail_loading() {
    for defs in [
        ".type A\n.extends A\n.end",
        ".type A<T>\n.extends A<A<T>>\n.end",
        ".interface A\n.end\n.type B\n.extends A\n.end",
        ".type A\n.end\n.interface B\n.extends A\n.end",
        ".type A\n.extends Int32\n.end",
        ".type A\n.field X Int32\n.end\n.type B\n.extends A\n.field X Int32\n.end",
        ".type A<T>\n.end\n.type B\n.extends A\n.end",
        ".type A\n.method instance F() -> Int32\nldc.i4 1\nret\n.end\n.end\n.type B\n.extends A\n.end",
    ] {
        assert!(program(defs, "ldc.i4 0").is_err(), "{defs}");
    }
}

#[test]
fn native_layout_and_base_value_conversions_are_not_silently_invented() {
    let defs = ".type Base\n.field X Int32\n.end\n.type Child\n.extends Base\n.field Y Int32\n.end";
    let p = program(defs, "sizeof Child");
    assert!(p.and_then(|p| p.run(Limits::default())).is_err());
    assert!(frontend::compile("record Base(X: int)\nrecord Child(Y: int): Base\nfunc Main() -> int { let child = Child(40, 2); let base: Base = child; return base.X }").is_err());
}

#[test]
fn base_absence_and_declared_field_indices_remain_explicit() {
    let source = "record Base(X: int)\nrecord Child(Y: int): Base\nfunc Main() -> int { typeof(Base).BaseType match { Some(let unexpected) => { return 1 }, None => {} }; let fields = typeof(Child).GetFields(); if fields.Length != 1 { return 2 }; return fields[0].DefinitionIndex }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(0));
}

#[test]
fn base_metadata_requires_explicit_module_reference_and_survives_linking() {
    let library = ".module Models\n.revision r1\n.type Base\n.field X Int32\n.end";
    let app = ".module App\n.references (Models#r1)\n.entry Main\n.type Child\n.extends Base\n.field Y Int32\n.end\n.function Main() -> Int32\nldc.i4 42\nldc.i4 2\nnewobj Child\nldfld Child::X\nret\n.end";
    let modules = neoclr::assembler::assemble_modules(&[app, library]).unwrap();
    let p = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    assert!(
        neoclr::assembler::assemble_modules(&[
            &app.replace(".references (Models#r1)", ".references ()"),
            library
        ])
        .is_err()
    );
}
