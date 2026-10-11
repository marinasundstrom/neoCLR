//! Operation-triggered retained-frame suspension, without generated state machines.
use neoclr::{CancellationToken, ExecutionOptions, FaultCode, Limits, LoadedProgram, Value};

fn program(main: &str, extra: &str) -> LoadedProgram {
    let source = format!(
        r#"
.module AsyncProbe
.entry Main
.function neoCLR.Runtime.RuntimeAsyncCreate() -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncDelay(Int32 ms) -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncComplete(Int64 id, Int32 result) -> Boolean
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncCancel(Int64 id) -> Boolean
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncAwait(Int64 id) -> Int32
.methodimpl InternalCall
.end
.function neoCLR.Runtime.SchedulerSubmit(fn<Void> callback) -> Int32
.methodimpl InternalCall
.end
.function neoCLR.Runtime.GCCollect() -> noresult
.methodimpl InternalCall
.end
{extra}
.function Main() -> Int32
.local Int64 operation
{main}
.end
"#
    );
    let module = neoclr::assemble(&source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
}

#[test]
fn completed_before_await_returns_result_and_completion_is_one_shot() {
    let result = program(
        r#"
call neoCLR.Runtime.RuntimeAsyncCreate()
stloc operation
ldloc operation
ldc.i4 42
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
brfalse Bad
ldloc operation
ldc.i4 99
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
brtrue Bad
ldloc operation
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ret
Bad:
fault "completion was not one-shot"
"#,
        "",
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
}

const PRODUCER: &str = r#"
.type class Producer
.field Operation Int64
.field Value Int32
.method instance .ctor() -> noresult
ret
.end
.method instance Complete() -> Void
ldc.i4 10
call neoCLR.Runtime.RuntimeAsyncDelay(Int32)
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
pop
call neoCLR.Runtime.GCCollect()
ldarg 0
ldc.i4 25
stfld Producer::Value
ldarg 0
ldfld Producer::Operation
ldc.i4 17
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
pop
ldvoid
ret
.end
.end
.function Nested(Producer producer) -> Int32
.local Int32 saved
ldc.i4 7
stloc saved
ldc.i4 10
ldarg producer
ldfld Producer::Operation
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
add
ldloc saved
add
ldarg producer
ldfld Producer::Value
add
ret
.end
"#;
const PENDING: &str = r#"
.local Producer producer
newobj instance Producer::.ctor()
stloc producer
ldloc producer
call neoCLR.Runtime.RuntimeAsyncCreate()
stfld Producer::Operation
ldloc producer
function.bind fn<Void> = instance Producer::Complete()
call neoCLR.Runtime.SchedulerSubmit(fn<Void>)
pop
ldloc producer
call Nested(Producer)
ret
"#;

#[test]
fn pending_operation_resumes_after_producer_itself_suspends_and_collects() {
    let start = std::time::Instant::now();
    let result = program(PENDING, PRODUCER).run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(59));
    assert!(start.elapsed() >= std::time::Duration::from_millis(10));
    assert!(result.heap.collections() > 1);
    assert!(result.heap.is_empty());
}

#[test]
fn callback_fault_terminates_pending_await() {
    let producer = PRODUCER.replace(
        "ldc.i4 10\ncall neoCLR.Runtime.RuntimeAsyncDelay(Int32)",
        "fault \"producer failure\"\nldc.i4 10\ncall neoCLR.Runtime.RuntimeAsyncDelay(Int32)",
    );
    assert_eq!(
        program(PENDING, &producer)
            .run(Limits::default())
            .unwrap_err()
            .code,
        FaultCode::UserFault
    );
}

#[test]
fn pending_frames_still_charge_frame_budget() {
    assert_eq!(
        program(PENDING, PRODUCER)
            .run(Limits {
                frames: 2,
                ..Limits::default()
            })
            .unwrap_err()
            .code,
        FaultCode::StackOverflow
    );
}

#[test]
fn cancelled_operation_faults_and_cannot_be_completed() {
    let fault = program(
        r#"
call neoCLR.Runtime.RuntimeAsyncCreate()
stloc operation
ldloc operation
call neoCLR.Runtime.RuntimeAsyncCancel(Int64)
brfalse Bad
ldloc operation
ldc.i4 42
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
brtrue Bad
ldloc operation
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ret
Bad:
fault "cancel lost terminal authority"
"#,
        "",
    )
    .run(Limits::default())
    .unwrap_err();
    assert_eq!(fault.code, FaultCode::ExecutionCancelled);
}

#[test]
fn host_cancellation_interrupts_an_incomplete_operation() {
    let p = program("call neoCLR.Runtime.RuntimeAsyncCreate()\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret", "");
    let token = CancellationToken::new();
    std::thread::scope(|scope| {
        let token_copy = token.clone();
        let worker = scope.spawn(|| {
            p.run(ExecutionOptions {
                cancellation: Some(token_copy),
                ..ExecutionOptions::default()
            })
        });
        std::thread::sleep(std::time::Duration::from_millis(20));
        token.cancel();
        assert_eq!(
            worker.join().unwrap().unwrap_err().code,
            FaultCode::ExecutionCancelled
        );
    });
}

#[test]
fn consumed_handle_cannot_be_awaited_twice() {
    let fault = program("ldc.i4 0\ncall neoCLR.Runtime.RuntimeAsyncDelay(Int32)\nstloc operation\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\npop\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret", "").run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("consumed"));
}

#[test]
fn reentrant_wait_on_same_operation_is_rejected() {
    let producer = PRODUCER.replace(
        "ldc.i4 10\ncall neoCLR.Runtime.RuntimeAsyncDelay(Int32)",
        "ldarg 0\nldfld Producer::Operation",
    );
    let fault = program(PENDING, &producer)
        .run(Limits::default())
        .unwrap_err();
    assert!(fault.message.contains("already has a waiter"));
}

#[test]
fn invalid_delay_and_unknown_handles_are_rejected() {
    assert!(program("ldc.i4 -1\ncall neoCLR.Runtime.RuntimeAsyncDelay(Int32)\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret", "").run(Limits::default()).unwrap_err().message.contains("nonnegative"));
    assert!(program(
        "ldc.i8 0\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret",
        ""
    )
    .run(Limits::default())
    .unwrap_err()
    .message
    .contains("Unknown"));
}

const INDEPENDENT: &str = r#"
.type class Exchange
.field Parent Int64
.field Reply Int64
.field Done Int64
.method instance .ctor() -> noresult
ret
.end
.method instance Run() -> Void
ldarg 0
ldfld Exchange::Parent
ldc.i4 17
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
pop
ldarg 0
ldfld Exchange::Reply
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ldc.i4 25
ceq
brfalse Bad
call neoCLR.Runtime.GCCollect()
ldarg 0
ldfld Exchange::Done
ldc.i4 42
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
pop
ldvoid
ret
Bad:
fault "detached callback lost reply"
.end
.end
"#;
const EXCHANGE: &str = r#"
.local Exchange exchange
newobj instance Exchange::.ctor()
stloc exchange
ldloc exchange
call neoCLR.Runtime.RuntimeAsyncCreate()
stfld Exchange::Parent
ldloc exchange
call neoCLR.Runtime.RuntimeAsyncCreate()
stfld Exchange::Reply
ldloc exchange
call neoCLR.Runtime.RuntimeAsyncCreate()
stfld Exchange::Done
ldloc exchange
function.bind fn<Void> = instance Exchange::Run()
call neoCLR.Runtime.SchedulerSubmit(fn<Void>)
pop
ldloc exchange
ldfld Exchange::Parent
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ldc.i4 17
ceq
brfalse Bad
call neoCLR.Runtime.GCCollect()
ldloc exchange
ldfld Exchange::Reply
ldc.i4 25
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
pop
ldloc exchange
ldfld Exchange::Done
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ret
Bad:
fault "parent lost result"
"#;

fn bounded_run(p: LoadedProgram) -> neoclr::Execution {
    let token = CancellationToken::new();
    let (stop, stopped) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        let cancel = token.clone();
        scope.spawn(move || {
            if stopped
                .recv_timeout(std::time::Duration::from_secs(5))
                .is_err()
            {
                cancel.cancel();
            }
        });
        let result = p.run(ExecutionOptions {
            cancellation: Some(token),
            ..Default::default()
        });
        let _ = stop.send(());
        result.unwrap()
    })
}

#[test]
fn parent_resumes_while_callback_waits_for_parent_reply() {
    let result = bounded_run(program(EXCHANGE, INDEPENDENT));
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.is_empty());
    assert!(result.heap.collections() > 2);
}

#[test]
fn entry_result_survives_detached_callback_completion_after_entry_return() {
    let main = EXCHANGE.replace(
        "ldloc exchange\nldfld Exchange::Done\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret",
        "local.reset exchange\ncall neoCLR.Runtime.GCCollect()\nldc.i4 99\nret",
    );
    let result = bounded_run(program(&main, INDEPENDENT));
    assert_eq!(result.value, Value::Int32(99));
    assert!(result.heap.is_empty());
    assert!(result.heap.collections() > 2);
}

#[test]
fn detached_arrays_still_count_against_invocation_budget() {
    let extra = INDEPENDENT.replace(".method instance Run() -> Void\n", ".method instance Run() -> Void\n.local arrayref<Byte> bytes\nldc.i4 8\nnewarr Byte\nstloc bytes\n");
    let main = EXCHANGE.replace(
        "call neoCLR.Runtime.GCCollect()",
        "ldc.i4 8\nnewarr Byte\npop\ncall neoCLR.Runtime.GCCollect()",
    );
    let fault = program(&main, &extra)
        .run(Limits {
            array_bytes: 12,
            ..Limits::default()
        })
        .unwrap_err();
    assert_eq!(fault.code, FaultCode::ArrayLimitExceeded);
}
