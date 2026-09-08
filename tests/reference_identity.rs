use neoclr::{Limits, LoadedProgram, Value, assemble, frontend};

#[test]
fn frame_heap_and_interface_identity_are_location_based() {
    let module =
        frontend::compile(include_str!("../examples/source/reference-identity.neo")).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let result = LoadedProgram::new(&loaded)
        .unwrap()
        .run(Limits::default())
        .unwrap();
    assert_eq!(
        result.output,
        [
            "Same location through interface",
            "Copy has a different location",
            "Same field location",
            "Object and field are distinct locations"
        ]
    );
}

#[test]
fn array_locations_and_live_identity_survive_collection() {
    let source = r#"
func Main() -> bool {
    let values = new int[2]
    let first = &values[0]
    for i in 0..<12 { let temporary = new int[1] }
    return ReferenceEquals(first, &values[0]) && !ReferenceEquals(first, &values[1])
}
"#;
    let module = frontend::compile(source).unwrap();
    let result = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits {
            heap_objects: 3,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Boolean(true));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn identity_requires_two_live_initialized_managed_references() {
    for body in [
        "ldc.i4 0\nldc.i4 0",
        "ptr.null Int32\nptr.null Int32",
        "ldloca a\nldloca b",
    ] {
        let source = format!(
            ".module App\n.entry Main\n.function Main() -> Boolean\n.local Int32 a\n.local Int32 b\nldc.i4 1\nstloc a\n{body}\nref.eq\nret\n.end"
        );
        let module = assemble(&source).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        assert!(program.verify().is_err());
        assert!(program.run(Limits::default()).is_err());
    }
    assert!(frontend::compile("func Main() -> bool { return ReferenceEquals(1, 1) }").is_err());
}

#[test]
fn value_comparison_and_user_functions_are_not_replaced_by_identity() {
    let source = "func Main() -> bool { var a = 1\nvar b = 1\nlet x = &a\nlet y = &b\nreturn x == y && !ReferenceEquals(x,y) }";
    let module = frontend::compile(source).unwrap();
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Boolean(true)
    );
    let source = "func ReferenceEquals(a: int,b: int) -> bool { return false }\nfunc Main() -> bool { return ReferenceEquals(1,1) }";
    let module = frontend::compile(source).unwrap();
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Boolean(false)
    );
}
