use neoclr::{
    ExecutionOptions, Limits, LoadedProgram,
    debugger::{Breakpoint, DebugCommand, DebugSnapshot, Debugger},
    frontend,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

fn launch(module: neoclr::Module, limits: Limits) -> (Debugger, std::thread::JoinHandle<()>) {
    let program = LoadedProgram::new(&module).unwrap();
    let debugger = Debugger::new();
    let control = debugger.clone();
    let worker = std::thread::spawn(move || {
        let _ = program.run(ExecutionOptions {
            debugger: Some(control.clone()),
            console: Some(Arc::new(control)),
            limits,
            ..Default::default()
        });
    });
    until(&debugger, |s| s.status == "paused");
    (debugger, worker)
}
fn until(debugger: &Debugger, predicate: impl Fn(&DebugSnapshot) -> bool) -> DebugSnapshot {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot = debugger.snapshot();
        if predicate(&snapshot) {
            return snapshot;
        }
        if Instant::now() >= deadline {
            let _ = debugger.command(DebugCommand::Stop);
            panic!("debug wait timed out: {snapshot:?}");
        }
        debugger.wait(snapshot.revision, Duration::from_millis(100));
    }
}
fn act(debugger: &Debugger, command: DebugCommand) -> DebugSnapshot {
    let revision = debugger.snapshot().revision;
    debugger.command(command).unwrap();
    until(debugger, |s| {
        s.revision > revision
            && matches!(
                s.status.as_str(),
                "paused" | "completed" | "faulted" | "stopped"
            )
    })
}
#[test]
fn inspect_step_break_next_and_out_with_source_and_heap_identity() {
    let module = frontend::compile_named(
        include_str!("../examples/source/debugger.neo"),
        "debugger.neo",
    )
    .unwrap();
    let (debugger, worker) = launch(module, Limits::default());
    let initial = debugger.snapshot();
    assert_eq!(initial.frames[0].source.as_ref().unwrap().line, 9);
    assert!(
        initial.frames[0].locals[0]
            .1
            .value
            .contains("uninitialized")
    );
    let stepped = act(&debugger, DebugCommand::Step);
    assert_eq!(stepped.frames[0].instruction, 1);
    assert!(stepped.frames[0].evaluation_stack[0].value.contains('1'));
    debugger
        .command(DebugCommand::Break(Breakpoint::Instruction {
            function: "Add".into(),
            instruction: 0,
        }))
        .unwrap();
    let stopped = act(&debugger, DebugCommand::Continue);
    assert_eq!(stopped.frames.len(), 2);
    assert_eq!(stopped.frames[0].function, "Add");
    assert_eq!(stopped.frames[0].arguments[0].0, "counter");
    assert!(stopped.frames[0].arguments[0].1.value.contains("heap#"));
    assert_eq!(stopped.heap.len(), 1);
    let next = act(&debugger, DebugCommand::Next);
    assert_eq!(next.frames[0].source.as_ref().unwrap().line, 5);
    assert!(next.heap[0].1.children[0].1.value.contains("11"));
    debugger.command(DebugCommand::ClearBreakpoints).unwrap();
    let outer = act(&debugger, DebugCommand::Out);
    assert_eq!(outer.frames.len(), 1);
    let final_state = act(&debugger, DebugCommand::Continue);
    assert_eq!(final_state.status, "completed");
    assert!(final_state.result.unwrap().value.contains("20"));
    assert!(final_state.heap.is_empty()); // snapshots did not become GC roots
    assert_eq!(final_state.gc["reclaimed"], 1);
    assert!(stopped.heap[0].1.children[0].1.value.contains("10")); // immutable snapshot
    worker.join().unwrap();
}
#[test]
fn step_enters_guest_calls_and_next_steps_over_them() {
    let source = "func Inner(value: int) -> int {\n return value + 1\n}\nfunc Main() -> int {\n let result = Inner(41)\n return result\n}";
    let (debugger, worker) = launch(frontend::compile(source).unwrap(), Limits::default());
    let call = act(&debugger, DebugCommand::Step);
    assert_eq!(call.frames[0].function, "Main");
    let inner = act(&debugger, DebugCommand::Step);
    assert_eq!(inner.frames.len(), 2);
    assert_eq!(inner.frames[0].function, "Inner");
    let outer = act(&debugger, DebugCommand::Out);
    assert_eq!(outer.frames.len(), 1);
    assert_eq!(outer.frames[0].function, "Main");
    assert_eq!(act(&debugger, DebugCommand::Continue).status, "completed");
    worker.join().unwrap();

    let (debugger, worker) = launch(frontend::compile(source).unwrap(), Limits::default());
    let next = act(&debugger, DebugCommand::Next);
    assert_eq!(next.frames.len(), 1);
    assert_eq!(next.frames[0].function, "Main");
    assert_eq!(next.frames[0].source.as_ref().unwrap().line, 6);
    assert!(next.frames[0].locals[0].1.value.contains("42"));
    assert_eq!(act(&debugger, DebugCommand::Continue).status, "completed");
    worker.join().unwrap();
}

#[test]
fn running_program_can_be_observed_paused_and_stopped() {
    let (debugger, worker) = launch(
        frontend::compile("func Main() -> () { var x = 0; loop { x = x + 1 } }").unwrap(),
        Limits {
            instructions: usize::MAX,
            ..Default::default()
        },
    );
    debugger.command(DebugCommand::Continue).unwrap();
    let running = until(&debugger, |s| s.status == "running" && s.revision > 1);
    assert!(running.frames[0].locals[0].1.value.contains("Int32"));
    let paused = act(&debugger, DebugCommand::Pause);
    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(paused.revision, debugger.snapshot().revision);
    let stopped = act(&debugger, DebugCommand::Stop);
    assert_eq!(stopped.status, "stopped");
    worker.join().unwrap();
}
#[test]
fn source_breakpoint_and_fault_snapshot_survive_execution_teardown() {
    let module = frontend::compile_named(
        "func Main() -> int {\n let x = 0\n return 1 / x\n}",
        "fault.neo",
    )
    .unwrap();
    let (debugger, worker) = launch(module, Limits::default());
    debugger
        .command(DebugCommand::Break(Breakpoint::Source {
            document: "fault.neo".into(),
            line: 3,
        }))
        .unwrap();
    let paused = act(&debugger, DebugCommand::Continue);
    assert_eq!(paused.frames[0].source.as_ref().unwrap().line, 3);
    debugger.command(DebugCommand::ClearBreakpoints).unwrap();
    let fault = act(&debugger, DebugCommand::Continue);
    assert_eq!(fault.status, "faulted");
    assert_eq!(fault.frames[0].source.as_ref().unwrap().line, 3);
    assert!(fault.fault.is_some());
    worker.join().unwrap();
}
#[test]
fn native_bytes_show_initialization_and_frame_ownership() {
    let module = neoclr::assemble(
        ".module A\n.entry Main\n.function Main() -> Void\nldc.i4 4\nlocalloc\nldvoid\nret\n.end",
    )
    .unwrap();
    let (debugger, worker) = launch(module, Limits::default());
    act(&debugger, DebugCommand::Step);
    let allocated = act(&debugger, DebugCommand::Step);
    assert!(allocated.native[0].frame_owned);
    assert_eq!(allocated.native[0].bytes, vec![None; 4]);
    act(&debugger, DebugCommand::Stop);
    worker.join().unwrap();
}
#[test]
fn debugger_console_accepts_input_while_guest_waits() {
    let module = frontend::compile("func Main() -> int { return Console.ReadByte() match { Ok(let option) => option match { Some(let byte) => int(byte), None => -1 }, Error(_) => -2 } }").unwrap();
    let (debugger, worker) = launch(module, Limits::default());
    debugger.command(DebugCommand::Continue).unwrap();
    let waiting = until(&debugger, |s| s.status == "waiting for input");
    assert!(waiting.frames.len() >= 2);
    assert!(waiting.frames[0].function.contains("ReadByte"));
    debugger.input(b"A").unwrap();
    let result = until(&debugger, |s| s.status == "completed");
    assert!(result.result.unwrap().value.contains("65"));
    worker.join().unwrap();
}
#[test]
fn source_maps_roundtrip_and_invalid_points_are_rejected() {
    let module =
        frontend::compile_named("func Main() -> int {\n return 42\n}", "sample.neo").unwrap();
    let json = serde_json::to_string(&module).unwrap();
    let loaded = neoclr::load(&json).unwrap();
    assert_eq!(
        module.functions[0].sequence_points,
        loaded.functions[0].sequence_points
    );
    for point in [
        serde_json::json!({"instruction":999,"document":"x","line":1,"column":1}),
        serde_json::json!({"instruction":0,"document":"x","line":0,"column":1}),
    ] {
        let mut bad = serde_json::to_value(&module).unwrap();
        bad["functions"][0]["sequence_points"] = serde_json::json!([point]);
        assert!(neoclr::load(&bad.to_string()).is_err());
    }
}
#[test]
fn neo_fault_stack_trace_keeps_source_location() {
    let module =
        frontend::compile_named("func Main() -> int {\n return 1 / 0\n}", "sample.neo").unwrap();
    let fault = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert_eq!(
        fault.stack_trace.unwrap().frames[0]
            .source
            .as_ref()
            .unwrap()
            .line,
        2
    );
}

#[test]
fn repeating_single_instruction_breakpoint_can_be_resumed() {
    let module = neoclr::assemble(
        ".module A\n.entry Main\n.function Main() -> Void\nAgain:\nbr Again\n.end",
    )
    .unwrap();
    let (debugger, worker) = launch(module, Limits::default());
    debugger
        .command(DebugCommand::Break(Breakpoint::Instruction {
            function: "Main".into(),
            instruction: 0,
        }))
        .unwrap();
    assert_eq!(act(&debugger, DebugCommand::Continue).status, "paused");
    assert_eq!(act(&debugger, DebugCommand::Continue).status, "paused");
    act(&debugger, DebugCommand::Stop);
    worker.join().unwrap();
}

#[test]
fn cancellation_works_while_paused() {
    let module = frontend::compile("func Main() -> int { return 42 }").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let debugger = Debugger::new();
    let cancellation = neoclr::CancellationToken::new();
    let control = debugger.clone();
    let token = cancellation.clone();
    let worker = std::thread::spawn(move || {
        assert!(
            program
                .run(ExecutionOptions {
                    debugger: Some(control),
                    cancellation: Some(token),
                    ..Default::default()
                })
                .is_err()
        );
    });
    until(&debugger, |s| s.status == "paused");
    cancellation.cancel();
    until(&debugger, |s| s.status == "faulted");
    worker.join().unwrap();
}

#[test]
fn terminal_launch_source_step_memory_and_quit_commands_work() {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        process::{Command, Stdio},
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["debug", "examples/source/debugger.neo"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    input.write_all(b"source\nstep\nstack\n").unwrap();
    let mut text = String::new();
    while !text.contains("eval[0]") {
        let mut line = String::new();
        assert!(
            reader.read_line(&mut line).unwrap() > 0,
            "terminal ended before inspection: {text}"
        );
        text.push_str(&line);
    }
    // An idle human pause lets the next instruction publish a periodic running
    // snapshot. The next command must wait beyond that to the stopped source line.
    std::thread::sleep(Duration::from_millis(150));
    input
        .write_all(b"next\nsource\nheap\nmemory\nquit\n")
        .unwrap();
    drop(input);
    reader.read_to_string(&mut text).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains("examples/source/debugger.neo:9:"));
    assert!(text.contains("local_0"));
    assert!(text.contains("examples/source/debugger.neo:10:"));
    assert!(
        !text.contains("[running]"),
        "next returned before stopping: {text}"
    );
}

#[test]
fn snapshots_bound_large_payloads_and_do_not_follow_heap_cycles() {
    let cycle = neoclr::assemble(".module A\n.entry Main\n.function Main() -> System.Value&\n.local System.Value& node\nldvoid\nvalue.pack Void\nheap.new\nstloc node\nldloc node\nldloc node\nvalue.pack System.Value&\nstobj System.Value\nldloc node\nret\n.end").unwrap();
    let (debugger, worker) = launch(cycle, Limits::default());
    let done = act(&debugger, DebugCommand::Continue);
    assert_eq!(done.status, "completed");
    assert_eq!(done.heap.len(), 1);
    assert!(done.heap[0].1.children[0].1.value.contains("heap#0"));
    assert!(done.heap[0].1.children[0].1.children.is_empty());
    worker.join().unwrap();
    let array = frontend::compile("func Main() -> int[]& { return new int[100] }").unwrap();
    let (debugger, worker) = launch(array, Limits::default());
    let done = act(&debugger, DebugCommand::Continue);
    assert_eq!(done.heap[0].1.children.len(), 64);
    assert!(done.heap[0].1.truncated);
    worker.join().unwrap();
}

#[test]
fn constructor_frames_show_shared_unpublished_storage() {
    let module = frontend::compile_named(
        include_str!("../examples/source/constructor-chaining.neo"),
        "constructor-chaining.neo",
    )
    .unwrap();
    let (debugger, worker) = launch(module, Limits::default());
    debugger
        .command(DebugCommand::Break(Breakpoint::Instruction {
            function: "Counter..ctor".into(),
            instruction: 0,
        }))
        .unwrap();
    let stopped = act(&debugger, DebugCommand::Continue);
    assert_eq!(stopped.frames[0].function, "Counter..ctor");
    assert!(
        stopped.frames[0].arguments[0]
            .1
            .value
            .contains(".construction")
    );
    let storage = stopped
        .frames
        .iter()
        .flat_map(|f| &f.locals)
        .find(|(name, _)| name == "<construction storage>")
        .unwrap();
    assert!(
        storage
            .1
            .children
            .iter()
            .any(|(_, value)| value.value.contains("Uninitialized"))
    );
    debugger.command(DebugCommand::Stop).unwrap();
    worker.join().unwrap();
}

#[test]
fn default_interface_breakpoint_shows_original_receiver_and_steps_into_explicit_body() {
    let module = frontend::compile_named(
        include_str!("../examples/source/default-interfaces.neo"),
        "defaults.neo",
    )
    .unwrap();
    let (debugger, worker) = launch(module, Limits::default());
    debugger
        .command(DebugCommand::Break(Breakpoint::Instruction {
            function: "Readable.Twice".into(),
            instruction: 0,
        }))
        .unwrap();
    let stopped = act(&debugger, DebugCommand::Continue);
    assert_eq!(stopped.frames[0].function, "Readable.Twice");
    assert!(stopped.frames[0].source.is_some());
    assert!(stopped.frames[0].arguments[0].1.value.contains("frame#"));
    let mut snapshot = stopped;
    for _ in 0..10 {
        snapshot = act(&debugger, DebugCommand::Step);
        if snapshot.frames[0].function == "Counter.Readable.Read" {
            break;
        }
    }
    assert_eq!(snapshot.frames[0].function, "Counter.Readable.Read");
    debugger.command(DebugCommand::Stop).unwrap();
    worker.join().unwrap();
}

#[test]
fn delegate_call_enters_target_and_next_preserves_caller_source() {
    let source = "delegate Reader() -> int
class Counter { var Value: int = 42; readonly func Read() -> int { return this.Value } }
func Main() -> int {
 let counter = new Counter()
 let callback = Reader(counter.Read)
 let result = callback()
 return result
}";
    let module = frontend::compile_named(source, "delegates.neo").unwrap();
    let main = module.functions.iter().find(|f| f.name == "Main").unwrap();
    let pc = main
        .body
        .iter()
        .position(
            |op| matches!(op, neoclr::metadata::Instruction::Call(t) if t.name == "Reader.Invoke"),
        )
        .unwrap();
    for step in [true, false] {
        let (debugger, worker) = launch(module.clone(), Limits::default());
        debugger
            .command(DebugCommand::Break(Breakpoint::Instruction {
                function: "Main".into(),
                instruction: pc,
            }))
            .unwrap();
        let stopped = act(&debugger, DebugCommand::Continue);
        let callback = stopped.frames[0]
            .locals
            .iter()
            .find(|(_, v)| v.value.contains("delegate Counter.Read"))
            .unwrap();
        assert!(callback.1.value.contains("delegate Counter.Read"));
        assert!(callback.1.children[0].1.value.contains("heap#"));
        debugger.command(DebugCommand::ClearBreakpoints).unwrap();
        if step {
            let entered = act(&debugger, DebugCommand::Step);
            assert_eq!(entered.frames[0].function, "Counter.Read");
            assert_eq!(entered.frames.len(), 2);
            assert_eq!(entered.frames[1].source.as_ref().unwrap().line, 6);
            act(&debugger, DebugCommand::Out);
        } else {
            let next = act(&debugger, DebugCommand::Next);
            assert_eq!(next.frames.len(), 1);
            assert_eq!(next.frames[0].source.as_ref().unwrap().line, 7);
        }
        assert_eq!(act(&debugger, DebugCommand::Continue).status, "completed");
        worker.join().unwrap();
    }
}
