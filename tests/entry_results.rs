use std::{fs, process::Command};

#[test]
fn integer_entry_becomes_process_exit_status_without_diagnostic_output() {
    let path = std::env::temp_dir().join(format!("neoclr-entry-{}.neoil", std::process::id()));
    fs::write(
        &path,
        ".module App\n.entry Main\n.function Main() -> Int32\nldc.i4 23\nret\n.end\n",
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .arg("run")
        .arg(&path)
        .args(["--execution", "interpreter"])
        .output()
        .unwrap();
    fs::remove_file(path).unwrap();
    assert_eq!(result.status.code(), Some(23));
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
}

#[test]
fn entry_dispatch_without_a_queue_resumes_and_preserves_stack_values() {
    let module = neoclr::assemble(".module App\n.entry Main\n.function Main() -> Int32\nldc.i4 42\ncall neoCLR.Runtime.DrainEntryTasks()\npop\nret\n.end\n.function neoCLR.Runtime.DrainEntryTasks() -> Void\n.methodimpl InternalCall\n.end\n").unwrap();
    let program = neoclr::LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(42)
    );
}

#[test]
fn no_result_entry_dispatch_preserves_the_caller_stack() {
    let module = neoclr::assemble(".module App\n.entry Main\n.function Main() -> Int32\nldc.i4 42\ncall neoCLR.Runtime.DrainEntryTasks()\nret\n.end\n.function neoCLR.Runtime.DrainEntryTasks() -> void\n.methodimpl InternalCall\n.end\n").unwrap();
    let program = neoclr::LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(42)
    );
}

fn dispatch_program(drain: &str) -> neoclr::LoadedProgram {
    dispatch_program_with_result(drain, false)
}

fn dispatch_program_with_result(drain: &str, no_result: bool) -> neoclr::LoadedProgram {
    let drain_result = if no_result { "void" } else { "Void" };
    let system = neoclr::assemble(&format!(
        r#".module System
.type class System.Concurrency.Tasks.TaskQueue
.method instance .ctor() -> void
ret
.end
.method instance Drain() -> void
{drain}
ret
.end
.end
.function neoCLR.Runtime.RegisterDefaultTaskQueue(System.Concurrency.Tasks.TaskQueue queue) -> Void
.methodimpl InternalCall
.end
.function neoCLR.Runtime.DrainEntryTasks() -> {drain_result}
.methodimpl InternalCall
.end
"#
    ))
    .unwrap();
    let source = r#".module App
.entry Main
.function Main() -> Int32
newobj instance System.Concurrency.Tasks.TaskQueue::.ctor()
call neoCLR.Runtime.RegisterDefaultTaskQueue(System.Concurrency.Tasks.TaskQueue)
pop
ldc.i4 42
call Wait()
pop
ret
.end
.function Wait() -> Void
call neoCLR.Runtime.DrainEntryTasks()
ret
.end
"#;
    let source = if no_result {
        source
            .replace("call Wait()\npop", "call Wait()")
            .replace(".function Wait() -> Void", ".function Wait() -> void")
    } else {
        source.to_owned()
    };
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        &system,
    )
    .unwrap()
    .remove(0);
    neoclr::LoadedProgram::with_library(&app, &system).unwrap()
}

#[test]
fn entry_dispatch_resumes_after_queue_return() {
    for no_result in [false, true] {
        let program = dispatch_program_with_result("", no_result);
        program.verify().unwrap();
        let result = program.run(neoclr::Limits::default()).unwrap();
        assert_eq!(result.value, neoclr::Value::Int32(42));
        assert_eq!(result.heap.len(), 0);
    }
}

#[test]
fn entry_dispatch_rejects_reentrant_queue_pumping() {
    let program = dispatch_program("call neoCLR.Runtime.DrainEntryTasks()\npop");
    program.verify().unwrap();
    let error = program.run(neoclr::Limits::default()).unwrap_err();
    assert!(error.to_string().contains("cannot be nested"), "{error}");
}

#[test]
fn entry_dispatch_counts_suspended_startup_against_frame_limit() {
    let program = dispatch_program("");
    let limits = neoclr::Limits {
        frames: 2,
        ..neoclr::Limits::default()
    };
    let error = program.run(limits).unwrap_err();
    assert_eq!(error.code, neoclr::FaultCode::StackOverflow);
}
