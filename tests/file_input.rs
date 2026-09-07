use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "neoclr-file-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, bytes: &[u8]) -> String {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).unwrap();
        path.to_str().unwrap().into()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn read(path: &str, limit: i32) -> Value {
    let program = LoadedProgram::new(&assemble(".module App").unwrap()).unwrap();
    program
        .resolve_function(&parse_function_ref("System.IO.File::ReadAllText(String,Int32)").unwrap())
        .unwrap()
        .invoke(
            vec![Value::String(path.into()), Value::Int32(limit)],
            Limits::default(),
        )
        .unwrap()
        .value
}

fn record(name: &str, fields: Vec<Value>) -> Value {
    Value::Object {
        ty: neoclr::assembler::parse_type(name).unwrap(),
        fields,
    }
}
fn carrier(name: &str, case: &str, fields: Vec<Value>) -> Value {
    record(name, vec![Value::Erased(Box::new(record(case, fields)))])
}
fn ok(text: &str) -> Value {
    carrier(
        "System.Result<String,Error>",
        "System.Result.Ok<String>",
        vec![Value::String(text.into())],
    )
}
fn err(text: &str) -> Value {
    carrier(
        "System.Result<String,Error>",
        "System.Result.Error<Error>",
        vec![Value::Error(text.into())],
    )
}

#[test]
fn reads_exact_bound_empty_unicode_nul_and_preserves_bom() {
    let fixture = Fixture::new();
    for text in ["", "hello", "é🌍\0", "\u{feff}21", "line\r\n"] {
        let path = fixture.write("input", text.as_bytes());
        assert_eq!(read(&path, text.len() as i32), ok(text));
    }
}

#[test]
fn enforces_bound_across_chunks_and_rejects_invalid_utf8() {
    let fixture = Fixture::new();
    let path = fixture.write("large", &vec![b'x'; 16385]);
    assert_eq!(read(&path, 16384), err("FileTooLarge"));
    assert_eq!(read(&path, 0), err("FileTooLarge"));
    assert_eq!(read(&path, 16385), ok(&"x".repeat(16385)));
    let invalid = fixture.write("invalid", &[0xff]);
    assert_eq!(read(&invalid, 1), err("InvalidUtf8"));
    assert_eq!(read(&invalid, 0), err("FileTooLarge"));
}

#[test]
fn expected_io_and_argument_failures_are_recoverable() {
    let fixture = Fixture::new();
    assert_eq!(read("", 1), err("InvalidPath"));
    assert_eq!(read("bad\0path", 1), err("InvalidPath"));
    assert_eq!(read("", -1), err("ArgumentOutOfRange"));
    assert_eq!(
        read(fixture.0.join("missing").to_str().unwrap(), 1),
        err("FileNotFound")
    );
    // Opening a directory differs between platforms; either way it is an Error.
    let directory = read(fixture.0.to_str().unwrap(), 1);
    assert!(
        [
            err("NotRegularFile"),
            err("AccessDenied"),
            err("FileReadFailed")
        ]
        .contains(&directory)
    );
    let path = fixture.write("valid", b"21");
    assert_eq!(read(&path, 2), ok("21"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn sample_round_trips_verifies_computes_and_handles_parse_errors() {
    let module = assemble(include_str!("../examples/file_input.neoil")).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&loaded).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().output,
        ["42", "InvalidFormat", "File input handled"]
    );
    let fixture = Fixture::new();
    let report = program
        .resolve_function(&parse_function_ref("Report(String)").unwrap())
        .unwrap();
    let missing = fixture.0.join("missing").to_str().unwrap().to_owned();
    assert_eq!(
        report
            .invoke(vec![Value::String(missing)], Limits::default())
            .unwrap()
            .output,
        ["FileNotFound"]
    );
}

#[test]
fn file_service_is_visible_without_opening_files_and_faults_keep_call_site() {
    let program = LoadedProgram::new(&assemble(".module App").unwrap()).unwrap();
    let target = parse_function_ref("System.IO.File::ReadAllText(String,Int32)").unwrap();
    let graph = program
        .analyze_reachability(std::slice::from_ref(&target), 16)
        .unwrap();
    assert_eq!(
        graph.required_services(),
        [RuntimeService::FileInput, RuntimeService::ValueStorage]
    );
    assert_eq!(graph.functions[1].target.name, "neoCLR.Runtime.ReadAllText");
    assert!(!graph.missing_services(&[]).is_empty());
    let fault = program
        .resolve_function(&target)
        .unwrap()
        .invoke(
            vec![Value::String("unused".into()), Value::Int32(0)],
            Limits {
                instructions: 0,
                ..Limits::default()
            },
        )
        .unwrap_err();
    assert_eq!(
        fault.stack_trace.unwrap().frames[0].function.name,
        "System.IO.File.ReadAllText"
    );
}
