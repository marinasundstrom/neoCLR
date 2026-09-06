use neoclr::{Limits, Value, assemble, load, run};

#[test]
fn field_names_normalize_to_indices_in_all_three_instructions() {
    let source = ".module Test\n.entry Main\n.function Main() -> Int32\n.local Point point\n.local Point* storage\nldc.i4 1\nldc.i4 2\nnewobj Point\nldc.i4 42\nstfld Point::Y\nstloc point\nldc.i4 1\nheap.alloc Point\nstloc storage\nldloc storage\nldloc point\nstobj Point\nldloc storage\nldflda Point::Y\nldind.i4\nldloc point\nldfld Point::X\nadd\nldloc storage\nheap.free\npop\nret\n.end\n.type Point\n.field X Int32\n.field Y Int32\n.end";
    let named = assemble(source).unwrap();
    let numeric = assemble(&source.replace("Point::Y", "1").replace("Point::X", "0")).unwrap();
    assert_eq!(
        serde_json::to_value(&named).unwrap(),
        serde_json::to_value(numeric).unwrap()
    );
    let loaded = load(&serde_json::to_string(&named).unwrap()).unwrap();
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().value,
        Value::Int32(43)
    );
}

#[test]
fn field_and_slot_name_scopes_are_independent() {
    let source = ".module Test\n.entry Main\n.type First\n.field value Int32\n.end\n.type Second\n.field padding Int32\n.field value Int32\n.method static Read(First value) -> Int32\n.local Int32 value\nldarg value\nldfld First::value\nstloc value\nldc.i4 0\nldloc value\nnewobj Second\nldfld Second::value\nret\n.end\n.end\n.function Main() -> Int32\nldc.i4 42\nnewobj First\ncall Second::Read(First)\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn unknown_and_malformed_field_names_report_source_lines() {
    for operand in [
        "Missing::X",
        "Point::Missing",
        "Point::x",
        "X",
        "Point::",
        "Point::X::Y",
        "Ptr<Point>::X",
    ] {
        let source = format!(
            ".module Test\n.type Point\n.field X Int32\n.end\n.function F(Point value) -> Int32\nldarg value\nldfld {operand}\nret\n.end"
        );
        let error = assemble(&source).unwrap_err();
        assert!(error.message.contains("line 7:"), "{operand}: {error}");
    }
}
