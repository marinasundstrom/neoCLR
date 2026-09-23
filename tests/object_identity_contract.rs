//! Characterize the runtime prerequisites for the pending Object equality API.
//! These are raw-IL contracts, not evidence that Object.ReferenceEquals is shipped.
use neoclr::{Limits, LoadedProgram, Value, assemble};

fn execute(body: &str, declarations: &str, heap_objects: usize) -> neoclr::Execution {
    let module = assemble(&format!(
        ".module Identity\n.entry Main\n.type class abstract System.Object\n.end\n{declarations}\n.function Main() -> Boolean\n{body}\nret\n.end"
    )).unwrap();
    // Identity must survive an artifact roundtrip, not depend on source bindings.
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
        .run(Limits {
            heap_objects,
            ..Limits::default()
        })
        .unwrap()
}

#[test]
fn object_and_interface_views_keep_class_identity_across_mutation_and_gc() {
    let declarations =
        ".interface Marker\n.end\n.type class Cell\n.implements Marker\n.field Number Int32\n.end";
    let mut body = String::from(
        ".local Cell cell\n.local System.Object root\n.local Marker view\nldc.i4 1\nnewobj Cell\nstloc cell\nldloc cell\ncastclass System.Object\nstloc root\nldloc cell\ncastclass Marker\nstloc view\nldloc cell\nldc.i4 9\nstfld Cell::Number\n",
    );
    for _ in 0..8 {
        body.push_str("ldc.i4 9\nnewobj Cell\npop\n");
    }
    body.push_str("ldloc root\nldloc view\nref.eq\n");
    let result = execute(&body, declarations, 2);
    assert_eq!(result.value, Value::Boolean(true));
    assert!(result.heap.collections() > 1);
    assert!(result.heap.reclaimed_objects() >= 8);
}

#[test]
fn separate_boxes_have_distinct_identity_but_a_box_alias_keeps_it() {
    for (right, same) in [("ldloc boxed", true), ("ldc.i4 42\nbox Int32", false)] {
        let body = format!(
            ".local System.Object boxed\nldc.i4 42\nbox Int32\nstloc boxed\nldloc boxed\n{right}\nref.eq"
        );
        assert_eq!(execute(&body, "", 3).value, Value::Boolean(same));
    }
}

#[test]
fn array_object_view_preserves_allocation_not_element_equality() {
    for (right, same) in [
        ("ldloc array\ncastclass System.Object", true),
        ("ldc.i4 0\nnewarr Int32\ncastclass System.Object", false),
    ] {
        let body = format!(
            ".local arrayref<Int32> array\nldc.i4 0\nnewarr Int32\nstloc array\nldloc array\ncastclass System.Object\n{right}\nref.eq"
        );
        assert_eq!(execute(&body, "", 3).value, Value::Boolean(same));
    }
}

#[test]
fn typed_nulls_compare_by_absence_not_their_declared_type() {
    let declarations = ".type class Cell\n.end";
    for (right, same) in [("ldloc cell", true), ("newobj Cell", false)] {
        let body = format!(
            ".local System.Object root\n.local Cell cell\nldloca root\ninitobj System.Object\nldloca cell\ninitobj Cell\nldloc root\n{right}\nref.eq"
        );
        assert_eq!(execute(&body, declarations, 2).value, Value::Boolean(same));
    }
}

#[test]
fn current_string_projection_allocates_wrappers_not_canonical_string_identity() {
    // This records an unresolved divergence, not the desired public API contract.
    for (right, same) in [
        ("ldloc root", true),
        ("ldloc text\ncastclass System.Object", false),
        (
            "ldloc root\ncastclass String\ncastclass System.Object",
            false,
        ),
    ] {
        let body = format!(
            ".local String text\n.local System.Object root\nldstr \"same\"\nstloc text\nldloc text\ncastclass System.Object\nstloc root\nldloc root\n{right}\nref.eq"
        );
        assert_eq!(execute(&body, "", 3).value, Value::Boolean(same));
    }
}

#[test]
fn diagnostic_allocation_ids_are_not_cross_execution_object_identity() {
    let module = assemble(".module Identity\n.entry Main\n.type class Cell\n.end\n.function Main() -> Cell\nnewobj Cell\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let first = program.run(Limits::default()).unwrap();
    let second = program.run(Limits::default()).unwrap();
    let (Value::ObjectReference(a), Value::ObjectReference(b)) = (&first.value, &second.value)
    else {
        panic!("expected class references");
    };
    assert_eq!(a.allocation_id(), b.allocation_id());
    assert_ne!(a, b);
}
