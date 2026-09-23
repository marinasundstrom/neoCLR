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
    let extra = NOTIFY_TYPES.replace("= Echo(String)", "= Fails(String)")
        + "\n.function Fails(String input) -> String\nfault \"producer failed\"\n.end\n";
    assert!(
        execute("call Launch()\npop\nldstr \"entry\"", &extra)
            .unwrap_err()
            .message
            .contains("producer failed")
    );
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
