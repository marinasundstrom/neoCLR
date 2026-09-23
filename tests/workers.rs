//! Worker execution boundaries are enforced below the Raven frontend.
use neoclr::{Limits, LoadedProgram, Value, assemble};

fn library() -> &'static neoclr::Module {
    static LIBRARY: std::sync::OnceLock<neoclr::Module> = std::sync::OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = std::process::Command::new("python3")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}
fn execute(body: &str, extra: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    execute_options(body, extra, Limits::default().into())
}
fn execute_options(
    body: &str,
    extra: &str,
    options: neoclr::ExecutionOptions,
) -> Result<neoclr::Execution, neoclr::Fault> {
    let source = format!(
        r#".module Workers
.entry Main
.function Echo(String text) -> String
ldarg text
ret
.end
{extra}
.function Main() -> String
{body}
ret
.end
"#
    );
    let library = library();
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        library,
    )?
    .remove(0);
    LoadedProgram::with_library(&app, library)?.run(options)
}

#[test]
fn dedicated_and_pooled_workers_exchange_owned_text() {
    for name in ["StartWorker", "QueueWorker"] {
        let result = execute(&format!("delegate.bind System.Func<String,String> = Echo(String)\nldstr \"👩‍💻\"\ncall neoCLR.Runtime.{name}(System.Func<String,String>,String)\ncall neoCLR.Runtime.JoinWorker(Int32)"), "").unwrap();
        assert_eq!(result.value, Value::String("👩‍💻".into()));
    }
}

#[test]
fn worker_results_share_a_byte_budget_with_captured_output() {
    let extra = ".function Print(String input) -> String\nldstr \"\"\ncall neoCLR.Runtime.WriteLine(String)\npop\nldarg input\ncall neoCLR.Runtime.WriteLine(String)\npop\nldarg input\nret\n.end";
    for name in ["StartWorker", "QueueWorker"] {
        for (callback, input, budget, succeeds) in [
            ("Echo", "", 0, true),
            ("Echo", "é", 2, true),
            ("Echo", "é", 1, false),
            ("Print", "é", 6, true), // empty line + UTF-8 line + returned text
            ("Print", "é", 5, false), // return exceeds remaining quota
            ("Print", "é", 3, false), // line exceeds remaining quota
            ("Print", "", 0, false), // empty lines still cost one byte
        ] {
            let body = format!(
                "delegate.bind System.Func<String,String> = {callback}(String)\nldstr \"{input}\"\ncall neoCLR.Runtime.{name}(System.Func<String,String>,String)\ncall neoCLR.Runtime.JoinWorker(Int32)"
            );
            let result = execute_options(
                &body,
                extra,
                Limits {
                    worker_result_bytes: budget,
                    ..Default::default()
                }
                .into(),
            );
            if succeeds {
                let execution =
                    result.unwrap_or_else(|e| panic!("{name} {callback} {input:?} {budget}: {e}"));
                assert_eq!(execution.value, Value::String(input.into()));
                let expected_bytes = if callback == "Print" {
                    format!("\n{input}\n").into_bytes()
                } else {
                    vec![]
                };
                assert_eq!(execution.stdout, expected_bytes);
                assert!(execution.stderr.is_empty());
                assert_eq!(
                    execution.output,
                    if callback == "Print" {
                        vec!["", input]
                    } else {
                        vec![]
                    }
                );
            } else {
                assert!(
                    result
                        .unwrap_err()
                        .message
                        .contains("Worker result byte limit exceeded")
                );
            }
        }
    }
}

#[test]
fn invalid_and_reused_handles_fault() {
    assert!(
        execute("ldc.i4 -1\ncall neoCLR.Runtime.JoinWorker(Int32)", "")
            .unwrap_err()
            .message
            .contains("Unknown")
    );
    let body = ".local Int32 handle\ndelegate.bind System.Func<String,String> = Echo(String)\nldstr \"ok\"\ncall neoCLR.Runtime.StartWorker(System.Func<String,String>,String)\nstloc handle\nldloc handle\ncall neoCLR.Runtime.JoinWorker(Int32)\npop\nldloc handle\ncall neoCLR.Runtime.JoinWorker(Int32)";
    assert!(
        execute(body, "")
            .unwrap_err()
            .message
            .contains("already joined")
    );
}

#[test]
fn worker_fault_reaches_join_and_nested_workers_are_rejected() {
    let extra = ".function Nested(String input) -> String\ndelegate.bind System.Func<String,String> = Echo(String)\nldarg input\ncall neoCLR.Runtime.StartWorker(System.Func<String,String>,String)\ncall neoCLR.Runtime.JoinWorker(Int32)\nret\n.end";
    let body = "delegate.bind System.Func<String,String> = Nested(String)\nldstr \"nested\"\ncall neoCLR.Runtime.QueueWorker(System.Func<String,String>,String)\ncall neoCLR.Runtime.JoinWorker(Int32)";
    assert!(
        execute(body, extra)
            .unwrap_err()
            .message
            .contains("Nested worker")
    );
}

const NOTIFY_TYPES: &str = r#"
.type class CopyCompletion
.field Handle Int32
.field Buffer arrayref<Byte>
.method instance Complete() -> Void
ldarg this
ldfld CopyCompletion::Handle
call neoCLR.Runtime.JoinWorker(Int32)
pop
ldarg this
ldfld CopyCompletion::Buffer
ldc.i4 0
ldelem Byte
conv.i4
ldc.i4 17
beq Retained
fault "Pending buffer was not retained"
Retained:
ldstr "copied"
call neoCLR.Runtime.WriteLine(String)
pop
ldvoid
ret
.end
.end
.function Ready() -> Void
ldstr "ready"
call neoCLR.Runtime.WriteLine(String)
pop
ldvoid
ret
.end
.function Launch() -> Void
.local Int32 handle
.local arrayref<Byte> buffer
call System.Tasks.TaskQueue::get_Default()
pop
delegate.bind System.Func<String,String> = Echo(String)
ldstr "data"
call neoCLR.Runtime.StartWorker(System.Func<String,String>,String)
stloc handle
ldc.i4 1
newarr Byte
stloc buffer
ldloc buffer
ldc.i4 0
ldc.i4 17
conv.u1
stelem Byte
ldloc handle
ldloc handle
ldloc buffer
newobj CopyCompletion
delegate.bind System.Func<Void> = instance CopyCompletion::Complete()
call neoCLR.Runtime.NotifyWorker(Int32,System.Func<Void>)
pop
ldvoid
ret
.end
"#;

#[test]
fn registered_completion_survives_both_vm_collection_paths_and_dispatches() {
    let body = r#"
.local Int32 index
call Launch()
pop
call System.Tasks.TaskQueue::get_Default()
delegate.bind System.Func<Void> = Ready()
call instance System.Tasks.TaskQueue::Post(System.Func<Void>)
ldc.i4 0
stloc index
Again:
ldc.i4 16
newarr Byte
pop
ldloc index
ldc.i4 1
add
stloc index
ldloc index
ldc.i4 100
blt Again
ldstr "entry"
"#;
    for limits in [
        Limits {
            heap_objects: 32,
            ..Limits::default()
        },
        Limits {
            array_elements: 128,
            ..Limits::default()
        },
    ] {
        let result = execute_options(body, NOTIFY_TYPES, limits.into()).unwrap();
        assert_eq!(result.value, Value::String("entry".into()));
        assert_eq!(result.output, ["ready", "copied"]);
        assert!(result.heap.statistics().reclaimed_objects >= 100);
        assert_eq!(result.heap.statistics().live_objects, 0);
    }
}

#[test]
fn notification_rejects_missing_dispatcher_unknown_and_duplicate_handles() {
    let call = "ldc.i4 0\ndelegate.bind System.Func<Void> = Ready()\ncall neoCLR.Runtime.NotifyWorker(Int32,System.Func<Void>)\npop\nldstr \"done\"";
    assert!(
        execute(call, NOTIFY_TYPES)
            .unwrap_err()
            .message
            .contains("default TaskQueue")
    );
    assert!(
        execute(
            &format!("call System.Tasks.TaskQueue::get_Default()\npop\n{call}"),
            NOTIFY_TYPES
        )
        .unwrap_err()
        .message
        .contains("Unknown")
    );
    assert!(
        execute(&format!("call Launch()\npop\n{call}"), NOTIFY_TYPES)
            .unwrap_err()
            .message
            .contains("registered")
    );
}

#[test]
fn notification_fault_and_instruction_exhaustion_teardown_workers() {
    let body = "call Launch()\npop\nfault \"stop invocation\"";
    assert!(
        execute(body, NOTIFY_TYPES)
            .unwrap_err()
            .message
            .contains("stop invocation")
    );
    let body = "call Launch()\npop\nLoop:\nbr Loop";
    assert!(
        execute_options(
            body,
            NOTIFY_TYPES,
            Limits {
                instructions: 4000,
                ..Limits::default()
            }
            .into()
        )
        .unwrap_err()
        .message
        .contains("instruction limit")
    );
}

#[test]
fn notified_worker_failure_reaches_invocation() {
    for (body, message, code) in [
        (
            "fault \"producer failed\"",
            "producer failed",
            neoclr::FaultCode::UserFault,
        ),
        (
            "ldc.i4 1\nldc.i4 0\ndiv\npop\nldarg input\nret",
            "division by zero",
            neoclr::FaultCode::DivideByZero,
        ),
    ] {
        let extra = NOTIFY_TYPES.replace("= Echo(String)", "= Fails(String)")
            + &format!("\n.function Fails(String input) -> String\n{body}\n.end\n");
        let fault = execute("call Launch()\npop\nldstr \"entry\"", &extra).unwrap_err();
        assert!(fault.message.contains(message));
        assert_eq!(fault.code, code);
    }
}

#[test]
fn oversized_notified_result_reaches_invocation_as_fault() {
    let result = execute_options(
        "call Launch()\npop\nldstr \"entry\"",
        NOTIFY_TYPES,
        Limits {
            worker_result_bytes: 3,
            ..Default::default()
        }
        .into(),
    );
    assert!(
        result
            .unwrap_err()
            .message
            .contains("Worker result byte limit exceeded")
    );
}

#[test]
fn worker_output_capture_keeps_console_input_unavailable() {
    let extra = ".function Read(String input) -> String\ncall neoCLR.Runtime.ConsoleReadByte()\nvalue.unpack Int32\nldc.i4 1\nbeq Unavailable\nfault \"Worker input must be unavailable\"\nUnavailable:\nldarg input\nret\n.end";
    for name in ["StartWorker", "QueueWorker"] {
        let body = format!(
            "delegate.bind System.Func<String,String> = Read(String)\nldstr \"ok\"\ncall neoCLR.Runtime.{name}(System.Func<String,String>,String)\ncall neoCLR.Runtime.JoinWorker(Int32)"
        );
        assert_eq!(
            execute(&body, extra).unwrap().value,
            Value::String("ok".into())
        );
    }
}

#[test]
fn cancellation_after_registration_stops_invocation_and_worker() {
    #[derive(Debug)]
    struct CancelOnWrite(neoclr::CancellationToken);
    impl neoclr::Console for CancelOnWrite {
        fn read_byte(&self) -> std::io::Result<Option<u8>> {
            Ok(None)
        }
        fn write_line(&self, _: &str) -> std::io::Result<()> {
            self.0.cancel();
            Ok(())
        }
    }
    let cancellation = neoclr::CancellationToken::default();
    let extra = NOTIFY_TYPES.replace("= Echo(String)", "= UntilCancelled(String)")
        + "\n.function UntilCancelled(String input) -> String\nLoop:\nbr Loop\n.end\n";
    let result = execute_options(
        "call Launch()\npop\nldstr \"cancel now\"\ncall neoCLR.Runtime.WriteLine(String)\npop\nldstr \"entry\"",
        &extra,
        neoclr::ExecutionOptions {
            console: Some(std::sync::Arc::new(CancelOnWrite(cancellation.clone()))),
            cancellation: Some(cancellation),
            limits: Limits {
                instructions: 1_000_000_000,
                ..Limits::default()
            },
            ..Default::default()
        },
    );
    assert!(result.unwrap_err().message.contains("cancel"));
}

#[test]
fn cancellation_during_worker_output_stops_delivery_and_guest_continuation() {
    #[derive(Debug)]
    struct CancelAfterLine {
        token: neoclr::CancellationToken,
        lines: std::sync::Mutex<Vec<String>>,
        after: usize,
    }
    impl neoclr::Console for CancelAfterLine {
        fn read_byte(&self) -> std::io::Result<Option<u8>> {
            Ok(None)
        }
        fn write_line(&self, text: &str) -> std::io::Result<()> {
            let mut lines = self.lines.lock().unwrap();
            lines.push(text.into());
            if lines.len() == self.after {
                self.token.cancel();
            }
            Ok(())
        }
    }
    let producer = ".function Print(String input) -> String\nldstr \"first\"\ncall neoCLR.Runtime.WriteLine(String)\npop\nldstr \"second\"\ncall neoCLR.Runtime.WriteLine(String)\npop\nldarg input\nret\n.end";
    for name in ["StartWorker", "QueueWorker"] {
        for notified in [false, true] {
            for after in [1, 2] {
                let console = std::sync::Arc::new(CancelAfterLine {
                    token: neoclr::CancellationToken::new(),
                    lines: Default::default(),
                    after,
                });
                let (body, extra) = if notified {
                    (
                        "call Launch()\npop\nldstr \"entry\"".into(),
                        NOTIFY_TYPES
                            .replace("= Echo(String)", "= Print(String)")
                            .replace("Runtime.StartWorker", &format!("Runtime.{name}"))
                            + producer,
                    )
                } else {
                    (
                        format!(
                            "delegate.bind System.Func<String,String> = Print(String)\nldstr \"data\"\ncall neoCLR.Runtime.{name}(System.Func<String,String>,String)\ncall neoCLR.Runtime.JoinWorker(Int32)"
                        ),
                        producer.into(),
                    )
                };
                let fault = execute_options(
                    &body,
                    &extra,
                    neoclr::ExecutionOptions {
                        cancellation: Some(console.token.clone()),
                        console: Some(console.clone()),
                        ..Default::default()
                    },
                )
                .unwrap_err();
                assert_eq!(fault.message, "execution cancelled");
                assert_eq!(*console.lines.lock().unwrap(), ["first", "second"][..after]);
            }
        }
    }
}

const BUSY_QUEUE: &str = r#"
.type class BusyQueue
.field Handle Int32
.field Done Int32
.field Queue System.Tasks.TaskQueue
.method instance Complete() -> Void
ldarg this
ldfld BusyQueue::Handle
call neoCLR.Runtime.JoinWorker(Int32)
pop
ldarg this
ldc.i4 1
stfld BusyQueue::Done
ldstr "completed"
call neoCLR.Runtime.WriteLine(String)
pop
ldvoid
ret
.end
.method instance Tick() -> Void
ldarg this
ldfld BusyQueue::Done
brtrue Finished
ldc.i4 16
newarr Byte
pop
ldarg this
ldfld BusyQueue::Queue
ldarg this
delegate.bind System.Func<Void> = instance BusyQueue::Tick()
call instance System.Tasks.TaskQueue::Post(System.Func<Void>)
ldvoid
ret
Finished:
ldstr "pump stopped"
call neoCLR.Runtime.WriteLine(String)
pop
ldvoid
ret
.end
.end
"#;

fn busy_queue_body(explicit: bool) -> String {
    let queue = if explicit {
        "newobj instance System.Tasks.TaskQueue::.ctor()"
    } else {
        "call System.Tasks.TaskQueue::get_Default()"
    };
    let drain = if explicit {
        "ldloc queue\ncall instance System.Tasks.TaskQueue::Drain()"
    } else {
        ""
    };
    format!(
        r#"
.local System.Tasks.TaskQueue queue
.local BusyQueue state
call System.Tasks.TaskQueue::get_Default()
pop
{queue}
stloc queue
delegate.bind System.Func<String,String> = Echo(String)
ldstr "ready"
call neoCLR.Runtime.StartWorker(System.Func<String,String>,String)
ldc.i4 0
ldloc queue
newobj BusyQueue
stloc state
ldloc state
ldfld BusyQueue::Handle
ldloc state
delegate.bind System.Func<Void> = instance BusyQueue::Complete()
call neoCLR.Runtime.NotifyWorker(Int32,System.Func<Void>)
pop
ldloc queue
ldloc state
delegate.bind System.Func<Void> = instance BusyQueue::Tick()
call instance System.Tasks.TaskQueue::Post(System.Func<Void>)
{drain}
ldstr "entry"
"#
    )
}

#[test]
fn completion_progresses_while_default_queue_reposts_work() {
    let result = execute_options(
        &busy_queue_body(false),
        BUSY_QUEUE,
        Limits {
            instructions: 1_000_000,
            heap_objects: 64,
            ..Limits::default()
        }
        .into(),
    )
    .unwrap();
    assert_eq!(result.output, ["completed", "pump stopped"]);
    assert_eq!(result.value, Value::String("entry".into()));
    assert_eq!(result.heap.statistics().live_objects, 0);
}

#[test]
fn explicit_queue_does_not_dispatch_default_queue_notifications() {
    let result = execute_options(
        &busy_queue_body(true),
        BUSY_QUEUE,
        Limits {
            instructions: 10_000,
            ..Limits::default()
        }
        .into(),
    );
    assert!(result.unwrap_err().message.contains("instruction limit"));
}

#[test]
fn individual_worker_cancellation_is_acknowledged_without_cancelling_siblings() {
    let producer = ".function UntilCancelled(String text) -> String\nAgain:\nbr Again\n.end";
    for start in ["StartWorker", "QueueWorker"] {
        let body = format!(
            r#"
.local Int32 cancelled
.local Int32 sibling
delegate.bind System.Func<String,String> = UntilCancelled(String)
ldstr ""
call neoCLR.Runtime.{start}(System.Func<String,String>,String)
stloc cancelled
delegate.bind System.Func<String,String> = Echo(String)
ldstr "sibling survived"
call neoCLR.Runtime.{start}(System.Func<String,String>,String)
stloc sibling
ldloc cancelled
call neoCLR.Runtime.RequestWorkerCancellation(Int32)
brtrue Requested
fault "first request was not accepted"
Requested:
ldloc cancelled
call neoCLR.Runtime.RequestWorkerCancellation(Int32)
brfalse Repeated
fault "duplicate request was accepted"
Repeated:
ldloc cancelled
call neoCLR.Runtime.JoinWorkerResult(Int32)
value.unpack Void
pop
ldloc sibling
call neoCLR.Runtime.JoinWorkerResult(Int32)
value.unpack String
"#
        );
        let execution = execute_options(
            &body,
            producer,
            Limits {
                instructions: 100_000_000,
                ..Limits::default()
            }
            .into(),
        )
        .unwrap();
        assert_eq!(execution.value, Value::String("sibling survived".into()));
    }
    assert!(
        execute(
            "ldc.i4 -1\ncall neoCLR.Runtime.RequestWorkerCancellation(Int32)\npop\nldstr \"bad\"",
            ""
        )
        .unwrap_err()
        .message
        .contains("Unknown")
    );
}

#[test]
fn cancelled_notification_keeps_receiver_alive_until_acknowledged_join() {
    let types = NOTIFY_TYPES.replace(
        "call neoCLR.Runtime.JoinWorker(Int32)\npop",
        "call neoCLR.Runtime.JoinWorkerResult(Int32)\nvalue.unpack Void\npop",
    ).replace("= Echo(String)", "= UntilCancelled(String)")
     .replace("stloc handle\nldc.i4 1", "stloc handle\nldloc handle\ncall neoCLR.Runtime.RequestWorkerCancellation(Int32)\npop\nldc.i4 1");
    let producer = ".function UntilCancelled(String text) -> String\nAgain:\nbr Again\n.end";
    let body = ".local Int32 index\ncall Launch()\npop\nldc.i4 0\nstloc index\nAgain:\nldc.i4 16\nnewarr Byte\npop\nldloc index\nldc.i4 1\nadd\nstloc index\nldloc index\nldc.i4 100\nblt Again\nldstr \"entry\"";
    let result = execute_options(
        body,
        &format!("{types}\n{producer}"),
        Limits {
            instructions: 100_000_000,
            heap_objects: 32,
            ..Limits::default()
        }
        .into(),
    )
    .unwrap();
    assert_eq!(result.output, ["copied"]);
    assert!(result.heap.statistics().reclaimed_objects >= 100);
    assert_eq!(result.heap.statistics().live_objects, 0);
}
