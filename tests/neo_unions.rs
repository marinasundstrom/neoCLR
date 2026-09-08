use neoclr::{Limits, LoadedProgram, Value, frontend};

fn program(source: &str) -> LoadedProgram {
    let module = frontend::compile(source).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
}

#[test]
fn both_declaration_forms_construct_convert_and_match() {
    let execution = program(include_str!("../examples/source/unions.neo"))
        .run(Limits::default())
        .unwrap();
    assert_eq!(execution.value, Value::Int32(0));
    assert_eq!(execution.output, ["1234", "cash", "9", "16", "6"]);
}

#[test]
fn inline_variants_are_distinct_nested_types_and_constructors_define_acceptance() {
    let module = frontend::compile(include_str!("../examples/source/unions.neo")).unwrap();
    let shape = module
        .type_definition(&neoclr::assembler::parse_type("Shape").unwrap())
        .unwrap();
    let circle = module
        .type_definition(&neoclr::assembler::parse_type("Shape.Circle").unwrap())
        .unwrap();
    assert_eq!(circle.declaring_type, shape.definition);
    assert_ne!(shape.definition, circle.definition);
    let constructors: Vec<_> = module
        .functions
        .iter()
        .filter(|f| {
            f.owner.as_ref() == Some(&neoclr::assembler::parse_type("Shape").unwrap())
                && f.name.ends_with("..ctor")
        })
        .collect();
    assert_eq!(constructors.len(), 3);
    assert!(constructors.iter().all(|f| f.parameters.len() == 1));
}

#[test]
fn variants_keep_copy_rules_and_reference_payloads_across_carriers() {
    let execution = program(
        r#"
record Item(Value: int)
union Message {
    case Data(item: Item&, count: int)
    case Empty
}
union Envelope(Message | Message.Empty)
func Add(item: Item&, count: int) -> int { item.Value = item.Value + count; return item.Value }
func Main() -> int {
    let item = new Item(40)
    let data = Message.Data(item, 2)
    var message: Message = data
    let envelope: Envelope = message
    let total = envelope match {
        Message(let value) => value match {
            Data(let payload) => Add(payload.item, payload.count),
            Empty => 0
        },
        Empty => -1
    }
    message = Message.Empty()
    if !(message match { Empty(let empty) => true, Data(_) => false }) { return -2 }
    return total
}
"#,
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(execution.value, Value::Int32(42));
}

#[test]
fn invalid_declarations_conversions_and_matches_are_rejected() {
    for source in [
        "union Empty()\nfunc Main() -> int { return 0 }",
        "union Bad(Unknown)\nfunc Main() -> int { return 0 }",
        "union Bad(Bad)\nfunc Main() -> int { return 0 }",
        "record A()\nunion Bad(A | A)\nfunc Main() -> int { return 0 }",
        "union Bad { case A\ncase A }\nfunc Main() -> int { return 0 }",
        "union U { case A\ncase B }\nfunc Main() -> U { return 42 }",
        "union U { case A }\nfunc Main() -> U { return U(42) }",
        "union U { case A }\nfunc Main() -> U { return default(U) }",
        "union U { case A\ncase B }\nfunc Main() -> int { let u: U = U.A(); return u match { A => 1 } }",
        "union U { case A }\nfunc Main() -> int { let u: U = U.A(); return u match { Missing => 1 } }",
        "union U { case A }\nfunc Main() -> int { let u: U = U.A(); return u match { A => 1, A => 2 } }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
}

#[test]
fn source_union_storage_cannot_hide_frame_references() {
    let module = frontend::compile(
        r#"
record Borrow(Value: int&)
union Stored(Borrow)
func Main() -> Stored {
    var value = 42
    return Borrow(&value)
}
"#,
    )
    .unwrap();
    let fault = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert!(fault.message.contains("frame-backed"), "{fault}");
}

#[test]
fn returned_union_retains_heap_payload_under_collection_pressure() {
    let execution = program(
        r#"
record Item(Value: int)
union Message { case Data(item: Item&) }
func Make(value: int) -> Message { return Message.Data(new Item(value)) }
func Main() -> int {
    let retained = Make(42)
    for i in 0..<40 { let ignored = Make(i) }
    return retained match { Data(let data) => data.item.Value }
}
"#,
    )
    .run(Limits {
        heap_objects: 8,
        ..Limits::default()
    })
    .unwrap();
    assert_eq!(execution.value, Value::Int32(42));
    assert!(execution.heap.statistics().collections > 0);
}

#[test]
fn union_name_can_be_shadowed_by_a_callable_local() {
    let execution = program(
        r#"
union Choice { case Number(value: int) }
func Main() -> int {
    let Choice: System.Func<int,int> = value => value + 1
    return Choice(41)
}
"#,
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(execution.value, Value::Int32(42));
}
