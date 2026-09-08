use neoclr::{
    Limits, Value, assemble, library, load, metadata::Instruction, run, run_with_library,
};

fn app(body: &str, returns: &str) -> neoclr::Module {
    assemble(&format!(
        ".module App\n.entry Main\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))
    .unwrap()
}

#[test]
fn runtime_library_is_assembled_platform_code() {
    let module = assemble(neoclr::library::system_source()).unwrap();
    assert!(module.entry.is_empty());
    assert!(
        module
            .functions
            .iter()
            .filter(|f| !f.is_internal_call()
                && !f
                    .owner
                    .as_ref()
                    .and_then(|ty| module.type_definition(ty))
                    .is_some_and(
                        |d| d.representation == neoclr::metadata::Representation::Interface
                    ))
            .all(|f| !f.body.is_empty())
    );
    assert_eq!(
        module
            .functions
            .iter()
            .filter(|f| f.is_internal_call())
            .count(),
        22 // Includes eight metadata-only reflection helpers.
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
                &format!("ldc.i4 {value}\ncall System.Math.Abs(int32)\ncall instance System.Result<Int32,System.OverflowError>::GetOkCase()\ncall instance System.Result.Ok<Int32>::get_Value()"),
                "Int32",
            ),
            Limits::default(),
        )
        .unwrap();
        assert_eq!(result.value, Value::Int32(expected));
    }
    let result = run(
        &app(
            "ldc.i4 -2147483648\ncall System.Math.Abs(int32)\ncall instance System.Result<Int32,System.OverflowError>::GetErrorCase()\ncall instance System.Result.Error<System.OverflowError>::get_Value()\ncall instance System.OverflowError::ToString()\ncall System.Error::FromMessage(String)",
            "Error",
        ),
        Limits::default(),
    )
    .unwrap();
    assert_eq!(result.value, Value::Error("Overflow".into()));
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
                &format!("ldc.i4 {a}\nldc.i4 {b}\ncall System.Int32.Divide(int32, int32)\ncall instance System.Result<Int32,System.IntegerDivisionError>::get_IsErrorCase()"),
                "Boolean",
            ),
            Limits::default(),
        )
        .unwrap();
        assert_eq!(result.value, Value::Boolean(true));
    }
}

#[test]
fn supplied_compiled_library_body_is_executed() {
    let mut compiled = library::system().unwrap().clone();
    let body = assemble(".module Replacement\n.function F() -> System.Result<Int32,System.IntegerDivisionError>\nldc.i4 123\nnewobj instance System.Result.Ok<Int32>::.ctor(Int32)\nnewobj instance System.Result<Int32,System.IntegerDivisionError>::.ctor(System.Result.Ok<Int32>)\nret\n.end").unwrap().functions[0].body.clone();
    compiled
        .functions
        .iter_mut()
        .find(|f| f.name == "System.Int32.Divide")
        .unwrap()
        .body = body;
    let loaded = load(&serde_json::to_string(&compiled).unwrap()).unwrap();
    let application = app(
        "ldc.i4 84\nldc.i4 2\ncall System.Int32.Divide(int32, int32)\ncall instance System.Result<Int32,System.IntegerDivisionError>::GetOkCase()\ncall instance System.Result.Ok<Int32>::get_Value()",
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
