//! Opt-in typed-stack, control-flow, and definite-local-initialization analysis.
//! This is not a memory-safety verifier.
use crate::{
    Fault, Module,
    metadata::{Function, Instruction as Op, Type},
};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct FunctionVerification {
    pub definition: Option<crate::metadata::MemberId>,
    /// Index in the linked module (application functions precede library functions).
    pub function_index: usize,
    pub name: String,
    pub owner: Option<Type>,
    pub parameters: Vec<Type>,
    pub instance: bool,
    pub maximum_stack: usize,
    pub reachable_instructions: usize,
}

#[derive(Debug, Clone)]
pub struct Verification {
    /// IL functions only; native imports/InternalCall declarations have no IL body.
    pub functions: Vec<FunctionVerification>,
}

pub fn verify(module: &Module) -> Result<Verification, Fault> {
    crate::LoadedProgram::new(module)?.verify()
}

pub fn verify_with_library(module: &Module, library: &Module) -> Result<Verification, Fault> {
    crate::LoadedProgram::with_library(module, library)?.verify()
}

pub(crate) fn analyze(module: &Module) -> Result<Verification, Fault> {
    let mut constructors = std::collections::HashSet::new();
    for function in &module.functions {
        for op in &function.body {
            if let Op::Construct(target) = op {
                constructors.insert(crate::vm::resolve_constructor(module, target)?.definition);
            }
        }
    }
    let mut functions = Vec::new();
    for (index, function) in module.functions.iter().enumerate() {
        if function.is_internal_call()
            || function.pinvoke.is_some()
            || crate::interfaces::is_contract(module, function)
        {
            continue;
        }
        let mut report = analyze_function(module, index, function, false)?;
        if constructors.contains(&function.definition) {
            let construction = analyze_function(module, index, function, true)?;
            report.maximum_stack = report.maximum_stack.max(construction.maximum_stack);
        }
        functions.push(report);
    }
    Ok(Verification { functions })
}

#[derive(Clone)]
struct State {
    stack: Vec<StackType>,
    initialized: Vec<bool>,
    receiver_initialized: bool,
}

fn analyze_function(
    module: &Module,
    index: usize,
    function: &Function,
    constructing: bool,
) -> Result<FunctionVerification, Fault> {
    let fault = |pc, message: &str| Fault {
        message: format!("verification: {message}"),
        function: Some(function.name.clone()),
        instruction: Some(pc),
        stack_trace: None,
    };
    let arity = function
        .owner
        .as_ref()
        .and_then(|t| module.type_definition(t))
        .map_or(0, |d| d.generic_parameters.len());
    let mut states: Vec<Option<State>> = vec![None; function.body.len()];
    states[0] = Some(State {
        stack: vec![],
        initialized: vec![false; function.locals.len()],
        receiver_initialized: !constructing
            || function
                .owner
                .as_ref()
                .and_then(|owner| crate::vm::record_fields(module, owner, arity).ok())
                .is_some_and(|fields| fields.is_empty()),
    });
    let mut queue = VecDeque::from([0]);
    let mut maximum_stack = 0;
    while let Some(pc) = queue.pop_front() {
        let mut state = states[pc].clone().expect("queued state exists");
        let op = &function.body[pc];
        let (pops, pushes) = effect(module, op, arity).map_err(|mut e| {
            e.function = Some(function.name.clone());
            e.instruction = Some(pc);
            e
        })?;
        if state.stack.len() < pops {
            return Err(fault(pc, "evaluation stack underflow"));
        }
        if matches!(op, Op::Return) && state.stack.len() != 1 {
            return Err(fault(pc, "ret requires exactly one value"));
        }
        if matches!(op, Op::Load(slot) if !state.initialized[*slot]) {
            return Err(fault(pc, "local is not initialized on every incoming path"));
        }
        if constructing && matches!(op, Op::Arg(0) | Op::Return) && !state.receiver_initialized {
            return Err(fault(
                pc,
                "constructor receiver is not initialized on every incoming path",
            ));
        }
        if matches!(op, Op::StoreArg(0)) {
            state.receiver_initialized = true;
        }
        if let Op::ResetLocal(slot) = op {
            state.initialized[*slot] = false;
        }
        if let Op::Store(slot) = op {
            state.initialized[*slot] = true;
        }
        let inputs = state.stack.split_off(state.stack.len() - pops);
        match (op, inputs.first()) {
            (
                Op::StoreObject(_) | Op::InitializeObject(_),
                Some(StackType::Slot {
                    local: Some(index), ..
                }),
            ) => state.initialized[*index] = true,
            (
                Op::ReferenceType
                | Op::LoadObject(_)
                | Op::FieldAddress(_)
                | Op::ArrayAddress(_)
                | Op::ArrayElement(_)
                | Op::StoreArrayElement(_)
                | Op::ArrayLength
                | Op::BorrowInterface(_)
                | Op::Store(_)
                | Op::Return,
                Some(StackType::Slot {
                    local: Some(index), ..
                }),
            ) if !state.initialized[*index] => {
                return Err(fault(pc, "referenced local is uninitialized"));
            }
            _ => (),
        }
        if constructing
            && !state.receiver_initialized
            && matches!(
                op,
                Op::LoadObject(_) | Op::FieldAddress(_) | Op::BorrowInterface(_) | Op::Store(_)
            )
            && matches!(
                inputs.first(),
                Some(StackType::Slot {
                    argument: Some(0),
                    ..
                })
            )
        {
            return Err(fault(
                pc,
                "constructor receiver is not initialized on every incoming path",
            ));
        }
        if constructing
            && matches!(op, Op::StoreObject(_) | Op::InitializeObject(_))
            && matches!(
                inputs.first(),
                Some(StackType::Slot {
                    argument: Some(0),
                    ..
                })
            )
        {
            state.receiver_initialized = true;
        }
        if matches!(op, Op::ReferenceEqual) {
            for input in &inputs {
                if let StackType::Slot {
                    local: Some(index), ..
                } = input
                {
                    if !state.initialized[*index] {
                        return Err(fault(pc, "reference equality requires initialized targets"));
                    }
                }
            }
        }
        if matches!(inputs.first(), Some(StackType::Readonly(_)))
            && matches!(
                op,
                Op::StoreObject(_)
                    | Op::InitializeObject(_)
                    | Op::StoreArrayElement(_)
                    | Op::StoreIndirectInt8
                    | Op::StoreIndirectInt16
                    | Op::StoreIndirectInt32
                    | Op::StoreIndirectInt64
                    | Op::StoreIndirectNative
                    | Op::StoreIndirectFloat32
                    | Op::StoreIndirectFloat64
            )
        {
            return Err(fault(pc, "cannot write through readonly reference"));
        }
        let mut conditional_outputs = vec![];
        if let Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) = op {
            let callee = crate::vm::resolve(module, target).map_err(|e| fault(pc, &e.message))?;
            let offset = inputs.len().saturating_sub(callee.parameters.len());
            for (argument, input) in inputs.iter().enumerate() {
                if matches!(input, StackType::Readonly(_))
                    && !(if argument < offset {
                        callee.receiver_readonly
                    } else {
                        callee.readonly_parameters.contains(&(argument - offset))
                            || matches!(
                                callee.parameters.get(argument - offset),
                                Some(Type::ReadOnlyByRef(_))
                            )
                    })
                {
                    return Err(fault(
                        pc,
                        "readonly reference cannot satisfy writable parameter or receiver",
                    ));
                }
                if let StackType::Slot {
                    local: Some(index), ..
                } = input
                {
                    if argument >= offset
                        && (callee.out_parameters.contains(&(argument - offset))
                            || callee.out_when_true.contains(&(argument - offset)))
                    {
                        continue;
                    }
                    if !state.initialized[*index] {
                        return Err(fault(pc, "reference argument requires initialized local"));
                    }
                }
            }
            for index in &callee.out_when_true {
                if let Some(StackType::Slot {
                    local: Some(slot), ..
                }) = inputs.get(index + offset)
                {
                    conditional_outputs.push(*slot);
                }
            }
            // Check all input preconditions before making any outputs initialized.
            for index in &callee.out_parameters {
                if let Some(StackType::Slot {
                    local: Some(slot), ..
                }) = inputs.get(index + offset)
                {
                    state.initialized[*slot] = true;
                }
            }
        }
        let mut outputs =
            typed_effect(module, function, op, &inputs, arity, state.stack.is_empty())
                .map_err(|e| fault(pc, &e.message))?;
        if !conditional_outputs.is_empty() {
            conditional_outputs.sort_unstable();
            conditional_outputs.dedup();
            outputs = vec![StackType::ConditionalOutput(conditional_outputs)];
        }
        debug_assert_eq!(outputs.len(), pushes);
        state.stack.extend(outputs);
        maximum_stack = maximum_stack.max(state.stack.len());
        let mut successors = Vec::new();
        match op {
            Op::Return | Op::Fault(_) => (),
            Op::Branch(target) => successors.push(*target),
            Op::BranchTrue(target)
            | Op::BranchFalse(target)
            | Op::BranchEqual(target)
            | Op::BranchNotEqual(target)
            | Op::BranchGreater(target)
            | Op::BranchGreaterUnsigned(target)
            | Op::BranchLess(target)
            | Op::BranchLessUnsigned(target)
            | Op::BranchGreaterEqual(target)
            | Op::BranchGreaterEqualUnsigned(target)
            | Op::BranchLessEqual(target)
            | Op::BranchLessEqualUnsigned(target) => {
                successors.extend([*target, pc + 1]);
            }
            Op::Switch(targets) => {
                successors.extend(targets);
                successors.push(pc + 1);
            }
            _ => successors.push(pc + 1),
        }
        for (edge, target) in successors.into_iter().enumerate() {
            let mut state = state.clone();
            match (op, edge, inputs.first()) {
                (Op::BranchTrue(_), 0, Some(StackType::ConditionalOutput(slots)))
                | (Op::BranchFalse(_), 1, Some(StackType::ConditionalOutput(slots))) => {
                    for slot in slots {
                        state.initialized[*slot] = true;
                    }
                }
                _ => (),
            }
            if target >= function.body.len() {
                return Err(fault(pc, "reachable fallthrough past end of function"));
            }
            if let Some(previous) = &mut states[target] {
                if previous.stack.len() != state.stack.len() {
                    return Err(fault(
                        target,
                        "incompatible stack heights at control-flow join",
                    ));
                }
                let mut changed = false;
                for (old, incoming) in previous.stack.iter_mut().zip(&state.stack) {
                    if old == incoming {
                        continue;
                    }
                    match (&*old, incoming) {
                        (StackType::Exact(Type::ByRef(a)), StackType::Readonly(Type::ByRef(b)))
                            if a == b =>
                        {
                            *old = incoming.clone();
                            changed = true;
                        }
                        (StackType::Readonly(Type::ByRef(a)), StackType::Exact(Type::ByRef(b)))
                            if a == b => {}
                        _ => {
                            return Err(fault(
                                target,
                                "incompatible stack types at control-flow join",
                            ));
                        }
                    }
                }
                if previous.receiver_initialized && !state.receiver_initialized {
                    previous.receiver_initialized = false;
                    changed = true;
                }
                for (old, incoming) in previous.initialized.iter_mut().zip(&state.initialized) {
                    if *old && !incoming {
                        *old = false;
                        changed = true;
                    }
                }
                if changed {
                    queue.push_back(target);
                }
            } else {
                states[target] = Some(state.clone());
                queue.push_back(target);
            }
        }
    }
    Ok(FunctionVerification {
        definition: function.definition.clone(),
        function_index: index,
        name: function.name.clone(),
        owner: function.owner.clone(),
        parameters: function.parameters.clone(),
        instance: function.instance,
        maximum_stack,
        reachable_instructions: states.iter().filter(|s| s.is_some()).count(),
    })
}

// Exhaustive: adding an instruction requires explicitly specifying its stack effect.
fn effect(module: &Module, op: &Op, arity: usize) -> Result<(usize, usize), Fault> {
    use Op::*;
    Result::Ok(match op {
        Unaligned(_) | Branch(_) | Fault(_) | ResetLocal(_) => (0, 0),
        Int(_)
        | Int64(_)
        | Float32 { .. }
        | Float64 { .. }
        | Bool(_)
        | String(_)
        | Void
        | Arg(_)
        | LocalAddress(_)
        | ArgumentAddress(_)
        | Load(_)
        | LoadTypeToken(_)
        | SizeOf(_)
        | AlignOf(_)
        | NullPointer(_)
        | Error(_) => (0, 1),
        Pop | Store(_) | StoreArg(_) | Return | BranchTrue(_) | BranchFalse(_) | Switch(_)
        | InitializeObject(_) => (1, 0),
        Dup => (1, 2),
        AllocateArray(_) | NewArray(_) | ArrayLength | ReferenceType => (1, 1),
        CreateArray(_) | ArrayElement(_) | ArrayAddress(_) => (2, 1),
        StoreArrayElement(_) => (3, 0),
        New(ty) => (crate::vm::record_fields(module, ty, arity)?.len(), 1),
        Call(target) | CallVirtual(target) => {
            (target.parameters.len() + usize::from(target.instance), 1)
        }
        Construct(target) => (target.parameters.len(), 1),
        BorrowInterface(_) | PackValue(_) | IsValue(_) | UnpackValue(_) => (1, 1),
        ReferenceEqual | SetField(_) | PointerAdd | BitAnd | BitOr | BitXor | ShiftLeft
        | ShiftRight | ShiftRightUnsigned | Remainder | RemainderUnsigned | Add | Sub | Mul
        | AddChecked | SubChecked | MulChecked | Divide | AddCheckedUnsigned
        | SubCheckedUnsigned | MulCheckedUnsigned | DivideUnsigned | Equal | Greater
        | GreaterUnsigned | Less | LessUnsigned => (2, 1),
        BranchEqual(_)
        | BranchNotEqual(_)
        | BranchGreater(_)
        | BranchGreaterUnsigned(_)
        | BranchLess(_)
        | BranchLessUnsigned(_)
        | BranchGreaterEqual(_)
        | BranchGreaterEqualUnsigned(_)
        | BranchLessEqual(_)
        | BranchLessEqualUnsigned(_)
        | CopyObject(_)
        | StoreObject(_)
        | StoreIndirectInt8
        | StoreIndirectInt16
        | StoreIndirectInt32
        | StoreIndirectInt64
        | StoreIndirectNative
        | StoreIndirectFloat32
        | StoreIndirectFloat64 => (2, 0),
        CopyBlock | InitializeBlock => (3, 0),
        CheckedInt8
        | CheckedUInt8
        | CheckedInt16
        | CheckedUInt16
        | CheckedInt32
        | CheckedUInt32
        | CheckedInt64
        | CheckedUInt64
        | CheckedNativeInt
        | CheckedNativeUInt
        | CheckedInt8Unsigned
        | CheckedUInt8Unsigned
        | CheckedInt16Unsigned
        | CheckedUInt16Unsigned
        | CheckedInt32Unsigned
        | CheckedUInt32Unsigned
        | CheckedInt64Unsigned
        | CheckedUInt64Unsigned
        | CheckedNativeIntUnsigned
        | CheckedNativeUIntUnsigned
        | ConvertInt8
        | ConvertUInt8
        | ConvertInt16
        | ConvertUInt16
        | ConvertUInt32
        | ConvertInt64
        | ConvertUInt64
        | ConvertNativeInt
        | ConvertNativeUInt
        | ConvertInt32
        | ConvertFloat32
        | ConvertFloat64
        | ConvertFloatUnsigned
        | CheckFinite
        | BitNot
        | Negate
        | PointerFromInt(_)
        | LoadIndirectInt8
        | LoadIndirectUInt8
        | LoadIndirectInt16
        | LoadIndirectUInt16
        | LoadIndirectUInt32
        | LoadIndirectInt32
        | LoadIndirectInt64
        | LoadIndirectNative
        | LoadIndirectFloat32
        | LoadIndirectFloat64
        | Field(_)
        | FieldAddress(_)
        | Allocate(_)
        | AllocateLocal
        | Free
        | PointerCast(_)
        | LoadObject(_)
        | HeapNew => (1, 1),
    })
}

// A loaded open parameter may normalize to a different runtime stack type (Byte
// becomes Int32, for example). Keep it distinct from a raw stored !n value.
#[derive(Debug, Clone, PartialEq, Eq)]
enum StackType {
    ConditionalOutput(Vec<usize>),
    Slot {
        ty: Type,
        local: Option<usize>,
        argument: Option<usize>,
    },
    Exact(Type),
    Readonly(Type),
    NormalizedParameter(u16),
}

fn loaded(ty: &Type) -> StackType {
    use Type::*;
    match ty {
        TypeParameter(index) => StackType::NormalizedParameter(*index),
        SByte | Byte | Int16 | UInt16 | Char | UInt32 => StackType::Exact(Int32),
        UInt64 => StackType::Exact(Int64),
        Single => StackType::Exact(Double),
        ReadOnlyByRef(target) => StackType::Readonly(ByRef(target.clone())),
        other => StackType::Exact(other.clone()),
    }
}

fn stored(value: &StackType, target: &Type) -> Result<(), Fault> {
    if matches!(value, StackType::Readonly(_)) && matches!(target, Type::ByRef(_)) {
        return Err(Fault::new(
            "readonly reference cannot satisfy writable storage contract",
        ));
    }
    if let Type::ReadOnlyByRef(inner) = target {
        if exact(value).is_ok_and(|ty| ty == &Type::ByRef(inner.clone())) {
            return Ok(());
        }
    }
    if exact(value).is_ok_and(|ty| ty == target) || *value == loaded(target) {
        Ok(())
    } else {
        Err(Fault::new(format!(
            "expected storage {target:?}, got {value:?}"
        )))
    }
}

fn exact(value: &StackType) -> Result<&Type, Fault> {
    match value {
        StackType::Exact(ty) | StackType::Readonly(ty) | StackType::Slot { ty, .. } => Ok(ty),
        StackType::ConditionalOutput(_) => Ok(&Type::Boolean),
        _ => Err(Fault::new(
            "operation requires a concrete stack type; open parameter normalization is unresolved",
        )),
    }
}

fn address(value: &StackType) -> Result<&Type, Fault> {
    match exact(value)? {
        Type::Ptr(t) | Type::ByRef(t) => Ok(t),
        _ => Err(Fault::new("expected pointer or managed slot reference")),
    }
}

fn pointer(value: &StackType) -> Result<&Type, Fault> {
    match exact(value)? {
        Type::Ptr(ty) => Ok(ty),
        _ => Err(Fault::new("expected native pointer")),
    }
}

fn integer(ty: &Type) -> bool {
    matches!(ty, Type::Int32 | Type::Int64 | Type::IntPtr | Type::UIntPtr)
}
fn count(ty: &Type) -> bool {
    matches!(ty, Type::Int32 | Type::IntPtr | Type::UIntPtr)
}
fn numeric(ty: &Type) -> bool {
    integer(ty) || *ty == Type::Double
}
fn require(condition: bool, message: &str) -> Result<(), Fault> {
    if condition {
        Ok(())
    } else {
        Err(Fault::new(message))
    }
}

fn typed_effect(
    module: &Module,
    function: &Function,
    op: &Op,
    values: &[StackType],
    arity: usize,
    rest_empty: bool,
) -> Result<Vec<StackType>, Fault> {
    use Op::*;
    use StackType::Exact as E;
    use Type as T;
    let one = |ty| Result::Ok(vec![E(ty)]);
    let field = |owner: &Type, index: usize| -> Result<Type, crate::Fault> {
        crate::access::check_field(module, function, owner, index)?;
        crate::vm::record_fields(module, owner, arity)?
            .get(index)
            .map(|f| f.ty.clone())
            .ok_or_else(|| crate::Fault::new("field index out of range"))
    };
    match op {
        AllocateArray(ty) | NewArray(ty) | CreateArray(ty) => {
            require(
                count(exact(&values[0])?),
                "array length requires Int32 or native integer",
            )?;
            if matches!(op, CreateArray(_)) {
                stored(&values[1], ty)?;
            }
            let array = T::Array(Box::new(ty.clone()));
            one(if matches!(op, NewArray(_) | AllocateArray(_)) {
                T::ByRef(Box::new(array))
            } else {
                array
            })
        }
        ArrayLength | ArrayElement(_) | StoreArrayElement(_) | ArrayAddress(_) => {
            let owner = exact(&values[0])?;
            let target = if let T::ByRef(t) = owner {
                t.as_ref()
            } else {
                owner
            };
            let T::Array(element) = target else {
                return Err(crate::Fault::new("array operation requires array"));
            };
            if matches!(op, ArrayLength) {
                return one(T::UIntPtr);
            }
            require(
                count(exact(&values[1])?),
                "array index requires Int32 or native integer",
            )?;
            let operand = match op {
                ArrayElement(t) | StoreArrayElement(t) | ArrayAddress(t) => t,
                _ => unreachable!(),
            };
            require(element.as_ref() == operand, "array element type mismatch")?;
            if matches!(op, ArrayElement(_)) {
                return Ok(vec![loaded(element)]);
            }
            require(
                matches!(owner, T::ByRef(_)),
                "array mutation/address requires managed array reference",
            )?;
            if matches!(op, StoreArrayElement(_)) {
                stored(&values[2], element)?;
                return Ok(vec![]);
            }
            Ok(vec![match &values[0] {
                StackType::Slot {
                    local, argument, ..
                } => StackType::Slot {
                    ty: T::ByRef(element.clone()),
                    local: *local,
                    argument: *argument,
                },
                StackType::Readonly(_) => StackType::Readonly(T::ByRef(element.clone())),
                _ => E(T::ByRef(element.clone())),
            }])
        }
        Unaligned(_) | Branch(_) | Fault(_) | Pop | ResetLocal(_) => Result::Ok(vec![]),
        Int(_) | SizeOf(_) | AlignOf(_) => one(T::Int32),
        Int64(_) => one(T::Int64),
        Float32 { .. } | Float64 { .. } => one(T::Double),
        Bool(_) => one(T::Boolean),
        String(_) => one(T::String),
        Error(_) => one(T::Error),
        Void => one(T::Void),
        LocalAddress(index) => Result::Ok(vec![StackType::Slot {
            ty: T::ByRef(Box::new(function.locals[*index].clone())),
            local: Some(*index),
            argument: None,
        }]),
        ArgumentAddress(index) => Result::Ok(vec![StackType::Slot {
            ty: T::ByRef(Box::new(function.argument_types()[*index].clone())),
            local: None,
            argument: Some(*index),
        }]),
        Arg(index) => Ok(vec![loaded(&function.argument_types()[*index])]),
        Load(index) => Result::Ok(vec![loaded(&function.locals[*index])]),
        Store(index) => {
            stored(&values[0], &function.locals[*index])?;
            Result::Ok(vec![])
        }
        StoreArg(index) => {
            stored(&values[0], &function.argument_types()[*index])?;
            Result::Ok(vec![])
        }
        Return => {
            require(
                !matches!(&values[0], StackType::Slot { .. }),
                "cannot return a managed reference to the current frame",
            )?;
            stored(&values[0], &function.returns)?;
            Result::Ok(vec![])
        }
        Dup => Result::Ok(vec![values[0].clone(), values[0].clone()]),
        BorrowInterface(interface) => match exact(&values[0])? {
            T::ByRef(concrete) => {
                crate::interfaces::ensure_implementation(module, concrete, interface)?;
                if matches!(values[0], StackType::Readonly(_)) {
                    Ok(vec![StackType::Readonly(T::ByRef(Box::new(
                        interface.clone(),
                    )))])
                } else {
                    one(T::ByRef(Box::new(interface.clone())))
                }
            }
            T::Ptr(concrete) | T::InterfaceRef(concrete) => {
                crate::interfaces::ensure_implementation(module, concrete, interface)?;
                one(T::InterfaceRef(Box::new(interface.clone())))
            }
            _ => Err(crate::Fault::new(
                "interface.borrow requires a typed pointer or managed slot reference",
            )),
        },
        CallVirtual(target) => {
            let callee = crate::vm::resolve(module, target)?;
            let interface = callee
                .owner
                .clone()
                .ok_or_else(|| crate::Fault::new("interface call requires owner"))?;
            match exact(&values[0])? {
                T::ByRef(actual) if **actual == interface => (),
                T::InterfaceRef(actual) if **actual == interface && !callee.receiver_byref => (),
                _ => {
                    return Err(crate::Fault::new(
                        "interface view or receiver mode mismatch",
                    ));
                }
            }
            for (value, ty) in values[1..].iter().zip(&callee.argument_types()[1..]) {
                stored(value, ty)?;
            }
            Result::Ok(vec![loaded(&callee.returns)])
        }
        Call(target) => {
            let callee = crate::vm::resolve(module, target)?;
            for (value, ty) in values.iter().zip(callee.argument_types()) {
                stored(value, &ty)?;
            }
            Result::Ok(vec![loaded(&callee.returns)])
        }
        LoadTypeToken(_) => one(Type::RuntimeTypeHandle),
        ReferenceEqual => {
            for value in values {
                require(
                    matches!(exact(value)?, Type::ByRef(_)),
                    "ref.eq requires managed references",
                )?;
            }
            one(Type::Boolean)
        }
        ReferenceType => {
            require(
                matches!(exact(&values[0])?, Type::ByRef(_)),
                "ref.type requires a managed reference",
            )?;
            one(Type::RuntimeTypeHandle)
        }
        PackValue(ty) => {
            stored(&values[0], ty)?;
            one(Type::Value)
        }
        IsValue(_) => {
            stored(&values[0], &Type::Value)?;
            one(Type::Boolean)
        }
        UnpackValue(ty) => {
            stored(&values[0], &Type::Value)?;
            Result::Ok(vec![loaded(ty)])
        }
        Construct(target) => {
            let callee = crate::vm::resolve_constructor(module, target)?;
            for (value, ty) in values.iter().zip(&callee.parameters) {
                stored(value, ty)?;
            }
            one(callee
                .owner
                .ok_or_else(|| crate::Fault::new("missing constructor owner"))?)
        }
        New(ty) => {
            for (value, field) in values
                .iter()
                .zip(crate::vm::record_fields(module, ty, arity)?)
            {
                stored(value, &field.ty)?;
            }
            one(ty.clone())
        }
        Field(index) => Result::Ok(vec![loaded(&field(exact(&values[0])?, *index)?)]),
        SetField(index) => {
            stored(&values[1], &field(exact(&values[0])?, *index)?)?;
            Result::Ok(vec![values[0].clone()])
        }
        FieldAddress(index) => match exact(&values[0])? {
            T::ByRef(owner) => {
                let ty = T::ByRef(Box::new(field(owner, *index)?));
                if matches!(values[0], StackType::Readonly(_)) {
                    Ok(vec![StackType::Readonly(ty)])
                } else if let StackType::Slot { local, .. } = &values[0] {
                    Ok(vec![StackType::Slot {
                        ty,
                        local: *local,
                        argument: None,
                    }])
                } else {
                    one(ty)
                }
            }
            _ => one(T::Ptr(Box::new(field(pointer(&values[0])?, *index)?))),
        },
        NullPointer(ty) => one(T::Ptr(Box::new(ty.clone()))),
        PointerCast(ty) => {
            pointer(&values[0])?;
            one(T::Ptr(Box::new(ty.clone())))
        }
        PointerFromInt(ty) => {
            require(
                matches!(exact(&values[0])?, T::IntPtr | T::UIntPtr),
                "ptr.fromint requires native integer",
            )?;
            one(T::Ptr(Box::new(ty.clone())))
        }
        PointerAdd => {
            pointer(&values[0])?;
            require(
                matches!(exact(&values[1])?, T::Int32 | T::IntPtr),
                "pointer offset requires Int32 or IntPtr",
            )?;
            Result::Ok(vec![values[0].clone()])
        }
        Allocate(ty) => {
            require(
                count(exact(&values[0])?),
                "allocation count requires Int32 or native integer",
            )?;
            one(T::Ptr(Box::new(ty.clone())))
        }
        AllocateLocal => {
            require(
                rest_empty,
                "localloc requires only its size on the evaluation stack",
            )?;
            require(
                count(exact(&values[0])?),
                "localloc requires Int32 or native integer",
            )?;
            one(T::Ptr(Box::new(T::Byte)))
        }
        Free => {
            pointer(&values[0])?;
            one(T::Void)
        }
        LoadObject(ty) => {
            require(
                address(&values[0])? == ty,
                "memory load pointer type mismatch",
            )?;
            Result::Ok(vec![loaded(ty)])
        }
        StoreObject(ty) => {
            require(
                address(&values[0])? == ty,
                "memory store pointer type mismatch",
            )?;
            stored(&values[1], ty)?;
            Result::Ok(vec![])
        }
        InitializeObject(ty) => {
            require(
                address(&values[0])? == ty,
                "memory operation pointer type mismatch",
            )?;
            if crate::vm::check_type(ty, module).is_ok() {
                if matches!(exact(&values[0])?, T::ByRef(_)) {
                    crate::initialization::default_value(module, ty)?;
                } else {
                    crate::memory::layout(module, ty)?;
                }
            }
            Result::Ok(vec![])
        }
        CopyObject(ty) => {
            for value in values {
                require(
                    pointer(value)? == ty,
                    "memory operation pointer type mismatch",
                )?;
            }
            Result::Ok(vec![])
        }
        CopyBlock | InitializeBlock => {
            pointer(&values[0])?;
            if matches!(op, CopyBlock) {
                pointer(&values[1])?;
            } else {
                require(
                    exact(&values[1])? == &T::Int32,
                    "initblk fill requires Int32",
                )?;
            }
            require(
                count(exact(&values[2])?),
                "block size requires Int32 or native integer",
            )?;
            Result::Ok(vec![])
        }
        LoadIndirectInt8 | LoadIndirectUInt8 | LoadIndirectInt16 | LoadIndirectUInt16
        | LoadIndirectUInt32 | LoadIndirectInt32 | LoadIndirectInt64 | LoadIndirectNative
        | LoadIndirectFloat32 | LoadIndirectFloat64 => {
            let ty = crate::numeric::indirect_type(op, pointer(&values[0])?)?;
            Result::Ok(vec![loaded(&ty)])
        }
        StoreIndirectInt8 | StoreIndirectInt16 | StoreIndirectInt32 | StoreIndirectInt64
        | StoreIndirectNative | StoreIndirectFloat32 | StoreIndirectFloat64 => {
            let ty = pointer(&values[0])?;
            crate::numeric::indirect_type(op, ty)?;
            stored(&values[1], ty)?;
            Result::Ok(vec![])
        }
        HeapNew => {
            require(
                !matches!(exact(&values[0])?, T::ByRef(_)),
                "managed reference cannot escape into heap storage",
            )?;
            one(T::ByRef(Box::new(exact(&values[0])?.clone())))
        }
        Equal | BranchEqual(_) | BranchNotEqual(_) => {
            require(
                values[0] == values[1],
                "equality requires matching stack types",
            )?;
            if matches!(op, Equal) {
                one(T::Boolean)
            } else {
                Result::Ok(vec![])
            }
        }
        BranchTrue(_) | BranchFalse(_) => {
            require(
                matches!(
                    exact(&values[0])?,
                    T::Boolean
                        | T::Int32
                        | T::Int64
                        | T::IntPtr
                        | T::UIntPtr
                        | T::Ptr(_)
                        | T::ByRef(_)
                ),
                "invalid branch condition type",
            )?;
            Result::Ok(vec![])
        }
        Switch(_) => {
            require(exact(&values[0])? == &T::Int32, "switch requires Int32")?;
            Result::Ok(vec![])
        }
        BitNot | Negate => {
            let ty = exact(&values[0])?;
            require(
                integer(ty) || (matches!(op, Negate) && *ty == T::Double),
                "invalid unary operand type",
            )?;
            one(ty.clone())
        }
        ShiftLeft | ShiftRight | ShiftRightUnsigned => {
            require(
                integer(exact(&values[0])?) && count(exact(&values[1])?),
                "invalid shift operand types",
            )?;
            Result::Ok(vec![values[0].clone()])
        }
        Add
        | Sub
        | Mul
        | Divide
        | Remainder
        | Greater
        | GreaterUnsigned
        | Less
        | LessUnsigned
        | BranchGreater(_)
        | BranchGreaterUnsigned(_)
        | BranchLess(_)
        | BranchLessUnsigned(_)
        | BranchGreaterEqual(_)
        | BranchGreaterEqualUnsigned(_)
        | BranchLessEqual(_)
        | BranchLessEqualUnsigned(_)
        | BitAnd
        | BitOr
        | BitXor
        | RemainderUnsigned
        | DivideUnsigned
        | AddChecked
        | SubChecked
        | MulChecked
        | AddCheckedUnsigned
        | SubCheckedUnsigned
        | MulCheckedUnsigned => {
            let ty = exact(&values[0])?;
            let floating = matches!(
                op,
                Add | Sub
                    | Mul
                    | Divide
                    | Remainder
                    | Greater
                    | GreaterUnsigned
                    | Less
                    | LessUnsigned
                    | BranchGreater(_)
                    | BranchGreaterUnsigned(_)
                    | BranchLess(_)
                    | BranchLessUnsigned(_)
                    | BranchGreaterEqual(_)
                    | BranchGreaterEqualUnsigned(_)
                    | BranchLessEqual(_)
                    | BranchLessEqualUnsigned(_)
            );
            require(
                values[0] == values[1] && (integer(ty) || (floating && *ty == T::Double)),
                "invalid binary operand types",
            )?;
            if matches!(
                op,
                BranchGreater(_)
                    | BranchGreaterUnsigned(_)
                    | BranchLess(_)
                    | BranchLessUnsigned(_)
                    | BranchGreaterEqual(_)
                    | BranchGreaterEqualUnsigned(_)
                    | BranchLessEqual(_)
                    | BranchLessEqualUnsigned(_)
            ) {
                Result::Ok(vec![])
            } else if matches!(op, Greater | GreaterUnsigned | Less | LessUnsigned) {
                one(T::Boolean)
            } else {
                one(ty.clone())
            }
        }
        CheckFinite => {
            require(
                exact(&values[0])? == &T::Double,
                "ckfinite requires floating-point value",
            )?;
            one(T::Double)
        }
        ConvertFloat32 | ConvertFloat64 | ConvertFloatUnsigned => {
            let ty = exact(&values[0])?;
            require(
                integer(ty) || (!matches!(op, ConvertFloatUnsigned) && *ty == T::Double),
                "invalid floating-point conversion source",
            )?;
            one(T::Double)
        }
        ConvertInt8
        | ConvertUInt8
        | ConvertInt16
        | ConvertUInt16
        | ConvertInt32
        | ConvertUInt32
        | ConvertInt64
        | ConvertUInt64
        | ConvertNativeInt
        | ConvertNativeUInt
        | CheckedInt8
        | CheckedUInt8
        | CheckedInt16
        | CheckedUInt16
        | CheckedInt32
        | CheckedUInt32
        | CheckedInt64
        | CheckedUInt64
        | CheckedNativeInt
        | CheckedNativeUInt
        | CheckedInt8Unsigned
        | CheckedUInt8Unsigned
        | CheckedInt16Unsigned
        | CheckedUInt16Unsigned
        | CheckedInt32Unsigned
        | CheckedUInt32Unsigned
        | CheckedInt64Unsigned
        | CheckedUInt64Unsigned
        | CheckedNativeIntUnsigned
        | CheckedNativeUIntUnsigned => {
            let ty = exact(&values[0])?;
            require(
                numeric(ty)
                    || (matches!(op, ConvertNativeInt | ConvertNativeUInt)
                        && matches!(ty, T::Ptr(_))),
                "invalid integer conversion source",
            )?;
            one(match op {
                ConvertInt64
                | ConvertUInt64
                | CheckedInt64
                | CheckedUInt64
                | CheckedInt64Unsigned
                | CheckedUInt64Unsigned => T::Int64,
                ConvertNativeInt | CheckedNativeInt | CheckedNativeIntUnsigned => T::IntPtr,
                ConvertNativeUInt | CheckedNativeUInt | CheckedNativeUIntUnsigned => T::UIntPtr,
                _ => T::Int32,
            })
        }
    }
}
