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

// Exercise the sample's actual service declarations with focused alternative inputs.
fn workflow_with_main(main: &str) -> String {
    let (declarations, _) = include_str!("../examples/source/order-workflow.neo")
        .split_once("func Main()")
        .unwrap();
    format!("{declarations}{main}")
}

#[test]
fn rejected_purchases_preserve_inventory_and_do_not_notify() {
    let result = run(&workflow_with_main(
        r#"
func Main() -> int {
    let product = new Product("Coffee", 12, 5)
    var notifications = ConsoleNotifications()
    let Error(zero) = Purchase(product, 0, &notifications) else { return 1 }
    let InvalidQuantity(zeroDetails) = zero else { return 2 }
    let Error(negative) = Purchase(product, -2, &notifications) else { return 3 }
    let InvalidQuantity(negativeDetails) = negative else { return 4 }
    let Error(oversized) = Purchase(product, 6, &notifications) else { return 5 }
    let OutOfStock(stockDetails) = oversized else { return 6 }
    if zeroDetails.quantity != 0 || negativeDetails.quantity != -2 { return 7 }
    if stockDetails.available != 5 || stockDetails.requested != 6 { return 8 }
    if product.Stock != 5 || product.Price != 12 { return 9 }
    return 0
}
"#,
    ));
    assert_eq!(result.value, Value::Int32(0));
    assert!(result.output.is_empty());
}

#[test]
fn catalog_lookup_preserves_identity_and_purchase_exhausts_exact_stock() {
    let result = run(&workflow_with_main(
        r#"
func Main() -> int {
    var products = System.Collections.ArrayList<Product&>.Allocate(0)
    let product = new Product("Tea", 8, 3)
    products.Add(product)
    var notifications = ConsoleNotifications()
    let Some(found) = FindProduct(&products, "Tea") else { return 1 }
    if !ReferenceEquals(found, product) { return 2 }
    if let Some(_) = FindProduct(&products, "Milk") { return 3 }
    let Ok(receipt) = Purchase(found, 3, &notifications) else { return 4 }
    product.Price = 99
    if product.Stock != 0 || receipt.Total != 24 || receipt.Quantity != 3 { return 5 }
    let Error(error) = Purchase(product, 1, &notifications) else { return 6 }
    let OutOfStock(details) = error else { return 7 }
    if details.available != 0 || details.requested != 1 { return 8 }
    return 0
}
"#,
    ));
    assert_eq!(result.value, Value::Int32(0));
    assert_eq!(result.output, ["Purchased: Tea", "24"]);
}

#[test]
fn nested_domain_error_needs_an_explicit_carrier_boundary() {
    let source = workflow_with_main(
        r#"
func Reject() -> Result<Receipt, PurchaseError> { return Error(InvalidQuantity(0)) }
func Main() -> () {}
"#,
    );
    assert!(frontend::compile(&source).is_err());
    let source = workflow_with_main(
        r#"
func Reject() -> Result<Receipt, PurchaseError> { return Error<PurchaseError>(InvalidQuantity(0)) }
func Main() -> int {
    let Error(error) = Reject() else { return 1 }
    let InvalidQuantity(details) = error else { return 2 }
    return details.quantity
}
"#,
    );
    assert_eq!(run(&source).value, Value::Int32(0));
}
