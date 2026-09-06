use crate::{
    Fault, Module, Value,
    metadata::{Case, FunctionRef, Instruction as Op, Representation, Type},
};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub instructions: usize,
    pub frames: usize,
    pub stack: usize,
    pub heap_objects: usize,
    pub pointer_bytes: usize,
    pub pointer_allocations: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            instructions: 100_000,
            frames: 256,
            stack: 4096,
            heap_objects: 4096,
            pointer_bytes: 16 * 1024 * 1024,
            pointer_allocations: 4096,
        }
    }
}

#[derive(Debug)]
pub struct Execution {
    pub value: Value,
    pub output: Vec<String>,
    /// Retained until the execution result is dropped; references index this arena.
    pub heap: Vec<Value>,
    pub memory: crate::memory::PointerHeap,
}

fn resolve(module: &Module, target: &FunctionRef) -> Option<usize> {
    module.functions.iter().position(|f| {
        f.name == target.name
            && f.parameters == target.parameters
            && f.instance == target.instance
            && target
                .owner
                .as_ref()
                .is_none_or(|owner| f.owner.as_ref() == Some(owner))
    })
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    if module.name == "System" {
        if !module.entry.is_empty() {
            return Err(Fault::new(
                "System is reserved for the runtime library without an entry point",
            ));
        }
        validate_linked(module)
    } else {
        crate::library::link(module, crate::library::system()?).map(|_| ())
    }
}

pub(crate) fn validate_linked(module: &Module) -> Result<(), Fault> {
    if module.name.is_empty() {
        return Err(Fault::new("module name must not be empty"));
    }
    if module.format != 3 {
        return Err(Fault::new("unsupported module format (expected 3)"));
    }
    let mut names = HashSet::new();
    for def in &module.types {
        if def.name.is_empty() || !names.insert(&def.name) {
            return Err(Fault::new("empty or duplicate type name"));
        }
        let ty = Type::from_name(&def.name);
        if ty.definition_name() != Some(def.name.as_str()) {
            return Err(Fault::new("type definitions must use canonical names"));
        }
        if (def.representation == Representation::Runtime) != ty.is_primitive() {
            return Err(Fault::new(
                "runtime representation is reserved for canonical primitive types",
            ));
        }
        if def.representation == Representation::Runtime && !def.fields.is_empty() {
            return Err(Fault::new(
                "runtime primitive types cannot declare record fields",
            ));
        }
        let mut fields = HashSet::new();
        for field in &def.fields {
            if field.name.is_empty() || !fields.insert(&field.name) {
                return Err(Fault::new("empty or duplicate field name"));
            }
            check_type(&field.ty, module)?;
        }
    }
    let mut signatures = HashSet::new();
    for function in &module.functions {
        if function.name.is_empty()
            || !signatures.insert((&function.name, &function.parameters, function.instance))
        {
            return Err(Fault::new(
                "empty name, duplicate or reserved function signature",
            ));
        }
    }
    for function in &module.functions {
        crate::metadata::validate_slot_names(
            &function.parameter_names,
            function.parameters.len(),
            function.instance,
        )?;
        crate::metadata::validate_slot_names(&function.local_names, function.locals.len(), false)?;
        if function.instance && function.owner.is_none() {
            return Err(Fault::new("instance method requires a declaring type"));
        }
        if let Some(owner) = &function.owner {
            check_type(owner, module)?;
            let def = module
                .type_definition(owner)
                .ok_or_else(|| Fault::new("method owner has no type definition"))?;
            let prefix = format!("{}.", def.name);
            let member = function
                .name
                .strip_prefix(&prefix)
                .ok_or_else(|| Fault::new("method name does not match its declaring type"))?;
            if member.is_empty() || member.contains('.') {
                return Err(Fault::new("invalid member name"));
            }
        }
        for ty in function
            .parameters
            .iter()
            .chain(&function.locals)
            .chain([&function.returns])
        {
            check_type(ty, module)?;
        }
        match function.impl_flags {
            crate::metadata::INTERNAL_CALL => {
                if !function.body.is_empty() || !function.locals.is_empty() {
                    return Err(Fault::new(
                        "InternalCall must not have an IL body or locals",
                    ));
                }
                crate::native::bind(function)?;
                continue;
            }
            0 => (),
            _ => return Err(Fault::new("unsupported method implementation flags")),
        }
        if function.body.is_empty() {
            return Err(Fault::new("empty function body"));
        }
        for op in &function.body {
            match op {
                Op::Branch(i) | Op::BranchTrue(i) if *i >= function.body.len() => {
                    return Err(Fault::new("branch outside function"));
                }
                Op::Arg(i) if *i >= function.argument_types().len() => {
                    return Err(Fault::new("argument index outside signature"));
                }
                Op::Load(i) | Op::Store(i) if *i >= function.locals.len() => {
                    return Err(Fault::new("local index outside signature"));
                }
                Op::Call(target) => {
                    if let Some(owner) = &target.owner {
                        check_type(owner, module)?;
                    }
                    for ty in &target.parameters {
                        check_type(ty, module)?;
                    }
                    if resolve(module, target).is_none() {
                        return Err(Fault::new(format!(
                            "unknown function overload {}({:?})",
                            target.name, target.parameters
                        )));
                    }
                }
                Op::New(name) => {
                    let def = module
                        .types
                        .iter()
                        .find(|t| &t.name == name)
                        .ok_or_else(|| Fault::new(format!("unknown type {name}")))?;
                    if def.representation != Representation::Record {
                        return Err(Fault::new(
                            "newobj cannot construct a runtime primitive as a record",
                        ));
                    }
                }
                Op::SizeOf(ty)
                | Op::AlignOf(ty)
                | Op::Allocate(ty)
                | Op::LoadObject(ty)
                | Op::StoreObject(ty) => {
                    check_type(ty, module)?;
                    crate::memory::layout(module, ty)?;
                }
                Op::None(ty)
                | Op::Ok(ty)
                | Op::Err(ty)
                | Op::NullPointer(ty)
                | Op::PointerCast(ty)
                | Op::PointerFromInt(ty) => check_type(ty, module)?,
                _ => (),
            }
        }
    }
    if !module.entry.is_empty()
        && !module.functions.iter().any(|f| {
            f.name == module.entry
                && f.parameters.is_empty()
                && !f.is_internal_call()
                && !f.instance
        })
    {
        return Err(Fault::new("parameterless entry function not found"));
    }
    Ok(())
}

fn check_type(ty: &Type, module: &Module) -> Result<(), Fault> {
    match ty {
        Type::Named(name) if Type::from_name(name).is_primitive() => Err(Fault::new(
            "primitive type requires canonical primitive signature encoding",
        )),
        Type::Named(name) if !module.types.iter().any(|t| &t.name == name) => {
            Err(Fault::new(format!("unknown type {name}")))
        }
        Type::Option(t) | Type::Ref(t) | Type::Ptr(t) => check_type(t, module),
        Type::Result(t, e) => {
            check_type(t, module)?;
            check_type(e, module)
        }
        _ => Ok(()),
    }
}

struct Frame {
    function: usize,
    pc: usize,
    args: Vec<Value>,
    locals: Vec<Option<Value>>,
    stack: Vec<Value>,
}

impl Frame {
    fn new(function: usize, args: Vec<Value>, module: &Module) -> Self {
        Self {
            function,
            pc: 0,
            args,
            locals: vec![None; module.functions[function].locals.len()],
            stack: vec![],
        }
    }
    fn pop(&mut self) -> Result<Value, Fault> {
        self.stack
            .pop()
            .ok_or_else(|| Fault::new("evaluation stack underflow"))
    }
    fn pointer(&mut self) -> Result<crate::memory::Pointer, Fault> {
        match self.pop()? {
            Value::Pointer(p) => Ok(p),
            _ => Err(Fault::new("expected Ptr")),
        }
    }
    fn args(&mut self, types: &[Type]) -> Result<Vec<Value>, Fault> {
        if self.stack.len() < types.len() {
            return Err(Fault::new("not enough arguments"));
        }
        let args = self.stack.split_off(self.stack.len() - types.len());
        args.into_iter()
            .zip(types)
            .map(|(value, ty)| value.for_storage(ty))
            .collect()
    }
}

fn expect(value: &Value, ty: &Type) -> Result<(), Fault> {
    if value.ty() == *ty {
        Ok(())
    } else {
        Err(Fault::new(format!("expected {ty:?}, got {:?}", value.ty())))
    }
}

pub fn run(module: &Module, limits: Limits) -> Result<Execution, Fault> {
    run_with_library(module, crate::library::system()?, limits)
}

/// Execute against an explicitly compiled System library artifact.
pub fn run_with_library(
    module: &Module,
    library: &Module,
    limits: Limits,
) -> Result<Execution, Fault> {
    if module.entry.is_empty() {
        return Err(Fault::new(
            "cannot execute a library without an entry point",
        ));
    }
    let linked = crate::library::link(module, library)?;
    interpret(&linked, limits)
}

fn interpret(module: &Module, limits: Limits) -> Result<Execution, Fault> {
    if limits.frames == 0 {
        return Err(Fault::new("frame limit exceeded"));
    }
    let entry = module
        .functions
        .iter()
        .position(|f| f.name == module.entry && f.parameters.is_empty() && !f.instance)
        .ok_or_else(|| Fault::new("missing entry"))?;
    let mut frames = vec![Frame::new(entry, vec![], module)];
    let mut heap: Vec<Value> = vec![];
    let mut memory = crate::memory::PointerHeap::default();
    let mut output = vec![];
    for _ in 0..limits.instructions {
        let frame = frames
            .last_mut()
            .ok_or_else(|| Fault::new("missing frame"))?;
        let function = &module.functions[frame.function];
        let pc = frame.pc;
        let op = function.body.get(pc).ok_or_else(|| Fault {
            message: "function fell through without ret".into(),
            function: Some(function.name.clone()),
            instruction: Some(pc),
        })?;
        frame.pc += 1;
        let context = function.name.clone();
        // Host Result propagates terminal faults; there is no guest exception machinery.
        let step = (|| -> Result<Option<Value>, Fault> {
            let frame = frames
                .last_mut()
                .ok_or_else(|| Fault::new("missing frame"))?;
            match op {
                Op::Int(n) => frame.stack.push(Value::Int32(*n)),
                Op::Int64(n) => frame.stack.push(Value::Int64(*n)),
                Op::Bool(b) => frame.stack.push(Value::Boolean(*b)),
                Op::String(s) => frame.stack.push(Value::String(s.clone())),
                Op::Void => frame.stack.push(Value::Void),
                Op::Error(s) => frame.stack.push(Value::Error(s.clone())),
                Op::Arg(i) => frame.stack.push(frame.args[*i].clone().on_stack()),
                Op::Load(i) => frame.stack.push(
                    frame.locals[*i]
                        .clone()
                        .ok_or_else(|| Fault::new("read of uninitialized local"))?
                        .on_stack(),
                ),
                Op::Store(i) => {
                    let value = frame.pop()?;
                    frame.locals[*i] = Some(value.for_storage(&function.locals[*i])?);
                }
                Op::Dup => {
                    let value = frame
                        .stack
                        .last()
                        .ok_or_else(|| Fault::new("evaluation stack underflow"))?
                        .clone();
                    frame.stack.push(value);
                }
                Op::Pop => {
                    frame.pop()?;
                }
                Op::Add
                | Op::Sub
                | Op::Mul
                | Op::AddChecked
                | Op::SubChecked
                | Op::MulChecked
                | Op::AddCheckedUnsigned
                | Op::SubCheckedUnsigned
                | Op::MulCheckedUnsigned
                | Op::Divide
                | Op::DivideUnsigned
                | Op::Less
                | Op::LessUnsigned => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    frame.stack.push(crate::numeric::binary(op, left, right)?);
                }
                Op::ConvertNativeInt
                | Op::ConvertNativeUInt
                | Op::ConvertInt32
                | Op::ConvertInt8
                | Op::ConvertUInt8
                | Op::ConvertInt16
                | Op::ConvertUInt16
                | Op::ConvertUInt32
                | Op::ConvertInt64
                | Op::ConvertUInt64 => {
                    let value = frame.pop()?;
                    frame.stack.push(crate::numeric::convert(op, value)?);
                }
                Op::PointerFromInt(ty) => {
                    let address = match frame.pop()? {
                        Value::IntPtr(n) => n as usize,
                        Value::UIntPtr(n) => n,
                        _ => return Err(Fault::new("ptr.fromint requires native integer")),
                    };
                    frame.stack.push(Value::Pointer(
                        memory.pointer_at_address(address, ty.clone()),
                    ));
                }
                Op::Equal => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    expect(&right, &left.ty())?;
                    frame.stack.push(Value::Boolean(left == right));
                }
                Op::Branch(i) => frame.pc = *i,
                Op::BranchTrue(i) => match frame.pop()? {
                    Value::Boolean(true) => frame.pc = *i,
                    Value::Boolean(false) => (),
                    _ => return Err(Fault::new("brtrue requires Boolean")),
                },
                Op::Call(target) => {
                    let index = resolve(module, target)
                        .ok_or_else(|| Fault::new("unknown function overload"))?;
                    let callee = &module.functions[index];
                    let args = frame.args(&callee.argument_types())?;
                    if callee.is_internal_call() {
                        let value = crate::native::bind(callee)?.invoke(args, &mut output)?;
                        expect(&value, &callee.returns)?;
                        frame.stack.push(value);
                    } else {
                        if frames.len() >= limits.frames {
                            return Err(Fault::new("frame limit exceeded"));
                        }
                        frames.push(Frame::new(index, args, module));
                    }
                }
                Op::Return => {
                    let value = frame.pop()?;
                    let value = value.for_storage(&function.returns)?;
                    if !frame.stack.is_empty() {
                        return Err(Fault::new("ret requires exactly one value"));
                    }
                    frames.pop();
                    if let Some(caller) = frames.last_mut() {
                        caller.stack.push(value.on_stack());
                    } else {
                        return Ok(Some(value));
                    }
                }
                Op::New(name) => {
                    let def = module
                        .types
                        .iter()
                        .find(|t| &t.name == name)
                        .ok_or_else(|| Fault::new("unknown type"))?;
                    let types: Vec<_> = def.fields.iter().map(|f| f.ty.clone()).collect();
                    let fields = frame.args(&types)?;
                    frame.stack.push(Value::Object {
                        name: name.clone(),
                        fields,
                    });
                }
                Op::Field(i) => {
                    let Value::Object { fields, .. } = frame.pop()? else {
                        return Err(Fault::new(
                            "ldfld requires object value (use heap.load for a reference)",
                        ));
                    };
                    frame.stack.push(
                        fields
                            .get(*i)
                            .ok_or_else(|| Fault::new("field index out of range"))?
                            .clone()
                            .on_stack(),
                    );
                }
                Op::SetField(i) => {
                    let value = frame.pop()?;
                    let Value::Object { name, mut fields } = frame.pop()? else {
                        return Err(Fault::new("stfld requires object value"));
                    };
                    let field = fields
                        .get_mut(*i)
                        .ok_or_else(|| Fault::new("field index out of range"))?;
                    *field = value.for_storage(&field.ty())?;
                    frame.stack.push(Value::Object { name, fields });
                }
                Op::SizeOf(ty) | Op::AlignOf(ty) => {
                    let layout = crate::memory::layout(module, ty)?;
                    let value = if matches!(op, Op::SizeOf(_)) {
                        layout.size
                    } else {
                        layout.alignment
                    };
                    frame.stack.push(Value::Int32(
                        i32::try_from(value)
                            .map_err(|_| Fault::new("layout exceeds Int32 range"))?,
                    ));
                }
                Op::Allocate(ty) => {
                    let count = match frame.pop()? {
                        Value::Int32(n) => usize::try_from(n).ok(),
                        Value::IntPtr(n) => usize::try_from(n).ok(),
                        Value::UIntPtr(n) => Some(n),
                        _ => return Err(Fault::new("allocation count requires integer")),
                    }
                    .ok_or_else(|| Fault::new("negative allocation count"))?;
                    let layout = crate::memory::layout(module, ty)?;
                    let pointer = memory.allocate(
                        ty.clone(),
                        &layout,
                        count,
                        limits.pointer_bytes,
                        limits.pointer_allocations,
                    )?;
                    frame.stack.push(Value::Pointer(pointer));
                }
                Op::Free => {
                    memory.free(&frame.pointer()?)?;
                    frame.stack.push(Value::Void);
                }
                Op::NullPointer(ty) => frame
                    .stack
                    .push(Value::Pointer(crate::memory::Pointer::null(ty.clone()))),
                Op::PointerCast(ty) => {
                    let mut pointer = frame.pointer()?;
                    pointer.target = ty.clone();
                    frame.stack.push(Value::Pointer(pointer));
                }
                Op::PointerAdd => {
                    let offset = match frame.pop()? {
                        Value::Int32(n) => n as isize,
                        Value::IntPtr(n) => n,
                        _ => return Err(Fault::new("pointer offset requires Int32 or IntPtr")),
                    };
                    let pointer = frame.pointer()?;
                    frame
                        .stack
                        .push(Value::Pointer(memory.offset(&pointer, offset)?));
                }
                Op::FieldAddress(index) => {
                    let pointer = frame.pointer()?;
                    if !matches!(pointer.target, Type::Named(_)) {
                        return Err(Fault::new("ldflda requires pointer to record"));
                    }
                    let layout = crate::memory::layout(module, &pointer.target)?;
                    frame
                        .stack
                        .push(Value::Pointer(memory.field(&pointer, &layout, *index)?));
                }
                Op::LoadObject(_)
                | Op::LoadIndirectInt32
                | Op::LoadIndirectInt8
                | Op::LoadIndirectUInt8
                | Op::LoadIndirectInt16
                | Op::LoadIndirectUInt16
                | Op::LoadIndirectUInt32
                | Op::LoadIndirectInt64
                | Op::LoadIndirectNative => {
                    let mut pointer = frame.pointer()?;
                    let ty = if let Op::LoadObject(ty) = op {
                        if pointer.target != *ty {
                            return Err(Fault::new("memory load pointer type mismatch"));
                        }
                        ty.clone()
                    } else {
                        crate::numeric::indirect_type(op, &pointer.target)?
                    };
                    pointer.target = ty.clone();
                    let layout = crate::memory::layout(module, &ty)?;
                    frame.stack.push(memory.read(&pointer, &layout)?.on_stack());
                }
                Op::StoreObject(_)
                | Op::StoreIndirectInt32
                | Op::StoreIndirectInt8
                | Op::StoreIndirectInt16
                | Op::StoreIndirectInt64
                | Op::StoreIndirectNative => {
                    let value = frame.pop()?;
                    let pointer = frame.pointer()?;
                    if let Op::StoreObject(ty) = op {
                        if pointer.target != *ty {
                            return Err(Fault::new("memory store pointer type mismatch"));
                        }
                    } else {
                        crate::numeric::indirect_type(op, &pointer.target)?;
                    }
                    let ty = &pointer.target;
                    let value = value
                        .for_storage(ty)
                        .map_err(|_| Fault::new("memory store type mismatch"))?;
                    let layout = crate::memory::layout(module, ty)?;
                    memory.write(&pointer, &layout, &value)?;
                }
                Op::HeapNew => {
                    if heap.len() >= limits.heap_objects {
                        return Err(Fault::new("heap object limit exceeded"));
                    }
                    let value = frame.pop()?;
                    let target = value.ty();
                    let index = heap.len();
                    heap.push(value);
                    frame.stack.push(Value::Reference { index, target });
                }
                Op::HeapLoad => {
                    let Value::Reference { index, .. } = frame.pop()? else {
                        return Err(Fault::new("heap.load requires Ref"));
                    };
                    frame.stack.push(
                        heap.get(index)
                            .ok_or_else(|| Fault::new("invalid reference"))?
                            .clone(),
                    );
                }
                Op::HeapStore => {
                    let value = frame.pop()?;
                    let Value::Reference { index, target } = frame.pop()? else {
                        return Err(Fault::new("heap.store requires Ref followed by value"));
                    };
                    expect(&value, &target)?;
                    *heap
                        .get_mut(index)
                        .ok_or_else(|| Fault::new("invalid reference"))? = value;
                    frame.stack.push(Value::Void);
                }
                Op::Some => {
                    let value = frame.pop()?;
                    frame.stack.push(Value::Union {
                        ty: Type::Option(Box::new(value.ty())),
                        case: Case::Some,
                        payload: Box::new(value),
                    });
                }
                Op::None(ty) => frame.stack.push(Value::Union {
                    ty: Type::Option(Box::new(ty.clone())),
                    case: Case::None,
                    payload: Box::new(Value::Void),
                }),
                Op::Ok(error) => {
                    let value = frame.pop()?;
                    let success = value.ty();
                    frame
                        .stack
                        .push(Value::result(value, success, error.clone(), Case::Ok));
                }
                Op::Err(success) => {
                    let value = frame.pop()?;
                    let error = value.ty();
                    frame
                        .stack
                        .push(Value::result(value, success.clone(), error, Case::Err));
                }
                Op::IsCase(wanted) | Op::LoadCase(wanted) => {
                    let Value::Union { ty, case, payload } = frame.pop()? else {
                        return Err(Fault::new("case instruction requires union"));
                    };
                    let valid = match ty {
                        Type::Option(_) => matches!(wanted, Case::Some | Case::None),
                        Type::Result(_, _) => matches!(wanted, Case::Ok | Case::Err),
                        _ => false,
                    };
                    if !valid {
                        return Err(Fault::new("case does not belong to union type"));
                    }
                    if matches!(op, Op::IsCase(_)) {
                        frame.stack.push(Value::Boolean(case == *wanted));
                    } else if case == *wanted {
                        frame.stack.push(*payload);
                    } else {
                        return Err(Fault::new("union case mismatch"));
                    }
                }
                Op::Fault(message) => return Err(Fault::new(message)),
            }
            Ok(None)
        })();
        match step {
            Ok(Some(value)) => {
                return Ok(Execution {
                    value,
                    output,
                    heap,
                    memory,
                });
            }
            Err(mut fault) => {
                fault.function = Some(context);
                fault.instruction = Some(pc);
                return Err(fault);
            }
            Ok(None) => (),
        }
        if frames.iter().any(|f| f.stack.len() > limits.stack) {
            return Err(Fault::new("evaluation stack limit exceeded"));
        }
    }
    Err(Fault::new("instruction limit exceeded"))
}
