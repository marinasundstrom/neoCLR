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
        .replace("Tasks.", "System.Threading.Tasks.");
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
.local Tasks.TaskCompletionSource<Int32> source
newobj instance Tasks.TaskQueue::.ctor()
newobj instance Tasks.TaskCompletionSource<Int32>::.ctor(Tasks.TaskQueue)
stloc source
"#;

#[test]
fn repeated_task_property_preserves_reference_identity() {
    let body = format!(
        r#"{SOURCE}
ldloc source
call instance Tasks.TaskCompletionSource<Int32>::get_Task()
ldloc source
call instance Tasks.TaskCompletionSource<Int32>::get_Task()
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
        "ldloc source\nnewobj instance Tasks.Task<Int32>::.ctor(Tasks.TaskCompletionSource<Int32>)\npop\nldc.i4 0",
        "ldloc source\ncall instance Tasks.TaskCompletionSource<Int32>::Read()",
        "ldloc source\ncall instance Tasks.TaskCompletionSource<Int32>::Completed()\npop\nldc.i4 0",
        "ldloc source\nldfld Tasks.TaskCompletionSource<Int32>::slot\npop\nldc.i4 0",
    ] {
        let error = load(&format!("{SOURCE}{body}"))
            .err()
            .expect("access must fail");
        assert!(
            error.contains("access") || error.contains("private") || error.contains("internal"),
            "{error}"
        );
    }
}

#[test]
fn unit_payload_uses_ordinary_generic_storage_and_first_completion_wins() {
    let body = r#"
.local Tasks.TaskCompletionSource<Void> source
newobj instance Tasks.TaskQueue::.ctor()
newobj instance Tasks.TaskCompletionSource<Void>::.ctor(Tasks.TaskQueue)
stloc source
ldloc source
ldvoid
call instance Tasks.TaskCompletionSource<Void>::TrySetResult(Void)
brfalse Bad
ldloc source
ldvoid
call instance Tasks.TaskCompletionSource<Void>::TrySetResult(Void)
brtrue Bad
ldloc source
call instance Tasks.TaskCompletionSource<Void>::get_Task()
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
call instance Tasks.TaskCompletionSource<Int32>::get_Task()
stloc wrong
ldc.i4 0
"#
    );
    assert!(load(&body).is_err());
}
