use neoclr::{ExecutionOptions, FaultCode, Limits, LoadedProgram, Value, assemble};

const SOURCE: &str = ".module Args\n.entry Main\n.function Main(arrayref<String>) -> Int32\nldarg 0\nldlen\nconv.i4\nret\n.end";

#[test]
fn entry_receives_user_arguments_and_keeps_environment_argv_separate() {
    let module = assemble(SOURCE).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    for (arguments, count) in [
        (vec![], 0),
        (vec!["app"], 0),
        (vec!["app", "Café", "two"], 2),
    ] {
        let execution = program
            .run(ExecutionOptions {
                arguments: arguments.into_iter().map(str::to_owned).collect(),
                ..ExecutionOptions::default()
            })
            .unwrap();
        assert_eq!(execution.value, Value::Int32(count));
    }
}

#[test]
fn entry_array_allocation_honors_limits_and_rejects_other_signatures() {
    let program = LoadedProgram::new(&assemble(SOURCE).unwrap()).unwrap();
    let fault = program
        .run(ExecutionOptions {
            arguments: vec!["app".into(), "arg".into()],
            limits: Limits {
                heap_objects: 0,
                ..Limits::default()
            },
            ..ExecutionOptions::default()
        })
        .unwrap_err();
    assert_eq!(fault.code, FaultCode::HeapLimitExceeded);
    assert!(assemble(&SOURCE.replace("arrayref<String>", "arrayref<Int32>")).is_err());
}

#[test]
fn ambiguous_entry_overloads_are_rejected() {
    let source = format!("{SOURCE}\n.function Main() -> Int32\nldc.i4 0\nret\n.end");
    assert!(
        assemble(&source)
            .unwrap_err()
            .message
            .contains("ambiguous entry")
    );
}
