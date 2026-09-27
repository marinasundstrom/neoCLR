use super::*;
use crate::shared_heap::Owner;

pub(super) fn library() -> Module {
    let mut source = String::from(
        ".module System\n.type class abstract System.Object\n.method instance virtual ToString() -> String\nfault \"Object formatting is outside the task fixture\"\n.end\n.end\n",
    );
    for path in [
        "runtime/System/Runtime/CompilerServices/UnionAttribute.neoil",
        "runtime/neoCLR/Runtime/EnumInspection.neoil",
        "runtime/neoCLR/Runtime/StringConcat.neoil",
        "runtime/raven/generated/Propagatable.methods.neoil",
        "runtime/raven/generated/Propagatable.helpers.neoil",
        "runtime/raven/EquatableTo.neoil",
        "runtime/raven/UnionProtocol.neoil",
        "runtime/raven/Tasks.neoil",
        "runtime/raven/ArrayList.neoil",
        "runtime/raven/CollectionContracts.neoil",
        "runtime/raven/List.neoil",
        "runtime/raven/Iterable.neoil",
        "runtime/raven/generated/Option.methods.neoil",
        "runtime/raven/generated/Option.helpers.neoil",
        "runtime/raven/generated/Result.methods.neoil",
        "runtime/raven/generated/Result.helpers.neoil",
        "runtime/raven/TaskOutcome.neoil",
        "runtime/raven/TaskState.neoil",
        "runtime/raven/Disposable.neoil",
        "runtime/raven/Iterator.neoil",
        "runtime/neoCLR/Runtime/Tasks.neoil",
        "runtime/System/Fault.neoil",
        "runtime/neoCLR/Runtime/Fault.neoil",
        "runtime/raven/generated/Func.methods.neoil",
        "runtime/raven/generated/Func.helpers.neoil",
    ] {
        source.push_str(&crate::source::read_source(path).unwrap());
        source.push('\n');
    }
    // Import only the real Concat implementation needed by generated diagnostics.
    let strings = include_str!("../runtime/raven/generated/String.bootstrap.methods.neoil");
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
.function MakeQueue() -> System.Tasks.TaskQueue
newobj instance System.Tasks.TaskQueue::.ctor()
ret
.end
.function DefaultQueue() -> System.Tasks.TaskQueue
call System.Tasks.TaskQueue::get_Default()
ret
.end
.function Post(System.Tasks.TaskQueue queue, Counter counter) -> Void
ldarg queue
ldarg counter
delegate.bind System.Func<Void> = instance Counter::Tick()
callvirt instance System.Tasks.TaskQueue::Post(System.Func<Void>)
ldvoid
ret
.end
.function Pump(System.Tasks.TaskQueue queue) -> Void
ldarg queue
callvirt instance System.Tasks.TaskQueue::Drain()
ldvoid
ret
.end
.function Register(System.Tasks.Promise<Int32> source, Counter counter) -> Void
ldarg source
callvirt instance System.Tasks.Promise<Int32>::get_Task()
ldarg counter
delegate.bind System.Func<Void> = instance Counter::Tick()
callvirt instance System.Tasks.Task<Int32>::OnCompleted(System.Func<Void>)
ldvoid
ret
.end
.function QueueOf(System.Tasks.Promise<Int32> source) -> System.Tasks.TaskQueue
ldarg source
callvirt instance System.Tasks.Promise<Int32>::Dispatcher()
ret
.end
.function Make() -> System.Tasks.Promise<Int32>
newobj instance System.Tasks.TaskQueue::.ctor()
newobj instance System.Tasks.Promise<Int32>::.ctor(System.Tasks.TaskQueue)
ret
.end
.function Complete(System.Tasks.Promise<Int32> source) -> Boolean
ldarg source
ldc.i4 42
callvirt instance System.Tasks.Promise<Int32>::Complete(Int32)
ret
.end
.function Cancel(System.Tasks.Promise<Int32> source) -> Boolean
ldarg source
callvirt instance System.Tasks.Promise<Int32>::Cancel()
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
                !task_atomic::active(&frames[index]),
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
        let module = crate::assemble(&format!(".module System\n.type class System.Tasks.TaskQueue\n.method instance Post() -> noresult\n{body}\n.end\n.end\n.function neoCLR.Runtime.ConsoleReadByte() -> Value\n.methodimpl InternalCall\n.end")).unwrap();
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
                    "System.Tasks.TaskQueue",
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
    let module = crate::assemble(".module Application\n.type class System.Tasks.TaskQueue\n.method instance Post() -> noresult\nagain:\nbr again\n.end\n.end").unwrap();
    let frame = Frame::new(
        module.functions[0].clone(),
        vec![Value::NullObjectReference(Type::from_name(
            "System.Tasks.TaskQueue",
        ))],
    )
    .unwrap();
    assert!(!task_atomic::active(&[frame]));
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
            assert!(!task_atomic::active(&frames));
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
            assert!(!task_atomic::active(&frames[index]));
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
