use neoclr::{Limits, LoadedProgram, Value, assemble, frontend};

#[test]
fn loop_declarations_allow_different_array_and_nested_record_shapes() {
    let source = r#"
record Bag(Items: int[])
func Main() -> int {
    var total = 0
    for i in 0..<4 {
        let items = array(i, 7)
        let bag = Bag(items)
        total = total + bag.Items.Length
    }
    return total
}
"#;
    let module = frontend::compile(source).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(6)
    );
}

#[test]
fn reset_requires_no_live_alias_and_restores_uninitialized_state() {
    for prefix in ["ldloca value", "ldloca value\nldc.i4 0\nldelema Int32"] {
        let source = format!(
            ".module App\n.entry Main\n.function Main() -> Void\n.local Int32[] value\nldc.i4 1\nldc.i4 0\narray.create Int32\nstloc value\n{prefix}\nlocal.reset value\npop\nldvoid\nret\n.end"
        );
        let module = assemble(&source).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        let fault = program.run(Limits::default()).unwrap_err();
        assert!(fault.message.contains("references are live"), "{fault:?}");
    }
    let source = ".module App\n.entry Main\n.function Main() -> Int32\n.local Int32 value\nldc.i4 1\nstloc value\nlocal.reset value\nldloc value\nret\n.end";
    let module = assemble(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(
        program
            .verify()
            .unwrap_err()
            .message
            .contains("not initialized")
    );
    assert!(
        program
            .run(Limits::default())
            .unwrap_err()
            .message
            .contains("uninitialized")
    );
}
