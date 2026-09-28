//! Internal guest adapter. Public Task.Run overloads remain separate from
//! invoking a callback on its own VM context and draining native work.
use super::*;
use std::sync::Arc;

pub(super) fn submit(
    module: Arc<Module>,
    invocation: Arc<crate::invocation::Invocation>,
    heap: &mut crate::shared_heap::Access<'_>,
    callback: Value,
    mut options: ExecutionOptions,
) -> Result<usize, Fault> {
    let Value::Function(binding) = &callback else {
        return Err(Fault::new("task submission requires a delegate"));
    };
    let contract = crate::function_objects::contract(&module, &binding.ty)?;
    if !contract.parameters.is_empty() {
        return Err(Fault::new("task callback must have no parameters"));
    }
    let function = resolve(&module, &binding.target)?;
    crate::function_objects::compatible(&contract, &function)?;
    options.limits = invocation.limits;
    let worker_invocation = invocation.clone();
    invocation
        .work
        .submit(heap, vec![callback], move |context, mut captures, _| {
            let Value::Function(binding) = captures.pop().expect("registered callback") else {
                unreachable!()
            };
            let args = binding
                .receiver
                .into_iter()
                .map(|receiver| *receiver)
                .collect();
            let mut frames = vec![Frame::new(function, args)?];
            let mut state = InstructionState::with_invocation(worker_invocation.clone());
            let mut memory = worker_invocation.memory.clone();
            context.execute(|participant, control| {
                state.work_control = Some(control.clone());
                options.cancellation = Some(control.token());
                drive_instructions(
                    &module,
                    &mut frames,
                    &options,
                    &mut None,
                    participant,
                    &mut memory,
                    &mut Vec::new(),
                    &mut [Vec::new(), Vec::new()],
                    &mut state,
                )
            })
        })
}

pub(super) fn transfer_task_output(
    state: &mut InstructionState,
    lines: &mut Vec<String>,
    bytes: &mut [Vec<u8>; 2],
) -> Result<(), Fault> {
    if state.work_control.is_none() {
        state.invocation.work.drain_output(lines, bytes);
        return Ok(());
    }
    let count = lines
        .iter()
        .fold(0usize, |sum, line| {
            sum.saturating_add(line.len()).saturating_add(1)
        })
        .saturating_add(bytes[0].len())
        .saturating_add(bytes[1].len());
    let total = state.work_output_bytes.saturating_add(count);
    if total > state.invocation.limits.worker_result_bytes {
        lines.clear();
        bytes.iter_mut().for_each(Vec::clear);
        return Err(Fault::new("task captured output byte limit exceeded"));
    }
    state.work_output_bytes = total;
    state.invocation.work.record_output(lines, bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_heap::Owner;
    use std::time::{Duration, Instant};

    fn fixture(body: &str, result: &str) -> Arc<Module> {
        Arc::new(
            crate::assemble(&format!(
                r#"
.module System
.delegate Callback
.method instance Invoke() -> {result}
.end
.end
.type class Capture
.field Value Int32
.method instance Run() -> {result}
{body}
.end
.end
.function Bind(Capture capture) -> Callback
ldarg capture
delegate.bind Callback = instance Capture::Run()
ret
.end
.function neoCLR.Runtime.WriteLine(String value) -> Void
.methodimpl InternalCall
.end
"#
            ))
            .unwrap(),
        )
    }

    fn bind(
        module: &Module,
        invocation: &Arc<crate::invocation::Invocation>,
        parent: &mut crate::shared_heap::Participant,
    ) -> (Value, Value) {
        let captured = {
            let mut heap = parent.enter();
            let id = heap
                .allocate(Value::Object {
                    ty: Type::from_name("Capture"),
                    fields: vec![Value::Int32(40)],
                })
                .unwrap();
            Value::ObjectReference(crate::value::ObjectReference {
                reference: heap.address(id).unwrap(),
                view: None,
            })
        };
        let function = module
            .functions
            .iter()
            .find(|f| f.name == "Bind")
            .unwrap()
            .clone();
        let callback = drive_instructions(
            module,
            &mut vec![Frame::new(function, vec![captured.clone()]).unwrap()],
            &ExecutionOptions::default(),
            &mut None,
            parent,
            &mut invocation.memory.clone(),
            &mut vec![],
            &mut [vec![], vec![]],
            &mut InstructionState::with_invocation(invocation.clone()),
        )
        .unwrap();
        (callback, captured)
    }

    fn wait(
        invocation: &crate::invocation::Invocation,
        id: usize,
    ) -> Result<crate::task_work::Completion, Fault> {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(completion) = invocation.work.take_ready(id)? {
                return Ok(completion);
            }
            assert!(Instant::now() < deadline, "guest callback did not finish");
            invocation.park();
        }
    }

    #[test]
    fn native_submission_completes_generated_promise_and_drains_its_continuation() {
        let system = super::super::task_atomic_tests::library();
        let source = r#"
.module App
.entry Main
.type class Job
.field Source System.Tasks.Promise<Int32>
.field Observed Int32
.method instance .ctor() -> noresult
ldarg 0
newobj instance System.Tasks.Promise<Int32>::.ctor()
stfld Job::Source
ret
.end
.method instance Run() -> Void
ldarg 0
ldfld Job::Source
ldc.i4 42
callvirt instance System.Tasks.Promise<Int32>::Complete(Int32)
pop
ldvoid
ret
.end
.method instance Observe() -> Void
ldarg 0
ldarg 0
ldfld Job::Source
callvirt instance System.Tasks.Promise<Int32>::get_Task()
callvirt instance System.Tasks.Task<Int32>::GetResult()
stfld Job::Observed
ldvoid
ret
.end
.end
.function Main() -> Job
.local Job job
newobj instance Job::.ctor()
stloc job
ldloc job
ldfld Job::Source
callvirt instance System.Tasks.Promise<Int32>::get_Task()
ldloc job
delegate.bind fn<Void> = instance Job::Observe()
callvirt instance System.Tasks.Task<Int32>::OnCompleted(fn<Void>)
ldloc job
delegate.bind fn<Void> = instance Job::Run()
call neoCLR.Runtime.ScheduleTask(fn<Void>)
pop
ldloc job
ret
.end
"#;
        for (body, fails) in [(source.to_owned(), false),
            (source.replace("ldc.i4 42\ncallvirt instance System.Tasks.Promise<Int32>::Complete(Int32)",
                "fault \"submitted guest failure\"\nldc.i4 42\ncallvirt instance System.Tasks.Promise<Int32>::Complete(Int32)"), true)] {
            let app = crate::assembler::read_modules(
                &[crate::assembler::ModuleInput::Source(&body)], &system,
            ).unwrap().remove(0);
            let program = crate::LoadedProgram::with_library(&app, &system).unwrap();
            program.verify().unwrap();
            let execution = program.run(Limits::default());
            if fails {
                assert!(execution.err().unwrap().message.contains("submitted guest failure"));
            } else {
                let execution = execution.unwrap();
                let Value::ObjectReference(result) = execution.value else { panic!() };
                assert_eq!(result.reference.read_field(1).unwrap(), Value::Int32(42));
            }
        }
    }

    #[test]
    fn entry_waits_without_spending_fuel_and_redrains_after_native_completion() {
        let module = crate::assemble(
            r#"
.module System
.type class System.Tasks.TaskQueue
.field Drains Int32
.field Pending Int32
.field Observed Int32
.method instance Drain() -> noresult
ldarg 0
ldarg 0
ldfld System.Tasks.TaskQueue::Drains
ldc.i4 1
add
stfld System.Tasks.TaskQueue::Drains
ldarg 0
ldarg 0
ldfld System.Tasks.TaskQueue::Pending
stfld System.Tasks.TaskQueue::Observed
ret
.end
.end
.function Main() -> Int32
ldc.i4 42
ret
.end
"#,
        )
        .unwrap();
        let invocation = crate::invocation::Invocation::new(Limits::default());
        let _scope = crate::invocation_work::Scope(invocation.clone());
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let (release, resume) = std::sync::mpsc::channel();
        let (queue, id) = {
            let mut heap = parent.enter();
            let id = heap
                .allocate(Value::Object {
                    ty: Type::from_name("System.Tasks.TaskQueue"),
                    fields: vec![Value::Int32(0); 3],
                })
                .unwrap();
            let queue = Value::ObjectReference(crate::value::ObjectReference {
                reference: heap.address(id).unwrap(),
                view: None,
            });
            let job = invocation
                .work
                .submit(
                    &mut heap,
                    vec![queue.clone()],
                    move |context, captures, _| {
                        resume.recv_timeout(Duration::from_secs(2)).unwrap();
                        context.complete(|_| {
                            let Value::ObjectReference(queue) = &captures[0] else {
                                panic!()
                            };
                            queue
                                .reference
                                .field(1, Type::Int32)?
                                .write(Value::Int32(7))?;
                            Ok(Value::Void)
                        })
                    },
                )
                .unwrap();
            (queue, job)
        };
        invocation.dispatch.lock().unwrap().default_task_queue = Some(queue.clone());
        let mut state = InstructionState::with_invocation(invocation.clone());
        let function = module
            .functions
            .iter()
            .find(|f| f.name == "Main")
            .unwrap()
            .clone();
        let mut frames = vec![Frame::new(function, vec![]).unwrap()];
        let mut run = |state: &mut InstructionState| {
            interpret_instructions(
                &module,
                &mut frames,
                &ExecutionOptions::default(),
                &mut None,
                &mut parent.enter(),
                &mut invocation.memory.clone(),
                &mut vec![],
                &mut [vec![], vec![]],
                state,
                1024,
            )
            .unwrap()
        };
        assert!(matches!(run(&mut state), InstructionProgress::Waiting));
        let fuel = invocation.budget.remaining();
        for _ in 0..3 {
            assert!(matches!(run(&mut state), InstructionProgress::Waiting));
            assert_eq!(invocation.budget.remaining(), fuel);
        }
        let Value::ObjectReference(queue) = queue else {
            panic!()
        };
        assert_eq!(queue.reference.read_field(0).unwrap(), Value::Int32(1));
        release.send(()).unwrap();
        let completion = wait(&invocation, id).unwrap();
        assert!(matches!(
            run(&mut state),
            InstructionProgress::Completed(Value::Int32(42))
        ));
        assert_eq!(queue.reference.read_field(0).unwrap(), Value::Int32(2));
        assert_eq!(queue.reference.read_field(2).unwrap(), Value::Int32(7));
        completion.receive(&mut parent.enter(), vec![]).unwrap();
    }

    #[test]
    fn guest_delegate_shares_capture_budget_and_does_not_wait_for_dispatch_owner() {
        let module = fixture(
            "ldarg 0\nldarg 0\nldfld Capture::Value\nldc.i4 2\nadd\nstfld Capture::Value\nldarg 0\nret",
            "Capture",
        );
        let invocation = crate::invocation::Invocation::new(Limits::default());
        let _scope = crate::invocation_work::Scope(invocation.clone());
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let (callback, captured) = bind(&module, &invocation, &mut parent);
        let before = invocation.budget.remaining();
        let incumbent = Arc::new(());
        invocation.dispatch.lock().unwrap().owner = Arc::downgrade(&incumbent);
        let id = submit(
            module,
            invocation.clone(),
            &mut parent.enter(),
            callback,
            ExecutionOptions::default(),
        )
        .unwrap();
        let completion = wait(&invocation, id).unwrap();
        {
            let mut heap = parent.enter();
            heap.collect(vec![], crate::CollectionReason::ExplicitRequest)
                .unwrap();
            let value = completion.receive(&mut heap, vec![]).unwrap();
            assert_eq!(value, captured);
        }
        let Value::ObjectReference(captured) = captured else {
            panic!()
        };
        assert_eq!(captured.reference.read_field(0).unwrap(), Value::Int32(42));
        assert!(invocation.budget.remaining() < before);
        assert!(Arc::ptr_eq(
            &incumbent,
            &invocation.dispatch.lock().unwrap().owner.upgrade().unwrap()
        ));
    }

    #[test]
    fn guest_output_is_forwarded_once_and_bounded() {
        for limit in [2, 64] {
            let module = fixture(
                "ldstr \"hello\"\ncall neoCLR.Runtime.WriteLine(String)\npop\nldvoid\nret",
                "Void",
            );
            let limits = Limits {
                worker_result_bytes: limit,
                ..Limits::default()
            };
            let invocation = crate::invocation::Invocation::new(limits);
            let _scope = crate::invocation_work::Scope(invocation.clone());
            let owner = Owner::new(2);
            let mut parent = owner.participant().unwrap();
            let (callback, _) = bind(&module, &invocation, &mut parent);
            let id = submit(
                module,
                invocation.clone(),
                &mut parent.enter(),
                callback,
                ExecutionOptions::default(),
            )
            .unwrap();
            let result = wait(&invocation, id);
            let mut lines = vec![];
            let mut bytes = [vec![], vec![]];
            invocation.work.drain_output(&mut lines, &mut bytes);
            if limit == 2 {
                assert!(result.err().unwrap().message.contains("output byte limit"));
                assert!(lines.is_empty());
            } else {
                assert_eq!(
                    result
                        .unwrap()
                        .receive(&mut parent.enter(), vec![])
                        .unwrap(),
                    Value::Void
                );
                assert_eq!(lines, ["hello"]);
                invocation.work.drain_output(&mut lines, &mut bytes);
                assert_eq!(lines, ["hello"]);
            }
        }
    }

    #[test]
    fn stopping_invocation_interrupts_running_guest_loop() {
        let module = fixture("again:\nbr again", "Void");
        let limits = Limits {
            instructions: usize::MAX,
            ..Limits::default()
        };
        let invocation = crate::invocation::Invocation::new(limits);
        let scope = crate::invocation_work::Scope(invocation.clone());
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let (callback, _) = bind(&module, &invocation, &mut parent);
        let before = invocation.budget.remaining();
        submit(
            module,
            invocation.clone(),
            &mut parent.enter(),
            callback,
            ExecutionOptions::default(),
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while invocation.budget.remaining() == before {
            assert!(Instant::now() < deadline, "guest loop never started");
            std::thread::yield_now();
        }
        drop(scope);
        assert_eq!(Arc::strong_count(&invocation), 1);
        assert!(owner.participant().is_ok());
    }
    #[test]
    fn blocking_guest_delegate_permits_caller_collection_and_capture_mutation() {
        #[derive(Debug)]
        struct BlockingConsole {
            started: std::sync::mpsc::Sender<()>,
            release: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
        }
        impl crate::Console for BlockingConsole {
            fn read_byte(&self) -> std::io::Result<Option<u8>> {
                Ok(None)
            }
            fn write_line(&self, _: &str) -> std::io::Result<()> {
                self.started.send(()).unwrap();
                self.release
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(2))
                    .map_err(std::io::Error::other)
            }
        }
        let (started, ready) = std::sync::mpsc::channel();
        let (release, resume) = std::sync::mpsc::channel();
        let module = fixture(
            "ldstr \"blocked\"\ncall neoCLR.Runtime.WriteLine(String)\npop\nldarg 0\nldfld Capture::Value\nret",
            "Int32",
        );
        let invocation = crate::invocation::Invocation::new(Limits::default());
        let _scope = crate::invocation_work::Scope(invocation.clone());
        let owner = Owner::new(2);
        let mut parent = owner.participant().unwrap();
        let (callback, capture) = bind(&module, &invocation, &mut parent);
        let options = ExecutionOptions {
            console: Some(Arc::new(BlockingConsole {
                started,
                release: std::sync::Mutex::new(resume),
            })),
            ..Default::default()
        };
        let id = submit(
            module,
            invocation.clone(),
            &mut parent.enter(),
            callback,
            options,
        )
        .unwrap();
        ready.recv_timeout(Duration::from_secs(2)).unwrap();
        {
            let deadline = Instant::now() + Duration::from_secs(1);
            let mut heap = parent
                .enter_cancellable(|| Instant::now() >= deadline)
                .unwrap();
            heap.collect(vec![], crate::CollectionReason::ExplicitRequest)
                .unwrap();
            let Value::ObjectReference(capture) = capture else {
                panic!()
            };
            capture
                .reference
                .field(0, Type::Int32)
                .unwrap()
                .write(Value::Int32(42))
                .unwrap();
        }
        release.send(()).unwrap();
        assert_eq!(
            wait(&invocation, id)
                .unwrap()
                .receive(&mut parent.enter(), vec![])
                .unwrap(),
            Value::Int32(42)
        );
        let mut output = vec![];
        invocation
            .work
            .drain_output(&mut output, &mut [vec![], vec![]]);
        // Live console output follows the existing embedding contract: do not
        // replay it into captured output when the worker hands control back.
        assert!(output.is_empty());
        assert!(ready.try_recv().is_err());
    }
}
