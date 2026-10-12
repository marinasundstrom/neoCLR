//! Private timer services; these do not expose HttpClient.Timeout or a public Timer API.
use neoclr::{Limits, LoadedProgram, Value};

fn program(setup: &str, body: &str) -> LoadedProgram {
    let source = format!(
        r#".module TimerConsumer
.entry Main
.type class System.Concurrency.Tasks.TaskQueue
.method instance .ctor() -> noresult
ret
.end
.method instance Post(fn<Void> callback) -> noresult
ldarg 1
call instance fn<Void>::Invoke()
pop
ret
.end
.method instance Drain() -> noresult
ret
.end
.end
.type class Counter
.field Value Int32
.method instance .ctor() -> noresult
ret
.end
.method instance Complete() -> Void
ldarg 0
ldc.i4 42
stfld Counter::Value
ldvoid
ret
.end
.end
.function neoCLR.Runtime.RegisterDefaultTaskQueue(System.Concurrency.Tasks.TaskQueue queue) -> Void
.methodimpl InternalCall
.end
.function neoCLR.Runtime.DrainEntryTasks() -> Void
.methodimpl InternalCall
.end
.function neoCLR.Runtime.TimerStart(Int32 milliseconds, fn<Void> callback) -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.TimerCancel(Int64 handle) -> Boolean
.methodimpl InternalCall
.end
.function Main() -> Int32
.local Counter counter
.local Int64 handle
{setup}
newobj instance Counter::.ctor()
stloc counter
{body}
ldloc counter
ldfld Counter::Value
ret
.end
"#
    );
    let module = neoclr::assemble(&source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
}
const QUEUE: &str = "newobj instance System.Concurrency.Tasks.TaskQueue::.ctor()\ncall neoCLR.Runtime.RegisterDefaultTaskQueue(System.Concurrency.Tasks.TaskQueue)\npop";
const START: &str = "ldc.i4 1\nldloc counter\nfunction.bind fn<Void> = instance Counter::Complete()\ncall neoCLR.Runtime.TimerStart(Int32, fn<Void>)\nstloc handle";

#[test]
fn entry_pump_delivers_timer_callback_and_preserves_return_frame() {
    let result = program(
        QUEUE,
        &format!("{START}\ncall neoCLR.Runtime.DrainEntryTasks()\npop"),
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.len(), 0);
}

#[test]
fn cancelled_timer_does_not_keep_entry_pump_alive_or_invoke_guest_code() {
    let start = START.replace("ldc.i4 1\n", "ldc.i4 2147483647\n");
    let result = program(QUEUE, &format!("{start}\nldloc handle\ncall neoCLR.Runtime.TimerCancel(Int64)\nbrfalse Bad\ncall neoCLR.Runtime.DrainEntryTasks()\npop\nbr Done\nBad:\nfault \"timer was not cancelled\"\nDone:"))
        .run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(0));
    assert_eq!(result.heap.len(), 0);
}

#[test]
fn timer_requires_a_default_queue_before_retaining_callback() {
    let error = program("", START).run(Limits::default()).unwrap_err();
    assert!(
        error
            .message
            .contains("Timer completion requires the default TaskQueue"),
        "{error}"
    );
}
