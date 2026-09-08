use neoclr::{Limits, LoadedProgram, Value, frontend};
fn run(source: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let module = frontend::compile(source)?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    program.run(Limits::default())
}
#[test]
fn arrays_demo_covers_copies_caller_borrows_heap_returns_and_strings() {
    let result = run(include_str!("../examples/source/arrays.neo")).unwrap();
    assert_eq!(result.output, ["2", "9", "3", "42", "Neo", "CLR"]);
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.reclaimed_objects(), 2);
}
#[test]
fn arrays_pass_by_value_and_record_elements_are_addressable() {
    let result = run("record Counter(Age: int)\nfunc Change(items: Counter[]) -> int { var copy = items; copy[0].Age = 99; return copy[0].Age }\nfunc Main() -> int { var items = array(2, Counter(42)); let age = &items[1].Age; age = 7; let ignored = Change(items); return items[0].Age + items[1].Age }").unwrap();
    assert_eq!(result.value, Value::Int32(49));
}
#[test]
fn invalid_types_mutability_bounds_and_escapes_are_rejected() {
    for source in [
        "func Main() -> int { let a = [1]; a[0] = 2; return 0 }",
        "func Main() -> int { var a = [1, true]; return 0 }",
        "func Main() -> int { let a = new string[2]; return 0 }",
        "func Main() -> int& { var a = [1]; return &a[0] }",
        "func Main() -> int { let a = new int[1]; return a[1] }",
        "func Main() -> int { var a = [1]; if true { var b = [2]; let c = &b[0] }; return 0 }",
    ] {
        assert!(run(source).is_err(), "accepted {source}");
    }
}
#[test]
fn empty_arrays_and_array_type_tokens_work() {
    let result = run("func Main() -> int { let a: int[] = array(0, 42); let t = typeof(int[]); return a.Length }").unwrap();
    assert_eq!(result.value, Value::Int32(0));
}

#[test]
fn array_type_nesting_is_bounded() {
    let source = format!(
        "func Main() -> int {{ let a: int{} = 0; return 0 }}",
        "[]".repeat(1000)
    );
    assert!(
        frontend::compile(&source)
            .unwrap_err()
            .message
            .contains("nesting")
    );
}

#[test]
fn array_example_round_trips_through_cli_artifact() {
    let path = std::env::temp_dir().join(format!("neoclr-arrays-{}.neo.json", std::process::id()));
    let build = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["assemble", "examples/source/arrays.neo"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let execution = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .arg("run")
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(
        execution.status.success(),
        "{}",
        String::from_utf8_lossy(&execution.stderr)
    );
    assert!(String::from_utf8_lossy(&execution.stdout).contains("2\n9\n3\n42\nNeo\nCLR"));
}
