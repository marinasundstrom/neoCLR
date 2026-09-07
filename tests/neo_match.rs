use neoclr::{Console, ExecutionOptions, Limits, LoadedProgram, Value, frontend};
use std::{
    collections::VecDeque,
    io,
    sync::{Arc, Mutex},
};

fn run(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap()
}

#[test]
fn match_expressions_statements_and_error_cases() {
    let result = run(include_str!("../examples/source/match.neo"));
    assert_eq!(result.output, ["Invalid number", "42"]);
    assert_eq!(result.value, Value::Int32(42));
    for (text, expected) in [("42", 42), ("bad", -1), ("2147483648", -2)] {
        let source = format!(
            "func Main() -> int {{ return System.Int32.Parse(\"{text}\") match {{ Ok(let n) => n, Error(let e) => e match {{ InvalidFormat => -1, Overflow => -2 }} }} }}"
        );
        assert_eq!(run(&source).value, Value::Int32(expected));
    }
    assert_eq!(run("func Main() -> int { Int32.Parse(\"42\") match { Ok(let n) => { return n }, Error(_) => { return 0 } } }").value, Value::Int32(42));
    assert_eq!(
        run("func Main() -> int { return Int32.Parse(\"x\") match { Ok(let n) => n, _ => 7 } }")
            .value,
        Value::Int32(7)
    );
}

#[test]
fn single_evaluation_arm_scopes_and_loop_transfers() {
    let source = "func Parse(n: int&) -> System.Result<int,System.Int32ParseError> { n = n + 1; return Int32.Parse(\"1\") }\nfunc Main() -> int { var n = 0; var count = 0; loop { Parse(&n) match { Ok(let v) => { count = count + v; if count < 3 { continue }; break }, Error(_) => { return -1 } } }; return n }";
    assert_eq!(run(source).value, Value::Int32(3));
    assert_eq!(run("func Main() -> int { var x = 1; let result = Int32.Parse(\"42\") match { Ok(_) => &x, Error(_) => &x }; result = 42; return x }").value, Value::Int32(42));
}

#[derive(Debug)]
struct Input(Mutex<VecDeque<u8>>);
impl Console for Input {
    fn read_byte(&self) -> io::Result<Option<u8>> {
        Ok(self.0.lock().unwrap().pop_front())
    }
    fn write_line(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn nested_result_option_handles_input_eof_and_unavailable() {
    let source = "func Read() -> Result<Option<byte>,System.IO.ConsoleReadError> { return Console.ReadByte() }\nfunc Main() -> int { return Read() match { Ok(let option) => option match { Some(_) => 1, None => 0 }, Error(let e) => e match { Unavailable => -1, ReadFailed => -2 } } }";
    let module = frontend::compile(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(-1)
    );
    for (input, expected) in [(vec![42], 1), (vec![], 0)] {
        let options = ExecutionOptions {
            console: Some(Arc::new(Input(Mutex::new(input.into())))),
            ..Default::default()
        };
        assert_eq!(program.run(options).unwrap().value, Value::Int32(expected));
    }
}

#[test]
fn rejected_patterns_types_and_lifetimes() {
    for arms in [
        "Ok(let n) => n",
        "Ok(_) => 1, Ok(_) => 2, Error(_) => 3",
        "_ => 1, Error(_) => 2",
        "Ok(_) => 1, Error(_) => 2, _ => 3",
        "Missing => 1, _ => 2",
        "Ok => 1, Error(_) => 2",
        "Ok(_) => true, Error(_) => 2",
        "Ok(let n) => { return n }, Error(_) => 0",
        "Ok(let n) => &n, Error(_) => 0",
    ] {
        let source =
            format!("func Main() -> int {{ return Int32.Parse(\"1\") match {{ {arms} }} }}");
        assert!(frontend::compile(&source).is_err(), "accepted {source}");
    }
    for source in [
        "func Main() -> int { return 1 match { _ => 0 } }",
        "record Result(Value: int)\nfunc Main() -> int { return 0 }",
        "func Main() -> int { Int32.Parse(\"1\") match { Ok(let n) => {}, Error(_) => {} }; return n }",
        "func Main() -> int { let System = 1; return System.Int32.Parse(\"1\") }",
        "func Main() -> int { return Int32.Parse(true) }",
        "func Main() -> () { let x: Option<int&> = Console.ReadByte() }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
    let nested = format!(
        "func Main() -> {}int{} {{ return 0 }}",
        "Option<".repeat(40),
        ">".repeat(40)
    );
    assert!(
        frontend::compile(&nested)
            .unwrap_err()
            .message
            .contains("nesting limit")
    );
}

#[test]
fn runtime_rejects_frame_escape_through_match_result() {
    let module = frontend::compile("func Main() -> int& { var x = 1; return Int32.Parse(\"1\") match { Ok(_) => &x, Error(_) => &x } }").unwrap();
    let error = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert!(error.message.contains("current frame"), "{error}");
    let result = run(
        "record Counter(Age: int)\nfunc Main() -> int& { let value = new Counter(42); return Int32.Parse(\"1\") match { Ok(_) => &value.Age, Error(_) => &value.Age } }",
    );
    let Value::SlotReference(reference) = &result.value else {
        panic!("expected managed reference")
    };
    assert_eq!(
        result.heap.read_reference(reference).unwrap(),
        Value::Int32(42)
    );
}
