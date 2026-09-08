//! Nominal, single-target managed callables. Binding is a checked capability.
use crate::{
    Fault, Module, Value,
    metadata::{Function, FunctionRef, Representation, Type},
};

/// Opaque runtime binding. Guest artifacts cannot manufacture live bindings.
#[derive(Debug, Clone)]
pub struct Delegate {
    pub(crate) ty: Type,
    pub(crate) target: FunctionRef,
    pub(crate) receiver: Option<Box<Value>>,
}

impl PartialEq for Delegate {
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
                (Some(a), Some(b)) => slot(a)
                    .zip(slot(b))
                    .is_some_and(|(a, b)| a.same_location(b)),
                _ => false,
            }
    }
}

pub(crate) fn is_contract(module: &Module, function: &Function) -> bool {
    function
        .owner
        .as_ref()
        .and_then(|t| module.type_definition(t))
        .is_some_and(|d| d.representation == Representation::Delegate)
}

pub(crate) fn contract(module: &Module, ty: &Type) -> Result<Function, Fault> {
    let def = module
        .type_definition(ty)
        .filter(|d| d.representation == Representation::Delegate)
        .ok_or_else(|| Fault::new("expected delegate type"))?;
    if !def.fields.is_empty()
        || !def.implements.is_empty()
        || def.base.is_some()
        || !def.properties.is_empty()
        || def.is_abstract
    {
        return Err(Fault::new(
            "delegate requires a fieldless declaration without inheritance",
        ));
    }
    let mut methods = module
        .functions
        .iter()
        .filter(|f| f.owner.as_ref() == Some(&def.open_type()));
    let f = methods
        .next()
        .ok_or_else(|| Fault::new("delegate requires one Invoke declaration"))?;
    if methods.next().is_some()
        || f.name != format!("{}.Invoke", def.name)
        || !f.instance
        || f.receiver_byref
        || f.receiver_readonly
        || f.is_abstract
        || f.is_virtual
        || f.is_override
        || !f.interface_implementations.is_empty()
        || !f.body.is_empty()
        || !f.locals.is_empty()
        || f.pinvoke.is_some()
        || f.impl_flags != 0
        || !f.generic_parameters.is_empty()
        || f.visibility != crate::metadata::Visibility::Public
    {
        return Err(Fault::new("invalid delegate Invoke declaration"));
    }
    let arguments = match ty {
        Type::Constructed { arguments, .. } => arguments.as_slice(),
        _ => &[],
    };
    f.map_types(|t| t.substitute_type_parameters(arguments))
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
            "delegate signature or reference contract mismatch",
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
            && !crate::interfaces::is_contract(module, &callee))
        || (callee.is_abstract
            && !callee.is_virtual
            && !crate::interfaces::is_contract(module, &callee))
    {
        return Err(Fault::new(
            "delegate target requires a concrete IL function or managed dispatch contract",
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
        let value = receiver.ok_or_else(|| Fault::new("delegate target requires receiver"))?;
        let (view, slot) = match value {
            Value::SlotReference(slot) => (slot.target().clone(), slot),
            Value::SlotInterface {
                interface,
                receiver,
            } => (interface, receiver),
            _ => {
                return Err(Fault::new(
                    "delegate receiver requires a heap managed reference",
                ));
            }
        };
        if callee.owner.as_ref() != Some(&view) {
            return Err(Fault::new("delegate receiver type mismatch"));
        }
        slot.assigned()?;
        if slot.allocation_id().is_none() {
            return Err(Fault::new(
                "frame-backed references cannot be retained by delegates",
            ));
        }
        if slot.is_readonly() && !callee.receiver_readonly {
            return Err(Fault::new(
                "delegate cannot bind mutating method through readonly receiver",
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
                "delegate implementation requires managed IL receiver",
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
            return Err(Fault::new("static delegate cannot have receiver"));
        }
        None
    };
    let target = FunctionRef {
        definition: callee.definition,
        name: callee.name,
        owner: callee.owner,
        instance: callee.instance,
        parameters: callee.parameters,
        generic_arguments: callee.generic_arguments,
    };
    Ok(Value::Delegate(Delegate {
        ty: ty.clone(),
        target,
        receiver,
    }))
}
