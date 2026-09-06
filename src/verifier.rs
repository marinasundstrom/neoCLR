//! Opt-in control-flow, stack-height, and definite-local-initialization analysis.
//! This is not a type or memory-safety verifier.
use crate::{
    Fault, Module,
    metadata::{Function, Instruction as Op, Type},
};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct FunctionVerification {
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
    if module.name == "System" {
        crate::vm::validate(module)?;
        analyze(module)
    } else {
        verify_with_library(module, crate::library::system()?)
    }
}

pub fn verify_with_library(module: &Module, library: &Module) -> Result<Verification, Fault> {
    let linked = crate::library::link(module, library)?;
    analyze(&linked)
}

fn analyze(module: &Module) -> Result<Verification, Fault> {
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
    height: usize,
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
    };
    let arity = function
        .owner
        .as_ref()
        .and_then(|t| module.type_definition(t))
        .map_or(0, |d| d.generic_parameters.len());
    let mut states: Vec<Option<State>> = vec![None; function.body.len()];
    states[0] = Some(State {
        height: 0,
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
        if state.height < pops {
            return Err(fault(pc, "evaluation stack underflow"));
        }
        if matches!(op, Op::Return) && state.height != 1 {
            return Err(fault(pc, "ret requires exactly one value"));
        }
        if matches!(op, Op::Load(slot) if !state.initialized[*slot]) {
            return Err(fault(pc, "local is not initialized on every incoming path"));
        }
        if let Op::Store(slot) = op {
            state.initialized[*slot] = true;
        }
        state.height = state.height - pops + pushes;
        maximum_stack = maximum_stack.max(state.height);
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
                if previous.height != state.height {
                    return Err(fault(
                        target,
                        "incompatible stack heights at control-flow join",
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
