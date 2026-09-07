use neoclr::{Limits, LoadedProgram, Value, assemble};

fn program(extra: &str, body: &str, returns: &str) -> LoadedProgram {
    let module = assemble(&format!(
        ".module App\n.entry Main\n{extra}\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))
    .unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    LoadedProgram::new(&module).unwrap()
}

fn limits(objects: usize) -> Limits {
    Limits {
        heap_objects: objects,
        ..Limits::default()
    }
}

#[test]
fn returned_interior_reference_roots_heap_storage_across_collection() {
    let p =
        LoadedProgram::new(&assemble(include_str!("../examples/heap_references.neoil")).unwrap())
            .unwrap();
    p.verify().unwrap();
    let result = p.run(limits(2)).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.is_empty());
    assert_eq!(result.heap.reclaimed_objects(), 3);
}

#[test]
fn managed_heap_access_share_storage_identity_and_field_paths() {
    let p = program(
        ".type Counter\n.field Age Int32\n.end",
        ".local Counter& owner\n.local Int32& age\nldc.i4 7\nnewobj Counter\nheap.new\nstloc owner\nldloc owner\nldflda Counter::Age\nstloc age\nldloc owner\nldc.i4 42\nnewobj Counter\nstobj Counter\nldloc age\nldobj Int32",
        "Int32",
    );
    p.verify().unwrap();
    assert_eq!(p.run(limits(1)).unwrap().value, Value::Int32(42));
    let p = program(
        "",
        ".local Int32& owner\nldc.i4 7\nheap.new\nstloc owner\nldloc owner\nldc.i4 42\nstobj Int32\nldloc owner\nldobj Int32",
        "Int32",
    );
    p.verify().unwrap();
    assert_eq!(p.run(limits(1)).unwrap().value, Value::Int32(42));
}

#[test]
fn heap_reference_identity_depends_on_root_and_field_path() {
    for (body, expected) in [
        (
            ".local Int32& owner\nldc.i4 7\nheap.new\nstloc owner\nldloc owner\nldloc owner\nceq",
            true,
        ),
        ("ldc.i4 7\nheap.new\nldc.i4 7\nheap.new\nceq", false),
    ] {
        let p = program("", body, "Boolean");
        p.verify().unwrap();
        assert_eq!(p.run(limits(2)).unwrap().value, Value::Boolean(expected));
    }
}

#[test]
fn heap_owner_tracing_preserves_transitive_edges_when_only_field_is_referenced() {
    let extra = ".type Holder\n.field Age Int32\n.field Child Int32&\n.end\n.function Make() -> Int32&\nldc.i4 42\nldc.i4 9\nheap.new\nnewobj Holder\nheap.new\nldflda Holder::Age\nret\n.end";
    let p = program(
        extra,
        ".local Int32& age\ncall Make()\nstloc age\nldc.i4 0\nheap.new\npop\nldc.i4 1\nheap.new\npop\nldloc age\nldobj Int32",
        "Int32",
    );
    p.verify().unwrap();
    let result = p.run(limits(3)).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.reclaimed_objects(), 4);
}

#[test]
fn ordinary_local_reference_still_cannot_escape_while_heap_reference_can() {
    let p = program(
        ".function Bad() -> Int32&\n.local Int32 x\nldc.i4 7\nstloc x\nldloca x\nret\n.end",
        "call Bad()\nldobj Int32",
        "Int32",
    );
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("current frame")
    );
    let p = program(
        ".function Good() -> Int32&\nldc.i4 42\nheap.new\nret\n.end",
        "call Good()\nldobj Int32",
        "Int32",
    );
    p.verify().unwrap();
    assert_eq!(p.run(limits(1)).unwrap().value, Value::Int32(42));
}

#[test]
fn frame_references_cannot_hide_in_fields_or_erased_payloads() {
    for (extra, tail, returns) in [
        (
            ".type Holder\n.field Item Int32&\n.end",
            "newobj Holder",
            "Holder",
        ),
        ("", "value.pack Int32&", "System.Value"),
        (
            ".type Holder\n.field Item Int32&\n.end",
            "newobj Holder\nheap.new",
            "Holder&",
        ),
    ] {
        let p = program(
            extra,
            &format!(".local Int32 local\nldc.i4 7\nstloc local\nldloca local\n{tail}"),
            returns,
        );
        let fault = p.run(Limits::default()).unwrap_err();
        assert!(fault.message.contains("frame-backed references"), "{fault}");
    }
}

#[test]
fn generic_heap_values_and_managed_interface_receivers_survive_collection() {
    let extra = ".type Box<T>\n.field Value T\n.method static Make(T value) -> Box<T>&\nldarg value\nnewobj Box<T>\nheap.new\nret\n.end\n.end";
    let p = program(
        extra,
        "ldstr \"hello\"\ncall Box<String>::Make(String)\nldflda Box<String>::Value\nldobj String",
        "String",
    );
    p.verify().unwrap();
    assert_eq!(
        p.run(limits(1)).unwrap().value,
        Value::String("hello".into())
    );
    let extra = ".interface Read\n.method instance Get() -> Int32\n.end\n.end\n.type Counter\n.implements Read\n.field Age Int32\n.method instance Get() -> Int32\nldarg this\nldfld Counter::Age\nret\n.end\n.end";
    let p = program(
        extra,
        ".local Read& view\nldc.i4 42\nnewobj Counter\nheap.new\ninterface.borrow Read\nstloc view\nldc.i4 1\nheap.new\npop\nldc.i4 2\nheap.new\npop\nldloc view\ncallvirt instance Read::Get()",
        "Int32",
    );
    p.verify().unwrap();
    assert_eq!(p.run(limits(2)).unwrap().value, Value::Int32(42));
}

#[test]
fn old_allocation_artifacts_are_rejected_before_deserialization() {
    let module = assemble(
        ".module App\n.entry Main\n.function Main() -> Int32&\nldc.i4 7\nheap.new\nret\n.end",
    )
    .unwrap();
    let mut value = serde_json::to_value(module).unwrap();
    value["format"] = 4.into();
    assert!(
        neoclr::load(&value.to_string())
            .unwrap_err()
            .message
            .contains("expected 5")
    );
    for instruction in ["heap.load", "heap.store", "unbox Int32"] {
        assert!(
            assemble(&format!(
                ".module App\n.function Main() -> Void\n{instruction}\nret\n.end"
            ))
            .is_err()
        );
    }
}

#[test]
fn host_can_inspect_returned_fields_only_in_the_owning_execution() {
    let p = program(
        ".type Counter\n.field Age Int32\n.end",
        "ldc.i4 42\nnewobj Counter\nheap.new\nldflda Counter::Age",
        "Int32&",
    );
    p.verify().unwrap();
    let first = p.run(limits(1)).unwrap();
    let second = p.run(limits(1)).unwrap();
    let Value::SlotReference(reference) = &first.value else {
        panic!("expected managed reference")
    };
    assert_eq!(
        first.heap.read_reference(reference).unwrap(),
        Value::Int32(42)
    );
    assert!(second.heap.read_reference(reference).is_err());
}
