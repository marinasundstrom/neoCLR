use neoclr::{
    Limits, LoadedProgram, Value, assemble, library,
    metadata::{Instruction, Type},
};

#[test]
fn prepared_generic_calls_and_type_queries_survive_source_changes() {
    let mut source = assemble(include_str!("../examples/member-identities.neoil")).unwrap();
    let before = serde_json::to_value(&source).unwrap();
    let program = LoadedProgram::new(&source).unwrap();
    assert_eq!(serde_json::to_value(&source).unwrap(), before);
    let signature = neoclr::assembler::parse_type("Choice<Int32>").unwrap();
    let identity = program.resolve_type_identity(&signature).unwrap();
    source.types.clear();
    source.functions.clear();
    for _ in 0..2 {
        assert_eq!(
            program.run(Limits::default()).unwrap().output,
            ["generic declaration", "integer declaration"]
        );
        assert_eq!(program.resolve_type_identity(&signature).unwrap(), identity);
        assert!(program.verify().is_ok());
    }
}

#[test]
fn execution_state_and_limits_do_not_persist_between_runs() {
    let module = assemble(
        ".module App\n.entry Main\n.function Main() -> Ref<Int32>\nldc.i4 42\nheap.new\nret\n.end",
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let limits = Limits {
        instructions: 1,
        ..Limits::default()
    };
    assert!(
        program
            .run(limits)
            .unwrap_err()
            .message
            .contains("instruction limit")
    );
    let mut first = program.run(Limits::default()).unwrap();
    first.heap[0] = Value::Int32(99);
    let second = program.run(Limits::default()).unwrap();
    assert_eq!(second.heap, [Value::Int32(42)]);
    assert_eq!(first.value, second.value);
}

#[test]
fn supplied_library_is_snapshotted_and_sources_can_be_dropped() {
    let program = {
        let app = assemble(include_str!("../examples/hello.neoil")).unwrap();
        let mut system = library::system().unwrap().clone();
        let program = LoadedProgram::with_library(&app, &system).unwrap();
        system
            .functions
            .iter_mut()
            .find(|f| f.name == "neoCLR.Runtime.WriteLine")
            .unwrap()
            .body = vec![Instruction::Fault("changed source".into())];
        program
    };
    assert_eq!(
        program.run(Limits::default()).unwrap().output,
        ["Hello, world!"]
    );
    assert!(program.verify().is_ok());
}

#[test]
fn preparation_validates_metadata_but_typed_verification_stays_explicit() {
    let mut module =
        assemble(".module App\n.entry Main\n.function Main() -> Int32\nldc.i4 1\nret\n.end")
            .unwrap();
    module.functions[0].body = vec![
        Instruction::String("wrong type".into()),
        Instruction::Return,
    ];
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().is_err());
    assert!(program.run(Limits::default()).is_err());
    module.functions[0].definition.as_mut().unwrap().index = 99;
    assert!(
        LoadedProgram::new(&module)
            .unwrap_err()
            .message
            .contains("noncanonical")
    );
}

#[test]
fn library_only_images_support_analysis_without_an_entry_point() {
    for module in [
        library::system().unwrap().clone(),
        assemble(".module Helpers\n.type Marker\n.end").unwrap(),
    ] {
        let program = LoadedProgram::new(&module).unwrap();
        assert!(program.verify().is_ok());
        assert!(program.resolve_type_identity(&Type::Int32).is_ok());
        assert!(
            program
                .run(Limits::default())
                .unwrap_err()
                .message
                .contains("without an entry point")
        );
    }
}

#[test]
fn preparation_and_analysis_do_not_activate_native_imports() {
    let module = assemble(".module App\n.entry Main\n.function Imported() -> Int32\n.pinvoke \"neoclr_nonexistent_preparation_fixture\" \"value\" cdecl\n.end\n.function Main() -> Int32\ncall Imported()\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().is_ok());
    assert!(program.resolve_type_identity(&Type::Int32).is_ok());
    assert!(
        program
            .run(Limits::default())
            .unwrap_err()
            .message
            .contains("native imports require trusted")
    );
}
