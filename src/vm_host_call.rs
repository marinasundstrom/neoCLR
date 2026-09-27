//! Prepared blocking calls. Run without graph access; resume under graph access.
use super::*;

pub(super) struct HostCall {
    callee: crate::metadata::Function,
    args: Vec<Value>,
    assembly: Option<String>,
    context: String,
    pc: usize,
    patch: Option<(crate::value::ObjectReference, usize)>,
    immediate: Option<Value>,
    join: Option<crate::workers::PendingJoin>,
}

impl HostCall {
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
                            patch = Some((array.clone(), offset));
                            args = vec![Value::Int32(*id), Value::Int32(count as i32)];
                        }
                        _ => immediate = Some(Value::Erased(Box::new(Value::Byte(7)))),
                    }
                }
                _ => {}
            }
        }
        Ok(Self {
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
        if self.callee.pinvoke.is_none()
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
        match binding {
            Binding::FileResource(_) | Binding::ConsoleWriteBytes => true,
            Binding::JoinWorker
            | Binding::JoinWorkerResult
            | Binding::ReadAllText
            | Binding::WriteAllText
            | Binding::WriteLine
            | Binding::ConsoleReadByte
            | Binding::ConsoleFlush => true,
            _ => false,
        }
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
            crate::gc::trace(&Value::ObjectReference(array.clone()), roots);
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
                for (index, byte) in elements.iter().enumerate() {
                    array
                        .reference
                        .element(offset + index, &Type::Byte)?
                        .write(byte.clone())?;
                }
                value = Value::Erased(Box::new(Value::Int32(elements.len() as i32)));
            }
        }
        frames
            .last_mut()
            .ok_or_else(|| Fault::new("missing host caller"))?
            .stack
            .push(value);
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
    ) -> Result<Value, Fault> {
        let result = (|| {
            options.check_cancellation(&self.context, self.pc)?;
            if let Some(value) = &self.immediate {
                return Ok(value.clone());
            }
            let value = if self.callee.pinvoke.is_some() {
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
                    // SAFETY: only run_with_native supplies this trusted session.
                    unsafe { libraries.prepare(&self.callee)? }
                };
                // SAFETY: the trusted ABI contract still covers exports and indirect
                // pointers. Tracked argument storage stays exclusively borrowed; no
                // heap, scheduler, library-table or memory mutex is held in native code.
                unsafe { call.invoke(self.args.clone(), memory)? }
            } else {
                let binding = crate::native::bind(&self.callee)?;
                let before = output.len();
                let value = match binding {
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
            Ok(if self.callee.pinvoke.is_some() {
                value.on_stack()
            } else {
                value
            })
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
            include_str!("../runtime/neoCLR/Runtime/FileStreams.neoil")
        ))
        .unwrap();
        module
            .functions
            .into_iter()
            .find(|f| f.name == name)
            .unwrap()
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
        assert!(call.array_usage(&mut usage, &Limits { array_elements: 0, ..Limits::default() }).is_err());
        call.array_usage(&mut crate::arrays::Usage::default(), &Limits::default()).unwrap();
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
