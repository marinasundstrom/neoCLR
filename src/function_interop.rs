//! Closed ABI adaptation for general function-signature interop methods.
use crate::{
    Fault, Module,
    metadata::{Function, Instruction as Op, Type},
};

/// Expand an owned object-array implementation into an ordinary closed method body.
/// Open generic definitions are retained without manufacturing an arity.
pub fn lower(module: &Module, function: &mut Function) -> Result<(), Fault> {
    let Some(object) = function.boxed_arguments.clone() else {
        return Ok(());
    };
    let Some(Type::Function(shape)) = &function.function_type else {
        return Ok(());
    };
    if !module.is_object_reference_type(&object)
        || function.receiver_byref
        || shape
            .parameters
            .iter()
            .any(|t| !crate::reflection_properties::supported_value(module, t))
        || !shape.no_result
            && shape.returns != Type::Void
            && !crate::reflection_properties::supported_value(module, &shape.returns)
    {
        return Err(Fault::new("unsupported boxed function method signature"));
    }
    let array = Type::ArrayRef(Box::new(object.clone()));
    let slot = function.locals.len();
    let argument = usize::from(function.instance);
    let mut body = vec![
        Op::Int(shape.parameters.len() as i32),
        Op::NewArray(object.clone()),
        Op::Store(slot),
    ];
    for (i, ty) in shape.parameters.iter().enumerate() {
        body.extend([Op::Load(slot), Op::Int(i as i32), Op::Arg(i + argument)]);
        body.push(if module.is_object_reference_type(ty) {
            Op::CastClass(object.clone())
        } else {
            Op::BoxValue(ty.clone())
        });
        body.push(Op::StoreArrayElement(object.clone()));
    }
    let mut offsets = vec![0; function.body.len() + 1];
    for (i, op) in function.body.iter().enumerate() {
        offsets[i] = body.len();
        match op {
            Op::Arg(index) if *index == argument => body.push(Op::Load(slot)),
            Op::StoreArg(index) if *index == argument => body.push(Op::Store(slot)),
            Op::ArgumentAddress(index) if *index == argument => body.push(Op::LocalAddress(slot)),
            Op::Receiver {
                argument: true,
                index,
                readonly_value,
            } if *index == argument => body.push(Op::Receiver {
                argument: false,
                index: slot,
                readonly_value: *readonly_value,
            }),
            Op::Return => {
                if shape.no_result {
                    body.push(Op::Pop);
                } else if shape.returns == Type::Void {
                    body.extend([Op::Pop, Op::Void]);
                } else if module.is_object_reference_type(&shape.returns) {
                    body.push(Op::CastClass(shape.returns.clone()));
                } else {
                    body.push(Op::UnboxAny(shape.returns.clone()));
                }
                body.push(Op::Return);
            }
            _ => body.push(op.clone()),
        }
    }
    offsets[function.body.len()] = body.len();
    let map = |target: &mut usize| -> Result<(), Fault> {
        *target = *offsets
            .get(*target)
            .ok_or_else(|| Fault::new("invalid interop branch target"))?;
        Ok(())
    };
    for op in &mut body {
        match op {
            Op::Branch(t)
            | Op::BranchTrue(t)
            | Op::BranchFalse(t)
            | Op::BranchEqual(t)
            | Op::BranchNotEqual(t)
            | Op::BranchGreater(t)
            | Op::BranchGreaterUnsigned(t)
            | Op::BranchLess(t)
            | Op::BranchLessUnsigned(t)
            | Op::BranchGreaterEqual(t)
            | Op::BranchGreaterEqualUnsigned(t)
            | Op::BranchLessEqual(t)
            | Op::BranchLessEqualUnsigned(t) => map(t)?,
            Op::Switch(targets) => {
                for target in targets {
                    map(target)?;
                }
            }
            _ => {}
        }
    }
    // Preserve source locations against the expanded instruction offsets.
    for point in &mut function.sequence_points {
        point.instruction = *offsets
            .get(point.instruction)
            .ok_or_else(|| Fault::new("invalid interop sequence point"))?;
    }
    function.locals.push(array);
    function.local_names.resize(slot, None);
    function.local_names.push(Some("$arguments".into()));
    function.body = body;
    function.boxed_arguments = None;
    Ok(())
}
