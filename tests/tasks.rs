//! Provisional Raven-profile Task contracts: runtime access checks, not just compiler checks.
use neoclr::{Limits, LoadedProgram, Module, Value, assemble};
use std::{process::Command, sync::OnceLock};

fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}
fn load(body: &str) -> Result<LoadedProgram, String> {
    let source = format!(".module App\n.entry Main\n.function Main() -> Int32\n{body}\nret\n.end")
        .replace("Tasks.", "System.Tasks.");
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        library(),
    )
    .map_err(|error| error.to_string())?
    .remove(0);
    let program =
        LoadedProgram::with_library(&app, library()).map_err(|error| error.to_string())?;
    program.verify().map_err(|error| error.to_string())?;
    Ok(program)
}
const SOURCE: &str = r#"
.local Tasks.Promise<Int32> source
newobj instance Tasks.TaskQueue::.ctor()
newobj instance Tasks.Promise<Int32>::.ctor(Tasks.TaskQueue)
stloc source
"#;

#[test]
fn repeated_task_property_preserves_reference_identity() {
    let body = format!(
        r#"{SOURCE}
ldloc source
call instance Tasks.Promise<Int32>::get_Task()
ldloc source
call instance Tasks.Promise<Int32>::get_Task()
ref.eq
brtrue Same
fault "Task identity changed"
Same:
ldc.i4 42
"#
    );
    assert_eq!(
        load(&body).unwrap().run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn guest_il_cannot_bypass_completion_capabilities() {
    for body in [
        "call Tasks.TaskQueue::get_Current()\npop\nldc.i4 0",
        "ldloc source\nnewobj instance Tasks.Task<Int32>::.ctor(Tasks.Promise<Int32>)\npop\nldc.i4 0",
        "ldloc source\ncall instance Tasks.Promise<Int32>::Read()",
        "ldloc source\ncall instance Tasks.Promise<Int32>::Dispatcher()\npop\nldc.i4 0",
        "ldloc source\ncall instance Tasks.Promise<Int32>::get_Task()\ncall instance Tasks.Task<Int32>::Dispatcher()\npop\nldc.i4 0",
        "ldloc source\ncall instance Tasks.Promise<Int32>::Completed()\npop\nldc.i4 0",
        "ldloc source\ncall instance Tasks.Promise<Int32>::Cancelled()\npop\nldc.i4 0",
        "ldloc source\nldfld Tasks.Promise<Int32>::cancelled\npop\nldc.i4 0",
        "ldloc source\nldfld Tasks.Promise<Int32>::slot\npop\nldc.i4 0",
    ] {
        let error = load(&format!("{SOURCE}{body}")).expect_err("access must fail");
        assert!(
            error.contains("access") || error.contains("private") || error.contains("internal"),
            "{error}"
        );
    }
}

#[test]
fn unit_payload_uses_ordinary_generic_storage_and_first_completion_wins() {
    let body = r#"
.local Tasks.Promise<Void> source
newobj instance Tasks.TaskQueue::.ctor()
newobj instance Tasks.Promise<Void>::.ctor(Tasks.TaskQueue)
stloc source
ldloc source
ldvoid
call instance Tasks.Promise<Void>::Complete(Void)
brfalse Bad
ldloc source
ldvoid
call instance Tasks.Promise<Void>::Complete(Void)
brtrue Bad
ldloc source
call instance Tasks.Promise<Void>::get_Task()
call instance Tasks.Task<Void>::GetResult()
pop
ldc.i4 42
ret
Bad:
fault "Invalid completion state"
"#;
    assert_eq!(
        load(body).unwrap().run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn task_payload_types_remain_invariant() {
    let body = format!(
        r#".local Tasks.Task<String> wrong
{SOURCE}
ldloc source
call instance Tasks.Promise<Int32>::get_Task()
stloc wrong
ldc.i4 0
"#
    );
    assert!(load(&body).is_err());
}

#[test]
fn default_queue_is_lazy_stable_and_invocation_local() {
    let body = r#"
call neoCLR.Runtime.DefaultTaskQueue()
ref.isnull
brtrue Empty
fault "Queue leaked from an earlier invocation"
Empty:
call Tasks.TaskQueue::get_Default()
call Tasks.TaskQueue::get_Default()
ref.eq
brtrue Same
fault "Default identity changed"
Same:
ldc.i4 42
"#;
    let program = load(body).unwrap();
    for _ in 0..2 {
        assert_eq!(
            program.run(Limits::default()).unwrap().value,
            Value::Int32(42)
        );
    }
}

#[test]
fn default_queue_registration_cannot_replace_the_dispatcher() {
    let body = "call Tasks.TaskQueue::get_Default()\npop\nnewobj instance Tasks.TaskQueue::.ctor()\ncall neoCLR.Runtime.RegisterDefaultTaskQueue(Tasks.TaskQueue)\npop\nldc.i4 0";
    let error = load(body)
        .unwrap()
        .run(Limits::default())
        .unwrap_err()
        .to_string();
    assert!(error.contains("registered once"), "{error}");
}

#[test]
fn cancellation_is_terminal_and_completion_cannot_replace_it() {
    let body = format!(
        r#"{SOURCE}
ldloc source
call instance Tasks.Promise<Int32>::Cancel()
brfalse Failed
ldloc source
ldc.i4 42
call instance Tasks.Promise<Int32>::Complete(Int32)
brtrue Failed
ldloc source
call instance Tasks.Promise<Int32>::Cancel()
brtrue Failed
ldloc source
call instance Tasks.Promise<Int32>::get_Task()
call instance Tasks.Task<Int32>::get_State()
call instance Tasks.TaskState::get_Value()
ldc.i4 2
ceq
brfalse Failed
ldloc source
call instance Tasks.Promise<Int32>::get_Task()
call instance Tasks.Task<Int32>::get_Outcome()
call instance System.Option<Tasks.TaskOutcome<Int32>>::GetSomeCase()
call instance System.Option.Some<Tasks.TaskOutcome<Int32>>::get_Value()
call instance Tasks.TaskOutcome<Int32>::get_IsCancelled()
brfalse Failed
ldc.i4 1
ret
Failed:
fault "Cancellation contract failed"
"#
    );
    assert_eq!(
        load(&body).unwrap().run(Limits::default()).unwrap().value,
        Value::Int32(1)
    );
}
