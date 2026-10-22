//! Scalar CFG acceptance and stack heights, independent of Cranelift lowering.
//! Definite local initialization remains the ordinary neoCLR verifier's job.
use super::{Error, console_call};
use cranelift_codegen::ir::condcodes::IntCC;
use neoclr::metadata::{Function, Instruction as Op, Type};
use std::collections::VecDeque;

pub(super) fn comparison(op: &Op) -> Option<(usize, IntCC)> {
    Some(match *op {
        Op::BranchEqual(target) => (target, IntCC::Equal),
        Op::BranchNotEqual(target) => (target, IntCC::NotEqual),
        Op::BranchGreater(target) => (target, IntCC::SignedGreaterThan),
        Op::BranchGreaterUnsigned(target) => (target, IntCC::UnsignedGreaterThan),
        Op::BranchLess(target) => (target, IntCC::SignedLessThan),
        Op::BranchLessUnsigned(target) => (target, IntCC::UnsignedLessThan),
        Op::BranchGreaterEqual(target) => (target, IntCC::SignedGreaterThanOrEqual),
        Op::BranchGreaterEqualUnsigned(target) => (target, IntCC::UnsignedGreaterThanOrEqual),
        Op::BranchLessEqual(target) => (target, IntCC::SignedLessThanOrEqual),
        Op::BranchLessEqualUnsigned(target) => (target, IntCC::UnsignedLessThanOrEqual),
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Int32,
    Int64,
    NativeInt,
    NativeUInt,
    Pointer,
    LocalAddress(usize),
    Boolean,
    Literal,
    Unit,
}

pub(super) type Stacks = Vec<Option<Vec<Kind>>>;

pub(super) fn analyze(
    function: &Function,
    functions: &[Function],
    native_imports: bool,
) -> Result<Stacks, Error> {
    let kind = |t: &Type| {
        if matches!(t, Type::Ptr(_)) {
            Kind::Pointer
        } else {
            match t {
                Type::Int64 | Type::UInt64 => Kind::Int64,
                Type::IntPtr => Kind::NativeInt,
                Type::UIntPtr => Kind::NativeUInt,
                _ => Kind::Int32,
            }
        }
    };
    let integer = |k| {
        matches!(
            k,
            Kind::Int32 | Kind::Int64 | Kind::NativeInt | Kind::NativeUInt
        )
    };
    let fail =
        |pc, message| -> Error { format!("{} instruction {pc}: {message}", function.name).into() };
    let count = function.body.len();
    if count == 0 {
        return Err(fail(0, "empty body"));
    }
    // Check opcodes even in dead code; unreachable unsupported behavior is not
    // silently accepted as part of the native profile.
    let mut effects = Vec::new();
    let mut edges = Vec::new();
    for (pc, op) in function.body.iter().enumerate() {
        let effect = match op {
            Op::Int(_) | Op::Bool(_) | Op::String(_) => (0, 1),
            Op::Int64(_) if native_imports => (0, 1),
            Op::ConvertInt64
            | Op::ConvertUInt64
            | Op::ConvertNativeInt
            | Op::ConvertNativeUInt
            | Op::ConvertInt32
                if native_imports =>
            {
                (1, 1)
            }
            Op::LocalAddress(index) if native_imports && *index < function.locals.len() => (0, 1),
            Op::InitializeObject(t)
                if native_imports && crate::scalar_abi::scalar_type(t).is_some() =>
            {
                (1, 0)
            }
            Op::NullPointer(t) if native_imports && crate::scalar_abi::pointer_target(t) => (0, 1),
            Op::PointerCast(t) if native_imports && crate::scalar_abi::pointer_target(t) => (1, 1),
            Op::LoadObject(Type::Int32 | Type::Byte)
            | Op::LoadIndirectInt32
            | Op::LoadIndirectUInt8
                if native_imports =>
            {
                (1, 1)
            }
            Op::StoreObject(Type::Int32 | Type::Byte)
            | Op::StoreIndirectInt32
            | Op::StoreIndirectInt8
                if native_imports =>
            {
                (2, 0)
            }
            Op::Arg(index) if *index < function.parameters.len() => (0, 1),
            Op::Load(index) if *index < function.locals.len() => (0, 1),
            Op::Store(index) if *index < function.locals.len() => (1, 0),
            Op::Dup => (1, 2),
            Op::Pop => (1, 0),
            Op::Equal | Op::Less | Op::Greater => (2, 1),
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
            | Op::Remainder
            | Op::RemainderUnsigned => (2, 1),
            Op::Call(target) if console_call(target) => (target.parameters.len(), 1),
            Op::Call(target) => {
                let callee = functions
                    .iter()
                    .find(|f| f.name == target.name && f.parameters == target.parameters)
                    .ok_or_else(|| fail(pc, "unresolved scalar call"))?;
                (target.parameters.len(), usize::from(!callee.no_result))
            }
            Op::Return => (usize::from(!function.no_result), 0),
            Op::Fault(_) => (0, 0),
            Op::Branch(_) => (0, 0),
            Op::BranchTrue(_) | Op::BranchFalse(_) => (1, 0),
            _ if comparison(op).is_some() => (2, 0),
            _ => return Err(fail(pc, "unsupported instruction")),
        };
        let successors = match op {
            Op::Return | Op::Fault(_) => vec![],
            Op::Branch(target) => vec![*target],
            Op::BranchTrue(target) | Op::BranchFalse(target) => vec![*target, pc + 1],
            _ if comparison(op).is_some() => vec![comparison(op).unwrap().0, pc + 1],
            _ => vec![pc + 1],
        };
        effects.push(effect);
        edges.push(successors);
    }
    let mut stacks: Stacks = vec![None; count];
    stacks[0] = Some(vec![]);
    let mut work = VecDeque::from([0]);
    while let Some(pc) = work.pop_front() {
        let mut stack = stacks[pc].as_ref().unwrap().clone();
        let depth = stack.len();
        let (pops, _) = effects[pc];
        let op = &function.body[pc];
        if matches!(op, Op::Return) && depth != usize::from(!function.no_result) {
            return Err(fail(
                pc,
                "invalid return: stack does not match result contract",
            ));
        }
        if depth < pops {
            return Err(fail(pc, "stack underflow"));
        }
        let popped = stack.split_off(depth - pops);
        let valid_operands = match op {
            Op::Dup | Op::Pop => true,
            Op::PointerCast(_) => popped == [Kind::Pointer],
            Op::LoadObject(_) | Op::LoadIndirectInt32 | Op::LoadIndirectUInt8 => {
                popped == [Kind::Pointer]
            }
            Op::StoreObject(_) | Op::StoreIndirectInt32 | Op::StoreIndirectInt8 => {
                popped == [Kind::Pointer, Kind::Int32]
            }
            Op::ConvertInt64
            | Op::ConvertUInt64
            | Op::ConvertNativeInt
            | Op::ConvertNativeUInt
            | Op::ConvertInt32 => popped.iter().copied().all(integer),
            Op::InitializeObject(t) => {
                matches!(popped.as_slice(), [Kind::LocalAddress(index)] if function.locals[*index] == *t)
            }
            Op::Equal | Op::BranchEqual(_) | Op::BranchNotEqual(_) => {
                popped[0] == popped[1]
                    && (matches!(popped[0], Kind::Int32 | Kind::Boolean)
                        || (native_imports
                            && matches!(
                                popped[0],
                                Kind::Pointer | Kind::Int64 | Kind::NativeInt | Kind::NativeUInt
                            )))
            }
            Op::BranchTrue(_) | Op::BranchFalse(_) => {
                matches!(popped[0], Kind::Int32 | Kind::Boolean)
                    || (native_imports
                        && matches!(
                            popped[0],
                            Kind::Pointer | Kind::Int64 | Kind::NativeInt | Kind::NativeUInt
                        ))
            }
            Op::Call(target) if console_call(target) => popped == [Kind::Literal],
            Op::Call(target) => popped
                .iter()
                .copied()
                .eq(target.parameters.iter().map(&kind)),
            Op::Store(index) => popped == [kind(&function.locals[*index])],
            Op::Return if function.no_result => popped.is_empty(),
            Op::Return => popped == [kind(&function.returns)],
            _ => popped.iter().all(|kind| *kind == Kind::Int32),
        };
        if !valid_operands {
            return Err(fail(
                pc,
                "unsupported operand type for scalar/console instruction",
            ));
        }
        match op {
            Op::String(_) => stack.push(Kind::Literal),
            Op::Int64(_) | Op::ConvertInt64 | Op::ConvertUInt64 => stack.push(Kind::Int64),
            Op::ConvertNativeInt => stack.push(Kind::NativeInt),
            Op::ConvertNativeUInt => stack.push(Kind::NativeUInt),
            Op::NullPointer(_) | Op::PointerCast(_) => stack.push(Kind::Pointer),
            Op::LocalAddress(index) => stack.push(Kind::LocalAddress(*index)),
            Op::Bool(_) | Op::Equal | Op::Less | Op::Greater => stack.push(Kind::Boolean),
            Op::Call(target) if console_call(target) => stack.push(Kind::Unit),
            Op::Arg(index) => stack.push(kind(&function.parameters[*index])),
            Op::Load(index) => stack.push(kind(&function.locals[*index])),
            Op::Call(target) => {
                let callee = functions
                    .iter()
                    .find(|f| f.name == target.name && f.parameters == target.parameters)
                    .ok_or_else(|| fail(pc, "unresolved scalar call"))?;
                if !callee.no_result {
                    stack.push(kind(&callee.returns));
                }
            }
            Op::Dup => {
                stack.extend_from_slice(&popped);
                stack.extend_from_slice(&popped);
            }
            _ => {
                for _ in 0..effects[pc].1 {
                    stack.push(Kind::Int32);
                }
            }
        }
        for &next in &edges[pc] {
            if next >= count {
                return Err(fail(pc, "control flow leaves function body"));
            }
            match &stacks[next] {
                Some(existing) if existing.len() != stack.len() => {
                    return Err(fail(
                        next,
                        "incompatible stack heights at control-flow join",
                    ));
                }
                Some(existing) if *existing != stack => {
                    return Err(fail(next, "incompatible stack types at control-flow join"));
                }
                Some(_) => (),
                None => {
                    stacks[next] = Some(stack.clone());
                    work.push_back(next);
                }
            }
        }
    }
    Ok(stacks)
}
