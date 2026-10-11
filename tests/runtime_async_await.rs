//! Operation-triggered retained-frame suspension, without generated state machines.
use neoclr::{CancellationToken, ExecutionOptions, FaultCode, Limits, LoadedProgram, Value};

fn program(main: &str, extra: &str) -> LoadedProgram {
    let source = format!(
        r#"
.module AsyncProbe
.entry Main
.function neoCLR.Runtime.RuntimeAsyncStart(fn<Int32> body) -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncStart(fn<Void> body) -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64 id) -> Void
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncCompleteUnit(Int64 id) -> Boolean
.methodimpl InternalCall
.end
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
.function neoCLR.Runtime.RuntimeAsyncIsCompleted(Int64 id) -> Boolean
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncIsCancelled(Int64 id) -> Boolean
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncWaitCompletion(Int64 id) -> Boolean
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncRelease(Int64 id) -> Boolean
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
fn completed_handle_can_be_awaited_repeatedly_until_released() {
    let result = program("ldc.i4 0\ncall neoCLR.Runtime.RuntimeAsyncDelay(Int32)\nstloc operation\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\npop\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret", "").run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(0));
    let fault = program("ldc.i4 0\ncall neoCLR.Runtime.RuntimeAsyncDelay(Int32)\nstloc operation\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\npop\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncRelease(Int64)\npop\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret", "").run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("released"));
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

const BODY: &str = r#"
.type class AsyncBody
.field Gate Int64
.field Phase Int32
.method instance .ctor() -> noresult
ret
.end
.method instance Run() -> Int32
ldarg 0
ldc.i4 1
stfld AsyncBody::Phase
ldarg 0
ldfld AsyncBody::Gate
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
pop
call neoCLR.Runtime.GCCollect()
ldarg 0
ldc.i4 2
stfld AsyncBody::Phase
ldc.i4 42
ret
.end
.end
"#;
const START_BODY: &str = r#"
.local AsyncBody body
newobj instance AsyncBody::.ctor()
stloc body
ldloc body
call neoCLR.Runtime.RuntimeAsyncCreate()
stfld AsyncBody::Gate
ldloc body
function.bind fn<Int32> = instance AsyncBody::Run()
call neoCLR.Runtime.RuntimeAsyncStart(fn<Int32>)
stloc operation
ldloc body
ldfld AsyncBody::Phase
ldc.i4 1
ceq
brfalse Bad
ldloc operation
ldc.i4 99
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
brtrue Bad
ldloc body
ldfld AsyncBody::Gate
ldc.i4 0
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
pop
ldloc operation
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ret
Bad:
fault "async body did not run synchronously until suspension or lost producer authority"
"#;

#[test]
fn async_start_runs_until_first_suspend_then_publishes_body_result() {
    let result = bounded_run(program(START_BODY, BODY));
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.is_empty());
}

#[test]
fn async_start_completed_body_returns_a_completed_operation() {
    let extra = BODY.replace(
        "ldarg 0\nldfld AsyncBody::Gate\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\npop",
        "",
    );
    let main = START_BODY.replace("ldc.i4 1\nceq", "ldc.i4 2\nceq");
    assert_eq!(bounded_run(program(&main, &extra)).value, Value::Int32(42));
}

#[test]
fn async_start_charges_frame_before_running_body() {
    let fault = program(START_BODY, BODY)
        .run(Limits {
            frames: 1,
            ..Limits::default()
        })
        .unwrap_err();
    assert_eq!(fault.code, FaultCode::StackOverflow);
}

#[test]
fn async_start_body_fault_remains_terminal_in_this_poc() {
    let extra = BODY.replace("ldc.i4 42\nret", "fault \"async body failure\"");
    let fault = program(START_BODY, &extra)
        .run(Limits::default())
        .unwrap_err();
    assert_eq!(fault.code, FaultCode::UserFault);
}

#[test]
fn nested_async_starts_return_to_their_own_callers_and_publish_results() {
    let extra = BODY.replace("ldarg 0\nldfld AsyncBody::Gate\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\npop", "ldarg 0\nfunction.bind fn<Int32> = instance AsyncBody::Inner()\ncall neoCLR.Runtime.RuntimeAsyncStart(fn<Int32>)\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\npop")
        .replace("ldc.i4 42\nret\n.end\n.end", "ldc.i4 42\nret\n.end\n.method instance Inner() -> Int32\nldarg 0\nldfld AsyncBody::Gate\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\npop\nldc.i4 7\nret\n.end\n.end");
    assert_eq!(
        bounded_run(program(START_BODY, &extra)).value,
        Value::Int32(42)
    );
}

#[test]
fn multiple_parked_bodies_share_completion_and_release_cannot_retract_delivery() {
    let main = r#"
.local AsyncBody first
.local AsyncBody second
.local Int64 firstTask
.local Int64 secondTask
call neoCLR.Runtime.RuntimeAsyncCreate()
stloc operation
newobj instance AsyncBody::.ctor()
stloc first
ldloc first
ldloc operation
stfld AsyncBody::Gate
newobj instance AsyncBody::.ctor()
stloc second
ldloc second
ldloc operation
stfld AsyncBody::Gate
ldloc first
function.bind fn<Int32> = instance AsyncBody::First()
call neoCLR.Runtime.RuntimeAsyncStart(fn<Int32>)
stloc firstTask
ldloc second
function.bind fn<Int32> = instance AsyncBody::Run()
call neoCLR.Runtime.RuntimeAsyncStart(fn<Int32>)
stloc secondTask
ldloc operation
ldc.i4 0
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
pop
ldloc firstTask
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
pop
ldloc secondTask
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
pop
ldloc operation
call neoCLR.Runtime.RuntimeAsyncRelease(Int64)
brfalse Bad
ldloc firstTask
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ldloc secondTask
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
add
ret
Bad:
fault "release retracted a parked delivery or failed after delivery"
"#;
    let first = r#".method instance First() -> Int32
ldarg 0
call instance AsyncBody::Run()
ldarg 0
ldfld AsyncBody::Gate
call neoCLR.Runtime.RuntimeAsyncRelease(Int64)
brtrue Bad
ret
Bad:
fault "release retracted the second parked waiter"
.end
"#;
    let extra = format!(
        "{}{}\n.end\n",
        BODY.trim_end().strip_suffix(".end").unwrap(),
        first
    );
    let result = bounded_run(program(main, &extra));
    assert_eq!(result.value, Value::Int32(84));
    assert!(result.heap.is_empty());
}

#[test]
fn cancelled_body_handle_cannot_be_released_while_body_is_still_parked() {
    let main = START_BODY.replace("ldloc operation\nldc.i4 99\ncall neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)\nbrtrue Bad", "ldloc operation\ncall neoCLR.Runtime.RuntimeAsyncCancel(Int64)\npop\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncRelease(Int64)\nbrtrue Bad")
        .replace("ldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret", "ldc.i4 0\nret");
    assert_eq!(bounded_run(program(&main, BODY)).value, Value::Int32(0));
}

#[test]
fn readiness_query_is_nonconsuming_and_validates_handles() {
    let main = r#"
call neoCLR.Runtime.RuntimeAsyncCreate()
stloc operation
ldloc operation
call neoCLR.Runtime.RuntimeAsyncIsCompleted(Int64)
brtrue Bad
ldloc operation
ldc.i4 42
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
pop
ldloc operation
call neoCLR.Runtime.RuntimeAsyncIsCompleted(Int64)
brfalse Bad
ldloc operation
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ret
Bad:
fault "readiness did not reflect operation state"
"#;
    assert_eq!(
        program(main, "").run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let fault = program(
        "ldc.i8 0\ncall neoCLR.Runtime.RuntimeAsyncIsCompleted(Int64)\npop\nldc.i4 0\nret",
        "",
    )
    .run(Limits::default())
    .unwrap_err();
    assert!(fault.message.contains("Unknown"));
}

#[test]
fn cancellation_delivery_resumes_without_reading_the_cancelled_result() {
    let result = program(
        r#"
.local Canceller canceller
call neoCLR.Runtime.RuntimeAsyncCreate()
stloc operation
newobj instance Canceller::.ctor()
stloc canceller
ldloc canceller
ldloc operation
stfld Canceller::Operation
ldloc canceller
function.bind fn<Void> = instance Canceller::Run()
call neoCLR.Runtime.SchedulerSubmit(fn<Void>)
pop
ldloc operation
call neoCLR.Runtime.RuntimeAsyncWaitCompletion(Int64)
brtrue Bad
ldloc operation
call neoCLR.Runtime.RuntimeAsyncIsCancelled(Int64)
brfalse Bad
ldloc operation
call neoCLR.Runtime.RuntimeAsyncWaitCompletion(Int64)
brtrue Bad
ldloc operation
call neoCLR.Runtime.RuntimeAsyncRelease(Int64)
brfalse Bad
ldc.i4 0
call neoCLR.Runtime.RuntimeAsyncDelay(Int32)
stloc operation
ldloc operation
call neoCLR.Runtime.RuntimeAsyncWaitCompletion(Int64)
brfalse Bad
ldloc operation
call neoCLR.Runtime.RuntimeAsyncIsCancelled(Int64)
brtrue Bad
ldloc operation
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ret
Bad:
fault "cancelled observer aborted delivery or corrupted another operation"
"#,
        r#"
.type class Canceller
.field Operation Int64
.method instance .ctor() -> noresult
ret
.end
.method instance Run() -> Void
ldarg 0
ldfld Canceller::Operation
call neoCLR.Runtime.RuntimeAsyncCancel(Int64)
pop
call neoCLR.Runtime.GCCollect()
ldvoid
ret
.end
.end
"#,
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(result.value, Value::Int32(0));
}

#[test]
fn completion_observation_rejects_unknown_and_released_handles() {
    for service in ["RuntimeAsyncIsCancelled", "RuntimeAsyncWaitCompletion"] {
        for setup in ["ldc.i8 0".to_owned(), format!("ldc.i4 0\ncall neoCLR.Runtime.RuntimeAsyncDelay(Int32)\nstloc operation\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncIsCompleted(Int64)\npop\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncRelease(Int64)\npop\nldloc operation")] {
            let fault = program(&format!("{setup}\ncall neoCLR.Runtime.{service}(Int64)\npop\nldc.i4 0\nret"), "").run(Limits::default()).unwrap_err();
            assert!(fault.message.contains("Unknown or released"));
        }
    }
}

fn self_wait_program(service: &str, cancel_before_resume: bool) -> LoadedProgram {
    let extra = BODY.replace(".field Phase Int32", ".field Phase Int32\n.field OwnOperation Int64")
        .replace("ldc.i4 42\nret", "ldarg 0\ncall instance AsyncBody::ObserveSelf()\npop\nldc.i4 42\nret")
        .replace(".end\n.end\n", &format!(".end\n.method instance ObserveSelf() -> Boolean\nldarg 0\nldfld AsyncBody::OwnOperation\ncall neoCLR.Runtime.{service}(Int64)\n{}ret\n.end\n.end\n", if service == "RuntimeAsyncAwait" { "pop\nldc.bool true\n" } else { "" }));
    let setup = format!(
        "stloc operation\nldloc body\nldloc operation\nstfld AsyncBody::OwnOperation\n{}",
        if cancel_before_resume {
            "ldloc operation\ncall neoCLR.Runtime.RuntimeAsyncCancel(Int64)\npop\n"
        } else {
            ""
        }
    );
    let main = START_BODY.replace("stloc operation", &setup)
        .replace("ldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret", "ldloc operation\ncall neoCLR.Runtime.RuntimeAsyncWaitCompletion(Int64)\npop\nldc.i4 0\nret");
    program(&main, &extra)
}

#[test]
fn pending_body_self_wait_fails_instead_of_hanging() {
    for service in ["RuntimeAsyncAwait", "RuntimeAsyncWaitCompletion"] {
        let p = self_wait_program(service, false);
        let token = CancellationToken::new();
        let (stop, stopped) = std::sync::mpsc::channel();
        let fault = std::thread::scope(|scope| {
            let cancel = token.clone();
            scope.spawn(move || {
                if stopped
                    .recv_timeout(std::time::Duration::from_secs(2))
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
            result.unwrap_err()
        });
        assert_eq!(fault.code, FaultCode::RuntimeError);
        assert!(fault
            .message
            .contains("cannot await its own pending operation"));
        assert!(fault.function.unwrap().contains("ObserveSelf"));
    }
}

#[test]
fn body_can_observe_its_own_terminal_cancellation() {
    assert_eq!(
        bounded_run(self_wait_program("RuntimeAsyncWaitCompletion", true)).value,
        Value::Int32(0)
    );
}

#[test]
fn independently_dispatched_callback_can_await_the_submitting_body() {
    let extra = BODY.replace(".field Phase Int32", ".field Phase Int32\n.field OwnOperation Int64")
        .replace("ldc.i4 42\nret", "ldarg 0\nfunction.bind fn<Void> = instance AsyncBody::Observe()\ncall neoCLR.Runtime.SchedulerSubmit(fn<Void>)\npop\ncall neoCLR.Runtime.RuntimeAsyncYield()\npop\nldc.i4 42\nret")
        .replace(".end\n.end\n", r#".end
.method instance Observe() -> Void
ldarg 0
ldfld AsyncBody::OwnOperation
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ldc.i4 42
ceq
brfalse Bad
call neoCLR.Runtime.GCCollect()
ldvoid
ret
Bad:
fault "independent observer lost body result"
.end
.end
"#);
    let main = START_BODY.replace(
        "stloc operation",
        "stloc operation\nldloc body\nldloc operation\nstfld AsyncBody::OwnOperation",
    );
    let extra = format!(".function neoCLR.Runtime.RuntimeAsyncYield() -> Void\n.methodimpl InternalCall\n.end\n{extra}");
    assert_eq!(bounded_run(program(&main, &extra)).value, Value::Int32(42));
}

#[test]
fn unit_body_suspends_and_repeated_awaits_preserve_unit_result() {
    let body = BODY
        .replace("Run() -> Int32", "Run() -> Void")
        .replace("ldc.i4 42\nret", "ldvoid\nret");
    let main = START_BODY.replace("fn<Int32>", "fn<Void>")
        .replace("call neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret", "call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)\npop\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)\npop\nldloc operation\ncall neoCLR.Runtime.RuntimeAsyncRelease(Int64)\nbrfalse Bad\nldloc body\nldfld AsyncBody::Phase\nret");
    assert_eq!(bounded_run(program(&main, &body)).value, Value::Int32(2));
}

#[test]
fn unit_body_can_complete_eagerly() {
    let body = BODY
        .replace("Run() -> Int32", "Run() -> Void")
        .replace(
            "ldarg 0\nldfld AsyncBody::Gate\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\npop",
            "",
        )
        .replace("ldc.i4 42\nret", "ldvoid\nret");
    let main = r#"
.local AsyncBody body
newobj instance AsyncBody::.ctor()
stloc body
ldloc body
function.bind fn<Void> = instance AsyncBody::Run()
call neoCLR.Runtime.RuntimeAsyncStart(fn<Void>)
stloc operation
ldloc operation
call neoCLR.Runtime.RuntimeAsyncIsCompleted(Int64)
brfalse Bad
ldloc operation
call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)
pop
ldloc body
ldfld AsyncBody::Phase
ret
Bad:
fault "unit body did not complete eagerly"
"#;
    assert_eq!(bounded_run(program(main, &body)).value, Value::Int32(2));
}

#[test]
fn unit_and_integer_results_cannot_be_confused() {
    let unit = BODY
        .replace("Run() -> Int32", "Run() -> Void")
        .replace("ldc.i4 42\nret", "ldvoid\nret");
    let fault = program(&START_BODY.replace("fn<Int32>", "fn<Void>"), &unit)
        .run(Limits::default())
        .unwrap_err();
    assert!(fault.message.contains("Int32 result"));
    let main = START_BODY.replace(
        "call neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret",
        "call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)\npop\nldc.i4 0\nret",
    );
    let fault = program(&main, BODY).run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("unit result"));
}

#[test]
fn manual_unit_completion_is_retained_and_one_shot() {
    let main = r#"
call neoCLR.Runtime.RuntimeAsyncCreate()
stloc operation
ldloc operation
call neoCLR.Runtime.RuntimeAsyncCompleteUnit(Int64)
brfalse Bad
ldloc operation
ldc.i4 42
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
brtrue Bad
ldloc operation
call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)
pop
ldloc operation
call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)
pop
ldc.i4 7
ret
Bad:
fault "unit completion was not one-shot"
"#;
    assert_eq!(bounded_run(program(main, "")).value, Value::Int32(7));
}

#[test]
fn cli_no_result_unit_await_does_not_push_a_value() {
    let module = neoclr::assemble(
        r#"
.module UnitAwait
.entry Main
.function neoCLR.Runtime.RuntimeAsyncCreate() -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncCompleteUnit(Int64 id) -> Boolean
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64 id) -> noresult
.methodimpl InternalCall
.end
.function Main() -> Int32
.local Int64 operation
call neoCLR.Runtime.RuntimeAsyncCreate()
stloc operation
ldloc operation
call neoCLR.Runtime.RuntimeAsyncCompleteUnit(Int64)
pop
ldc.i4 42
ldloc operation
call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)
ret
.end
"#,
    )
    .unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    assert_eq!(bounded_run(p).value, Value::Int32(42));
}

#[test]
fn entry_drain_tracks_its_frame_across_detached_unit_body_calls() {
    let body = BODY
        .replace("Run() -> Int32", "Run() -> Void")
        .replace("ldc.i4 42\nret", "call Helper()\nldvoid\nret");
    let extra = format!(
        r#"{body}
.function Helper() -> noresult
ret
.end
.function neoCLR.Runtime.DrainEntryTasks() -> Void
.methodimpl InternalCall
.end
.type class Observer
.field Handle Int64
.method instance .ctor() -> noresult
ret
.end
.method instance Run() -> Void
ldarg 0
ldfld Observer::Handle
call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)
pop
ldvoid
ret
.end
.end
"#
    );
    let tail = r#"
newobj instance Observer::.ctor()
stloc observer
ldloc observer
ldloc operation
stfld Observer::Handle
ldloc observer
function.bind fn<Void> = instance Observer::Run()
call neoCLR.Runtime.SchedulerSubmit(fn<Void>)
pop
call neoCLR.Runtime.DrainEntryTasks()
pop
ldloc operation
call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)
pop
ldc.i4 42
ret
"#;
    let main = START_BODY.replace(".local AsyncBody body", ".local AsyncBody body\n.local Observer observer").replace("fn<Int32>", "fn<Void>").replace(
        "ldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\nret",
        tail,
    );
    assert_eq!(bounded_run(program(&main, &extra)).value, Value::Int32(42));
}

#[test]
fn parked_activations_keep_managed_completion_dispatch_alive() {
    assert_managed_dispatch_while_parked("", false);
}

#[test]
fn suspended_managed_queue_callback_is_not_drained_recursively() {
    assert_managed_dispatch_while_parked(
        "ldc.i4 1\ncall neoCLR.Runtime.RuntimeAsyncDelay(Int32)\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\npop",
        false,
    );
}

#[test]
fn managed_notifications_are_drained_again_after_an_activation_reparks() {
    assert_managed_dispatch_while_parked("", true);
}

fn assert_managed_dispatch_while_parked(pause: &str, repeat_direct_wait: bool) {
    let source = r#".module App
.entry Main
.function Main() -> Int32
.local Int64 operation
.local System.Tasks.TaskQueue queue
.local Int64 body
newobj instance System.Tasks.TaskQueue::.ctor()
stloc queue
ldloc queue
call neoCLR.Runtime.RegisterDefaultTaskQueue(System.Tasks.TaskQueue)
pop
call neoCLR.Runtime.RuntimeAsyncCreate()
stloc operation
ldloc queue
ldloc operation
stfld System.Tasks.TaskQueue::Gate
ldloc queue
function.bind fn<Int32> = instance System.Tasks.TaskQueue::Body()
call neoCLR.Runtime.RuntimeAsyncStart(fn<Int32>)
stloc body
ldloc body
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ret
.end
"#;
    let source = if repeat_direct_wait {
        source.replace(
            "ldloc queue\nfunction.bind fn<Int32> = instance System.Tasks.TaskQueue::Body()\ncall neoCLR.Runtime.RuntimeAsyncStart(fn<Int32>)\nstloc body\nldloc body",
            "ldloc operation\ncall neoCLR.Runtime.RuntimeAsyncAwait(Int64)\npop\ncall neoCLR.Runtime.RuntimeAsyncCreate()\nstloc operation\nldloc queue\nldloc operation\nstfld System.Tasks.TaskQueue::Gate\nldloc operation",
        )
    } else {
        source.to_owned()
    };
    let system = neoclr::assemble(&format!(
        r#".module System
.function neoCLR.Runtime.RuntimeAsyncDelay(Int32 ms) -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncCreate() -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncStart(fn<Int32> body) -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncAwait(Int64 id) -> Int32
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncComplete(Int64 id, Int32 value) -> Boolean
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RegisterDefaultTaskQueue(System.Tasks.TaskQueue queue) -> Void
.methodimpl InternalCall
.end
.type class System.Tasks.TaskQueue
.field Gate Int64
.field Running Boolean
.method instance .ctor() -> noresult
ret
.end
.method instance Drain() -> noresult
ldarg 0
ldfld System.Tasks.TaskQueue::Running
brfalse Begin
fault "queue was drained recursively"
Begin:
ldarg 0
ldc.bool true
stfld System.Tasks.TaskQueue::Running
ldarg 0
function.bind fn<Void> = instance System.Tasks.TaskQueue::Signal()
call instance fn<Void>::Invoke()
pop
ldarg 0
ldc.bool false
stfld System.Tasks.TaskQueue::Running
ret
.end
.method instance Signal() -> Void
{pause}
ldarg 0
ldfld System.Tasks.TaskQueue::Gate
ldc.i4 42
call neoCLR.Runtime.RuntimeAsyncComplete(Int64, Int32)
pop
ldvoid
ret
.end
.method instance Body() -> Int32
ldarg 0
ldfld System.Tasks.TaskQueue::Gate
call neoCLR.Runtime.RuntimeAsyncAwait(Int64)
ret
.end
.end
"#,
    ))
    .unwrap();
    let app =
        neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(&source)], &system)
            .unwrap()
            .remove(0);
    let p = LoadedProgram::with_library(&app, &system).unwrap();
    p.verify().unwrap();
    assert_eq!(bounded_run(p).value, Value::Int32(42));
}

#[test]
fn managed_body_ownership_reclaims_abandoned_frames_and_signals() {
    let extra = r#"
.type class System.Object
.end
.type class Owned
.extends System.Object
.method instance Run() -> Void
call neoCLR.Runtime.RuntimeAsyncCurrentOwner()
castclass Owned
ldarg 0
ref.eq
brfalse Bad
ldarg 0
call neoCLR.Runtime.RuntimeAsyncCreateOwned(System.Object)
call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)
pop
fault "abandoned body resumed"
Bad:
fault "managed body lost current owner"
.end
.end
.function neoCLR.Runtime.RuntimeAsyncStartManaged(System.Object owner, fn<Void> body) -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncCreateOwned(System.Object owner) -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.RuntimeAsyncCurrentOwner() -> System.Object
.methodimpl InternalCall
.end
.function Abandon() -> noresult
.local Owned owner
newobj Owned
stloc owner
ldloc owner
ldloc owner
function.bind fn<Void> = instance Owned::Run()
call neoCLR.Runtime.RuntimeAsyncStartManaged(System.Object, fn<Void>)
pop
ret
.end
"#;
    let main = r#"
.local Int32 iteration
ldc.i4 0
stloc iteration
Again:
call Abandon()
call neoCLR.Runtime.GCCollect()
ldloc iteration
ldc.i4 1
add
stloc iteration
ldloc iteration
ldc.i4 80
clt
brtrue Again
ldc.i4 42
ret
"#;
    assert_eq!(bounded_run(program(main, extra)).value, Value::Int32(42));
}

#[test]
fn ready_managed_body_survives_collection_after_completion_owner_is_dropped() {
    let extra = r#"
.type class System.Object
.end
.type class Owned
.extends System.Object
.field Gate Int64
.field Done Int64
.method instance Run() -> Void
ldarg 0
ldfld Owned::Gate
call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)
pop
call neoCLR.Runtime.GCCollect()
ldarg 0
ldfld Owned::Done
call neoCLR.Runtime.RuntimeAsyncCompleteUnit(Int64)
pop
ldvoid
ret
.end
.end
.function neoCLR.Runtime.RuntimeAsyncStartManaged(System.Object owner, fn<Void> body) -> Int64
.methodimpl InternalCall
.end
.function CompleteAndDrop(Int64 gate, Int64 done) -> noresult
.local Owned owner
ldarg gate
ldarg done
newobj Owned
stloc owner
ldloc owner
ldloc owner
function.bind fn<Void> = instance Owned::Run()
call neoCLR.Runtime.RuntimeAsyncStartManaged(System.Object, fn<Void>)
pop
ldarg gate
call neoCLR.Runtime.RuntimeAsyncCompleteUnit(Int64)
pop
ret
.end
"#;
    let main = r#"
.local Int64 done
call neoCLR.Runtime.RuntimeAsyncCreate()
stloc operation
call neoCLR.Runtime.RuntimeAsyncCreate()
stloc done
ldloc operation
ldloc done
call CompleteAndDrop(Int64, Int64)
call neoCLR.Runtime.GCCollect()
ldloc done
call neoCLR.Runtime.RuntimeAsyncAwaitUnit(Int64)
pop
ldloc operation
call neoCLR.Runtime.RuntimeAsyncRelease(Int64)
pop
ldloc done
call neoCLR.Runtime.RuntimeAsyncRelease(Int64)
pop
ldc.i4 42
ret
"#;
    assert_eq!(bounded_run(program(main, extra)).value, Value::Int32(42));
    let draining = main.replace("call neoCLR.Runtime.GCCollect()", "call neoCLR.Runtime.GCCollect()\ncall neoCLR.Runtime.DrainEntryTasks()\npop");
    let extra = format!("{extra}\n.function neoCLR.Runtime.DrainEntryTasks() -> Void\n.methodimpl InternalCall\n.end\n");
    assert_eq!(bounded_run(program(&draining, &extra)).value, Value::Int32(42));
}
