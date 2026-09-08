use neoclr::{
    ExecutionOptions, Limits, LoadedProgram, RuntimeService, Value, assembler::parse_function_ref,
};

fn program() -> LoadedProgram {
    let module =
        neoclr::frontend::compile(include_str!("../examples/source/environment.neo")).unwrap();
    let p = LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap())
        .unwrap();
    p.verify().unwrap();
    p
}

#[test]
fn arguments_are_per_execution_owned_values_and_respect_array_limits() {
    let p = program();
    let f = p
        .resolve_function(&parse_function_ref("System.Environment::GetCommandLineArgs()").unwrap())
        .unwrap();
    for args in [
        vec![],
        vec!["demo.neo".to_owned(), "a b".to_owned(), "é".to_owned()],
    ] {
        let result = f
            .invoke(
                vec![],
                ExecutionOptions {
                    arguments: args.clone(),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            result.value,
            Value::Array {
                element: neoclr::metadata::Type::String,
                elements: args.into_iter().map(Value::String).collect()
            }
        );
    }
    let error = f
        .invoke(
            vec![],
            ExecutionOptions {
                arguments: vec!["one".into(), "two".into()],
                limits: Limits {
                    array_elements: 1,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(error.message.contains("array"), "{error}");
    let services = p
        .analyze_reachability(
            &[parse_function_ref("neoCLR.Runtime.EnvironmentArguments()").unwrap()],
            8,
        )
        .unwrap()
        .required_services();
    assert!(services.contains(&RuntimeService::ProcessEnvironment));
    assert!(services.contains(&RuntimeService::ManagedArrays));
}

#[test]
fn cli_preserves_guest_arguments_missing_empty_and_present_variables() {
    for value in [None, Some(""), Some("hello é")] {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"));
        command.args([
            "run",
            "examples/source/environment.neo",
            "--",
            "a b",
            "--gc-stats",
        ]);
        command.env_remove("NEO_DEMO");
        if let Some(value) = value {
            command.env("NEO_DEMO", value);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let lines: Vec<_> = stdout.lines().collect();
        assert_eq!(
            &lines[..3],
            ["examples/source/environment.neo", "a b", "--gc-stats"]
        );
        assert_eq!(lines[3], std::env::current_dir().unwrap().to_str().unwrap());
        assert_eq!(lines[4], value.unwrap_or("NEO_DEMO is not set"));
    }
}

#[test]
fn invalid_names_and_non_unicode_values_return_errors() {
    let module = neoclr::frontend::compile(
        r#"
func Main() -> int {
    System.Environment.GetEnvironmentVariable("bad=name") match {
        Ok(let value) => { return 1 },
        Error(let error) => { return 0 }
    }
}
"#,
    )
    .unwrap();
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(0)
    );
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
            .args(["run", "examples/source/environment.neo"])
            .env("NEO_DEMO", std::ffi::OsString::from_vec(vec![0xff]))
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("EnvironmentUnavailable\n=> Int32(2)")
        );
    }
}
