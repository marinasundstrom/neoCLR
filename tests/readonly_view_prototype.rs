use neoclr::{Limits, Value, assemble, run, verify};

const PROTOTYPE: &str = include_str!("../docs/experiments/readonly-views/view.neoil");

fn with_main(body: &str, result: &str) -> neoclr::Module {
    let declarations = PROTOTYPE.split(".function Main()").next().unwrap();
    assemble(&format!(
        "{declarations}.function Main() -> {result}\n{body}\nret\n.end"
    ))
    .unwrap()
}

#[test]
fn readonly_adapter_observes_original_array_without_copying_elements() {
    let module = assemble(PROTOTYPE).unwrap();
    verify(&module).unwrap();
    let result = run(&module, Limits::default()).unwrap();
    assert_eq!(result.value, Value::Boolean(true));
    // One array, two Dogs, and one adapter. No second buffer or copied elements.
    assert_eq!(result.heap.statistics().allocated_objects, 4);
}

#[test]
fn escaped_view_retains_array_and_element_across_collection() {
    let module = with_main(
        ".local AnimalView view\ncall Make()\nstloc view\n\
         newobj Dog\npop\nnewobj Dog\npop\n\
         ldloc view\nldc.i4 0\ncallvirt instance AnimalView::get_Item(Int32)\n\
         ldloc view\nldc.i4 0\ncallvirt instance AnimalView::get_Item(Int32)\nref.eq",
        "Boolean",
    );
    verify(&module).unwrap();
    let result = run(
        &module,
        Limits {
            heap_objects: 4,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Boolean(true));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn handwritten_il_cannot_expose_buffer_or_element_address_through_adapter() {
    for (body, result) in [
        ("call Make()\ncastclass arrayref<Dog>", "arrayref<Dog>"),
        (
            "call Make()\ncastclass arrayref<Animal>",
            "arrayref<Animal>",
        ),
        (
            "call Make()\ncastclass DogArrayView\nldfld DogArrayView::Data",
            "arrayref<Dog>",
        ),
        ("call Make()\nldc.i4 0\nldelema Dog", "Dog&"),
    ] {
        let module = with_main(body, result);
        // A class-to-array cast can be structurally verifiable, but execution
        // must still check the concrete allocation. Other invalid operations
        // are rejected during verification as well.
        assert!(run(&module, Limits::default()).is_err(), "{body}");
        if !body.contains("castclass arrayref") {
            assert!(verify(&module).is_err(), "{body}");
        }
    }
}

#[test]
fn readonly_adapter_retains_bounds_checks() {
    for body in [
        "call Make()\nldc.i4 1\ncallvirt instance AnimalView::get_Item(Int32)",
        "call Make()\nldc.i4 -1\ncallvirt instance AnimalView::get_Item(Int32)",
    ] {
        let module = with_main(body, "Animal");
        verify(&module).unwrap();
        assert!(run(&module, Limits::default()).is_err(), "{body}");
    }
}

#[test]
fn readonly_adapter_does_not_invent_a_non_null_element_contract() {
    let module = with_main(
        "ldc.i4 1\nnewarr Dog\nnewobj instance DogArrayView::.ctor(arrayref<Dog>)\nldc.i4 0\ncallvirt instance AnimalView::get_Item(Int32)",
        "Animal",
    );
    verify(&module).unwrap();
    assert!(matches!(
        run(&module, Limits::default()).unwrap().value,
        Value::NullObjectReference(_)
    ));
}

#[test]
fn handwritten_setter_call_is_rejected_when_resolving_the_contract() {
    let declarations = PROTOTYPE.split(".function Main()").next().unwrap();
    let source = format!(
        "{declarations}.function Main() -> void\ncall Make()\nldc.i4 0\nnewobj Dog\ncallvirt instance AnimalView::set_Item(Int32,Animal)\nret\n.end"
    );
    assert!(
        assemble(&source)
            .unwrap_err()
            .message
            .contains("unknown function overload")
    );
}
