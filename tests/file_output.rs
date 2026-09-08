use neoclr::{ExecutionOptions, Limits, LoadedProgram, Value, assembler::parse_function_ref};

fn temp_dir() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "neoclr-output-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&path).unwrap();
    path
}

#[test]
fn writes_utf8_truncates_and_rejects_limits_before_modifying_files() {
    let module = neoclr::assemble(
        r#"
.module Test
.function Write(String path, String text, Int32 limit) -> String
.local System.Result<Void,System.IO.FileWriteError> result
ldarg path
ldarg text
ldarg limit
call System.IO.File::WriteAllText(String,String,Int32)
stloc result
ldloc result
call instance System.Result<Void,System.IO.FileWriteError>::get_IsOk()
brfalse Error
ldstr "Ok"
ret
Error:
ldloc result
call instance System.Result<Void,System.IO.FileWriteError>::GetErrorCase()
call instance System.Result.Error<System.IO.FileWriteError>::get_Value()
call instance System.IO.FileWriteError::ToString()
ret
.end
"#,
    )
    .unwrap();
    let p = LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap())
        .unwrap();
    p.verify().unwrap();
    let f = p
        .resolve_function(&parse_function_ref("Write(String,String,Int32)").unwrap())
        .unwrap();
    let write = |path: &str, text: &str, limit| {
        f.invoke(
            vec![
                Value::String(path.into()),
                Value::String(text.into()),
                Value::Int32(limit),
            ],
            Limits::default(),
        )
        .unwrap()
        .value
    };
    let directory = temp_dir();
    let path = directory.join("out.txt");
    let name = path.to_str().unwrap();
    assert_eq!(write(name, "é\0hello", 8), Value::String("Ok".into()));
    assert_eq!(std::fs::read(&path).unwrap(), "é\0hello".as_bytes());
    assert_eq!(write(name, "é", 1), Value::String("TooLarge".into()));
    assert_eq!(write(name, "", -1), Value::String("InvalidLimit".into()));
    assert_eq!(std::fs::read(&path).unwrap(), "é\0hello".as_bytes());
    assert_eq!(write(name, "x", 1), Value::String("Ok".into()));
    assert_eq!(std::fs::read(&path).unwrap(), b"x");
    assert_eq!(write(name, "", 0), Value::String("Ok".into()));
    assert!(std::fs::read(&path).unwrap().is_empty());
    assert_eq!(write("", "", 0), Value::String("InvalidPath".into()));
    assert_eq!(
        write("bad\0path", "", 0),
        Value::String("InvalidPath".into())
    );
    assert_eq!(
        write(directory.join("missing/out").to_str().unwrap(), "", 0),
        Value::String("NotFound".into())
    );
    // Host platforms may report directory opens as access denied or as non-regular.
    assert!(
        matches!(write(directory.to_str().unwrap(),"",0), Value::String(s) if s == "NotRegularFile" || s == "AccessDenied")
    );
    let services = p
        .analyze_reachability(
            &[parse_function_ref("neoCLR.Runtime.WriteAllText(String,String,Int32)").unwrap()],
            8,
        )
        .unwrap()
        .required_services();
    assert_eq!(services, vec![neoclr::RuntimeService::FileOutput]);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn report_runs_from_artifact_and_cli() {
    let module =
        neoclr::frontend::compile(include_str!("../examples/source/file-report.neo")).unwrap();
    let p = LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap())
        .unwrap();
    p.verify().unwrap();
    let directory = temp_dir();
    let input = directory.join("input.txt");
    std::fs::write(&input, "hello é\n").unwrap();
    let arguments = vec![
        "file-report.neo".into(),
        input.to_str().unwrap().into(),
        directory.to_str().unwrap().into(),
    ];
    let result = p
        .run(ExecutionOptions {
            arguments,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(0));
    assert_eq!(
        std::fs::read_to_string(directory.join("summary.txt")).unwrap(),
        "Input: input.txt\nUTF-8 bytes: 9\n"
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["run", "examples/source/file-report.neo", "--"])
        .arg(&input)
        .arg(&directory)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("=> Int32(0)")
    );
    std::fs::remove_dir_all(directory).unwrap();
}
