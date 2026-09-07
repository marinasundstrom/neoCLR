use neoclr::{
    Console, ExecutionOptions, Limits, LoadedProgram, RuntimeService, Value, assemble,
    assembler::parse_function_ref,
    metadata::{Case, Type},
};
use std::{
    collections::VecDeque,
    io,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
struct State {
    input: VecDeque<u8>,
    lines: Vec<String>,
    events: Vec<&'static str>,
}
#[derive(Debug, Default)]
struct TestConsole {
    state: Mutex<State>,
    fail_read: bool,
    fail_write: bool,
}
impl TestConsole {
    fn input(bytes: &[u8]) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                input: bytes.iter().copied().collect(),
                ..State::default()
            }),
            ..Self::default()
        })
    }
}
impl Console for TestConsole {
    fn read_byte(&self) -> io::Result<Option<u8>> {
        let mut state = self.state.lock().unwrap();
        state.events.push("read");
        if self.fail_read {
            return Err(io::Error::other("injected read failure"));
        }
        Ok(state.input.pop_front())
    }
    fn write_line(&self, text: &str) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.events.push("write");
        if self.fail_write {
            return Err(io::Error::other("injected write failure"));
        }
        state.lines.push(text.into());
        Ok(())
    }
}
fn options(console: Arc<TestConsole>) -> ExecutionOptions {
    ExecutionOptions {
        console: Some(console),
        ..ExecutionOptions::default()
    }
}
fn program() -> LoadedProgram {
    let module = assemble(include_str!("../examples/console_input.neoil")).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
}

#[test]
fn il_sample_handles_digits_empty_input_eof_and_invalid_input() {
    let program = program();
    for (input, expected) in [
        (&b"21\n"[..], "42"),
        (&b"21\r\n"[..], "42"),
        (&b"21"[..], "42"),
        (&b""[..], "End of input"),
        (&b"\n"[..], "Empty input"),
        (&b"x\n"[..], "Expected ASCII digits"),
        (&b"1234567890"[..], "At most nine digits"),
        (&b"999999999\n"[..], "1999999998"),
        (&b"\xff"[..], "Expected ASCII digits"),
    ] {
        let console = TestConsole::input(input);
        let execution = program.run(options(console.clone())).unwrap();
        assert!(execution.output.is_empty());
        assert_eq!(execution.value, Value::Void);
        let state = console.state.lock().unwrap();
        assert_eq!(
            state.lines,
            ["Enter an unsigned number (up to nine digits):", expected]
        );
        assert_eq!(&state.events[..2], &["write", "read"]);
    }
}

#[test]
fn default_embedding_captures_output_and_does_not_read_process_stdin() {
    let execution = program().run(Limits::default()).unwrap();
    assert_eq!(
        execution.output,
        [
            "Enter an unsigned number (up to nine digits):",
            "ConsoleUnavailable"
        ]
    );
}

#[test]
fn input_error_is_a_result_and_later_invocations_can_continue() {
    let program = program();
    let console = Arc::new(TestConsole {
        fail_read: true,
        ..TestConsole::default()
    });
    program.run(options(console.clone())).unwrap();
    assert_eq!(console.state.lock().unwrap().lines[1], "ConsoleReadFailed");
    let next = TestConsole::input(b"2\n");
    program.run(options(next.clone())).unwrap();
    assert_eq!(next.state.lock().unwrap().lines[1], "4");
}

#[test]
fn output_failure_is_a_terminal_fault_before_input_and_precancellation_has_no_io() {
    let program = program();
    let console = Arc::new(TestConsole {
        fail_write: true,
        ..TestConsole::default()
    });
    let fault = program.run(options(console.clone())).unwrap_err();
    assert_eq!(fault.message, "console output failed");
    assert_eq!(
        fault.stack_trace.unwrap().frames[0].function.name,
        "System.Console.WriteLine"
    );
    assert_eq!(console.state.lock().unwrap().events, ["write"]);
    let console = TestConsole::input(b"21\n");
    let token = neoclr::CancellationToken::new();
    token.cancel();
    let fault = program
        .run(ExecutionOptions {
            cancellation: Some(token),
            ..options(console.clone())
        })
        .unwrap_err();
    assert_eq!(fault.message, "execution cancelled");
    assert!(console.state.lock().unwrap().events.is_empty());
}

#[test]
fn bytes_and_eof_have_exact_owned_union_types_and_shared_host_position() {
    let program = program();
    let read = program
        .resolve_function(&parse_function_ref("System.Console::ReadByte()").unwrap())
        .unwrap();
    let console = TestConsole::input(&[0, 255]);
    for byte in [Some(0), Some(255), None] {
        let option = Value::Union {
            ty: Type::Option(Box::new(Type::Byte)),
            case: if byte.is_some() {
                Case::Some
            } else {
                Case::None
            },
            payload: Box::new(byte.map(Value::Byte).unwrap_or(Value::Void)),
        };
        assert_eq!(
            read.invoke(vec![], options(console.clone())).unwrap().value,
            Value::result(
                option,
                Type::Option(Box::new(Type::Byte)),
                Type::Error,
                Case::Ok
            )
        );
    }
}

#[test]
fn console_dependencies_are_discovered_without_executing_host_io() {
    let graph = program()
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 12)
        .unwrap();
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::ConsoleInput)
    );
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::ConsoleOutput)
    );
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "neoCLR.Runtime.ConsoleReadByte")
    );
}

#[test]
fn byte_input_uses_typed_local_storage_before_integer_arithmetic() {
    let module = assemble(".module App\n.function Increment(Option<Byte> value) -> Int32\n.local Byte byte\nldarg value\nldcase Some\nstloc byte\nldloc byte\nldc.i4 1\nadd\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let function = program
        .resolve_function(&parse_function_ref("Increment(Option<Byte>)").unwrap())
        .unwrap();
    let value = Value::Union {
        ty: Type::Option(Box::new(Type::Byte)),
        case: Case::Some,
        payload: Box::new(Value::Byte(255)),
    };
    assert_eq!(
        function
            .invoke(vec![value], Limits::default())
            .unwrap()
            .value,
        Value::Int32(256)
    );
}

#[test]
fn typed_console_adapter_exposes_nested_cases() {
    let module = assemble(
        ".module App\n.entry Main\n.function Main() -> Int32\ncall System.Console::ReadByteTyped()\ncall instance System.Result<System.Option<Byte>,System.Error>::GetOkCase()\ncall instance System.Result.Ok<System.Option<Byte>>::get_Value()\ncall instance System.Option<System.Byte>::GetSomeCase()\ncall instance System.Option.Some<System.Byte>::get_Value()\nret\n.end",
    )
    .unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::with_library(&module, neoclr::library::system().unwrap()).unwrap();
    program.verify().unwrap();
    let console = TestConsole::input(&[42]);
    assert_eq!(
        program.run(options(console)).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn cli_flushes_prompt_before_waiting_for_redirected_input_without_duplicate_output() {
    use std::{
        io::{BufRead, Read, Write},
        process::{Command, Stdio},
        sync::mpsc,
        time::Duration,
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["run", "examples/console_input.neoil"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let (send, receive) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut output = io::BufReader::new(stdout);
        let mut prompt = String::new();
        output.read_line(&mut prompt).unwrap();
        send.send(prompt).unwrap();
        let mut rest = String::new();
        output.read_to_string(&mut rest).unwrap();
        rest
    });
    let prompt = receive.recv_timeout(Duration::from_secs(10));
    if prompt.is_err() {
        child.kill().unwrap();
        child.wait().unwrap();
        reader.join().unwrap();
        panic!("CLI did not make its prompt visible before reading input");
    }
    assert_eq!(
        prompt.unwrap(),
        "Enter an unsigned number (up to nine digits):\n"
    );
    input.write_all(b"21\n").unwrap();
    drop(input);
    assert!(child.wait().unwrap().success());
    assert_eq!(reader.join().unwrap(), "42\n=> Void\n");
}
