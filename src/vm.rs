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
    /// Retains libraries for the lifetime of returned native addresses.
    pub native_libraries: Option<crate::interop::NativeLibraries>,
}

pub(crate) fn resolve(
    module: &Module,
    target: &FunctionRef,
) -> Result<crate::metadata::Function, Fault> {
    let mut found = None;
    for (index, definition) in module.functions.iter().enumerate() {
        let identity = definition
            .definition
            .clone()
            .unwrap_or_else(|| crate::metadata::MemberId {
                module: module.name.clone(),
                index: index as u32,
            });
        if target
            .definition
            .as_ref()
            .is_some_and(|wanted| wanted != &identity)
        {
            continue;
        }
        if definition.name != target.name || definition.instance != target.instance {
            continue;
        }
        let arity = definition
            .owner
            .as_ref()
            .and_then(|o| module.type_definition(o))
            .map_or(0, |d| d.generic_parameters.len());
        let mut candidate = if arity > 0 {
            let Some(Type::Constructed {
                definition: owner,
                arguments,
            }) = &target.owner
            else {
                continue;
            };
            if definition.owner.as_ref().and_then(Type::definition_name) != Some(owner.as_str())
                || arguments.len() != arity
            {
                continue;
            }
            let mut instantiated =
                definition.map_types(|ty| ty.substitute_type_parameters(arguments))?;
            instantiated.owner = target.owner.clone();
            instantiated
        } else {
            if target
                .owner
                .as_ref()
                .is_some_and(|owner| definition.owner.as_ref() != Some(owner))
            {
                continue;
            }
            definition.clone()
        };
        candidate.definition = Some(identity);
        if candidate.parameters == target.parameters {
            if found.is_some() {
                return Err(Fault::new(
                    "ambiguous function overload after type substitution",
                ));
            }
            found = Some(candidate);
        }
    }
    found.ok_or_else(|| {
        Fault::new(format!(
            "unknown function overload {}({:?})",
            target.name, target.parameters
        ))
    })
}

pub(crate) fn record_fields(
    module: &Module,
    ty: &Type,
    arity: usize,
) -> Result<Vec<crate::metadata::Field>, Fault> {
    check_type_context(ty, module, arity, 0)?;
    let (name, arguments): (&str, &[Type]) = match ty {
        Type::Named(name) => (name, &[]),
        Type::Constructed {
            definition,
            arguments,
        } => (definition, arguments),
        _ => return Err(Fault::new("expected record type reference")),
    };
    let def = module
        .types
        .iter()
        .find(|d| d.name == name && d.representation == Representation::Record)
        .ok_or_else(|| Fault::new("expected record definition"))?;
    def.fields
        .iter()
        .map(|f| {
            Ok(crate::metadata::Field {
                name: f.name.clone(),
                ty: f.ty.substitute_type_parameters(arguments)?,
            })
        })
        .collect()
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    if module.name == "System" {
        if !module.entry.is_empty() {
            return Err(Fault::new(
                "System is reserved for the runtime library without an entry point",
            ));
        }
        let mut normalized = module.clone();
        normalized.normalize_definition_ids()?;
        validate_linked(&normalized)
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
    let mut type_identities = HashSet::new();
    for def in &module.types {
        if def
            .definition
            .as_ref()
            .is_some_and(|id| !type_identities.insert(id))
        {
            return Err(Fault::new("duplicate type definition identity"));
        }
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
        if def.generic_parameters.len() > u16::MAX as usize + 1 {
            return Err(Fault::new("too many type parameters"));
        }
        let mut parameter_names = HashSet::new();
        for name in def.generic_parameters.iter().flatten() {
            if !crate::metadata::valid_slot_name(name)
                || Type::from_name(name).is_primitive()
                || !parameter_names.insert(name)
            {
                return Err(Fault::new("invalid or duplicate type parameter name"));
            }
        }
        if !def.generic_parameters.is_empty()
            && (def.representation != Representation::Record
                || matches!(def.name.as_str(), "Option" | "Result" | "Ref" | "Ptr"))
        {
            return Err(Fault::new(
                "reserved type cannot declare generic parameters",
            ));
        }
        if def.packing.is_some() || def.minimum_size.is_some() {
            if def.representation != Representation::Record {
                return Err(Fault::new("runtime types cannot override layout"));
            }
            if def.generic_parameters.is_empty() {
                crate::memory::layout(module, &ty)?;
            } else if def
                .packing
                .is_some_and(|p| !matches!(p, 0 | 1 | 2 | 4 | 8 | 16 | 32 | 64 | 128))
                || def.minimum_size.is_some_and(|s| s > i32::MAX as u32)
            {
                return Err(Fault::new("invalid generic record layout controls"));
            }
        }
        let mut fields = HashSet::new();
        for field in &def.fields {
            if field.name.is_empty() || !fields.insert(&field.name) {
                return Err(Fault::new("empty or duplicate field name"));
            }
            check_type_context(&field.ty, module, def.generic_parameters.len(), 0)?;
        }
    }
    let mut identities = HashSet::new();
    let mut signatures = HashSet::new();
    for function in &module.functions {
        if function
            .definition
            .as_ref()
            .is_some_and(|identity| !identities.insert(identity))
        {
            return Err(Fault::new("duplicate function definition identity"));
        }
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
        let arity = function
            .owner
            .as_ref()
            .and_then(|owner| module.type_definition(owner))
            .map_or(0, |d| d.generic_parameters.len());
        let check = |ty: &Type| check_type_context(ty, module, arity, 0);
        if let Some(owner) = &function.owner {
            if arity == 0 {
                check_type(owner, module)?;
            }
            let def = module
                .type_definition(owner)
                .ok_or_else(|| Fault::new("method owner has no type definition"))?;
            if matches!((&def.definition, &function.definition), (Some(ty), Some(method)) if ty.module != method.module)
            {
                return Err(Fault::new(
                    "method and declaring type must belong to the same module",
                ));
            }
            let prefix = format!("{}.", def.name);
            let member = function
                .name
                .strip_prefix(&prefix)
                .ok_or_else(|| Fault::new("method name does not match its declaring type"))?;
            if member.is_empty() || (member.contains('.') && member != ".ctor") {
                return Err(Fault::new("invalid member name"));
            }
            if member == ".ctor" && (!function.instance || function.returns != Type::Void) {
                return Err(Fault::new(
                    ".ctor requires an instance method returning Void",
                ));
            }
        }
        for ty in function
            .parameters
            .iter()
            .chain(&function.locals)
            .chain([&function.returns])
        {
            check(ty)?;
        }
        if arity > 0 && (function.pinvoke.is_some() || function.impl_flags != 0) {
            return Err(Fault::new("generic owners require IL methods"));
        }
        if function.pinvoke.is_some() {
            crate::interop::validate(function)?;
            continue;
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
        let valid_target = |index: usize| {
            index < function.body.len()
                && (index == 0 || !matches!(function.body[index - 1], Op::Unaligned(_)))
        };
        for (pc, op) in function.body.iter().enumerate() {
            match op {
                Op::Unaligned(alignment) => {
                    if !matches!(alignment, 1 | 2 | 4) {
                        return Err(Fault::new("unaligned. requires alignment 1, 2, or 4"));
                    }
                    if !function.body.get(pc + 1).is_some_and(Op::accepts_unaligned) {
                        return Err(Fault::new(
                            "unaligned. must immediately precede a supported memory access",
                        ));
                    }
                }
                Op::Branch(i)
                | Op::BranchTrue(i)
                | Op::BranchFalse(i)
                | Op::BranchEqual(i)
                | Op::BranchNotEqual(i)
                | Op::BranchGreater(i)
                | Op::BranchGreaterUnsigned(i)
                | Op::BranchLess(i)
                | Op::BranchLessUnsigned(i)
                | Op::BranchGreaterEqual(i)
                | Op::BranchGreaterEqualUnsigned(i)
                | Op::BranchLessEqual(i)
                | Op::BranchLessEqualUnsigned(i)
                    if !valid_target(*i) =>
                {
                    return Err(Fault::new(
                        "branch outside function or into prefixed instruction",
                    ));
                }
                Op::Switch(targets) if targets.iter().any(|i| !valid_target(*i)) => {
                    return Err(Fault::new(
                        "switch target outside function or into prefixed instruction",
                    ));
                }
                Op::Arg(i) | Op::StoreArg(i) if *i >= function.argument_types().len() => {
                    return Err(Fault::new("argument index outside signature"));
                }
                Op::Load(i) | Op::Store(i) if *i >= function.locals.len() => {
                    return Err(Fault::new("local index outside signature"));
                }
                Op::Call(target) => {
                    if let Some(owner) = &target.owner {
                        check(owner)?;
                    }
                    for ty in &target.parameters {
                        check(ty)?;
                    }
                    resolve(module, target)?;
                }
                Op::New(ty) => {
                    record_fields(module, ty, arity)?;
                }
                Op::SizeOf(ty)
                | Op::AlignOf(ty)
                | Op::Allocate(ty)
                | Op::LoadObject(ty)
                | Op::StoreObject(ty)
                | Op::CopyObject(ty)
                | Op::InitializeObject(ty) => {
                    check(ty)?;
                    if check_type(ty, module).is_ok() {
                        crate::memory::layout(module, ty)?;
                    }
                }
                Op::None(ty)
                | Op::Ok(ty)
                | Op::Err(ty)
                | Op::NullPointer(ty)
                | Op::PointerCast(ty)
                | Op::PointerFromInt(ty) => check(ty)?,
                _ => (),
            }
        }
    }
    for attributes in module
        .types
        .iter()
        .map(|d| &d.custom_attributes)
        .chain(module.functions.iter().map(|f| &f.custom_attributes))
    {
        for attribute in attributes {
            validate_attribute(module, attribute)?;
        }
    }
    if !module.entry.is_empty()
        && !module.functions.iter().any(|f| {
            f.name == module.entry
                && f.parameters.is_empty()
                && !f.is_internal_call()
                && f.pinvoke.is_none()
                && !f.instance
                && f.owner
                    .as_ref()
                    .and_then(|o| module.type_definition(o))
                    .is_none_or(|d| d.generic_parameters.is_empty())
        })
    {
        return Err(Fault::new("parameterless entry function not found"));
    }
    Ok(())
}

pub(crate) fn check_type(ty: &Type, module: &Module) -> Result<(), Fault> {
    check_type_context(ty, module, 0, 0)
}

fn check_type_context(ty: &Type, module: &Module, arity: usize, depth: usize) -> Result<(), Fault> {
    if depth > 32 {
        return Err(Fault::new("type nesting exceeds 32"));
    }
    let nested = |ty: &Type| check_type_context(ty, module, arity, depth + 1);
    match ty {
        Type::TypeParameter(index) if *index as usize >= arity => {
            Err(Fault::new("type parameter outside declaring context"))
        }
        Type::Named(name) => {
            if Type::from_name(name).is_primitive() {
                return Err(Fault::new(
                    "primitive type requires canonical primitive signature encoding",
                ));
            }
            let def = module
                .types
                .iter()
                .find(|t| &t.name == name)
                .ok_or_else(|| Fault::new(format!("unknown type {name}")))?;
            if !def.generic_parameters.is_empty() {
                return Err(Fault::new("generic type requires type arguments"));
            }
            Ok(())
        }
        Type::Constructed {
            definition,
            arguments,
        } => {
            let def = module
                .types
                .iter()
                .find(|t| &t.name == definition)
                .ok_or_else(|| Fault::new(format!("unknown generic type {definition}")))?;
            if def.generic_parameters.is_empty() || def.generic_parameters.len() != arguments.len()
            {
                return Err(Fault::new("generic type argument count mismatch"));
            }
            for argument in arguments {
                nested(argument)?;
            }
            Ok(())
        }
        Type::Option(t) | Type::Ref(t) | Type::Ptr(t) => nested(t),
        Type::Result(t, e) => {
            nested(t)?;
            nested(e)
        }
        _ => Ok(()),
    }
}

struct Frame {
    function: std::rc::Rc<crate::metadata::Function>,
    pc: usize,
    args: Vec<Value>,
    locals: Vec<Option<Value>>,
    stack: Vec<Value>,
    allocations: Vec<crate::memory::Pointer>,
}

impl Frame {
    fn new(function: crate::metadata::Function, args: Vec<Value>) -> Self {
        let local_count = function.locals.len();
        Self {
            function: std::rc::Rc::new(function),
            pc: 0,
            args,
            locals: vec![None; local_count],
            stack: vec![],
            allocations: vec![],
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
    crate::LoadedProgram::with_library(module, library)?.run(limits)
}

/// Execute a trusted module with native imports enabled.
///
/// # Safety
/// Every executed native declaration must match its exported C ABI signature.
/// Native code and library initializers/destructors must uphold pointer validity,
/// allocation lifetimes, and Rust's memory safety requirements. Guest metadata alone
/// cannot establish these guarantees. The caller must trust the code being executed.
pub unsafe fn run_with_native(
    module: &Module,
    library: &Module,
    limits: Limits,
) -> Result<Execution, Fault> {
    if module.entry.is_empty() {
        return Err(Fault::new(
            "cannot execute a library without an entry point",
        ));
    }
    // SAFETY: the caller accepts the same native-code contract as LoadedProgram.
    unsafe { crate::LoadedProgram::with_library(module, library)?.run_with_native(limits) }
}

pub(crate) fn interpret(
    module: &Module,
    limits: Limits,
    mut native_libraries: Option<crate::interop::NativeLibraries>,
) -> Result<Execution, Fault> {
    if limits.frames == 0 {
        return Err(Fault::new("frame limit exceeded"));
    }
    let entry = module
        .functions
        .iter()
        .position(|f| f.name == module.entry && f.parameters.is_empty() && !f.instance)
        .ok_or_else(|| Fault::new("missing entry"))?;
    let mut frames = vec![Frame::new(module.functions[entry].clone(), vec![])];
    let mut heap: Vec<Value> = vec![];
    let mut memory = crate::memory::PointerHeap::default();
    let mut output = vec![];
    for _ in 0..limits.instructions {
        let frame = frames
            .last_mut()
            .ok_or_else(|| Fault::new("missing frame"))?;
        let function = frame.function.clone();
        let pc = frame.pc;
        let op = function.body.get(pc).ok_or_else(|| Fault {
            message: "function fell through without ret".into(),
            function: Some(function.name.clone()),
            instruction: Some(pc),
        })?;
        let access_alignment = match pc.checked_sub(1).and_then(|i| function.body.get(i)) {
            Some(Op::Unaligned(alignment)) => Some(*alignment as usize),
            _ => None,
        };
        frame.pc += 1;
        let context = function.name.clone();
        // Host Result propagates terminal faults; there is no guest exception machinery.
        let step = (|| -> Result<Option<Value>, Fault> {
            let frame = frames
                .last_mut()
                .ok_or_else(|| Fault::new("missing frame"))?;
            match op {
                Op::Unaligned(_) => (),
                Op::Int(n) => frame.stack.push(Value::Int32(*n)),
                Op::Float32 { bits } => frame
                    .stack
                    .push(Value::Double(f32::from_bits(*bits) as f64)),
                Op::Float64 { bits } => frame.stack.push(Value::Double(f64::from_bits(*bits))),
                Op::CheckFinite => {
                    let value = frame.pop()?;
                    match value {
                        Value::Double(n) if n.is_finite() => frame.stack.push(value),
                        Value::Double(_) => {
                            return Err(Fault::new("non-finite floating-point value"));
                        }
                        _ => return Err(Fault::new("ckfinite requires floating-point value")),
                    }
                }
                Op::ConvertFloat32 | Op::ConvertFloat64 | Op::ConvertFloatUnsigned => {
                    let value = frame.pop()?;
                    frame.stack.push(crate::floating::convert(op, value)?);
                }
                Op::Int64(n) => frame.stack.push(Value::Int64(*n)),
                Op::Bool(b) => frame.stack.push(Value::Boolean(*b)),
                Op::String(s) => frame.stack.push(Value::String(s.clone())),
                Op::Void => frame.stack.push(Value::Void),
                Op::Error(s) => frame.stack.push(Value::Error(s.clone())),
                Op::Arg(i) => frame.stack.push(frame.args[*i].clone().on_stack()),
                Op::StoreArg(i) => {
                    let value = frame.pop()?;
                    frame.args[*i] = value.for_storage(&function.argument_types()[*i])?;
                }
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
                Op::BitNot | Op::Negate => {
                    let value = frame.pop()?;
                    frame.stack.push(crate::numeric::unary(op, value)?);
                }
                Op::ShiftLeft | Op::ShiftRight | Op::ShiftRightUnsigned => {
                    let count = frame.pop()?;
                    let value = frame.pop()?;
                    frame.stack.push(crate::numeric::shift(op, value, count)?);
                }
                Op::BitAnd
                | Op::BitOr
                | Op::BitXor
                | Op::Remainder
                | Op::RemainderUnsigned
                | Op::Add
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
                | Op::LessUnsigned
                | Op::Greater
                | Op::GreaterUnsigned => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    frame.stack.push(crate::numeric::binary(op, left, right)?);
                }
                Op::CheckedInt8
                | Op::CheckedUInt8
                | Op::CheckedInt16
                | Op::CheckedUInt16
                | Op::CheckedInt32
                | Op::CheckedUInt32
                | Op::CheckedInt64
                | Op::CheckedUInt64
                | Op::CheckedNativeInt
                | Op::CheckedNativeUInt
                | Op::CheckedInt8Unsigned
                | Op::CheckedUInt8Unsigned
                | Op::CheckedInt16Unsigned
                | Op::CheckedUInt16Unsigned
                | Op::CheckedInt32Unsigned
                | Op::CheckedUInt32Unsigned
                | Op::CheckedInt64Unsigned
                | Op::CheckedUInt64Unsigned
                | Op::CheckedNativeIntUnsigned
                | Op::CheckedNativeUIntUnsigned => {
                    let value = frame.pop()?;
                    frame.stack.push(crate::checked::convert(op, value)?);
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
                Op::BranchTrue(i) | Op::BranchFalse(i) => {
                    let condition = match frame.pop()? {
                        Value::Boolean(value) => value,
                        Value::Int32(value) => value != 0,
                        Value::Int64(value) => value != 0,
                        Value::IntPtr(value) => value != 0,
                        Value::UIntPtr(value) => value != 0,
                        // A condition tests the address, not pointee validity or lifetime.
                        Value::Pointer(pointer) => pointer.address != 0,
                        // The prototype Ref arena has no null reference representation.
                        Value::Reference { .. } => true,
                        _ => {
                            return Err(Fault::new(
                                "conditional branch requires Boolean, integer, pointer, or Ref",
                            ));
                        }
                    };
                    if condition == matches!(op, Op::BranchTrue(_)) {
                        frame.pc = *i;
                    }
                }
                Op::BranchEqual(i)
                | Op::BranchNotEqual(i)
                | Op::BranchGreater(i)
                | Op::BranchGreaterUnsigned(i)
                | Op::BranchLess(i)
                | Op::BranchLessUnsigned(i)
                | Op::BranchGreaterEqual(i)
                | Op::BranchGreaterEqualUnsigned(i)
                | Op::BranchLessEqual(i)
                | Op::BranchLessEqualUnsigned(i) => {
                    let right = frame.pop()?;
                    let left = frame.pop()?;
                    if crate::numeric::branch_condition(op, left, right)? {
                        frame.pc = *i;
                    }
                }
                Op::Switch(targets) => {
                    let Value::Int32(index) = frame.pop()? else {
                        return Err(Fault::new("switch requires Int32"));
                    };
                    if let Some(target) = targets.get(index as u32 as usize) {
                        frame.pc = *target;
                    }
                }
                Op::Call(target) => {
                    let callee = resolve(module, target)?;
                    callee.map_types(|ty| {
                        check_type(ty, module)?;
                        Ok(ty.clone())
                    })?;
                    let args = frame.args(&callee.argument_types())?;
                    if callee.pinvoke.is_some() {
                        let libraries = native_libraries.as_mut().ok_or_else(|| {
                            Fault::new("native imports require trusted run_with_native execution")
                        })?;
                        // SAFETY: a native library session is only supplied by run_with_native,
                        // whose caller accepts the native ABI and memory safety contract.
                        let value = unsafe { libraries.invoke(&callee, args, &memory)? };
                        expect(&value, &callee.returns)?;
                        frame.stack.push(value.on_stack());
                    } else if callee.is_internal_call() {
                        let value = crate::native::bind(&callee)?.invoke(args, &mut output)?;
                        expect(&value, &callee.returns)?;
                        frame.stack.push(value);
                    } else {
                        if frames.len() >= limits.frames {
                            return Err(Fault::new("frame limit exceeded"));
                        }
                        frames.push(Frame::new(callee, args));
                    }
                }
                Op::Return => {
                    let value = frame.pop()?;
                    let value = value.for_storage(&function.returns)?;
                    if !frame.stack.is_empty() {
                        return Err(Fault::new("ret requires exactly one value"));
                    }
                    for pointer in &frame.allocations {
                        memory.release_local(pointer)?;
                    }
                    frames.pop();
                    if let Some(caller) = frames.last_mut() {
                        caller.stack.push(value.on_stack());
                    } else {
                        return Ok(Some(value));
                    }
                }
                Op::New(ty) => {
                    let definitions = module.instantiated_fields(ty)?;
                    let types: Vec<_> = definitions.iter().map(|f| f.ty.clone()).collect();
                    let fields = frame.args(&types)?;
                    frame.stack.push(Value::Object {
                        ty: ty.clone(),
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
                    let Value::Object { ty, mut fields } = frame.pop()? else {
                        return Err(Fault::new("stfld requires object value"));
                    };
                    let field = fields
                        .get_mut(*i)
                        .ok_or_else(|| Fault::new("field index out of range"))?;
                    *field = value.for_storage(&field.ty())?;
                    frame.stack.push(Value::Object { ty, fields });
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
                Op::AllocateLocal => {
                    let size = match frame.pop()? {
                        Value::Int32(n) => usize::try_from(n).ok(),
                        Value::IntPtr(n) => usize::try_from(n).ok(),
                        Value::UIntPtr(n) => Some(n),
                        _ => return Err(Fault::new("localloc size requires integer")),
                    }
                    .ok_or_else(|| Fault::new("negative localloc size"))?;
                    if !frame.stack.is_empty() {
                        return Err(Fault::new(
                            "localloc requires only its size on the evaluation stack",
                        ));
                    }
                    let pointer = memory.allocate_local(
                        size,
                        limits.pointer_bytes,
                        limits.pointer_allocations,
                    )?;
                    frame.allocations.push(pointer.clone());
                    frame.stack.push(Value::Pointer(pointer));
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
                    if !matches!(pointer.target, Type::Named(_) | Type::Constructed { .. }) {
                        return Err(Fault::new("ldflda requires pointer to record"));
                    }
                    let layout = crate::memory::layout(module, &pointer.target)?;
                    frame
                        .stack
                        .push(Value::Pointer(memory.field(&pointer, &layout, *index)?));
                }
                Op::CopyObject(ty) | Op::InitializeObject(ty) => {
                    let source = if matches!(op, Op::CopyObject(_)) {
                        Some(frame.pointer()?)
                    } else {
                        None
                    };
                    let destination = frame.pointer()?;
                    if destination.target != *ty || source.as_ref().is_some_and(|p| p.target != *ty)
                    {
                        return Err(Fault::new("memory operation pointer type mismatch"));
                    }
                    let layout = crate::memory::layout(module, ty)?;
                    if let Some(source) = source {
                        let value = memory.read(&source, &layout)?;
                        memory.write(&destination, &layout, &value)?;
                    } else {
                        memory.fill(&destination, &layout, 0)?;
                    }
                }
                Op::CopyBlock | Op::InitializeBlock => {
                    let size = match frame.pop()? {
                        Value::Int32(n) => usize::try_from(n).ok(),
                        Value::IntPtr(n) => usize::try_from(n).ok(),
                        Value::UIntPtr(n) => Some(n),
                        _ => return Err(Fault::new("block size requires integer")),
                    }
                    .ok_or_else(|| Fault::new("negative block size"))?;
                    let layout = crate::memory::Layout {
                        size,
                        alignment: 1,
                        fields: vec![],
                    };
                    if matches!(op, Op::CopyBlock) {
                        let source = frame.pointer()?;
                        let destination = frame.pointer()?;
                        memory.copy_block(&destination, &source, &layout)?;
                    } else {
                        let Value::Int32(value) = frame.pop()? else {
                            return Err(Fault::new("initblk fill requires Int32"));
                        };
                        let destination = frame.pointer()?;
                        memory.fill(&destination, &layout, value as u8)?;
                    }
                }
                Op::LoadObject(_)
                | Op::LoadIndirectInt32
                | Op::LoadIndirectInt8
                | Op::LoadIndirectUInt8
                | Op::LoadIndirectInt16
                | Op::LoadIndirectUInt16
                | Op::LoadIndirectUInt32
                | Op::LoadIndirectInt64
                | Op::LoadIndirectNative
                | Op::LoadIndirectFloat32
                | Op::LoadIndirectFloat64 => {
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
                    let mut layout = crate::memory::layout(module, &ty)?;
                    if let Some(alignment) = access_alignment {
                        layout.alignment = layout.alignment.min(alignment);
                    }
                    frame.stack.push(memory.read(&pointer, &layout)?.on_stack());
                }
                Op::StoreObject(_)
                | Op::StoreIndirectInt32
                | Op::StoreIndirectInt8
                | Op::StoreIndirectInt16
                | Op::StoreIndirectInt64
                | Op::StoreIndirectNative
                | Op::StoreIndirectFloat32
                | Op::StoreIndirectFloat64 => {
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
                    let mut layout = crate::memory::layout(module, ty)?;
                    if let Some(alignment) = access_alignment {
                        layout.alignment = layout.alignment.min(alignment);
                    }
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
                    native_libraries,
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

fn validate_attribute(
    module: &Module,
    attribute: &crate::metadata::CustomAttribute,
) -> Result<(), Fault> {
    let target = &attribute.constructor;
    let owner = target
        .owner
        .as_ref()
        .ok_or_else(|| Fault::new("attribute requires a constructor owner"))?;
    // Attribute metadata is closed even when attached to an open generic definition.
    record_fields(module, owner, 0)?;
    let name = match owner {
        Type::Named(name) => name,
        Type::Constructed { definition, .. } => definition,
        _ => return Err(Fault::new("attribute owner must be a record type")),
    };
    if !target.instance || !target.parameters.is_empty() || target.name != format!("{name}..ctor") {
        return Err(Fault::new(
            "marker attribute requires instance Type::.ctor()",
        ));
    }
    let constructor = resolve(module, target)?;
    if constructor.returns != Type::Void {
        return Err(Fault::new("attribute constructor must return Void"));
    }
    Ok(())
}
