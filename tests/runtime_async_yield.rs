//! Private interpreter frame-retention experiment, with no async state machine.
use neoclr::{FaultCode, Limits, LoadedProgram, Value};

fn program(callback: &str, main: &str) -> LoadedProgram {
    let source = format!(
        r#"
.module RuntimeYield
.entry Main
.function neoCLR.Runtime.RuntimeAsyncYield() -> Void
.methodimpl InternalCall
.end
.function neoCLR.Runtime.SchedulerSubmit(fn<Void> callback) -> Int32
.methodimpl InternalCall
.end
.function neoCLR.Runtime.GCCollect() -> void
.methodimpl InternalCall
.end
.type class Counter
.field Value Int32
.method instance .ctor() -> void
ret
.end
.method instance Complete() -> Void
{callback}
ldvoid
ret
.end
.end
.function Nested(Counter counter) -> Int32
.local Int32 saved
ldc.i4 7
stloc saved
ldc.i4 10
call neoCLR.Runtime.RuntimeAsyncYield()
pop
ldloc saved
add
ldarg counter
ldfld Counter::Value
add
ret
.end
.function Main() -> Int32
{main}
.end
"#
    );
    let module = neoclr::assemble(&source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
}

const START: &str = r#"
.local Counter counter
newobj instance Counter::.ctor()
stloc counter
ldloc counter
function.bind fn<Void> = instance Counter::Complete()
call neoCLR.Runtime.SchedulerSubmit(fn<Void>)
pop
ldloc counter
call Nested(Counter)
ret
"#;

#[test]
fn resumes_nested_call_with_locals_evaluation_stack_and_heap_roots() {
    let result = program(
        "call neoCLR.Runtime.GCCollect()\nldarg 0\nldc.i4 25\nstfld Counter::Value",
        START,
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
    assert!(result.heap.is_empty());
}

#[test]
fn empty_yield_resumes_and_nested_callback_yield_returns_to_its_own_caller() {
    let result = program(
        "call neoCLR.Runtime.RuntimeAsyncYield()\npop\nldarg 0\nldc.i4 25\nstfld Counter::Value",
        START,
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn callback_fault_does_not_resume_the_parked_caller() {
    let fault = program("fault \"yield callback fault\"", START)
        .run(Limits::default())
        .unwrap_err();
    assert_eq!(fault.code, FaultCode::UserFault);
    assert!(fault.message.contains("yield callback fault"));
}

#[test]
fn retained_call_chain_remains_charged_to_frame_budget() {
    let fault = program("", START)
        .run(Limits {
            frames: 2,
            ..Limits::default()
        })
        .unwrap_err();
    assert_eq!(fault.code, FaultCode::StackOverflow);
}

#[test]
fn yield_dispatches_one_callback_per_boundary() {
    let main = r#"
.local Counter counter
newobj instance Counter::.ctor()
stloc counter
ldloc counter
function.bind fn<Void> = instance Counter::Complete()
call neoCLR.Runtime.SchedulerSubmit(fn<Void>)
pop
ldloc counter
function.bind fn<Void> = instance Counter::Complete()
call neoCLR.Runtime.SchedulerSubmit(fn<Void>)
pop
call neoCLR.Runtime.RuntimeAsyncYield()
pop
ldloc counter
ldfld Counter::Value
ldc.i4 1
ceq
brfalse Bad
call neoCLR.Runtime.RuntimeAsyncYield()
pop
ldloc counter
ldfld Counter::Value
ret
Bad:
fault "yield drained more than one callback"
"#;
    let callback = "ldarg 0\nldarg 0\nldfld Counter::Value\nldc.i4 1\nadd\nstfld Counter::Value";
    assert_eq!(
        program(callback, main)
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(2)
    );
}

#[test]
fn reserved_signature_requires_no_parameters_and_unit_result() {
    for signature in ["Int32", "Void"] {
        let source = format!(
            ".module Test\n.function neoCLR.Runtime.RuntimeAsyncYield() -> {signature}\n.methodimpl InternalCall\n.end\n"
        );
        assert_eq!(neoclr::assemble(&source).is_ok(), signature == "Void");
    }
    assert!(neoclr::assemble(".module Test\n.function neoCLR.Runtime.RuntimeAsyncYield(Int32 value) -> Void\n.methodimpl InternalCall\n.end\n").is_err());
}

#[test]
fn resumed_unit_result_obeys_evaluation_stack_budget() {
    let fault = program("", START)
        .run(Limits {
            stack: 1,
            ..Limits::default()
        })
        .unwrap_err();
    assert_eq!(fault.code, FaultCode::EvaluationStackOverflow);
}

#[test]
fn cli_no_result_yield_resumes_without_pushing_unit() {
    let module = neoclr::assemble(".module Test\n.entry Main\n.function neoCLR.Runtime.RuntimeAsyncYield() -> void\n.methodimpl InternalCall\n.end\n.function Main() -> Int32\nldc.i4 42\ncall neoCLR.Runtime.RuntimeAsyncYield()\nret\n.end\n").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program
            .run(Limits {
                stack: 1,
                ..Limits::default()
            })
            .unwrap()
            .value,
        Value::Int32(42)
    );
}
