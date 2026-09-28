//! Guest execution acceptance for instruction-boundary suspension. No public Run
//! facade is implied; each test keeps unsupported shared host services out of scope.
use super::*;
use crate::{CollectionReason, shared_heap::Owner, task_work::Work};
use std::sync::mpsc;

#[test]
fn native_guest_callback_preserves_capture_and_result_through_collection_between_steps() {
    let module = crate::assemble(
        r#"
.module CapturedWork
.type class Capture
.field Number Int32
.end
.function Callback(Capture captured) -> Capture
.local Capture saved
ldarg captured
stloc saved
ldc.i4 0
newobj Capture
pop
ldloc saved
ldloc saved
ldfld Capture::Number
ldc.i4 2
add
stfld Capture::Number
ldloc saved
ret
.end
"#,
    )
    .unwrap();
    let function = module.functions[0].clone();
    let owner = Owner::new(2);
    let mut caller = owner.participant().unwrap();
    let (step_tx, step_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let (mut work, job, id) = {
        let mut heap = caller.enter();
        let ty = Type::from_name("Capture");
        let id = heap
            .allocate(Value::Object {
                ty,
                fields: vec![Value::Int32(40)],
            })
            .unwrap();
        let capture = Value::ObjectReference(crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        });
        let mut work = Work::new(&heap, 1, crate::CancellationToken::new(), Limits::default());
        let job = work
            .submit(
                &mut heap,
                vec![capture],
                move |mut context, mut captures, control| {
                    let capture = captures.pop().unwrap();
                    // Frame metadata and native resources are created on the worker itself.
                    let mut frames = vec![Frame::new(function, vec![capture])?];
                    let options = ExecutionOptions::default();
                    let mut state = InstructionState::new(options.limits);
                    let mut memory = state.invocation.memory.clone();
                    let mut output = Vec::new();
                    let mut bytes = [Vec::new(), Vec::new()];
                    let mut libraries = None;
                    let value = loop {
                        let progress = {
                            let mut heap = context.enter()?;
                            interpret_instructions(
                                &module,
                                &mut frames,
                                &options,
                                &mut libraries,
                                &mut heap,
                                &mut memory,
                                &mut output,
                                &mut bytes,
                                &mut state,
                                1,
                            )?
                        };
                        match progress {
                            InstructionProgress::Completed(value) => break value,
                            InstructionProgress::Waiting => panic!("unexpected host wait"),
                            InstructionProgress::HostCall(_) => panic!("unexpected host call"),
                            InstructionProgress::Suspended => {
                                step_tx
                                    .send(())
                                    .map_err(|_| Fault::new("test observer closed"))?;
                                loop {
                                    match resume_rx
                                        .recv_timeout(std::time::Duration::from_millis(10))
                                    {
                                        Ok(()) => break,
                                        Err(mpsc::RecvTimeoutError::Timeout)
                                            if !control.is_cancelled() => {}
                                        _ => {
                                            return Err(Fault::coded(
                                                crate::FaultCode::ExecutionCancelled,
                                                "test cancelled",
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    };
                    drop(state); // Any source teardown happens outside managed graph access.
                    context.complete(|_| Ok(value))
                },
            )
            .unwrap();
        (work, job, id)
    };
    let mut pauses = 0;
    while step_rx.recv().is_ok() {
        let mut heap = caller.enter();
        heap.collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(heap.get(id).is_some());
        pauses += 1;
        drop(heap);
        resume_tx.send(()).unwrap();
    }
    assert!(pauses >= 10);
    let completion = work.join(job).unwrap();
    {
        let mut heap = caller.enter();
        heap.collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        let result = completion.receive(&mut heap, vec![]).unwrap();
        let Value::ObjectReference(result) = result else {
            panic!()
        };
        assert_eq!(result.allocation_id(), id);
        assert_eq!(result.reference.read_field(0).unwrap(), Value::Int32(42));
        assert_eq!(heap.len(), 1);
        heap.collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(heap.is_empty());
    }
    work.shutdown().unwrap();
    drop(caller);
    owner.into_heap().unwrap();
}

#[test]
fn suspension_does_not_reset_instruction_budget() {
    let module =
        crate::assemble(".module Loop\n.function Spin() -> Void\nagain:\nbr again\n.end").unwrap();
    let mut frames = vec![Frame::new(module.functions[0].clone(), vec![]).unwrap()];
    let options = ExecutionOptions {
        limits: Limits {
            instructions: 5,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut state = InstructionState::new(options.limits);
    let owner = Owner::new(1);
    let mut participant = owner.participant().unwrap();
    let mut memory = state.invocation.memory.clone();
    let mut output = Vec::new();
    let mut bytes = [Vec::new(), Vec::new()];
    for step in 0..5 {
        let mut heap = participant.enter();
        let result = interpret_instructions(
            &module,
            &mut frames,
            &options,
            &mut None,
            &mut heap,
            &mut memory,
            &mut output,
            &mut bytes,
            &mut state,
            1,
        );
        if step < 4 {
            assert!(matches!(result, Ok(InstructionProgress::Suspended)));
        } else {
            assert!(matches!(
                result,
                Err(Fault {
                    code: crate::FaultCode::InstructionLimitExceeded,
                    ..
                })
            ));
        }
    }
    assert_eq!(state.budget.remaining(), 0);
}

#[test]
fn zero_quantum_is_rejected_without_consuming_budget() {
    let module =
        crate::assemble(".module A\n.function F() -> Int32\nldc.i4 42\nret\n.end").unwrap();
    let mut frames = vec![Frame::new(module.functions[0].clone(), vec![]).unwrap()];
    let options = ExecutionOptions::default();
    let mut state = InstructionState::new(options.limits);
    let owner = Owner::new(1);
    let mut participant = owner.participant().unwrap();
    let mut heap = participant.enter();
    let result = interpret_instructions(
        &module,
        &mut frames,
        &options,
        &mut None,
        &mut heap,
        &mut Default::default(),
        &mut Vec::new(),
        &mut [Vec::new(), Vec::new()],
        &mut state,
        0,
    );
    assert!(result.is_err());
    assert_eq!(state.budget.remaining(), options.limits.instructions);
}

#[test]
fn queue_and_entry_drain_state_survive_collection_at_every_pause() {
    let system = crate::assemble(
        r#"
.module System
.type class System.Tasks.TaskQueue
.field Drains Int32
.method instance .ctor() -> noresult
ret
.end
.method instance Drain() -> noresult
ldarg 0
ldarg 0
ldfld System.Tasks.TaskQueue::Drains
ldc.i4 1
add
stfld System.Tasks.TaskQueue::Drains
ret
.end
.end
.function neoCLR.Runtime.RegisterDefaultTaskQueue(System.Tasks.TaskQueue queue) -> Void
.methodimpl InternalCall
.end
.function neoCLR.Runtime.DrainEntryTasks() -> Void
.methodimpl InternalCall
.end
"#,
    )
    .unwrap();
    let source = r#"
.module App
.entry Main
.function Main() -> System.Tasks.TaskQueue
.local System.Tasks.TaskQueue queue
newobj instance System.Tasks.TaskQueue::.ctor()
stloc queue
ldloc queue
call neoCLR.Runtime.RegisterDefaultTaskQueue(System.Tasks.TaskQueue)
pop
call neoCLR.Runtime.DrainEntryTasks()
pop
ldloc queue
ret
.end
"#;
    let app =
        crate::assembler::read_modules(&[crate::assembler::ModuleInput::Source(source)], &system)
            .unwrap()
            .remove(0);
    let module = crate::library::link(&app, &system).unwrap();
    let entry = module
        .functions
        .iter()
        .find(|f| f.name == module.entry)
        .unwrap()
        .clone();
    let mut frames = vec![Frame::new(entry, vec![]).unwrap()];
    let options = ExecutionOptions::default();
    let mut state = InstructionState::new(options.limits);
    let owner = Owner::new(2);
    let mut participant = owner.participant().unwrap();
    let mut collector = owner.participant().unwrap();
    let mut memory = state.invocation.memory.clone();
    let mut output = Vec::new();
    let mut bytes = [Vec::new(), Vec::new()];
    let mut pauses = 0;
    let value = loop {
        let progress = {
            let mut heap = participant.enter();
            interpret_instructions(
                &module,
                &mut frames,
                &options,
                &mut None,
                &mut heap,
                &mut memory,
                &mut output,
                &mut bytes,
                &mut state,
                1,
            )
            .unwrap()
        };
        match progress {
            InstructionProgress::Completed(value) => break value,
            InstructionProgress::Waiting => panic!("unexpected host wait"),
            InstructionProgress::HostCall(_) => panic!("unexpected host call"),
            InstructionProgress::Suspended => {
                pauses += 1;
                collector
                    .enter()
                    .collect(vec![], CollectionReason::ExplicitRequest)
                    .unwrap();
            }
        }
    };
    // Queue bookkeeping now retains graph access; ordinary guest work still yields.
    assert!(pauses > 0);
    let Value::ObjectReference(queue) = value else {
        panic!()
    };
    let heap = participant.enter();
    assert_eq!(
        heap.read_reference(&queue.reference).unwrap(),
        Value::Object {
            ty: queue.reference.target().clone(),
            fields: vec![Value::Int32(2)],
        }
    );
}

#[test]
fn completion_wait_releases_graph_and_resumes_once() {
    completion_wait_probe(false);
}

#[test]
fn completion_wait_observes_cancellation_without_a_guest_frame() {
    completion_wait_probe(true);
}

fn completion_wait_probe(cancel: bool) {
    use crate::socket_io::Operation;
    let module = crate::assemble(
        r#"
.module System
.type class System.Tasks.TaskQueue
.field Posts Int32
.field Drains Int32
.method instance Post(fn<Void> callback) -> noresult
ldarg 0
ldarg 0
ldfld System.Tasks.TaskQueue::Posts
ldc.i4 1
add
stfld System.Tasks.TaskQueue::Posts
ret
.end
.method instance Drain() -> noresult
ldarg 0
ldarg 0
ldfld System.Tasks.TaskQueue::Drains
ldc.i4 1
add
stfld System.Tasks.TaskQueue::Drains
ret
.end
.end
.type class Result
.field Number Int32
.field Queue System.Tasks.TaskQueue
.end
.function Main(Result result) -> Result
ldarg result
ret
.end
"#,
    )
    .unwrap();
    let token = crate::CancellationToken::new();
    let options = ExecutionOptions {
        cancellation: Some(token.clone()),
        ..Default::default()
    };
    let mut state = InstructionState::new(options.limits);
    let owner = Owner::new(2);
    let mut participant = owner.participant().unwrap();
    let mut collector = owner.participant().unwrap();
    let (queue, result, port) = {
        let mut heap = participant.enter();
        let id = heap
            .allocate(Value::Object {
                ty: Type::from_name("System.Tasks.TaskQueue"),
                fields: vec![Value::Int32(0), Value::Int32(0)],
            })
            .unwrap();
        let queue = Value::ObjectReference(crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        });
        let callback = Value::Function(crate::Function {
            object_view: false,
            identity: std::sync::Arc::new(()),
            ty: crate::assembler::parse_type("fn<Void>").unwrap(),
            target: crate::assembler::parse_function_ref(
                "instance System.Tasks.TaskQueue::Drain()",
            )
            .unwrap(),
            receiver: Some(Box::new(queue.clone())),
        });
        let Value::Erased(listener) = state
            .invocation
            .dispatch
            .lock()
            .unwrap()
            .scheduler
            .sockets
            .invoke(
                Operation::Listen,
                &[
                    Value::String("127.0.0.1".into()),
                    Value::Int32(0),
                    Value::Int32(1),
                ],
                &heap,
            )
            .unwrap()
        else {
            panic!()
        };
        let Value::Erased(port) = state
            .invocation
            .dispatch
            .lock()
            .unwrap()
            .scheduler
            .sockets
            .invoke(Operation::LocalPort, &[*listener.clone()], &heap)
            .unwrap()
        else {
            panic!()
        };
        let Value::Int32(port) = *port else { panic!() };
        let Value::Erased(operation) = state
            .invocation
            .dispatch
            .lock()
            .unwrap()
            .scheduler
            .sockets
            .invoke(Operation::Accept, &[*listener, callback], &heap)
            .unwrap()
        else {
            panic!()
        };
        assert!(matches!(*operation, Value::Int64(_)));
        let result_id = heap
            .allocate(Value::Object {
                ty: Type::from_name("Result"),
                fields: vec![Value::Int32(42), queue.clone()],
            })
            .unwrap();
        let result = Value::ObjectReference(crate::value::ObjectReference {
            reference: heap.address(result_id).unwrap(),
            view: None,
        });
        (queue, result, port)
    };
    state.invocation.dispatch.lock().unwrap().default_task_queue = Some(queue.clone());
    let entry = module
        .functions
        .iter()
        .find(|f| f.name == "Main")
        .unwrap()
        .clone();
    let mut frames = vec![Frame::new(entry, vec![result.clone()]).unwrap()];
    let mut memory = state.invocation.memory.clone();
    let mut output = Vec::new();
    let mut bytes = [Vec::new(), Vec::new()];
    let mut run = |state: &mut InstructionState, frames: &mut Vec<Frame>| {
        let mut heap = participant.enter();
        interpret_instructions(
            &module,
            frames,
            &options,
            &mut None,
            &mut heap,
            &mut memory,
            &mut output,
            &mut bytes,
            state,
            1024,
        )
    };
    assert!(matches!(
        run(&mut state, &mut frames),
        Ok(InstructionProgress::Waiting)
    ));
    assert!(frames.is_empty());
    let fuel = state.budget.remaining();
    for _ in 0..3 {
        state.invocation.park();
        assert!(matches!(
            run(&mut state, &mut frames),
            Ok(InstructionProgress::Waiting)
        ));
        assert_eq!(state.budget.remaining(), fuel);
    }
    // Exercise access from another native participant while no guest frame exists.
    let observed = queue.clone();
    std::thread::spawn(move || {
        let mut heap = collector.enter();
        heap.collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        let Value::ObjectReference(queue) = observed else {
            panic!()
        };
        assert_eq!(queue.reference.read_field(1).unwrap(), Value::Int32(1));
        assert_eq!(heap.len(), 2);
    })
    .join()
    .unwrap();
    if cancel {
        token.cancel();
        state.invocation.park();
        assert!(matches!(
            run(&mut state, &mut frames),
            Err(Fault {
                code: crate::FaultCode::ExecutionCancelled,
                ..
            })
        ));
        assert_eq!(state.budget.remaining(), fuel);
        return;
    }
    let _peer = std::net::TcpStream::connect(("127.0.0.1", port as u16)).unwrap();
    let value = loop {
        match run(&mut state, &mut frames).unwrap() {
            InstructionProgress::Waiting => state.invocation.park(),
            InstructionProgress::HostCall(_) => panic!("unexpected host call"),
            InstructionProgress::Suspended => panic!("unexpected instruction quantum"),
            InstructionProgress::Completed(value) => break value,
        }
    };
    assert_eq!(value, result);
    let Value::ObjectReference(result) = value else {
        panic!()
    };
    assert_eq!(result.reference.read_field(0).unwrap(), Value::Int32(42));
    let Value::ObjectReference(queue) = queue else {
        panic!()
    };
    assert_eq!(queue.reference.read_field(0).unwrap(), Value::Int32(1));
    assert_eq!(queue.reference.read_field(1).unwrap(), Value::Int32(2));
}

#[test]
fn blocking_guest_console_call_allows_shared_collection_and_mutation() {
    struct CollectingConsole {
        participant: std::sync::Mutex<crate::shared_heap::Participant>,
        capture: crate::value::ObjectReference,
    }
    impl std::fmt::Debug for CollectingConsole {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("CollectingConsole")
        }
    }
    impl crate::Console for CollectingConsole {
        fn read_byte(&self) -> std::io::Result<Option<u8>> {
            let start = std::time::Instant::now();
            let mut participant = self.participant.lock().unwrap();
            let mut heap = participant
                .enter_cancellable(|| start.elapsed().as_secs() >= 2)
                .map_err(|_| std::io::Error::other("host call retained graph access"))?;
            heap.collect(vec![], CollectionReason::ExplicitRequest)
                .unwrap();
            self.capture
                .reference
                .field(0, Type::Int32)
                .unwrap()
                .write(Value::Int32(42))
                .unwrap();
            Ok(Some(65))
        }
        fn write_line(&self, _: &str) -> std::io::Result<()> {
            unreachable!()
        }
    }
    let module = crate::assemble(
        r#"
.module System
.function neoCLR.Runtime.ConsoleReadByte() -> Value
.methodimpl InternalCall
.end
.type class Capture
.field Number Int32
.end
.function Main(Capture capture) -> Capture
call neoCLR.Runtime.ConsoleReadByte()
pop
ldarg capture
ret
.end
"#,
    )
    .unwrap();
    let owner = Owner::new(2);
    let mut participant = owner.participant().unwrap();
    let other = owner.participant().unwrap();
    let capture = {
        let mut heap = participant.enter();
        let id = heap
            .allocate(Value::Object {
                ty: Type::from_name("Capture"),
                fields: vec![Value::Int32(0)],
            })
            .unwrap();
        crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        }
    };
    let options = ExecutionOptions {
        console: Some(std::sync::Arc::new(CollectingConsole {
            participant: std::sync::Mutex::new(other),
            capture: capture.clone(),
        })),
        ..Default::default()
    };
    let mut state = InstructionState::new(options.limits);
    let mut frames = vec![
        Frame::new(
            module
                .functions
                .iter()
                .find(|f| f.name == "Main")
                .unwrap()
                .clone(),
            vec![Value::ObjectReference(capture.clone())],
        )
        .unwrap(),
    ];
    let result = drive_instructions(
        &module,
        &mut frames,
        &options,
        &mut None,
        &mut participant,
        &mut Default::default(),
        &mut Vec::new(),
        &mut [Vec::new(), Vec::new()],
        &mut state,
    )
    .unwrap();
    assert_eq!(result, Value::ObjectReference(capture.clone()));
    assert_eq!(capture.reference.read_field(0).unwrap(), Value::Int32(42));
}

#[test]
fn guest_contexts_share_instruction_fuel_and_live_frame_admission() {
    let module =
        crate::assemble(".module SharedFuel\n.function Spin() -> Void\nloop:\nbr loop\n.end")
            .unwrap();
    let options = ExecutionOptions {
        limits: Limits {
            instructions: 5,
            frames: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    let invocation = crate::invocation::Invocation::new(options.limits);
    let budget = invocation.budget.clone();
    let mut first = InstructionState::with_invocation(invocation.clone());
    let mut second = InstructionState::with_invocation(invocation);
    let owner = Owner::new(1);
    let mut participant = owner.participant().unwrap();
    let mut first_frames = vec![Frame::new(module.functions[0].clone(), vec![]).unwrap()];
    let mut second_frames = vec![Frame::new(module.functions[0].clone(), vec![]).unwrap()];
    let mut run = |frames: &mut Vec<Frame>, state: &mut InstructionState| {
        interpret_instructions(
            &module,
            frames,
            &options,
            &mut None,
            &mut participant.enter(),
            &mut Default::default(),
            &mut Vec::new(),
            &mut [Vec::new(), Vec::new()],
            state,
            1,
        )
    };
    assert!(matches!(
        run(&mut first_frames, &mut first),
        Ok(InstructionProgress::Suspended)
    ));
    assert_eq!(budget.remaining(), 4);
    assert!(matches!(
        run(&mut second_frames, &mut second),
        Err(Fault {
            code: crate::FaultCode::StackOverflow,
            ..
        })
    ));
    assert_eq!(
        budget.remaining(),
        4,
        "failed frame admission consumes no instructions"
    );
    drop(first_frames);
    for _ in 0..3 {
        assert!(matches!(
            run(&mut second_frames, &mut second),
            Ok(InstructionProgress::Suspended)
        ));
    }
    assert!(matches!(
        run(&mut second_frames, &mut second),
        Err(Fault {
            code: crate::FaultCode::InstructionLimitExceeded,
            ..
        })
    ));
    assert_eq!(first.budget.remaining(), 0);
    assert_eq!(second.budget.remaining(), 0);
}

#[test]
fn guest_contexts_share_intern_owner_and_cannot_reset_its_limit() {
    let module = crate::assemble(
        r#"
.module System
.function neoCLR.Runtime.StringIntern(String value) -> String
.methodimpl InternalCall
.end
.function Canon(String input) -> String
ldarg input
call neoCLR.Runtime.StringIntern(String)
ret
.end
"#,
    )
    .unwrap();
    let options = ExecutionOptions {
        limits: Limits {
            intern_entries: 1,
            intern_bytes: 4,
            ..Default::default()
        },
        ..Default::default()
    };
    let invocation = crate::invocation::Invocation::new(options.limits);
    let owner = Owner::new(1);
    let run = |text: &str| {
        let mut participant = owner.participant().unwrap();
        let mut state = InstructionState::with_invocation(invocation.clone());
        let mut frames = vec![
            Frame::new(
                module
                    .functions
                    .iter()
                    .find(|f| f.name == "Canon")
                    .unwrap()
                    .clone(),
                vec![Value::String(text.into())],
            )
            .unwrap(),
        ];
        drive_instructions(
            &module,
            &mut frames,
            &options,
            &mut None,
            &mut participant,
            &mut Default::default(),
            &mut Vec::new(),
            &mut [Vec::new(), Vec::new()],
            &mut state,
        )
    };
    let Value::String(first) = run("same").unwrap() else {
        panic!()
    };
    let Value::String(second) = run("same").unwrap() else {
        panic!()
    };
    assert!(first.same_owner(&second));
    assert_eq!(
        run("next").unwrap_err().code,
        crate::FaultCode::InternPoolLimitExceeded
    );
}

#[test]
fn guest_contexts_observe_one_default_queue_after_submitter_exits() {
    let module = crate::assemble(
        r#"
.module System
.type class System.Tasks.TaskQueue
.field Drains Int32
.method instance Drain() -> noresult
ldarg 0
ldarg 0
ldfld System.Tasks.TaskQueue::Drains
ldc.i4 1
add
stfld System.Tasks.TaskQueue::Drains
ret
.end
.end
.function neoCLR.Runtime.DefaultTaskQueue() -> System.Tasks.TaskQueue
.methodimpl InternalCall
.end
.function ReadQueue() -> System.Tasks.TaskQueue
call neoCLR.Runtime.DefaultTaskQueue()
ret
.end
"#,
    )
    .unwrap();
    let owner = Owner::new(1);
    let options = ExecutionOptions::default();
    let invocation = crate::invocation::Invocation::new(options.limits);
    let queue = {
        let mut participant = owner.participant().unwrap();
        let mut heap = participant.enter();
        let id = heap
            .allocate(Value::Object {
                ty: Type::from_name("System.Tasks.TaskQueue"),
                fields: vec![Value::Int32(0)],
            })
            .unwrap();
        let queue = Value::ObjectReference(crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        });
        let mut dispatch = invocation.dispatch.lock().unwrap();
        dispatch.bind(&mut heap).unwrap();
        dispatch.default_task_queue = Some(queue.clone());
        dispatch.publish(&mut heap).unwrap();
        queue
    };
    for _ in 0..2 {
        let mut participant = owner.participant().unwrap();
        participant
            .enter()
            .collect(vec![], CollectionReason::ExplicitRequest)
            .unwrap();
        let mut state = InstructionState::with_invocation(invocation.clone());
        let entry = module
            .functions
            .iter()
            .find(|f| f.name == "ReadQueue")
            .unwrap()
            .clone();
        let result = drive_instructions(
            &module,
            &mut vec![Frame::new(entry, vec![]).unwrap()],
            &options,
            &mut None,
            &mut participant,
            &mut Default::default(),
            &mut vec![],
            &mut [vec![], vec![]],
            &mut state,
        )
        .unwrap();
        assert_eq!(result, queue);
    }
    let Value::ObjectReference(queue) = queue else {
        panic!()
    };
    assert_eq!(queue.reference.read_field(0).unwrap(), Value::Int32(2));
    drop(invocation);
    let mut participant = owner.participant().unwrap();
    participant
        .enter()
        .collect(vec![], CollectionReason::ExplicitRequest)
        .unwrap();
    assert!(participant.enter().is_empty());
}

#[test]
fn contending_dispatch_context_observes_cancellation_without_stealing_ownership() {
    let module =
        crate::assemble(".module System\n.function Finish() -> Void\nldvoid\nret\n.end").unwrap();
    let token = crate::CancellationToken::new();
    let options = ExecutionOptions {
        cancellation: Some(token.clone()),
        ..Default::default()
    };
    let invocation = crate::invocation::Invocation::new(options.limits);
    let incumbent = std::sync::Arc::new(());
    invocation.dispatch.lock().unwrap().owner = std::sync::Arc::downgrade(&incumbent);
    let mut state = InstructionState::with_invocation(invocation.clone());
    let owner = Owner::new(1);
    let mut participant = owner.participant().unwrap();
    let mut frames = vec![Frame::new(module.functions[0].clone(), vec![]).unwrap()];
    let mut step = |state: &mut InstructionState, frames: &mut Vec<Frame>| {
        interpret_instructions(
            &module,
            frames,
            &options,
            &mut None,
            &mut participant.enter(),
            &mut Default::default(),
            &mut vec![],
            &mut [vec![], vec![]],
            state,
            1024,
        )
    };
    assert!(matches!(
        step(&mut state, &mut frames),
        Ok(InstructionProgress::Waiting)
    ));
    assert!(frames.is_empty());
    let remaining = state.budget.remaining();
    token.cancel();
    assert!(matches!(
        step(&mut state, &mut frames),
        Err(Fault {
            code: crate::FaultCode::ExecutionCancelled,
            ..
        })
    ));
    assert_eq!(state.budget.remaining(), remaining);
    assert!(std::sync::Arc::ptr_eq(
        &incumbent,
        &invocation.dispatch.lock().unwrap().owner.upgrade().unwrap()
    ));
}

#[test]
fn guest_native_buffer_survives_submitter_and_shares_quota() {
    let module = crate::assemble(
        r#"
.module System
.function Allocate() -> Int32*
ldc.i4 1
heap.alloc Int32
ret
.end
.function Update(Int32* captured) -> Int32
ldarg captured
ldc.i4 42
stind.i4
ldarg captured
ldind.i4
ret
.end
"#,
    )
    .unwrap();
    let options = ExecutionOptions {
        limits: Limits {
            pointer_bytes: 4,
            ..Limits::default()
        },
        ..Default::default()
    };
    let invocation = crate::invocation::Invocation::new(options.limits);
    let owner = Owner::new(1);
    let run = |index: usize, args: Vec<Value>| {
        let mut state = InstructionState::with_invocation(invocation.clone());
        drive_instructions(
            &module,
            &mut vec![Frame::new(module.functions[index].clone(), args).unwrap()],
            &options,
            &mut None,
            &mut owner.participant().unwrap(),
            &mut invocation.memory.clone(),
            &mut vec![],
            &mut [vec![], vec![]],
            &mut state,
        )
    };
    let captured = run(0, vec![]).unwrap();
    assert!(run(0, vec![]).is_err());
    let passed = captured.clone();
    std::thread::scope(|scope| {
        let result = scope
            .spawn(|| run(1, vec![passed]))
            .join()
            .unwrap()
            .unwrap();
        assert_eq!(result, Value::Int32(42));
    });
    let Value::Pointer(pointer) = captured else {
        panic!()
    };
    assert_eq!(
        invocation
            .memory
            .read(
                &pointer,
                &crate::memory::Layout {
                    size: 4,
                    alignment: 4,
                    fields: vec![],
                }
            )
            .unwrap(),
        Value::Int32(42)
    );
    invocation.memory.free(&pointer).unwrap();
}

#[test]
fn parked_inline_arrays_charge_other_guest_contexts_and_release_on_exit() {
    let module =
        crate::assemble(".module System\n.function Hold(Int32[] value) -> Void\nldvoid\nret\n.end")
            .unwrap();
    let options = ExecutionOptions {
        limits: Limits {
            array_elements: 3,
            ..Limits::default()
        },
        ..Default::default()
    };
    let invocation = crate::invocation::Invocation::new(options.limits);
    let owner = Owner::new(2);
    let array = || Value::Array {
        element: Type::Int32,
        elements: vec![Value::Int32(1); 2],
    };
    let mut first = owner.participant().unwrap();
    let mut state = InstructionState::with_invocation(invocation.clone());
    let mut frames = vec![Frame::new(module.functions[0].clone(), vec![array()]).unwrap()];
    assert!(matches!(
        interpret_instructions(
            &module,
            &mut frames,
            &options,
            &mut None,
            &mut first.enter(),
            &mut invocation.memory.clone(),
            &mut vec![],
            &mut [vec![], vec![]],
            &mut state,
            1
        )
        .unwrap(),
        InstructionProgress::Suspended
    ));
    let run = || {
        let mut participant = owner.participant().unwrap();
        drive_instructions(
            &module,
            &mut vec![Frame::new(module.functions[0].clone(), vec![array()]).unwrap()],
            &options,
            &mut None,
            &mut participant,
            &mut invocation.memory.clone(),
            &mut vec![],
            &mut [vec![], vec![]],
            &mut InstructionState::with_invocation(invocation.clone()),
        )
    };
    assert_eq!(
        std::thread::scope(|scope| scope.spawn(run).join().unwrap())
            .unwrap_err()
            .code,
        crate::FaultCode::ArrayLimitExceeded
    );
    drop(frames);
    drop(state);
    drop(first);
    assert_eq!(run().unwrap(), Value::Void);
}

#[test]
fn completed_inline_array_remains_charged_until_participant_release() {
    let module = crate::assemble(".module System\n.function Make() -> Int32[]\nldc.i4 2\nldc.i4 0\narray.create Int32\nret\n.end").unwrap();
    let options = ExecutionOptions {
        limits: Limits {
            array_elements: 2,
            ..Limits::default()
        },
        ..Default::default()
    };
    let invocation = crate::invocation::Invocation::new(options.limits);
    let owner = Owner::new(2);
    let run = |participant: &mut crate::shared_heap::Participant| {
        drive_instructions(
            &module,
            &mut vec![Frame::new(module.functions[0].clone(), vec![]).unwrap()],
            &options,
            &mut None,
            participant,
            &mut invocation.memory.clone(),
            &mut vec![],
            &mut [vec![], vec![]],
            &mut InstructionState::with_invocation(invocation.clone()),
        )
    };
    let mut first = owner.participant().unwrap();
    let result = run(&mut first).unwrap();
    assert!(matches!(result, Value::Array { .. }));
    let mut second = owner.participant().unwrap();
    assert_eq!(
        run(&mut second).unwrap_err().code,
        crate::FaultCode::ArrayLimitExceeded
    );
    drop(result);
    drop(first);
    assert!(matches!(run(&mut second).unwrap(), Value::Array { .. }));
}

#[test]
fn native_work_failure_is_observed_before_more_guest_instructions() {
    let module =
        crate::assemble(".module System\n.function Main() -> Int32\nldc.i4 42\nret\n.end").unwrap();
    let options = ExecutionOptions::default();
    let invocation = crate::invocation::Invocation::new(options.limits);
    let scope = crate::invocation_work::Scope(invocation.clone());
    let owner = Owner::new(2);
    let mut parent = owner.participant().unwrap();
    invocation
        .work
        .submit(&mut parent.enter(), vec![], |_, _, _| {
            Err(Fault::coded(
                crate::FaultCode::UserFault,
                "native callback failed",
            ))
        })
        .unwrap();
    assert!(invocation.wake.park(std::time::Duration::from_secs(2)));
    let mut state = InstructionState::with_invocation(invocation.clone());
    let before = state.budget.remaining();
    let error = drive_instructions(
        &module,
        &mut vec![Frame::new(module.functions[0].clone(), vec![]).unwrap()],
        &options,
        &mut None,
        &mut parent,
        &mut invocation.memory.clone(),
        &mut vec![],
        &mut [vec![], vec![]],
        &mut state,
    )
    .unwrap_err();
    assert_eq!(error.code, crate::FaultCode::UserFault);
    assert_eq!(error.message, "native callback failed");
    assert_eq!(state.budget.remaining(), before);
    drop(scope);
}
