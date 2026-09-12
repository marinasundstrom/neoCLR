use neoclr::{Limits, Value, assemble, run, verify};
const TYPES: &str =
    ".type class Counter\n.field Age Int32\n.end\n.type Point\n.field X Int32\n.end\n";
fn module(body: &str) -> neoclr::Module {
    assemble(&format!(".module Classes\n.entry Main\n{TYPES}{body}")).unwrap()
}
#[test]
fn class_aliases_share_mutations_across_static_calls() {
    let m = module(
        ".function Main() -> Int32\n.local Counter first\n.local Counter alias\nldc.i4 1\nnewobj Counter\nstloc first\nldloc first\nstloc alias\nldloc alias\ncall Change(Counter)\npop\nldloc first\nldfld Counter::Age\nret\n.end\n.function Change(Counter) -> Void\nldarg 0\nldc.i4 42\nstfld Counter::Age\npop\nldvoid\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn value_assignment_preserves_independent_contents() {
    let m = module(
        ".function Main() -> Int32\n.local Point first\n.local Point copy\nldc.i4 1\nnewobj Point\nstloc first\nldloc first\nstloc copy\nldloc copy\nldc.i4 42\nstfld Point::X\nstloc copy\nldloc first\nldfld Point::X\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(1));
}
#[test]
fn returned_class_reference_outlives_its_frame_and_remains_rooted() {
    let m = module(
        ".function Main() -> Counter\ncall Make()\nret\n.end\n.function Make() -> Counter\nldc.i4 42\nnewobj Counter\nret\n.end",
    );
    verify(&m).unwrap();
    let execution = run(&m, Limits::default()).unwrap();
    let Value::ObjectReference(reference) = execution.value else {
        panic!("expected object handle")
    };
    assert_eq!(
        reference.target(),
        &neoclr::metadata::Type::Named("Counter".into())
    );
    assert_eq!(
        execution.heap.get(reference.allocation_id()),
        Some(Value::Object {
            ty: reference.target().clone(),
            fields: vec![Value::Int32(42)]
        })
    );
}
#[test]
fn reference_equality_distinguishes_aliases_and_equal_payloads() {
    for (right, expected) in [("ldloc first", true), ("ldc.i4 1\nnewobj Counter", false)] {
        let m = module(&format!(
            ".function Main() -> Boolean\n.local Counter first\nldc.i4 1\nnewobj Counter\nstloc first\nldloc first\n{right}\nref.eq\nret\n.end"
        ));
        verify(&m).unwrap();
        assert_eq!(
            run(&m, Limits::default()).unwrap().value,
            Value::Boolean(expected)
        );
    }
}
#[test]
fn class_creation_obeys_heap_budget() {
    let m = module(".function Main() -> Counter\nldc.i4 1\nnewobj Counter\nret\n.end");
    let limits = Limits {
        heap_objects: 0,
        ..Limits::default()
    };
    assert!(
        run(&m, limits)
            .unwrap_err()
            .message
            .contains("heap object limit")
    );
}

#[test]
fn value_containing_class_reference_copies_handle_and_gc_traces_it() {
    let m = module(
        ".type Holder\n.field Item Counter\n.end\n.function Main() -> Holder\n.local Holder holder\nldc.i4 42\nnewobj Counter\nnewobj Holder\nstloc holder\nldc.i4 7\nnewobj Counter\npop\nldloc holder\nret\n.end",
    );
    verify(&m).unwrap();
    let execution = run(
        &m,
        Limits {
            heap_objects: 1,
            ..Limits::default()
        },
    );
    // The nested handle keeps the first object live when the second allocation is attempted.
    assert!(execution.unwrap_err().message.contains("heap object limit"));
    let execution = run(&m, Limits::default()).unwrap();
    assert_eq!(execution.heap.len(), 1);
    assert_eq!(execution.heap.statistics().reclaimed_objects, 1);
}

#[test]
fn rebinding_parameter_does_not_rebind_caller_but_byref_does() {
    for (parameter, argument, body, expected) in [
        (
            "Counter",
            "ldloc first",
            "ldc.i4 9\nnewobj Counter\nstarg 0",
            1,
        ),
        (
            "Counter&",
            "ldloca first",
            "ldarg 0\nldc.i4 9\nnewobj Counter\nstobj Counter",
            9,
        ),
    ] {
        let m = module(&format!(
            ".function Main() -> Int32\n.local Counter first\nldc.i4 1\nnewobj Counter\nstloc first\n{argument}\ncall Replace({parameter})\npop\nldloc first\nldfld Counter::Age\nret\n.end\n.function Replace({parameter}) -> Void\n{body}\nldvoid\nret\n.end"
        ));
        verify(&m).unwrap();
        assert_eq!(
            run(&m, Limits::default()).unwrap().value,
            Value::Int32(expected)
        );
    }
}

#[test]
fn unsupported_class_metadata_and_default_initialization_are_rejected() {
    let baseline = module(".function Main() -> Void\nldvoid\nret\n.end");
    for representation in [
        neoclr::metadata::Representation::Interface,
        neoclr::metadata::Representation::Runtime,
    ] {
        let mut m = baseline.clone();
        m.types[0].representation = representation;
        assert!(verify(&m).is_err());
        assert!(run(&m, Limits::default()).is_err());
    }
    let json = serde_json::to_string(&baseline).unwrap();
    let roundtrip: neoclr::Module = serde_json::from_str(&json).unwrap();
    assert!(roundtrip.is_reference_type(&neoclr::metadata::Type::Named("Counter".into())));
    let ty = neoclr::metadata::Type::Named("Counter".into());
    assert!(neoclr::memory::layout(&baseline, &ty).is_err());
    // Reserve a slot and attempt to create a class through value default initialization.
    let invalid = assemble(&format!(
        ".module Invalid\n.entry Main\n{TYPES}.function Main() -> Void\n.local Counter c\nldloca c\ninitobj Counter\nldvoid\nret\n.end"
    ));
    if let Ok(m) = invalid {
        assert!(run(&m, Limits::default()).is_err());
    }
}

#[test]
fn host_cannot_supply_an_owned_record_for_a_class_parameter() {
    use neoclr::{LoadedProgram, assembler::parse_function_ref};
    let m = module(
        ".function Main() -> Void\nldvoid\nret\n.end\n.function Read(Counter) -> Int32\nldarg 0\nldfld Counter::Age\nret\n.end",
    );
    let program = LoadedProgram::new(&m).unwrap();
    let error = program
        .resolve_function(&parse_function_ref("Read(Counter)").unwrap())
        .unwrap_err();
    assert!(error.message.contains("class instances"));
}

#[test]
fn nominal_classes_cannot_satisfy_notreference_constraints() {
    use neoclr::metadata::{ConstraintKind, GenericConstraint};
    let mut m = module(
        ".function Main() -> Counter\nldc.i4 1\nnewobj Counter\ncall Identity<Counter>(Counter)\nret\n.end\n.function Identity<T>(T) -> T\nldarg 0\nret\n.end",
    );
    m.functions
        .iter_mut()
        .find(|f| f.name == "Identity")
        .unwrap()
        .generic_constraints
        .push(GenericConstraint {
            parameter: 0,
            kind: ConstraintKind::NotReference,
        });
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}
