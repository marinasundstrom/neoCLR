use neoclr::{
    Limits, Value, assemble, library, load,
    metadata::{Case, Instruction, Type},
    run, run_with_library,
};

fn app(body: &str, returns: &str) -> neoclr::Module {
    assemble(&format!(
        ".module App\n.entry Main\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))
    .unwrap()
}

#[test]
fn runtime_library_is_assembled_platform_code() {
    let module = assemble(include_str!("../runtime/System.neoil")).unwrap();
    assert!(module.entry.is_empty());
    assert_eq!(
        module
            .functions
            .iter()
            .filter(|f| !f.is_internal_call())
            .count(),
        21
    );
    assert_eq!(
        module
            .functions
            .iter()
            .filter(|f| f.is_internal_call())
            .count(),
        8
    );
    assert!(
        module
            .functions
            .iter()
            .find(|f| f.name == "System.Int32.Divide")
            .unwrap()
            .body
            .iter()
            .any(|op| matches!(op, Instruction::Divide))
    );
    assert!(
        run(&module, Limits::default())
            .unwrap_err()
            .message
            .contains("without an entry point")
    );
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(loaded.functions.len(), module.functions.len());
}

#[test]
fn platform_abs_handles_signs_and_overflow() {
    for (value, expected) in [
        (0, 0),
        (1, 1),
        (-1, 1),
        (i32::MAX, i32::MAX),
        (-2147483647, 2147483647),
    ] {
        let result = run(
            &app(
                &format!("ldc.i4 {value}\ncall System.Math.Abs(int32)"),
                "Result<Int32,Error>",
            ),
            Limits::default(),
        )
        .unwrap();
        assert_eq!(
            result.value,
            Value::result(Value::Int32(expected), Type::Int32, Type::Error, Case::Ok)
        );
    }
    let result = run(
        &app(
            "ldc.i4 -2147483648\ncall System.Math.Abs(int32)",
            "Result<Int32,Error>",
        ),
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        result.value,
        Value::result(
            Value::Error("Overflow".into()),
            Type::Int32,
            Type::Error,
            Case::Err
        )
    );
}

#[test]
fn div_truncates_toward_zero_and_faults_without_library_handling() {
    for (a, b, expected) in [
        (7, 2, 3),
        (-7, 2, -3),
        (7, -2, -3),
        (-7, -2, 3),
        (i32::MIN, 1, i32::MIN),
    ] {
        assert_eq!(
            run(
                &app(&format!("ldc.i4 {a}\nldc.i4 {b}\ndiv"), "Int32"),
                Limits::default()
            )
            .unwrap()
            .value,
            Value::Int32(expected)
        );
    }
    for (a, b) in [(1, 0), (i32::MIN, -1)] {
        assert!(
            run(
                &app(&format!("ldc.i4 {a}\nldc.i4 {b}\ndiv"), "Int32"),
                Limits::default()
            )
            .is_err()
        );
        let result = run(
            &app(
                &format!("ldc.i4 {a}\nldc.i4 {b}\ncall System.Int32.Divide(int32, int32)"),
                "Result<Int32,Error>",
            ),
            Limits::default(),
        )
        .unwrap();
        assert!(matches!(
            result.value,
            Value::Union {
                case: Case::Err,
                ..
            }
        ));
    }
}

#[test]
fn supplied_compiled_library_body_is_executed() {
    let mut compiled = library::system().unwrap().clone();
    compiled
        .functions
        .iter_mut()
        .find(|f| f.name == "System.Int32.Divide")
        .unwrap()
        .body = vec![
        Instruction::Int(123),
        Instruction::Ok(Type::Error),
        Instruction::Return,
    ];
    let loaded = load(&serde_json::to_string(&compiled).unwrap()).unwrap();
    let application = app(
        "ldc.i4 84\nldc.i4 2\ncall System.Int32.Divide(int32, int32)\nldcase Ok",
        "Int32",
    );
    assert_eq!(
        run_with_library(&application, &loaded, Limits::default())
            .unwrap()
            .value,
        Value::Int32(123)
    );
    assert_eq!(
        run(&application, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn library_calls_use_guest_frames_and_instruction_budget() {
    let application = app(
        "ldstr \"hello\"\ncall System.Console.WriteLine(string)",
        "Void",
    );
    assert!(
        run(
            &application,
            Limits {
                frames: 1,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .message
        .contains("frame limit")
    );
    assert!(
        run(
            &application,
            Limits {
                instructions: 3,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .message
        .contains("instruction limit")
    );
}
