use neoclr::metadata::Type;
use neoclr::{Limits, Value, assemble, run, verify};

fn execute(body: &str) -> Value {
    let module = assemble(&format!(".module Strings\n.entry Main\n{body}")).unwrap();
    verify(&module).unwrap();
    run(&module, Limits::default()).unwrap().value
}

#[test]
fn string_array_elements_have_null_defaults_and_can_be_replaced() {
    assert_eq!(
        execute(
            ".function Main() -> String\nldc.i4 1\nnewarr String\nldc.i4 0\nldelem String\nret\n.end"
        ),
        Value::NullObjectReference(Type::String)
    );
    assert_eq!(
        execute(
            ".function Main() -> String\n.local arrayref<String> a\nldc.i4 1\nnewarr String\nstloc a\nldloc a\nldc.i4 0\nldstr \"text\"\nstelem String\nldloc a\nldc.i4 0\nldelem String\nret\n.end"
        ),
        Value::String("text".into())
    );
}

#[test]
fn string_local_and_field_defaults_are_null_not_empty() {
    assert_eq!(
        execute(
            ".function Main() -> String\n.local String s\nldloca s\ninitobj String\nldloc s\nret\n.end"
        ),
        Value::NullObjectReference(Type::String)
    );
    assert_eq!(
        execute(
            ".type class Holder\n.field Text String\n.method instance .ctor() -> noresult\nret\n.end\n.end\n.function Main() -> String\nnewobj instance Holder::.ctor()\nldfld Holder::Text\nret\n.end"
        ),
        Value::NullObjectReference(Type::String)
    );
}
