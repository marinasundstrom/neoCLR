use super::*;
use crate::shared_heap::Owner;

pub(super) fn library() -> Module {
    let mut source = String::from(
        ".module System\n.type class abstract System.Object\n.method instance virtual ToString() -> String\nfault \"Object formatting is outside the task fixture\"\n.end\n.end\n",
    );
    for path in [
        "runtime/legacy/bootstrap/System/Runtime/CompilerServices/UnionAttribute.neoil",
        "runtime/legacy/bootstrap/neoCLR/Runtime/EnumInspection.neoil",
        "runtime/legacy/bootstrap/neoCLR/Runtime/StringConcat.neoil",
        "runtime/legacy/raven/generated/Propagatable.methods.neoil",
        "runtime/legacy/raven/generated/Propagatable.helpers.neoil",
        "runtime/legacy/raven/EquatableTo.neoil",
        "runtime/legacy/raven/UnionProtocol.neoil",
        "runtime/legacy/raven/Tasks.neoil",
        "runtime/legacy/raven/ArrayList.neoil",
        "runtime/legacy/raven/CollectionContracts.neoil",
        "runtime/legacy/raven/List.neoil",
        "runtime/legacy/raven/Iterable.neoil",
        "runtime/legacy/raven/generated/Optional.methods.neoil",
        "runtime/legacy/raven/generated/Optional.helpers.neoil",
        "runtime/legacy/raven/generated/Result.methods.neoil",
        "runtime/legacy/raven/generated/Result.helpers.neoil",
        "runtime/legacy/raven/TaskOutcome.neoil",
        "runtime/legacy/raven/TaskState.neoil",
        "runtime/legacy/raven/Disposable.neoil",
        "runtime/legacy/raven/Iterator.neoil",
        "runtime/legacy/bootstrap/neoCLR/Runtime/Tasks.neoil",
        "runtime/legacy/bootstrap/System/Fault.neoil",
        "runtime/legacy/bootstrap/neoCLR/Runtime/Fault.neoil",
    ] {
        source.push_str(&crate::source::read_source(path).unwrap());
        source.push('\n');
    }
    // Import only the real Concat implementation needed by generated diagnostics.
    let strings = include_str!("../runtime/legacy/raven/generated/String.bootstrap.methods.neoil");
    let concat = strings
        .split(".method static Concat(")
        .nth(1)
        .unwrap()
        .split(".end")
        .next()
        .unwrap();
    source.push_str(&format!(
        ".type System.String\n.method static Concat({concat}.end\n.end\n"
    ));
    source.push_str(
        r#"
.type class Counter
.field Count Int32
.method instance Tick() -> Void
ldarg 0
ldarg 0
ldfld Counter::Count
ldc.i4 1
add
stfld Counter::Count
ldvoid
ret
.end
.end
.function MakeQueue() -> System.Concurrency.Tasks.TaskQueue
newobj instance System.Concurrency.Tasks.TaskQueue::.ctor()
ret
.end
.function DefaultQueue() -> System.Concurrency.Tasks.TaskQueue
call System.Concurrency.Tasks.TaskQueue::get_Default()
ret
.end
.function Post(System.Concurrency.Tasks.TaskQueue queue, Counter counter) -> Void
ldarg queue
ldarg counter
function.bind fn<Void> = instance Counter::Tick()
callvirt instance System.Concurrency.Tasks.TaskQueue::Post(fn<Void>)
ldvoid
ret
.end
.function Pump(System.Concurrency.Tasks.TaskQueue queue) -> Void
ldarg queue
callvirt instance System.Concurrency.Tasks.TaskQueue::Drain()
ldvoid
ret
.end
.function Register(System.Concurrency.Tasks.Promise<Int32> source, Counter counter) -> Void
ldarg source
callvirt instance System.Concurrency.Tasks.Promise<Int32>::get_Task()
ldarg counter
function.bind fn<Void> = instance Counter::Tick()
callvirt instance System.Concurrency.Tasks.Task<Int32>::OnCompleted(fn<Void>)
ldvoid
ret
.end
.function QueueOf(System.Concurrency.Tasks.Promise<Int32> source) -> System.Concurrency.Tasks.TaskQueue
ldarg source
callvirt instance System.Concurrency.Tasks.Promise<Int32>::Dispatcher()
ret
.end
.function Make() -> System.Concurrency.Tasks.Promise<Int32>
newobj instance System.Concurrency.Tasks.TaskQueue::.ctor()
newobj instance System.Concurrency.Tasks.Promise<Int32>::.ctor(System.Concurrency.Tasks.TaskQueue)
ret
.end
.function Complete(System.Concurrency.Tasks.Promise<Int32> source) -> Boolean
ldarg source
ldc.i4 42
callvirt instance System.Concurrency.Tasks.Promise<Int32>::Complete(Int32)
ret
.end
.function Cancel(System.Concurrency.Tasks.Promise<Int32> source) -> Boolean
ldarg source
callvirt instance System.Concurrency.Tasks.Promise<Int32>::Cancel()
ret
.end
"#,
    );
    crate::assemble(&source).unwrap()
}

#[test]
fn generated_promise_terminal_transition_is_not_split_by_one_instruction_quanta() {
    let module = library();
    let options = ExecutionOptions::default();
    let invocation = crate::invocation::Invocation::new(options.limits);
    let owner = Owner::new(2);
    let mut first = owner.participant().unwrap();
    let mut second = owner.participant().unwrap();
    let function = |name: &str| {
        module
            .functions
            .iter()
            .find(|f| f.name == name)
            .unwrap()
            .clone()
    };
    let capture = drive_instructions(
        &module,
        &mut vec![Frame::new(function("Make"), vec![]).unwrap()],
        &options,
        &mut None,
        &mut first,
        &mut invocation.memory.clone(),
        &mut vec![],
        &mut [vec![], vec![]],
        &mut InstructionState::with_invocation(invocation.clone()),
    )
    .unwrap();
    let mut frames = [
        vec![Frame::new(function("Complete"), vec![capture.clone()]).unwrap()],
        vec![Frame::new(function("Cancel"), vec![capture]).unwrap()],
    ];
    let mut states = [
        InstructionState::with_invocation(invocation.clone()),
        InstructionState::with_invocation(invocation.clone()),
    ];
    let mut results = [None, None];
    for _ in 0..100 {
        for (index, participant) in [&mut first, &mut second].into_iter().enumerate() {
            if results[index].is_some() {
                continue;
            }
            let progress = interpret_instructions(
                &module,
                &mut frames[index],
                &options,
                &mut None,
                &mut participant.enter(),
                &mut invocation.memory.clone(),
                &mut vec![],
                &mut [vec![], vec![]],
                &mut states[index],
                1,
            )
            .unwrap();
            assert!(
                !task_atomic::active(&module, &frames[index]),
                "must not release graph access mid-transition"
            );
            if let InstructionProgress::Completed(value) = progress {
                results[index] = Some(value);
            }
        }
        if results.iter().all(Option::is_some) {
            break;
        }
    }
    assert_eq!(
        results
            .iter()
            .filter(|r| **r == Some(Value::Boolean(true)))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|r| **r == Some(Value::Boolean(false)))
            .count(),
        1
    );
}

#[test]
fn guarded_mutations_keep_instruction_limits_and_reject_host_suspension() {
    for (body, cancelled) in [
        ("again:\nbr again", false),
        ("again:\nbr again", true),
        ("call neoCLR.Runtime.ConsoleReadByte()\npop\nret", false),
    ] {
        let module = crate::assemble(&format!(".module System\n.type class System.Concurrency.Tasks.TaskQueue\n.method instance Post() -> void\n{body}\n.end\n.end\n.function neoCLR.Runtime.ConsoleReadByte() -> Value\n.methodimpl InternalCall\n.end")).unwrap();
        let cancellation = crate::CancellationToken::new();
        if cancelled {
            cancellation.cancel();
        }
        let options = ExecutionOptions {
            cancellation: Some(cancellation),
            limits: Limits {
                instructions: 8,
                ..Limits::default()
            },
            ..Default::default()
        };
        let mut state = InstructionState::new(options.limits);
        let owner = Owner::new(1);
        let mut participant = owner.participant().unwrap();
        let function = module
            .functions
            .iter()
            .find(|f| f.name.ends_with(".Post"))
            .unwrap()
            .clone();
        let mut frames = vec![
            Frame::new(
                function,
                vec![Value::NullObjectReference(Type::from_name(
                    "System.Concurrency.Tasks.TaskQueue",
                ))],
            )
            .unwrap(),
        ];
        let fault = interpret_instructions(
            &module,
            &mut frames,
            &options,
            &mut None,
            &mut participant.enter(),
            &mut state.invocation.memory.clone(),
            &mut vec![],
            &mut [vec![], vec![]],
            &mut state,
            1,
        )
        .err()
        .unwrap();
        if cancelled {
            assert_eq!(fault.code, crate::FaultCode::ExecutionCancelled);
        } else if body.starts_with("again") {
            assert_eq!(fault.code, crate::FaultCode::InstructionLimitExceeded);
        } else {
            assert!(fault.message.contains("cannot suspend for host I/O"));
        }
    }
}

#[test]
fn application_names_do_not_receive_system_atomic_policy() {
    let module = crate::assemble(".module Application\n.type class System.Concurrency.Tasks.TaskQueue\n.method instance Post() -> void\nagain:\nbr again\n.end\n.end").unwrap();
    let frame = Frame::new(
        module.functions[0].clone(),
        vec![Value::NullObjectReference(Type::from_name(
            "System.Concurrency.Tasks.TaskQueue",
        ))],
    )
    .unwrap();
    assert!(!task_atomic::active(&module, &[frame]));
}

#[test]
fn generated_queue_callbacks_yield_and_posts_during_drain_are_not_lost() {
    let module = library();
    let options = ExecutionOptions::default();
    let invocation = crate::invocation::Invocation::new(options.limits);
    let owner = Owner::new(2);
    let mut caller = owner.participant().unwrap();
    let mut pump = owner.participant().unwrap();
    let function = |name: &str| {
        module
            .functions
            .iter()
            .find(|f| f.name == name)
            .unwrap()
            .clone()
    };
    let run = |name: &str, args, participant: &mut crate::shared_heap::Participant| {
        drive_instructions(
            &module,
            &mut vec![Frame::new(function(name), args).unwrap()],
            &options,
            &mut None,
            participant,
            &mut invocation.memory.clone(),
            &mut vec![],
            &mut [vec![], vec![]],
            &mut InstructionState::with_invocation(invocation.clone()),
        )
        .unwrap()
    };
    let queue = run("MakeQueue", vec![], &mut caller);
    let counter = {
        let mut access = caller.enter();
        let id = access
            .allocate(Value::Object {
                ty: Type::from_name("Counter"),
                fields: vec![Value::Int32(0)],
            })
            .unwrap();
        Value::ObjectReference(crate::value::ObjectReference {
            reference: access.address(id).unwrap(),
            view: None,
        })
    };
    run("Post", vec![queue.clone(), counter.clone()], &mut caller);
    let mut frames = vec![Frame::new(function("Pump"), vec![queue.clone()]).unwrap()];
    let mut state = InstructionState::with_invocation(invocation.clone());
    let mut callback_yielded = false;
    for _ in 0..1000 {
        let progress = interpret_instructions(
            &module,
            &mut frames,
            &options,
            &mut None,
            &mut pump.enter(),
            &mut invocation.memory.clone(),
            &mut vec![],
            &mut [vec![], vec![]],
            &mut state,
            1,
        )
        .unwrap();
        if frames.iter().any(|frame| frame.queue_callback) && !callback_yielded {
            assert!(!task_atomic::active(&module, &frames));
            // Other guest work can submit while a callback is suspended.
            run("Post", vec![queue.clone(), counter.clone()], &mut caller);
            callback_yielded = true;
        }
        if matches!(progress, InstructionProgress::Completed(_)) {
            break;
        }
    }
    assert!(callback_yielded);
    assert!(frames.is_empty());
    let Value::ObjectReference(counter) = counter else {
        panic!()
    };
    assert_eq!(counter.reference.read_field(0).unwrap(), Value::Int32(2));
}

fn interleave(
    module: &Module,
    invocation: &std::sync::Arc<crate::invocation::Invocation>,
    participants: [&mut crate::shared_heap::Participant; 2],
    mut frames: [Vec<Frame>; 2],
) -> [Value; 2] {
    let options = ExecutionOptions::default();
    let mut states = [
        InstructionState::with_invocation(invocation.clone()),
        InstructionState::with_invocation(invocation.clone()),
    ];
    let mut results = [None, None];
    for _ in 0..1000 {
        for index in 0..2 {
            if results[index].is_some() {
                continue;
            }
            let progress = interpret_instructions(
                module,
                &mut frames[index],
                &options,
                &mut None,
                &mut participants[index].enter(),
                &mut invocation.memory.clone(),
                &mut vec![],
                &mut [vec![], vec![]],
                &mut states[index],
                1,
            )
            .unwrap();
            assert!(!task_atomic::active(module, &frames[index]));
            if let InstructionProgress::Completed(value) = progress {
                results[index] = Some(value);
            }
        }
        if results.iter().all(Option::is_some) {
            break;
        }
    }
    results.map(Option::unwrap)
}

#[test]
fn generated_default_queue_creation_has_one_identity() {
    let module = library();
    let invocation = crate::invocation::Invocation::new(Limits::default());
    let owner = Owner::new(2);
    let mut first = owner.participant().unwrap();
    let mut second = owner.participant().unwrap();
    let function = module
        .functions
        .iter()
        .find(|f| f.name == "DefaultQueue")
        .unwrap();
    let results = interleave(
        &module,
        &invocation,
        [&mut first, &mut second],
        [
            vec![Frame::new(function.clone(), vec![]).unwrap()],
            vec![Frame::new(function.clone(), vec![]).unwrap()],
        ],
    );
    assert_eq!(results[0], results[1]);
    assert!(matches!(results[0], Value::ObjectReference(_)));
}

#[test]
fn generated_registration_and_completion_post_callback_exactly_once() {
    let module = library();
    for register_first in [false, true] {
        let options = ExecutionOptions::default();
        let invocation = crate::invocation::Invocation::new(options.limits);
        let owner = Owner::new(2);
        let mut first = owner.participant().unwrap();
        let mut second = owner.participant().unwrap();
        let function = |name: &str| {
            module
                .functions
                .iter()
                .find(|f| f.name == name)
                .unwrap()
                .clone()
        };
        let run = |name: &str, args, participant: &mut crate::shared_heap::Participant| {
            drive_instructions(
                &module,
                &mut vec![Frame::new(function(name), args).unwrap()],
                &options,
                &mut None,
                participant,
                &mut invocation.memory.clone(),
                &mut vec![],
                &mut [vec![], vec![]],
                &mut InstructionState::with_invocation(invocation.clone()),
            )
            .unwrap()
        };
        let source = run("Make", vec![], &mut first);
        let queue = run("QueueOf", vec![source.clone()], &mut first);
        let counter = {
            let mut access = first.enter();
            let id = access
                .allocate(Value::Object {
                    ty: Type::from_name("Counter"),
                    fields: vec![Value::Int32(0)],
                })
                .unwrap();
            Value::ObjectReference(crate::value::ObjectReference {
                reference: access.address(id).unwrap(),
                view: None,
            })
        };
        let mut frames = [
            vec![Frame::new(function("Complete"), vec![source.clone()]).unwrap()],
            vec![Frame::new(function("Register"), vec![source, counter.clone()]).unwrap()],
        ];
        if register_first {
            frames.swap(0, 1);
        }
        interleave(&module, &invocation, [&mut first, &mut second], frames);
        run("Pump", vec![queue], &mut first);
        let Value::ObjectReference(counter) = counter else {
            panic!()
        };
        assert_eq!(counter.reference.read_field(0).unwrap(), Value::Int32(1));
    }
}
