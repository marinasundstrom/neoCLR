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
        let mut work = Work::new(&heap, 1, crate::CancellationToken::new());
        let job = work
            .submit(&mut heap, vec![id], move |mut context, control| {
                // Frame metadata and native resources are created on the worker itself.
                let mut frames = vec![Frame::new(function, vec![capture])?];
                let options = ExecutionOptions::default();
                let mut state = InstructionState::new(options.limits);
                let mut memory = crate::memory::PointerHeap::default();
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
                                match resume_rx.recv_timeout(std::time::Duration::from_millis(10)) {
                                    Ok(()) => break,
                                    Err(mpsc::RecvTimeoutError::Timeout)
                                        if !control.is_cancelled() =>
                                    {
                                        ()
                                    }
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
            })
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
    let mut memory = crate::memory::PointerHeap::default();
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
    assert_eq!(state.remaining_instructions, 0);
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
    assert!(matches!(result, Err(_)));
    assert_eq!(state.remaining_instructions, options.limits.instructions);
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
    let mut memory = crate::memory::PointerHeap::default();
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
    assert!(pauses > 20);
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
.delegate System.Func<T>
.method instance Invoke() -> T
.end
.end
.type class System.Tasks.TaskQueue
.field Posts Int32
.field Drains Int32
.method instance Post(System.Func<Void> callback) -> noresult
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
        let callback = Value::Delegate(crate::Delegate {
            ty: crate::assembler::parse_type("System.Func<Void>").unwrap(),
            target: crate::assembler::parse_function_ref(
                "instance System.Tasks.TaskQueue::Drain()",
            )
            .unwrap(),
            receiver: Some(Box::new(queue.clone())),
        });
        let Value::Erased(listener) = state
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
            .scheduler
            .sockets
            .invoke(Operation::LocalPort, &[*listener.clone()], &heap)
            .unwrap()
        else {
            panic!()
        };
        let Value::Int32(port) = *port else { panic!() };
        let Value::Erased(operation) = state
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
    state.default_task_queue = Some(queue.clone());
    let entry = module
        .functions
        .iter()
        .find(|f| f.name == "Main")
        .unwrap()
        .clone();
    let mut frames = vec![Frame::new(entry, vec![result.clone()]).unwrap()];
    let mut memory = crate::memory::PointerHeap::default();
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
    let fuel = state.remaining_instructions;
    for _ in 0..3 {
        state.scheduler.park();
        assert!(matches!(
            run(&mut state, &mut frames),
            Ok(InstructionProgress::Waiting)
        ));
        assert_eq!(state.remaining_instructions, fuel);
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
        state.scheduler.park();
        assert!(matches!(
            run(&mut state, &mut frames),
            Err(Fault {
                code: crate::FaultCode::ExecutionCancelled,
                ..
            })
        ));
        assert_eq!(state.remaining_instructions, fuel);
        return;
    }
    let _peer = std::net::TcpStream::connect(("127.0.0.1", port as u16)).unwrap();
    let value = loop {
        match run(&mut state, &mut frames).unwrap() {
            InstructionProgress::Waiting => state.scheduler.park(),
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
