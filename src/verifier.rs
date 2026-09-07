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
    let mut functions = Vec::new();
    for (index, function) in module.functions.iter().enumerate() {
        if function.is_internal_call() || function.pinvoke.is_some() {
            continue;
        }
        functions.push(analyze_function(module, index, function)?);
    }
    Ok(Verification { functions })
}

#[derive(Clone)]
struct State {
    stack: Vec<StackType>,
    initialized: Vec<bool>,
}

fn analyze_function(
    module: &Module,
    index: usize,
    function: &Function,
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
        if let Op::Store(slot) = op {
            state.initialized[*slot] = true;
        }
        let inputs = state.stack.split_off(state.stack.len() - pops);
        let outputs = typed_effect(module, function, op, &inputs, arity, state.stack.is_empty())
            .map_err(|e| fault(pc, &e.message))?;
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
        for target in successors {
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
                if previous.stack != state.stack {
                    return Err(fault(
                        target,
                        "incompatible stack types at control-flow join",
                    ));
                }
                let mut changed = false;
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
        Unaligned(_) | Branch(_) | Fault(_) => (0, 0),
        Int(_)
        | Int64(_)
        | Float32 { .. }
        | Float64 { .. }
        | Bool(_)
        | String(_)
        | Void
        | Arg(_)
        | Load(_)
        | SizeOf(_)
        | AlignOf(_)
        | NullPointer(_)
        | None(_)
        | Error(_) => (0, 1),
        Pop | Store(_) | StoreArg(_) | Return | BranchTrue(_) | BranchFalse(_) | Switch(_)
        | InitializeObject(_) => (1, 0),
        Dup => (1, 2),
        New(ty) => (crate::vm::record_fields(module, ty, arity)?.len(), 1),
        Call(target) => (target.parameters.len() + usize::from(target.instance), 1),
        SetField(_) | PointerAdd | HeapStore | BitAnd | BitOr | BitXor | ShiftLeft | ShiftRight
        | ShiftRightUnsigned | Remainder | RemainderUnsigned | Add | Sub | Mul | AddChecked
        | SubChecked | MulChecked | Divide | AddCheckedUnsigned | SubCheckedUnsigned
        | MulCheckedUnsigned | DivideUnsigned | Equal | Greater | GreaterUnsigned | Less
        | LessUnsigned => (2, 1),
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
        | HeapNew
        | HeapLoad
        | Some
        | Ok(_)
        | Err(_)
        | IsCase(_)
        | LoadCase(_) => (1, 1),
    })
}

// A loaded open parameter may normalize to a different runtime stack type (Byte
// becomes Int32, for example). Keep it distinct from a raw stored !n value.
#[derive(Debug, Clone, PartialEq, Eq)]
enum StackType {
    Exact(Type),
    NormalizedParameter(u16),
}

fn loaded(ty: &Type) -> StackType {
    use Type::*;
    match ty {
        TypeParameter(index) => StackType::NormalizedParameter(*index),
        SByte | Byte | Int16 | UInt16 | Char | UInt32 => StackType::Exact(Int32),
        UInt64 => StackType::Exact(Int64),
        Single => StackType::Exact(Double),
        other => StackType::Exact(other.clone()),
    }
}

fn stored(value: &StackType, target: &Type) -> Result<(), Fault> {
    if *value == StackType::Exact(target.clone()) || *value == loaded(target) {
        Ok(())
    } else {
        Err(Fault::new(format!(
            "expected storage {target:?}, got {value:?}"
        )))
    }
}

fn exact(value: &StackType) -> Result<&Type, Fault> {
    match value {
        StackType::Exact(ty) => Ok(ty),
        _ => Err(Fault::new(
            "operation requires a concrete stack type; open parameter normalization is unresolved",
        )),
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

fn argument_types(function: &Function, arity: usize) -> Vec<Type> {
    let mut args = function.argument_types();
    if let (true, Some(Type::Named(name))) = (function.instance && arity > 0, &function.owner) {
        args[0] = Type::Constructed {
            definition: name.clone(),
            arguments: (0..arity).map(|i| Type::TypeParameter(i as u16)).collect(),
        };
    }
    args
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
        crate::vm::record_fields(module, owner, arity)?
            .get(index)
            .map(|f| f.ty.clone())
            .ok_or_else(|| crate::Fault::new("field index out of range"))
    };
    match op {
        Unaligned(_) | Branch(_) | Fault(_) | Pop => Result::Ok(vec![]),
        Int(_) | SizeOf(_) | AlignOf(_) => one(T::Int32),
        Int64(_) => one(T::Int64),
        Float32 { .. } | Float64 { .. } => one(T::Double),
        Bool(_) => one(T::Boolean),
        String(_) => one(T::String),
        Error(_) => one(T::Error),
        Void => one(T::Void),
        Arg(index) => Result::Ok(vec![loaded(&argument_types(function, arity)[*index])]),
        Load(index) => Result::Ok(vec![loaded(&function.locals[*index])]),
        Store(index) => {
            stored(&values[0], &function.locals[*index])?;
            Result::Ok(vec![])
        }
        StoreArg(index) => {
            stored(&values[0], &argument_types(function, arity)[*index])?;
            Result::Ok(vec![])
        }
        Return => {
            stored(&values[0], &function.returns)?;
            Result::Ok(vec![])
        }
        Dup => Result::Ok(vec![values[0].clone(), values[0].clone()]),
        Call(target) => {
            let callee = crate::vm::resolve(module, target)?;
            for (value, ty) in values.iter().zip(callee.argument_types()) {
                stored(value, &ty)?;
            }
            Result::Ok(vec![loaded(&callee.returns)])
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
        FieldAddress(index) => one(T::Ptr(Box::new(field(pointer(&values[0])?, *index)?))),
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
                pointer(&values[0])? == ty,
                "memory load pointer type mismatch",
            )?;
            Result::Ok(vec![loaded(ty)])
        }
        StoreObject(ty) => {
            require(
                pointer(&values[0])? == ty,
                "memory store pointer type mismatch",
            )?;
            stored(&values[1], ty)?;
            Result::Ok(vec![])
        }
        CopyObject(ty) | InitializeObject(ty) => {
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
        HeapNew => one(T::Ref(Box::new(exact(&values[0])?.clone()))),
        HeapLoad | HeapStore => {
            let T::Ref(ty) = exact(&values[0])? else {
                return Result::Err(crate::Fault::new("expected Ref value"));
            };
            if matches!(op, HeapStore) {
                require(
                    values[1] == E(*ty.clone()),
                    "heap.store requires exact stored type",
                )?;
                one(T::Void)
            } else {
                one(*ty.clone())
            }
        }
        Some => one(T::Option(Box::new(exact(&values[0])?.clone()))),
        None(ty) => one(T::Option(Box::new(ty.clone()))),
        Ok(error) => one(T::Result(
            Box::new(exact(&values[0])?.clone()),
            Box::new(error.clone()),
        )),
        Err(success) => one(T::Result(
            Box::new(success.clone()),
            Box::new(exact(&values[0])?.clone()),
        )),
        IsCase(case) | LoadCase(case) => {
            use crate::metadata::Case;
            let payload = match (exact(&values[0])?, case) {
                (T::Option(ty), Case::Some) => *ty.clone(),
                (T::Option(_), Case::None) => T::Void,
                (T::Result(ty, _), Case::Ok) | (T::Result(_, ty), Case::Err) => *ty.clone(),
                _ => return Result::Err(crate::Fault::new("case does not belong to union type")),
            };
            one(if matches!(op, IsCase(_)) {
                T::Boolean
            } else {
                payload
            })
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
                        | T::Ref(_)
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
