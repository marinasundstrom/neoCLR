//! Prepared blocking calls. Run without graph access; resume under graph access.
use super::*;

pub(super) struct HostCall {
    indirect: Option<std::sync::Arc<crate::interop::NativeCall>>,
    callee: crate::metadata::Function,
    args: Vec<Value>,
    assembly: Option<String>,
    context: String,
    pc: usize,
    patch: Option<(Value, usize)>,
    immediate: Option<Value>,
    join: Option<crate::workers::PendingJoin>,
}

impl HostCall {
    pub(super) fn indirect(
        call: std::sync::Arc<crate::interop::NativeCall>,
        args: Vec<Value>,
        assembly: Option<String>,
        context: String,
        pc: usize,
    ) -> Self {
        Self {
            callee: call.function.clone(),
            indirect: Some(call),
            args,
            assembly,
            context,
            pc,
            patch: None,
            immediate: None,
            join: None,
        }
    }

    pub(super) fn new(
        callee: crate::metadata::Function,
        mut args: Vec<Value>,
        assembly: Option<String>,
        context: String,
        pc: usize,
    ) -> Result<Self, Fault> {
        let mut patch = None;
        let mut immediate = None;
        if callee.pinvoke.is_none() {
            use crate::{file_streams::Operation, native::Binding};
            match crate::native::bind(&callee)? {
                Binding::ConsoleWriteBytes | Binding::FileResource(Operation::Write)
                    if args.len() == 2 =>
                {
                    let view =
                        if matches!(crate::native::bind(&callee)?, Binding::ConsoleWriteBytes) {
                            crate::file_streams::console_bytes(&args[1])?
                        } else {
                            crate::file_streams::memory(&args[1], true)?
                        };
                    if view.length() > crate::file_streams::MAX_TRANSFER {
                        immediate = Some(Value::Erased(Box::new(Value::Byte(8))));
                    } else {
                        let count = view.length() as i32;
                        let snapshot = crate::file_streams::snapshot(view)?;
                        args = vec![
                            args[0].clone(),
                            snapshot,
                            Value::Int32(0),
                            Value::Int32(count),
                        ];
                    }
                }
                Binding::FileResource(Operation::ReadInto) if args.len() == 2 => {
                    let view = crate::file_streams::memory(&args[1], false)?;
                    if view.length() > crate::file_streams::MAX_TRANSFER {
                        immediate = Some(Value::Erased(Box::new(Value::Byte(8))));
                    } else {
                        let count = view.length() as i32;
                        patch = Some((args[1].clone(), 0));
                        args = vec![args[0].clone(), Value::Int32(count)];
                    }
                }
                Binding::ConsoleWriteBytes | Binding::FileResource(Operation::Write) => {
                    if let Some(Value::ObjectReference(array)) = args.get(1) {
                        args[1] = array.reference.read()?;
                    }
                }
                Binding::FileResource(Operation::ReadInto) => {
                    let [
                        Value::Int32(id),
                        Value::ObjectReference(array),
                        Value::Int32(offset),
                        Value::Int32(count),
                    ] = args.as_slice()
                    else {
                        return Err(Fault::new("Invalid file read arguments"));
                    };
                    if array.reference.target() != &Type::Array(Box::new(Type::Byte)) {
                        return Err(Fault::new("File read requires a byte array"));
                    }
                    array.reference.require_writable()?;
                    let length = array.reference.array_length()?;
                    match (usize::try_from(*offset), usize::try_from(*count)) {
                        (Ok(offset), Ok(count)) if offset <= length && count <= length - offset => {
                            patch = Some((Value::ObjectReference(array.clone()), offset));
                            args = vec![Value::Int32(*id), Value::Int32(count as i32)];
                        }
                        _ => immediate = Some(Value::Erased(Box::new(Value::Byte(7)))),
                    }
                }
                _ => {}
            }
        }
        Ok(Self {
            indirect: None,
            callee,
            args,
            assembly,
            context,
            pc,
            patch,
            immediate,
            join: None,
        })
    }

    pub(super) fn prepare_join(
        &mut self,
        scheduler: &mut crate::scheduler::Scheduler,
    ) -> Result<(), Fault> {
        if self.indirect.is_none()
            && self.callee.pinvoke.is_none()
            && matches!(
                crate::native::bind(&self.callee)?,
                crate::native::Binding::JoinWorker | crate::native::Binding::JoinWorkerResult
            )
        {
            self.join = Some(scheduler.workers.prepare_join(self.args.clone()).map_err(
                |mut fault| {
                    fault.function = Some(self.context.clone());
                    fault.instruction = Some(self.pc);
                    fault
                },
            )?);
        }
        Ok(())
    }

    pub(super) fn supports(binding: &crate::native::Binding) -> bool {
        use crate::native::Binding;
        matches!(
            binding,
            Binding::FileResource(_)
                | Binding::ConsoleWriteBytes
                | Binding::JoinWorker
                | Binding::JoinWorkerResult
                | Binding::ReadAllText
                | Binding::WriteAllText
                | Binding::WriteLine
                | Binding::ConsoleReadByte
                | Binding::ConsoleTerminalOpen
                | Binding::ConsoleTerminalReadByte
                | Binding::ConsoleTerminalClose
                | Binding::ConsoleFlush
        )
    }

    pub(super) fn array_usage(
        &self,
        usage: &mut crate::arrays::Usage,
        limits: &Limits,
    ) -> Result<(), Fault> {
        for arg in &self.args {
            crate::arrays::measure(arg, usage, limits)?;
        }
        if let Some(value) = &self.immediate {
            crate::arrays::measure(value, usage, limits)?;
        }
        Ok(())
    }

    pub(super) fn trace_roots(&self, roots: &mut Vec<usize>) {
        if let Some((array, _)) = &self.patch {
            crate::gc::trace(array, roots);
        }
        for arg in &self.args {
            crate::gc::trace(arg, roots);
        }
    }

    /// Commit only the transferred elements, preserving concurrent changes elsewhere.
    pub(super) fn resume(&self, mut value: Value, frames: &mut [Frame]) -> Result<(), Fault> {
        if let (Some((array, offset)), Value::Erased(payload)) = (&self.patch, &value) {
            if let Value::Array {
                element: Type::Byte,
                elements,
            } = payload.as_ref()
            {
                match array {
                    Value::MemoryView(view) => {
                        if elements.len() > view.length() {
                            return Err(Fault::new("file read exceeds destination memory"));
                        }
                        let destination = view.as_slice();
                        for (index, byte) in elements.iter().enumerate() {
                            destination.set(index, byte.clone())?;
                        }
                    }
                    Value::ObjectReference(array) => {
                        for (index, byte) in elements.iter().enumerate() {
                            array
                                .reference
                                .element(offset + index, &Type::Byte)?
                                .write(byte.clone())?;
                        }
                    }
                    _ => return Err(Fault::new("Invalid file read destination")),
                }
                value = Value::Erased(Box::new(Value::Int32(elements.len() as i32)));
            }
        }
        let caller = frames
            .last_mut()
            .ok_or_else(|| Fault::new("missing host caller"))?;
        if !self.callee.no_result {
            caller.stack.push(value);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn run(
        &mut self,
        module: &Module,
        state: &mut InstructionState,
        _native_libraries: &mut Option<crate::interop::NativeLibraries>,
        memory: &crate::memory::SharedMemory,
        output: &mut Vec<String>,
        bytes: &mut [Vec<u8>; 2],
        options: &ExecutionOptions,
        participant: &mut crate::shared_heap::Participant,
        frame_depth: usize,
    ) -> Result<Value, Fault> {
        let result = (|| {
            options.check_cancellation(&self.context, self.pc)?;
            if let Some(value) = &self.immediate {
                return Ok(value.clone());
            }
            let value = if self.indirect.is_some() || self.callee.pinvoke.is_some() {
                let _borrow = memory.borrow_native(&self.args)?;
                let call = {
                    let mut libraries = state
                        .invocation
                        .native_libraries
                        .lock()
                        .expect("native library lock poisoned");
                    let libraries = libraries.as_mut().ok_or_else(|| {
                        Fault::new("native imports require trusted run_with_native execution")
                    })?;
                    match &self.indirect {
                        Some(call) => call.clone(),
                        // SAFETY: only trusted execution supplies this library session.
                        None => std::sync::Arc::new(unsafe { libraries.prepare(&self.callee)? }),
                    }
                };
                let mut runner = |callback: &Value, arguments: Vec<Value>| {
                    foreign_callbacks::invoke(
                        module,
                        callback,
                        arguments,
                        state,
                        participant,
                        options,
                        frame_depth,
                        output,
                        bytes,
                    )
                };
                // SAFETY: the validated ABI and caller's native trust cover this call.
                // All input storage, callback contexts and loaded code remain alive.
                // No graph or library lock is held while entering native code.
                unsafe { call.invoke(self.args.clone(), memory, &mut runner)? }
            } else {
                let binding = crate::native::bind(&self.callee)?;
                let before = output.len();
                let value = match binding {
                    crate::native::Binding::ConsoleTerminalOpen
                    | crate::native::Binding::ConsoleTerminalReadByte
                    | crate::native::Binding::ConsoleTerminalClose => state
                        .invocation
                        .terminal
                        .invoke(&binding, &self.args, options.console.as_deref())?,

                    crate::native::Binding::FileResource(
                        crate::file_streams::Operation::ReadInto,
                    ) => {
                        let [Value::Int32(id), Value::Int32(count)] = self.args.as_slice() else {
                            return Err(Fault::new("Invalid prepared file read"));
                        };
                        state
                            .invocation
                            .files
                            .lock()
                            .expect("file table lock poisoned")
                            .read_transfer(*id, *count)
                    }
                    crate::native::Binding::FileResource(operation) => state
                        .invocation
                        .files
                        .lock()
                        .expect("file table lock poisoned")
                        .invoke(operation, &self.args, &state.invocation.limits)?,
                    crate::native::Binding::JoinWorkerResult => Value::Erased(Box::new(
                        self.join
                            .take()
                            .expect("prepared worker join")
                            .wait(output, options, true)?,
                    )),
                    crate::native::Binding::JoinWorker => self
                        .join
                        .take()
                        .expect("prepared worker join")
                        .wait(output, options, false)?,
                    _ => binding.invoke(
                        self.args.clone(),
                        self.assembly.as_deref(),
                        module,
                        &options.limits,
                        output,
                        bytes,
                        options,
                    )?,
                };
                if options.console.is_none()
                    && matches!(
                        binding,
                        crate::native::Binding::JoinWorker
                            | crate::native::Binding::JoinWorkerResult
                    )
                {
                    for line in &output[before..] {
                        bytes[0].extend_from_slice(line.as_bytes());
                        bytes[0].push(b'\n');
                    }
                }
                value
            };
            expect(&value, &self.callee.returns)?;
            Ok(
                if self.indirect.is_some() || self.callee.pinvoke.is_some() {
                    value.on_stack()
                } else {
                    value
                },
            )
        })();
        result.map_err(|mut fault: Fault| {
            fault.function = Some(self.context.clone());
            fault.instruction = Some(self.pc);
            fault
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(name: &str) -> crate::metadata::Function {
        let module = crate::assemble(&format!(
            ".module System\n{}",
            include_str!("../runtime/legacy/bootstrap/neoCLR/Runtime/FileStreams.neoil")
        ))
        .unwrap();
        module
            .functions
            .into_iter()
            .find(|f| f.name == name)
            .unwrap()
    }

    fn memory_service(readonly: bool) -> crate::metadata::Function {
        let mut function = service(if readonly {
            "neoCLR.Runtime.FileWriteChunk"
        } else {
            "neoCLR.Runtime.FileReadInto"
        });
        function.parameters = vec![Type::Int32, crate::file_streams::memory_type(readonly)];
        function.parameter_names.clear();
        function
    }

    #[test]
    fn persistent_read_roots_only_owner_until_completion_and_preserves_window() {
        let owner = crate::shared_heap::Owner::new(2);
        let mut participant = owner.participant().unwrap();
        let mut other = owner.participant().unwrap();
        let mut heap = participant.enter();
        let id = heap
            .allocate(Value::Array {
                element: Type::Byte,
                elements: vec![Value::Byte(9); 4],
            })
            .unwrap();
        let reference = heap.address(id).unwrap();
        let view = crate::memory_views::Memory::new(&heap, reference.clone(), 1, 2).unwrap();
        let call = HostCall::new(
            memory_service(false),
            vec![
                Value::Int32(1),
                Value::MemoryView(crate::MemoryView::persistent(view)),
            ],
            None,
            "Caller".into(),
            0,
        )
        .unwrap();
        let mut roots = vec![];
        call.trace_roots(&mut roots);
        assert_eq!(roots.len(), 1);
        heap.publish(roots).unwrap();
        drop(heap);
        let mut heap = other.enter();
        heap.collect(vec![], crate::CollectionReason::ExplicitRequest)
            .unwrap();
        reference
            .element(3, &Type::Byte)
            .unwrap()
            .write(Value::Byte(99))
            .unwrap();
        let caller =
            crate::assemble(".module Test\n.function Caller() -> Void\nret\n.end").unwrap();
        let mut frames = vec![Frame::new(caller.functions[0].clone(), vec![]).unwrap()];
        let oversized = Value::Erased(Box::new(Value::Array {
            element: Type::Byte,
            elements: vec![Value::Byte(7); 3],
        }));
        assert!(call.resume(oversized, &mut frames).is_err());
        assert_eq!(
            reference.element(1, &Type::Byte).unwrap().read().unwrap(),
            Value::Byte(9)
        );
        call.resume(
            Value::Erased(Box::new(Value::Array {
                element: Type::Byte,
                elements: vec![Value::Byte(42)],
            })),
            &mut frames,
        )
        .unwrap();
        assert_eq!(
            reference.read().unwrap(),
            Value::Array {
                element: Type::Byte,
                elements: vec![
                    Value::Byte(9),
                    Value::Byte(42),
                    Value::Byte(9),
                    Value::Byte(99)
                ]
            }
        );
        drop(heap);
        let mut heap = participant.enter();
        drop(call);
        heap.publish(vec![]).unwrap();
        heap.collect(vec![], crate::CollectionReason::ExplicitRequest)
            .unwrap();
        assert_eq!(heap.statistics().live_objects, 0);
    }

    #[test]
    fn persistent_write_snapshots_only_selected_initialized_bytes() {
        for mode in 0..3 {
            let console = mode != 0;
            let borrowed = mode == 2;
            let mut function = memory_service(true);
            let first = if console {
                function.name = "neoCLR.Runtime.ConsoleWriteBytes".into();
                function.parameters[0] = Type::Boolean;
                if borrowed {
                    function.parameters[1] = crate::file_streams::console_slice_type();
                }
                Value::Boolean(true)
            } else {
                Value::Int32(1)
            };
            let mut heap = crate::ManagedHeap::default();
            let id = heap
                .allocate(Value::Array {
                    element: Type::Byte,
                    elements: vec![
                        Value::Uninitialized(Type::Byte),
                        Value::Byte(42),
                        Value::Uninitialized(Type::Byte),
                    ],
                })
                .unwrap();
            let reference = heap.address(id).unwrap();
            let view = crate::memory_views::Memory::new(&heap, reference.clone(), 1, 1)
                .unwrap()
                .as_readonly();
            let call = HostCall::new(
                function.clone(),
                vec![
                    first.clone(),
                    Value::MemoryView(if borrowed {
                        crate::MemoryView::persistent(view).as_slice()
                    } else {
                        crate::MemoryView::persistent(view)
                    }),
                ],
                None,
                "Caller".into(),
                0,
            )
            .unwrap();
            reference
                .element(1, &Type::Byte)
                .unwrap()
                .write(Value::Byte(99))
                .unwrap();
            assert_eq!(
                call.args,
                vec![
                    first.clone(),
                    Value::Array {
                        element: Type::Byte,
                        elements: vec![Value::Byte(42)]
                    },
                    Value::Int32(0),
                    Value::Int32(1)
                ]
            );
            let mut roots = vec![];
            call.trace_roots(&mut roots);
            assert!(roots.is_empty());
            let limits = Limits {
                array_elements: 1,
                array_bytes: std::mem::size_of::<Value>(),
                ..Limits::default()
            };
            call.array_usage(&mut crate::arrays::Usage::default(), &limits)
                .unwrap();
            let uninitialized = crate::memory_views::Memory::new(&heap, reference, 0, 1)
                .unwrap()
                .as_readonly();
            assert!(
                HostCall::new(
                    function.clone(),
                    vec![
                        first.clone(),
                        Value::MemoryView(if borrowed {
                            crate::MemoryView::persistent(uninitialized).as_slice()
                        } else {
                            crate::MemoryView::persistent(uninitialized)
                        })
                    ],
                    None,
                    "Caller".into(),
                    0
                )
                .is_err()
            );
        }
    }

    #[test]
    fn file_memory_bindings_require_exact_persistent_byte_capabilities() {
        for readonly in [false, true] {
            let mut function = memory_service(readonly);
            assert!(crate::native::bind(&function).is_ok());
            function.parameters[1] = crate::file_streams::memory_type(!readonly);
            assert!(crate::native::bind(&function).is_err());
            function.parameters[1] = crate::cryptography::view_type(readonly);
            assert!(crate::native::bind(&function).is_err());
            let Type::Constructed { arguments, .. } = &mut function.parameters[1] else {
                unreachable!()
            };
            arguments[0] = Type::Int32;
            assert!(crate::native::bind(&function).is_err());
        }
    }

    #[test]
    fn detached_read_preserves_destination_and_concurrent_tail_changes() {
        let owner = crate::shared_heap::Owner::new(2);
        let mut participant = owner.participant().unwrap();
        let mut other = owner.participant().unwrap();
        let mut heap = participant.enter();
        let id = heap
            .allocate(Value::Array {
                element: Type::Byte,
                elements: vec![Value::Byte(0); 4],
            })
            .unwrap();
        let reference = crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        };
        let call = HostCall::new(
            service("neoCLR.Runtime.FileReadInto"),
            vec![
                Value::Int32(1),
                Value::ObjectReference(reference.clone()),
                Value::Int32(1),
                Value::Int32(2),
            ],
            None,
            "Caller".into(),
            7,
        )
        .unwrap();
        let mut roots = vec![];
        call.trace_roots(&mut roots);
        heap.publish(roots).unwrap();
        drop(heap);
        let mut heap = other.enter();
        heap.collect(vec![], crate::CollectionReason::ExplicitRequest)
            .unwrap();
        reference
            .reference
            .element(3, &Type::Byte)
            .unwrap()
            .write(Value::Byte(99))
            .unwrap();
        drop(heap);
        let _heap = participant.enter();
        let caller =
            crate::assemble(".module Test\n.function Caller() -> Void\nret\n.end").unwrap();
        let mut frames = vec![Frame::new(caller.functions[0].clone(), vec![]).unwrap()];
        call.resume(
            Value::Erased(Box::new(Value::Array {
                element: Type::Byte,
                elements: vec![Value::Byte(42)],
            })),
            &mut frames,
        )
        .unwrap();
        assert_eq!(
            frames[0].stack,
            vec![Value::Erased(Box::new(Value::Int32(1)))]
        );
        assert_eq!(
            reference.reference.read().unwrap(),
            Value::Array {
                element: Type::Byte,
                elements: vec![
                    Value::Byte(0),
                    Value::Byte(42),
                    Value::Byte(0),
                    Value::Byte(99)
                ],
            }
        );
    }

    #[test]
    fn prepared_write_is_detached_from_subsequent_mutation() {
        let owner = crate::shared_heap::Owner::new(1);
        let mut participant = owner.participant().unwrap();
        let mut heap = participant.enter();
        let id = heap
            .allocate(Value::Array {
                element: Type::Byte,
                elements: vec![Value::Byte(42)],
            })
            .unwrap();
        let reference = crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        };
        let call = HostCall::new(
            service("neoCLR.Runtime.FileWriteChunk"),
            vec![
                Value::Int32(1),
                Value::ObjectReference(reference.clone()),
                Value::Int32(0),
                Value::Int32(1),
            ],
            None,
            "Caller".into(),
            7,
        )
        .unwrap();
        let mut usage = crate::arrays::Usage::default();
        assert!(
            call.array_usage(
                &mut usage,
                &Limits {
                    array_elements: 0,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        call.array_usage(&mut crate::arrays::Usage::default(), &Limits::default())
            .unwrap();
        reference
            .reference
            .element(0, &Type::Byte)
            .unwrap()
            .write(Value::Byte(99))
            .unwrap();
        heap.collect(vec![], crate::CollectionReason::ExplicitRequest)
            .unwrap();
        assert_eq!(
            call.args[1],
            Value::Array {
                element: Type::Byte,
                elements: vec![Value::Byte(42)],
            }
        );
    }
}
