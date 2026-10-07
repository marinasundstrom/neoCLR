//! Scalar CFG acceptance and stack heights, independent of Cranelift lowering.
//! Definite local initialization remains the ordinary neoCLR verifier's job.
use super::{Error, console_call};
use cranelift_codegen::ir::condcodes::IntCC;
use neoclr::metadata::{Function, Instruction as Op};
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
    Literal,
    Unit,
}

pub(super) type Stacks = Vec<Option<Vec<Kind>>>;

pub(super) fn analyze(function: &Function) -> Result<Stacks, Error> {
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
            Op::Int(_) | Op::String(_) => (0, 1),
            Op::Arg(index) if *index < function.parameters.len() => (0, 1),
            Op::Load(index) if *index < function.locals.len() => (0, 1),
            Op::Store(index) if *index < function.locals.len() => (1, 0),
            Op::Dup => (1, 2),
            Op::Pop => (1, 0),
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
            Op::Call(target) => (target.parameters.len(), 1),
            Op::Return => (1, 0),
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
        if matches!(op, Op::Return) && depth != 1 {
            return Err(fail(pc, "invalid return: expected exactly one value"));
        }
        if depth < pops {
            return Err(fail(pc, "stack underflow"));
        }
        let popped = stack.split_off(depth - pops);
        let expected = if matches!(op, Op::Call(target) if console_call(target)) {
            Kind::Literal
        } else {
            Kind::Int32
        };
        if !matches!(op, Op::Dup | Op::Pop) && popped.iter().any(|kind| *kind != expected) {
            return Err(fail(
                pc,
                "unsupported operand type for scalar/console instruction",
            ));
        }
        match op {
            Op::String(_) => stack.push(Kind::Literal),
            Op::Call(target) if console_call(target) => stack.push(Kind::Unit),
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
