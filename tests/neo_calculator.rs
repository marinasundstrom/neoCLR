use neoclr::{Console, ExecutionOptions, Limits, LoadedProgram, Value, frontend};
use std::{
    collections::VecDeque,
    io,
    io::Write,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

const SOURCE: &str = include_str!("../examples/source/calculator.neo");
#[derive(Debug, Default)]
struct State {
    input: VecDeque<u8>,
    output: Vec<String>,
    reads: usize,
}
#[derive(Debug, Default)]
struct TestConsole {
    state: Mutex<State>,
    fail_after: Option<usize>,
}
impl Console for TestConsole {
    fn read_byte(&self) -> io::Result<Option<u8>> {
        let mut state = self.state.lock().unwrap();
        if self.fail_after == Some(state.reads) {
            return Err(io::Error::other("test input failure"));
        }
        state.reads += 1;
        Ok(state.input.pop_front())
    }
    fn write_line(&self, text: &str) -> io::Result<()> {
        self.state.lock().unwrap().output.push(text.into());
        Ok(())
    }
}
fn program() -> LoadedProgram {
    LoadedProgram::new(&frontend::compile(SOURCE).unwrap()).unwrap()
}
fn execute(input: &[u8]) -> (neoclr::Execution, Arc<TestConsole>) {
    let console = Arc::new(TestConsole {
        state: Mutex::new(State {
            input: input.iter().copied().collect(),
            ..Default::default()
        }),
        ..Default::default()
    });
    let result = program()
        .run(ExecutionOptions {
            console: Some(console.clone()),
            ..Default::default()
        })
        .unwrap();
    (result, console)
}

#[test]
fn repeated_calculation_signed_input_crlf_and_partial_eof() {
    let (result, console) = execute(b"84\r\n2\r\n-7\n3");
    assert_eq!(result.value, Value::Int32(0));
    assert_eq!(
        console.state.lock().unwrap().output,
        [
            "Integer division: one signed decimal number per line; EOF exits",
            "Dividend:",
            "Divisor:",
            "= 42",
            "Dividend:",
            "Divisor:",
            "= -2",
        ]
    );
    assert_eq!(result.heap.statistics().allocated_objects, 0);
    for (input, last) in [
        (&b""[..], "Dividend:"),
        (&b"42"[..], "End of input before divisor"),
        (&b"+9\n+2\n"[..], "Dividend:"),
    ] {
        let (result, console) = execute(input);
        assert_eq!(result.value, Value::Int32(0));
        assert_eq!(console.state.lock().unwrap().output.last().unwrap(), last);
    }
}

#[test]
fn recoverable_parse_and_division_errors_allow_retry() {
    let (result, console) = execute(b"84\nbad\n0\n2147483648\n-2147483648\n-1\n9\n2");
    assert_eq!(result.value, Value::Int32(0));
    let state = console.state.lock().unwrap();
    let reports: Vec<_> = state
        .output
        .iter()
        .filter(|line| !line.ends_with(':') && !line.starts_with("Integer division:"))
        .map(String::as_str)
        .collect();
    assert_eq!(
        reports,
        [
            "Expected at most 32 ASCII decimal characters; try again",
            "Cannot divide by zero",
            "Number out of range; try again",
            "Division overflow",
            "= 4"
        ]
    );
    let (_, console) = execute(b"\n-\n42\n2\n");
    assert_eq!(
        console
            .state
            .lock()
            .unwrap()
            .output
            .iter()
            .filter(|s| s.as_str() == "Invalid number; try again")
            .count(),
        2
    );
}

#[test]
fn line_and_session_limits_do_not_grow_without_bound() {
    let mut input = vec![b'1'; 33];
    input.extend_from_slice(b"\n84\n2\n");
    let (result, console) = execute(&input);
    assert_eq!(result.value, Value::Int32(0));
    assert!(
        console
            .state
            .lock()
            .unwrap()
            .output
            .contains(&"= 42".into())
    );
    assert!(
        console
            .state
            .lock()
            .unwrap()
            .output
            .contains(&"Expected at most 32 ASCII decimal characters; try again".into())
    );
    let (result, console) = execute(&vec![b'9'; 300]);
    assert_eq!(result.value, Value::Int32(2));
    assert_eq!(console.state.lock().unwrap().reads, 256);
    assert_eq!(
        console.state.lock().unwrap().output.last().unwrap(),
        "Input limit reached (256 bytes)"
    );
    let (result, console) = execute(b"\xff\n84\n2\n");
    assert_eq!(result.value, Value::Int32(0));
    assert!(
        console
            .state
            .lock()
            .unwrap()
            .output
            .contains(&"= 42".into())
    );
}

#[test]
fn io_errors_are_results_but_execution_limits_are_faults() {
    let result = program().run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(1));
    assert_eq!(result.output.last().unwrap(), "ConsoleUnavailable");
    let console = Arc::new(TestConsole {
        fail_after: Some(0),
        ..Default::default()
    });
    let result = program()
        .run(ExecutionOptions {
            console: Some(console.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(1));
    assert_eq!(
        console.state.lock().unwrap().output.last().unwrap(),
        "ConsoleReadFailed"
    );
    let fault = program()
        .run(Limits {
            instructions: 10,
            ..Default::default()
        })
        .unwrap_err();
    assert!(fault.message.contains("instruction"));
}

#[test]
fn instance_calls_and_explicit_byte_conversion_use_typed_il() {
    let source = "func Main() -> int { let value = 42; Console.WriteLine(value.ToString()); return \"42\".SliceUtf8(0, 2) match { Ok(let s) => Int32.Parse(s) match { Ok(let n) => n, Error(_) => 0 }, Error(_) => 0 } }";
    let result = LoadedProgram::new(&frontend::compile(source).unwrap())
        .unwrap()
        .run(Limits::default())
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["42"]);
    for source in [
        "func Main() -> int { return int(true) }",
        "func Main() -> int { return int(\"42\") }",
        "func Main() -> int { return int(1,2) }",
        "func Main() -> () { let n = 1; n.Missing() }",
        "func Main() -> () { let n = 1; n.ToString(1) }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
    let il = frontend::lower_to_il(SOURCE).unwrap();
    assert!(il.contains("conv.ovf.i4"));
    assert!(il.contains("call instance Int32::ToString()"));
}

#[test]
fn loop_and_match_heap_roots_survive_collection_pressure() {
    let source = "record Counter(Value: int)\nfunc Main() -> int { var total = 0; for i in 0..<16 { let counter = new Counter(i); Int32.Parse(\"1\") match { Ok(let n) => { total = total + counter.Value + n }, Error(_) => { return -1 } } }; return total }";
    let result = LoadedProgram::new(&frontend::compile(source).unwrap())
        .unwrap()
        .run(Limits {
            heap_objects: 2,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(136));
    let stats = result.heap.statistics();
    assert_eq!(stats.allocated_objects, 16);
    assert_eq!(stats.reclaimed_objects, 16);
    assert!(stats.peak_objects <= 2);
}

#[test]
fn documented_cli_supports_redirected_input_and_gc_diagnostics() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args([
            "run",
            "examples/source/calculator.neo",
            "--gc-stats",
            "--gc-events",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"84\n2\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("= 42\n"));
    assert!(stdout.ends_with("=> Int32(0)\n"));
    assert!(!stdout.contains("GC"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("GC"));
}
