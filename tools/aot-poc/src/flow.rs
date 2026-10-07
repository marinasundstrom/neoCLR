//! Scalar CFG acceptance and stack heights, independent of Cranelift lowering.
//! Definite local initialization remains the ordinary neoCLR verifier's job.
use super::Error;
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

pub(super) fn analyze(function: &Function) -> Result<Vec<Option<usize>>, Error> {
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
            Op::Int(_) => (0, 1),
            Op::Arg(index) if *index < function.parameters.len() => (0, 1),
            Op::Load(index) if *index < function.locals.len() => (0, 1),
            Op::Store(index) if *index < function.locals.len() => (1, 0),
            Op::Dup => (1, 2),
            Op::Pop => (1, 0),
            Op::Add | Op::Sub | Op::Mul => (2, 1),
            Op::Call(target) => (target.parameters.len(), 1),
            Op::Return => (1, 0),
            Op::Branch(_) => (0, 0),
            Op::BranchTrue(_) | Op::BranchFalse(_) => (1, 0),
            _ if comparison(op).is_some() => (2, 0),
            _ => return Err(fail(pc, "unsupported instruction")),
        };
        let successors = match op {
            Op::Return => vec![],
            Op::Branch(target) => vec![*target],
            Op::BranchTrue(target) | Op::BranchFalse(target) => vec![*target, pc + 1],
            _ if comparison(op).is_some() => vec![comparison(op).unwrap().0, pc + 1],
            _ => vec![pc + 1],
        };
        effects.push(effect);
        edges.push(successors);
    }
    let mut depths = vec![None; count];
    depths[0] = Some(0);
    let mut work = VecDeque::from([0]);
    while let Some(pc) = work.pop_front() {
        let depth = depths[pc].unwrap();
        let (pops, pushes) = effects[pc];
        if matches!(function.body[pc], Op::Return) && depth != 1 {
            return Err(fail(pc, "invalid return: expected exactly one value"));
        }
        if depth < pops {
            return Err(fail(pc, "stack underflow"));
        }
        let next_depth = depth - pops + pushes;
        for &next in &edges[pc] {
            if next >= count {
                return Err(fail(pc, "control flow leaves function body"));
            }
            match depths[next] {
                Some(existing) if existing != next_depth => {
                    return Err(fail(
                        next,
                        "incompatible stack heights at control-flow join",
                    ));
                }
                Some(_) => (),
                None => {
                    depths[next] = Some(next_depth);
                    work.push_back(next);
                }
            }
        }
    }
    Ok(depths)
}
