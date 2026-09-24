use neoclr::{FaultCode, Limits, Value};

fn module(body: &str) -> neoclr::Module {
    neoclr::assemble(&format!(
        ".module Fields\n.entry Main\n.type class Cell\n.field Number Int32\n.end\n.function Main() -> Int32\n{body}\nret\n.end"
    )).unwrap()
}

#[test]
fn interior_reference_retains_owner_after_original_reference_is_cleared() {
    let mut body = ".local Cell cell\n.local Int32& field\nldc.i4 40\nnewobj Cell\nstloc cell\nldloc cell\nldflda Cell::Number\nstloc field\nldloca cell\ninitobj Cell\n".to_string();
    for _ in 0..8 {
        body.push_str("ldc.i4 0\nnewobj Cell\npop\n");
    }
    body.push_str("ldloc field\nldc.i4 42\nstobj Int32\nldloc field\nldobj Int32");
    let module = module(&body);
    neoclr::verify(&module).unwrap();
    let result = neoclr::run(
        &module,
        Limits {
            heap_objects: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
    assert!(result.heap.is_empty());
}

#[test]
fn null_field_address_has_null_reference_fault() {
    let module = module(
        ".local Cell cell\nldloca cell\ninitobj Cell\nldloc cell\nldflda Cell::Number\nldobj Int32",
    );
    neoclr::verify(&module).unwrap();
    assert_eq!(
        neoclr::run(&module, Limits::default()).unwrap_err().code,
        FaultCode::NullReference
    );
}

#[test]
fn field_address_does_not_bypass_private_access() {
    let mut m = module("ldc.i4 42\nnewobj Cell\nldflda Cell::Number\nldobj Int32");
    m.types[0].fields[0].visibility = neoclr::metadata::Visibility::Private;
    assert!(neoclr::verify(&m).is_err());
    assert!(neoclr::run(&m, Limits::default()).is_err());
}
