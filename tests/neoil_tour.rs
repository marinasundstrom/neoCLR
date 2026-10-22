use neoclr::{Limits, Value, assemble, run, verify};

#[test]
fn website_il_tour_assembles_verifies_and_executes() {
    let module = assemble(include_str!("../samples/neoil-tour/main.neoil")).unwrap();
    verify(&module).unwrap();
    let serialized = serde_json::to_string(&module).unwrap();
    let loaded = neoclr::load(&serialized).unwrap();
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
