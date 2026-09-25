use neoclr::{Limits, Value, assemble, load, run};

const CELL: &str = ".type class Cell<T>\n.field deferred Pending T\n.method instance .ctor() -> noresult\nret\n.end\n.end\n";

fn app(body: &str, declarations: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Deferred\n.entry Main\n{declarations}{CELL}.function Main() -> Int32\n{body}\n.end"
    ))
    .unwrap()
}

#[test]
fn deferred_nondefaultable_field_can_be_assigned_after_construction() {
    let m = app(
        ".local Cell<Value> cell\nnewobj instance Cell<Value>::.ctor()\nstloc cell\nldloc cell\nldc.i4 42\nvalue.pack Int32\nstfld Cell<Value>::Pending\nldloc cell\nldfld Cell<Value>::Pending\nvalue.unpack Int32\nret",
        "",
    );
    let serialized = serde_json::to_string(&m).unwrap();
    let restored = load(&serialized).unwrap();
    assert_eq!(
        run(&restored, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn deferred_read_before_assignment_faults_and_ordinary_constructor_rule_remains() {
    let m = app(
        "newobj instance Cell<Value>::.ctor()\nldfld Cell<Value>::Pending\nvalue.unpack Int32\nret",
        "",
    );
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("uninitialized")
    );
    let mut ordinary = m;
    ordinary
        .types
        .iter_mut()
        .find(|t| t.name == "Cell")
        .unwrap()
        .fields[0]
        .deferred = false;
    assert!(
        run(&ordinary, Limits::default())
            .unwrap_err()
            .message
            .contains("constructor returned with an uninitialized field")
    );
}

#[test]
fn deferred_fields_are_not_value_type_partial_initialization() {
    assert!(assemble(".module Invalid\n.type Cell\n.field deferred Pending Value\n.end").is_err());
    let mut m = app("ldc.i4 0\nret", "");
    m.types
        .iter_mut()
        .find(|t| t.name == "Cell")
        .unwrap()
        .is_reference_type = false;
    assert!(load(&serde_json::to_string(&m).unwrap()).is_err());
}

#[test]
fn deferred_payload_retains_heap_references_across_collection() {
    let pressure = "ldc.i4 0\nnewobj Item\npop\n".repeat(24);
    let m = app(
        &format!(
            ".local Cell<Value> cell\nnewobj instance Cell<Value>::.ctor()\nstloc cell\nldloc cell\nldc.i4 42\nnewobj Item\nvalue.pack Item\nstfld Cell<Value>::Pending\n{pressure}ldloc cell\nldfld Cell<Value>::Pending\nvalue.unpack Item\nldfld Item::Number\nret"
        ),
        ".type class Item\n.field Number Int32\n.end\n",
    );
    let result = run(
        &m,
        Limits {
            heap_objects: 4,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
    assert_eq!(result.heap.len(), 0);
}
