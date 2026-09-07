use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};

#[test]
fn io_error_cases_round_trip_format_and_reject_wrong_extraction() {
    for (owner, cases) in [
        (
            "System.IO.ConsoleReadError",
            vec![
                ("Unavailable", "ConsoleUnavailable"),
                ("ReadFailed", "ConsoleReadFailed"),
            ],
        ),
        (
            "System.IO.FileReadError",
            vec![
                ("InvalidLimit", "ArgumentOutOfRange"),
                ("InvalidPath", "InvalidPath"),
                ("NotFound", "FileNotFound"),
                ("AccessDenied", "AccessDenied"),
                ("NotRegularFile", "NotRegularFile"),
                ("ReadFailed", "FileReadFailed"),
                ("TooLarge", "FileTooLarge"),
                ("InvalidUtf8", "InvalidUtf8"),
            ],
        ),
    ] {
        for (case, message) in &cases {
            let source = format!(
                ".module App\n.entry Main\n.function Main() -> {owner}\nnewobj instance {owner}.{case}::.ctor()\nnewobj instance {owner}::.ctor({owner}.{case})\nret\n.end"
            );
            let module = assemble(&source).unwrap();
            let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
            let program = LoadedProgram::new(&module).unwrap();
            program.verify().unwrap();
            let error = program.run(Limits::default()).unwrap().value;
            let resolve = |name: &str| {
                program
                    .resolve_function(&parse_function_ref(name).unwrap())
                    .unwrap()
            };
            assert_eq!(
                resolve(&format!("instance {owner}::ToString()"))
                    .invoke_instance(error.clone(), vec![], Limits::default())
                    .unwrap()
                    .value,
                Value::String((*message).into())
            );
            for (other, _) in &cases {
                assert_eq!(
                    resolve(&format!("instance {owner}::get_Is{other}()"))
                        .invoke_instance(error.clone(), vec![], Limits::default())
                        .unwrap()
                        .value,
                    Value::Boolean(case == other)
                );
                let result = resolve(&format!("instance {owner}::Get{other}()")).invoke_instance(
                    error.clone(),
                    vec![],
                    Limits::default(),
                );
                if case == other {
                    assert_eq!(
                        result.unwrap().value.ty(),
                        neoclr::assembler::parse_type(&format!("{owner}.{case}")).unwrap()
                    );
                } else {
                    assert!(result.is_err());
                }
            }
        }
    }
}
