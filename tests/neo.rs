use neoclr::{Limits, LoadedProgram, Value, frontend};
use std::{fs, process::Command};

fn execute(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap()
}

#[test]
fn counter_program_exercises_value_copies_and_heap_field_references() {
    let source = include_str!("../examples/source/counter.neo");
    let il = frontend::lower_to_il(source).unwrap();
    assert!(il.contains("heap.new"));
    assert!(il.contains("ldloca"));
    assert!(il.contains("ldflda Counter::Age"));
    let result = execute(source);
    assert_eq!(result.output, ["1", "2", "42"]);
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 1);
    assert_eq!(result.heap.statistics().reclaimed_objects, 1);
}

#[test]
fn expression_precedence_literals_and_void_calls_lower_to_existing_il() {
    for (expression, expected) in [
        ("2 + 3 * 4", 14),
        ("(2 + 3) * 4", 20),
        ("20 / 2 / 2", 5),
        ("-2147483648", i32::MIN),
        ("-(2 + 3)", -5),
        ("42 - 2 * -3", 48),
    ] {
        assert_eq!(
            execute(&format!("func Main() -> int {{ return {expression} }}")).value,
            Value::Int32(expected)
        );
    }
    let result = execute("func Main() -> () { Console.WriteLine(\"Hello, λ\\nNeo\") }");
    assert_eq!(result.value, Value::Void);
    assert_eq!(result.output, ["Hello, λ\nNeo"]);
    assert_eq!(
        execute("func Main() -> bool { return true }").value,
        Value::Boolean(true)
    );
}

#[test]
fn mutable_bindings_can_rebind_references_and_mutate_nested_values() {
    let result = execute(
        "record Inner(Value: int)\nrecord Outer(Child: Inner)\nfunc Main() -> int {\nvar local = Outer(Inner(1))\nlocal.Child.Value = 42\nvar shared = new Outer(Inner(2))\nshared = &new Outer(Inner(3))\nshared.Child.Value = local.Child.Value\nreturn shared.Child.Value\n}",
    );
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 2);
}

#[test]
fn forward_calls_and_reference_parameters_are_typed() {
    let result = execute(
        "func Main() -> int { var value: int = 40; Add(&value, 2); return value }\nfunc Add(value: int&, amount: int) -> () { value = value + amount }",
    );
    assert_eq!(result.value, Value::Int32(42));
    for source in [
        "func Main() -> int { return Missing(1) }",
        "func Main() -> int { return Add(true) }\nfunc Add(value: int) -> int { return value }",
        "func Main() -> int { return Add() }\nfunc Add(value: int) -> int { return value }",
        "func Main() -> int { let x: bool = 1; return 0 }",
        "func Main() -> int { return true }",
        "func Main() -> int { let Add = 1; return Add(2) }\nfunc Add(x: int) -> int { return x }",
        "func Main() -> () { WriteLine(1) }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
}

#[test]
fn immutable_values_reject_writable_access_but_reference_targets_remain_mutable() {
    for statement in [
        "value = Counter(2)",
        "value.Age = 2",
        "let reference = &value",
    ] {
        let source = format!(
            "record Counter(Age: int)\nfunc Main() -> int {{ let value = Counter(1); {statement}; return value.Age }}"
        );
        assert!(
            frontend::compile(&source)
                .unwrap_err()
                .message
                .contains("immutable")
        );
    }
    assert_eq!(execute("record Counter(Age: int)\nfunc Main() -> int { let value = new Counter(1); value.Age = 42; return value.Age }").value, Value::Int32(42));
}

#[test]
fn compiler_and_runtime_preserve_reference_escape_rules() {
    assert!(
        frontend::compile("func Main() -> int& { var value = 42; return &value }")
            .unwrap_err()
            .message
            .contains("current frame")
    );
    let source = "record Holder(Item: int&)\nfunc Main() -> Holder { var value = 42; return Holder(&value) }";
    let module = frontend::compile(source).unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap_err()
            .message
            .contains("frame-backed references")
    );
    let result = execute(
        "record Counter(Age: int)\nfunc Main() -> int& { let counter = new Counter(42); return &counter.Age }",
    );
    let Value::SlotReference(reference) = &result.value else {
        panic!("expected reference")
    };
    assert_eq!(
        result.heap.read_reference(reference).unwrap(),
        Value::Int32(42)
    );
}

#[test]
fn unsupported_and_malformed_source_has_bounded_diagnostics() {
    for source in [
        "func Main() -> int { return 2147483648 }",
        "func Main() -> int { return -2147483649 }",
        "func Main() -> int { return 1 }\nfunc Main() -> int { return 2 }",
        "record Counter(Age: int, Age: int)\nfunc Main() -> int { return 0 }",
        "func Main() -> int { let x = 1; let x = 2; return x }",
        "func Main(x: int) -> int { return x }",
        "func Main() -> int { while 1 {} return 0 }",
        "func Main() -> int { return 1; return 2 }",
        "func Main() -> int { let value = 1 }",
        "func Main() -> int { return \"unterminated }",
        "import Unsupported.*\nfunc Main() -> int { return 1 }",
        "func Main() -> int { return new Make() }\nfunc Make() -> int { return 42 }",
        "func Main() -> int { return 1 2 }",
    ] {
        let error = frontend::compile(source).unwrap_err();
        assert!(error.message.starts_with("source "), "{source}: {error}");
    }
    let nested = format!(
        "func Main() -> int {{ return {}1{} }}",
        "(".repeat(150),
        ")".repeat(150)
    );
    assert!(
        frontend::compile(&nested)
            .unwrap_err()
            .message
            .contains("nesting limit")
    );
    let long = format!("func Main() -> int {{ return {}1 }}", "1+".repeat(150));
    assert!(
        frontend::compile(&long)
            .unwrap_err()
            .message
            .contains("nesting limit")
    );
}

#[test]
fn cli_runs_checks_and_compiles_neo_to_an_ordinary_artifact() {
    let source = "examples/source/counter.neo";
    for command in ["check", "verify", "run"] {
        let output = Command::new(env!("CARGO_BIN_EXE_neoclr"))
            .args([command, source])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if command == "run" {
            assert!(String::from_utf8_lossy(&output.stdout).starts_with("1\n2\n42\n"));
        }
    }
    let path = std::env::temp_dir().join(format!("neoclr-neo-{}.neo.json", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["assemble", source, path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let module = neoclr::load(&fs::read_to_string(&path).unwrap()).unwrap();
    fs::remove_file(&path).unwrap();
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    let output = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["run", source, "--module", "examples/hello.neoil"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("one input file"));
}
