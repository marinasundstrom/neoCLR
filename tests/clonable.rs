use neoclr::{Limits, LoadedProgram, Value, assemble, load};

const SAMPLE: &str = include_str!("../examples/clonable.neoil");

#[test]
fn generic_clone_dispatch_round_trips_and_preserves_the_original() {
    let module = assemble(SAMPLE).unwrap();
    let artifact = serde_json::to_string(&module).unwrap();
    let program = LoadedProgram::new(&load(&artifact).unwrap()).unwrap();
    program.verify().unwrap();
    let execution = program.run(Limits::default()).unwrap();
    assert_eq!(execution.output, ["original", "changed clone"]);
    assert_eq!(execution.value, Value::Void);
    assert_eq!(execution.memory.live_allocations(), 0);
}

#[test]
fn clone_requires_declared_conformance_and_matching_receiver_and_return() {
    for source in [
        SAMPLE.replace("instance byref Clone", "instance Clone"),
        SAMPLE.replace("Clone() -> Snapshot<T>", "Clone() -> String"),
    ] {
        assert!(assemble(&source).is_err());
    }
    let undeclared = SAMPLE.replace(".implements System.Clonable<Snapshot<T>>", "");
    let module = assemble(&undeclared).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().is_err());
    assert!(program.run(Limits::default()).is_err());
}

#[test]
fn ordinary_value_copies_do_not_invoke_clone() {
    let source = r#"
.module CopyContract
.entry Main
.type Item
    .implements System.Clonable<Item>
    .field Number Int32
    .method instance byref Clone() -> Item
        fault "Clone must be explicit"
    .end
.end
.function Main() -> Int32
    .local Item original
    .local Item copied
    ldc.i4 42
    newobj Item
    stloc original
    ldloc original
    dup
    pop
    stloc copied
    ldloc copied
    ldfld Item::Number
    ret
.end
"#;
    let module = assemble(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );

    let explicit = source.replace(
        "ldloc copied\n    ldfld Item::Number",
        "ldloca copied\n    interface.borrow System.Clonable<Item>\n    callvirt instance System.Clonable<Item>::Clone()\n    ldfld Item::Number",
    );
    let module = assemble(&explicit).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let fault = program.run(Limits::default()).unwrap_err();
    assert_eq!(fault.message, "Clone must be explicit");
    assert_eq!(fault.function.as_deref(), Some("Item.Clone"));
}
