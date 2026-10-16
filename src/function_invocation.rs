//! Provisional dynamic Function invocation lowered to ordinary checked IL.
use crate::{
    Fault, Module,
    metadata::{Function, FunctionRef, Instruction as Op, Type},
};

/// Build an adapter for InvokeFunction or its non-invoking InvokeFunctionCheck service.
///
/// Check returns 0 for valid input, 2 for unsupported signatures, 6 for invalid
/// receivers and 8 for invalid arguments; it never enters the target function.
///
/// Experimental compiler/runtime primitive, not a stable hosting ABI. Arguments use
/// Object[] with exact boxed scalar types or assignable references; no coercion,
/// ref/out copyback or binder policy is provided. A no-result call returns null.
/// The supplied shape inventory must cover all reachable function bindings.
/// Invoke faults on invalid inputs; Check returns a status before entering the target.
pub fn lower(module: &Module, service: &Function, shapes: &[Type]) -> Result<Function, Fault> {
    let check = service.name == "neoCLR.Runtime.InvokeFunctionCheck";
    let object = service.parameters.first().cloned().unwrap_or(Type::Void);
    if (!check && service.name != "neoCLR.Runtime.InvokeFunction")
        || service.returns != if check { Type::Int32 } else { object.clone() }
        || service.instance
        || service.owner.is_some()
        || service.parameters != [object.clone(), Type::ArrayRef(Box::new(object.clone()))]
        || service.no_result
        || service.impl_flags != 0x1000
        || !service.body.is_empty()
        || !service.locals.is_empty()
        || !service.generic_parameters.is_empty()
        || service.pinvoke.is_some()
        || !service.generic_constraints.is_empty()
        || !service.out_parameters.is_empty()
        || !service.readonly_parameters.is_empty()
        || !module.is_object_reference_type(&object)
    {
        return Err(Fault::new("invalid dynamic function invocation contract"));
    }
    let mut result = service.clone();
    result.impl_flags = 0;
    result.locals = vec![object.clone()];
    result.local_names = vec![None];
    let mut body = vec![
        Op::Arg(1),
        Op::ReferenceIsNull,
        Op::BranchFalse(4),
        Op::Fault("function arguments require a non-null Object array".into()),
    ];
    if check {
        body[3] = Op::Int(8);
        body.insert(4, Op::Return);
        body[2] = Op::BranchFalse(5);
    }
    let mut seen = vec![];
    for ty in shapes {
        let Type::Function(shape) = ty else {
            return Err(Fault::new("expected a function shape"));
        };
        if seen.contains(ty) {
            continue;
        }
        seen.push(ty.clone());
        shape.validate()?;
        let branch = body.len() + 3;
        body.extend([
            Op::Arg(0),
            Op::IsInstance(ty.clone()),
            Op::ReferenceIsNull,
            Op::BranchTrue(0),
        ]);
        let supported = shape
            .parameters
            .iter()
            .all(|t| crate::reflection_properties::supported_value(module, t))
            && (shape.no_result
                || crate::reflection_properties::supported_value(module, &shape.returns));
        if supported && check {
            let count_ok = body.len() + 8;
            body.extend([
                Op::Arg(1),
                Op::ArrayLength,
                Op::ConvertInt32,
                Op::Int(shape.parameters.len() as i32),
                Op::Equal,
                Op::BranchTrue(count_ok),
                Op::Int(8),
                Op::Return,
            ]);
            for (i, parameter) in shape.parameters.iter().enumerate() {
                let mut null_branch = None;
                if module.is_object_reference_type(parameter) {
                    body.extend([
                        Op::Arg(1),
                        Op::Int(i as i32),
                        Op::ArrayElement(object.clone()),
                        Op::ReferenceIsNull,
                    ]);
                    null_branch = Some(body.len());
                    body.push(Op::BranchTrue(0));
                }
                let valid = body.len() + 8;
                body.extend([
                    Op::Arg(1),
                    Op::Int(i as i32),
                    Op::ArrayElement(object.clone()),
                    Op::IsInstance(parameter.clone()),
                    Op::ReferenceIsNull,
                    Op::BranchFalse(valid),
                    Op::Int(8),
                    Op::Return,
                ]);
                if let Some(index) = null_branch {
                    body[index] = Op::BranchTrue(body.len());
                }
            }
            body.extend([Op::Int(0), Op::Return]);
        } else if supported {
            let arity_ok = body.len() + 7;
            body.extend([
                Op::Arg(1),
                Op::ArrayLength,
                Op::ConvertInt32,
                Op::Int(shape.parameters.len() as i32),
                Op::Equal,
                Op::BranchTrue(arity_ok),
                Op::Fault("function argument count mismatch".into()),
                Op::Arg(0),
                Op::CastClass(ty.clone()),
            ]);
            for (i, parameter) in shape.parameters.iter().enumerate() {
                body.extend([
                    Op::Arg(1),
                    Op::Int(i as i32),
                    Op::ArrayElement(object.clone()),
                    if module.is_object_reference_type(parameter) {
                        Op::CastClass(parameter.clone())
                    } else {
                        Op::UnboxAny(parameter.clone())
                    },
                ]);
            }
            body.push(Op::Call(FunctionRef {
                definition: None,
                name: "$Function.Invoke".into(),
                owner: Some(ty.clone()),
                instance: true,
                generic_arguments: vec![],
                parameters: shape.parameters.clone(),
            }));
            if shape.no_result {
                body.extend([
                    Op::LocalAddress(0),
                    Op::InitializeObject(object.clone()),
                    Op::Load(0),
                ]);
            } else if module.is_object_reference_type(&shape.returns) {
                body.push(Op::CastClass(object.clone()));
            } else {
                body.push(Op::BoxValue(shape.returns.clone()));
            }
            body.push(Op::Return);
        } else if check {
            body.extend([Op::Int(2), Op::Return]);
        } else {
            body.push(Op::Fault("unsupported dynamic function signature".into()));
        }
        body[branch] = Op::BranchTrue(body.len());
    }
    if check {
        body.extend([Op::Int(6), Op::Return]);
    } else {
        body.push(Op::Fault(
            "dynamic invocation requires a supported non-null function".into(),
        ));
    }
    result.body = body;
    Ok(result)
}
