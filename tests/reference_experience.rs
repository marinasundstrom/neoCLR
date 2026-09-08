use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    let json = serde_json::to_string(&module).unwrap();
    LoadedProgram::new(&neoclr::load(&json).unwrap())
        .unwrap()
        .run(Limits::default())
        .unwrap()
}

#[test]
fn order_workflow_preserves_shared_inventory_and_receipt_snapshots() {
    let result = run(include_str!("../examples/source/order-workflow.neo"));
    assert_eq!(result.value, Value::Int32(0));
    assert_eq!(
        result.output,
        [
            "Purchased: Coffee",
            "24",
            "7",
            "24",
            "Out of stock",
            "Orders complete"
        ]
    );
}

#[test]
fn list_copy_shares_complete_state_across_growth() {
    let result = run(include_str!(
        "../docs/experiments/reference-experience/list-copy.neo"
    ));
    assert_eq!(result.value, Value::Int32(0));
    assert_eq!(result.output, ["20", "2", "2", "50", "50"]);
}

#[test]
fn value_element_edit_requires_writeback_but_reference_element_does_not() {
    let result = run(r#"
record Product(Stock: int)
func Main() -> int {
    var values = System.Collections.ArrayList<Product>.Allocate(0)
    values.Add(Product(5))
    var detached = values[0]
    detached.Stock = 3
    if values[0].Stock != 5 { return 1 }
    values[0] = detached
    if values[0].Stock != 3 { return 2 }
    var references = System.Collections.ArrayList<Product&>.Allocate(0)
    references.Add(new Product(5))
    let shared = references[0]
    shared.Stock = 3
    if references[0].Stock != 3 { return 3 }
    return 0
}
"#);
    assert_eq!(result.value, Value::Int32(0));
}

#[test]
fn returned_callback_rejects_frame_reference_even_when_used_in_owner_frame() {
    let module = frontend::compile(include_str!(
        "../docs/experiments/reference-experience/frame-callback.neo"
    ))
    .unwrap();
    let error = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("frame-backed references cannot escape into stored values")
    );
}

#[test]
fn explicit_copy_is_independent_but_reference_elements_keep_identity() {
    let result = run(r#"
record Product(Stock: int)
func Main() -> int {
    var values = System.Collections.ArrayList<int>.Allocate(8)
    values.Add(10)
    let readonlyValues: readonly System.Collections.ArrayList<int>& = &values
    var copy = readonlyValues.Copy()
    if copy.Capacity != 1 { return 1 }
    copy[0] = 20
    copy.Add(30)
    if values[0] != 10 || values.Count != 1 { return 2 }
    values.Add(40)
    if copy[1] != 30 { return 3 }
    var refs = System.Collections.ArrayList<Product&>.Allocate(0)
    refs.Add(new Product(5))
    var copiedRefs = refs.Copy()
    copiedRefs[0].Stock = 3
    if refs[0].Stock != 3 { return 4 }
    copiedRefs[0] = new Product(9)
    if refs[0].Stock != 3 { return 5 }
    var empty = System.Collections.ArrayList<Product&>.Allocate(0)
    var copiedEmpty = empty.Copy()
    copiedEmpty.Add(new Product(1))
    if empty.Count != 0 { return 6 }
    return 0
}
"#);
    assert_eq!(result.value, Value::Int32(0));
}
