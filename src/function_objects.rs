//! Checked single-target objects instantiated from structural Function shapes.
use crate::{
    Fault, Module, Value,
    metadata::{Function, FunctionRef, Type},
};

/// Opaque runtime binding. Guest artifacts cannot manufacture live bindings.
#[derive(Debug, Clone)]
pub struct FunctionObject {
    pub(crate) ty: Type,
    pub(crate) target: FunctionRef,
    pub(crate) receiver: Option<Box<Value>>,
}

impl PartialEq for FunctionObject {
    fn eq(&self, other: &Self) -> bool {
        fn slot(value: &Value) -> Option<&crate::SlotReference> {
            match value {
                Value::SlotReference(r) | Value::SlotInterface { receiver: r, .. } => Some(r),
                _ => None,
            }
        }
        self.ty == other.ty
            && self.target == other.target
            && match (&self.receiver, &other.receiver) {
                (None, None) => true,
                (Some(a), Some(b)) => match (a.as_ref(), b.as_ref()) {
                    (Value::ObjectReference(a), Value::ObjectReference(b)) => {
                        a.reference.same_location(&b.reference)
                    }
                    _ => slot(a)
                        .zip(slot(b))
                        .is_some_and(|(a, b)| a.same_location(b)),
                },
                _ => false,
            }
    }
}

pub(crate) fn is_contract(_module: &Module, function: &Function) -> bool {
    matches!(function.owner, Some(Type::Function(_)))
}

pub(crate) fn contract(_module: &Module, ty: &Type) -> Result<Function, Fault> {
    if let Type::Function(shape) = ty {
        shape.validate()?;
        return Ok(Function {
            origin: None,
            sequence_points: vec![],
            visibility: crate::metadata::Visibility::Public,
            definition: None,
            custom_attributes: vec![],
            name: "$Function.Invoke".into(),
            owner: Some(ty.clone()),
            instance: true,
            parameters: shape.parameters.clone(),
            parameter_names: vec![],
            receiver_byref: false,
            receiver_readonly: false,
            is_virtual: false,
            is_override: false,
            is_abstract: false,
            interface_implementations: vec![],
            generic_parameters: vec![],
            generic_constraints: vec![],
            generic_arguments: vec![],
            out_parameters: shape.out_parameters.clone(),
            out_when_true: shape.out_when_true.clone(),
            readonly_parameters: vec![],
            no_result: shape.no_result,
            returns: shape.returns.clone(),
            locals: vec![],
            local_names: vec![],
            impl_flags: 0,
            pinvoke: None,
            body: vec![],
        });
    }
    Err(Fault::new("expected structural Function type"))
}

fn readonly(f: &Function, i: usize) -> bool {
    f.readonly_parameters.contains(&i) || matches!(f.parameters[i], Type::ReadOnlyByRef(_))
}
fn parameter(t: &Type) -> Type {
    match t {
        Type::ReadOnlyByRef(t) => Type::ByRef(t.clone()),
        _ => t.clone(),
    }
}
pub(crate) fn compatible(contract: &Function, target: &Function) -> Result<(), Fault> {
    if contract.parameters.len() != target.parameters.len()
        || contract.no_result != target.no_result
        || contract.returns != target.returns
        || contract
            .parameters
            .iter()
            .zip(&target.parameters)
            .any(|(a, b)| parameter(a) != parameter(b))
        || (0..contract.parameters.len()).any(|i| {
            readonly(contract, i) != readonly(target, i)
                || contract.out_parameters.contains(&i) != target.out_parameters.contains(&i)
                || contract.out_when_true.contains(&i) != target.out_when_true.contains(&i)
        })
    {
        return Err(Fault::new(
            "Function signature or reference contract mismatch",
        ));
    }
    Ok(())
}

pub(crate) fn validate_binding(
    module: &Module,
    caller: &Function,
    ty: &Type,
    target: &FunctionRef,
) -> Result<Function, Fault> {
    let signature = contract(module, ty)?;
    let callee = crate::vm::resolve(module, target)?;
    crate::access::check_call(module, Some(caller), &callee)?;
    compatible(&signature, &callee)?;
    if callee.name.ends_with("..ctor")
        || callee.pinvoke.is_some()
        || callee.impl_flags != 0
        || is_contract(module, &callee)
        || (callee.instance
            && !callee.receiver_byref
            && !crate::interfaces::is_contract(module, &callee)
            && !callee
                .owner
                .as_ref()
                .is_some_and(|owner| module.is_reference_type(owner)))
        || (callee.is_abstract
            && !callee.is_virtual
            && !crate::interfaces::is_contract(module, &callee))
    {
        return Err(Fault::new(
            "Function target requires a concrete IL function or managed dispatch contract",
        ));
    }
    Ok(callee)
}

pub(crate) fn bind(
    module: &Module,
    caller: &Function,
    ty: &Type,
    target: &FunctionRef,
    receiver: Option<Value>,
) -> Result<Value, Fault> {
    let mut callee = validate_binding(module, caller, ty, target)?;
    let receiver = if callee.instance {
        let value = receiver.ok_or_else(|| Fault::new("Function target requires receiver"))?;
        if let Value::ObjectReference(mut object) = value {
            let owner = callee.owner.as_ref().unwrap();
            if object.target() != owner && !module.reference_assignable(object.target(), owner) {
                return Err(Fault::new("Function receiver type mismatch"));
            }
            object.reference.assigned()?;
            if crate::interfaces::is_contract(module, &callee) {
                callee = crate::interfaces::implementation(
                    module,
                    &object.concrete_type(),
                    owner,
                    &callee,
                )?;
            } else if callee.is_virtual {
                callee = crate::inheritance::dispatch(module, &object.concrete_type(), &callee)?;
            }
            if callee.receiver_byref
                || callee.is_abstract
                || crate::interfaces::is_contract(module, &callee)
            {
                return Err(Fault::new(
                    "Function binding requires a concrete class implementation",
                ));
            }
            compatible(&contract(module, ty)?, &callee)?;
            object.view = callee.owner.clone();
            return Ok(finish(
                ty,
                callee,
                Some(Box::new(Value::ObjectReference(object))),
            ));
        }
        let (view, slot) = match value {
            Value::SlotReference(slot) => (slot.target().clone(), slot),
            Value::SlotInterface {
                interface,
                receiver,
            } => (interface, receiver),
            _ => {
                return Err(Fault::new(
                    "Function receiver requires a heap managed reference",
                ));
            }
        };
        if callee.owner.as_ref() != Some(&view) {
            return Err(Fault::new("Function receiver type mismatch"));
        }
        slot.assigned()?;
        if slot.allocation_id().is_none() {
            return Err(Fault::new(
                "frame-backed references cannot be retained by Function objects",
            ));
        }
        if slot.is_readonly() && !callee.receiver_readonly {
            return Err(Fault::new(
                "Function cannot bind mutating method through readonly receiver",
            ));
        }
        if crate::interfaces::is_contract(module, &callee) {
            callee =
                crate::interfaces::implementation(module, &slot.stored_type()?, &view, &callee)?;
        } else if callee.is_virtual {
            callee = crate::inheritance::dispatch(module, &slot.stored_type()?, &callee)?;
        }
        if callee.is_abstract
            || (slot.is_readonly() && !callee.receiver_readonly)
            || callee.pinvoke.is_some()
            || callee.impl_flags != 0
            || (!callee.receiver_byref && !crate::interfaces::is_contract(module, &callee))
        {
            return Err(Fault::new(
                "Function implementation requires managed IL receiver",
            ));
        }
        compatible(&contract(module, ty)?, &callee)?;
        let value = if crate::interfaces::is_contract(module, &callee) {
            Value::SlotInterface {
                interface: callee.owner.clone().unwrap(),
                receiver: slot,
            }
        } else {
            Value::SlotReference(slot.dispatch_view(module, callee.owner.as_ref().unwrap())?)
        };
        Some(Box::new(value))
    } else {
        if receiver.is_some() {
            return Err(Fault::new("static Function cannot have receiver"));
        }
        None
    };
    Ok(finish(ty, callee, receiver))
}

fn finish(ty: &Type, callee: Function, receiver: Option<Box<Value>>) -> Value {
    let target = FunctionRef {
        definition: callee.definition,
        name: callee.name,
        owner: callee.owner,
        instance: callee.instance,
        parameters: callee.parameters,
        generic_arguments: callee.generic_arguments,
    };
    Value::Function(FunctionObject {
        ty: ty.clone(),
        target,
        receiver,
    })
}
