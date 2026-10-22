//! Runtime submission does not require a managed TaskQueue.
use neoclr::{Limits, LoadedProgram, Value};

fn program(callback_body: &str) -> LoadedProgram {
    let source = format!(
        r#".module SubmissionConsumer
.entry Main
.type class Counter
.field Value Int32
.method instance .ctor() -> void
ret
.end
.method instance Complete() -> Void
{callback_body}
ldvoid
ret
.end
.end
.function neoCLR.Runtime.SchedulerSubmit(fn<Void> callback) -> Int32
.methodimpl InternalCall
.end
.function neoCLR.Runtime.DrainEntryTasks() -> Void
.methodimpl InternalCall
.end
.function neoCLR.Runtime.GCCollect() -> void
.methodimpl InternalCall
.end
.function Main() -> Int32
.local Counter counter
newobj instance Counter::.ctor()
stloc counter
ldloc counter
function.bind fn<Void> = instance Counter::Complete()
call neoCLR.Runtime.SchedulerSubmit(fn<Void>)
pop
call neoCLR.Runtime.GCCollect()
ldloc counter
ldfld Counter::Value
brtrue Inline
call neoCLR.Runtime.DrainEntryTasks()
pop
ldloc counter
ldfld Counter::Value
ret
Inline:
fault "submission ran inline"
.end
"#
    );
    let module = neoclr::assemble(&source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
}

#[test]
fn queue_free_submission_retains_receiver_and_preserves_entry_stack() {
    let result = program("ldarg 0\nldc.i4 42\nstfld Counter::Value")
        .run(Limits::default())
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.len(), 0);
}

#[test]
fn submitted_callback_fault_is_terminal() {
    let error = program("fault \"submitted callback fault\"")
        .run(Limits::default())
        .unwrap_err();
    assert!(error.message.contains("submitted callback fault"));
}
