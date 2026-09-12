use neoclr::{Limits, Value, assemble, run, verify};
const TYPES: &str = ".type class Counter\n.field Age Int32\n.method instance .ctor() -> noresult\nret\n.end\n.end\n.type class Holder\n.field Item Counter\n.method instance .ctor() -> noresult\nret\n.end\n.end\n";
fn module(main: &str, returns: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Defaults\n.entry Main\n{TYPES}.function Main() -> {returns}\n{main}\n.end"
    ))
    .unwrap()
}
#[test]
fn class_fields_default_to_typed_null_without_allocating_targets() {
    let m = module(
        "newobj instance Holder::.ctor()\nldfld Holder::Item\nret",
        "Counter",
    );
    verify(&m).unwrap();
    let execution = run(&m, Limits::default()).unwrap();
    assert_eq!(
        execution.value,
        Value::NullObjectReference(neoclr::metadata::Type::Named("Counter".into()))
    );
    assert_eq!(execution.heap.statistics().allocated_objects, 1);
    assert_eq!(execution.heap.len(), 0);
}
#[test]
fn initobj_resets_class_slot_without_modifying_other_aliases() {
    let m = module(
        ".local Counter original\n.local Counter alias\nnewobj instance Counter::.ctor()\nstloc original\nldloc original\nstloc alias\nldloca original\ninitobj Counter\nldloc alias\nldc.i4 42\nstfld Counter::Age\nldloc original\ndup\nref.eq\npop\nldloc alias\nldfld Counter::Age\nret",
        "Int32",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn null_reference_comparison_and_field_access_are_explicit() {
    for (right, expected) in [
        ("ldloc empty", true),
        ("newobj instance Counter::.ctor()", false),
    ] {
        let m = module(
            &format!(
                ".local Counter empty\nldloca empty\ninitobj Counter\nldloc empty\n{right}\nref.eq\nret"
            ),
            "Boolean",
        );
        verify(&m).unwrap();
        assert_eq!(
            run(&m, Limits::default()).unwrap().value,
            Value::Boolean(expected)
        );
    }
    for (code, returns) in [
        ("ldfld Counter::Age\nret", "Int32"),
        ("ldc.i4 1\nstfld Counter::Age\nret", "noresult"),
    ] {
        let m = module(
            &format!(".local Counter empty\nldloca empty\ninitobj Counter\nldloc empty\n{code}"),
            returns,
        );
        verify(&m).unwrap();
        assert!(
            run(&m, Limits::default())
                .unwrap_err()
                .message
                .contains("null object reference")
        );
    }
}
#[test]
fn null_is_distinct_from_uninitialized_storage() {
    let m = module(".local Counter empty\nldloc empty\nret", "Counter");
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}
