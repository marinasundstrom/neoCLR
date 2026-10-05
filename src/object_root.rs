//! Host-selected Object identity. This is load context, not serialized metadata.
use crate::{
    Fault, Module,
    metadata::{Function, Representation, Type, Visibility},
};

pub(crate) fn owns_slot(module: &Module, method: &Function) -> bool {
    let Some(member) = &method.definition else {
        return false;
    };
    match &module.object_root {
        Some(root) => {
            member.module == root.module
                && member.revision == root.revision
                && method.owner.as_ref() == Some(&Type::from_name("System.Object"))
        }
        None => member.module == "System",
    }
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    let Some(identity) = &module.object_root else {
        return Ok(());
    };
    let candidates: Vec<_> = module
        .types
        .iter()
        .filter(|ty| ty.name == "System.Object")
        .collect();
    let root = match candidates.as_slice() {
        [root] if root.definition.as_ref() == Some(identity) => *root,
        _ => {
            return Err(Fault::new(
                "selected Object root is missing, conflicting or has the wrong identity",
            ));
        }
    };
    if !root.is_reference_type
        || !root.is_abstract
        || root.is_sealed
        || root.visibility != Visibility::Public
        || root.representation != Representation::Record
        || root.base.is_some()
        || root.declaring_type.is_some()
        || !root.fields.is_empty()
        || !root.generic_parameters.is_empty()
        || !root.generic_constraints.is_empty()
        || root.enum_info.is_some()
    {
        return Err(Fault::new(
            "selected Object root requires a public abstract fieldless nongeneric root class",
        ));
    }
    for (name, parameters, result) in [
        ("ToString", vec![], Type::String),
        (
            "Equals",
            vec![Type::from_name("System.Object")],
            Type::Boolean,
        ),
        ("GetHashCode", vec![], Type::Int32),
    ] {
        let full_name = format!("System.Object.{name}");
        let slots: Vec<_> = module
            .functions
            .iter()
            .filter(|method| method.name == full_name && method.parameters == parameters)
            .collect();
        let slot = match slots.as_slice() {
            [slot] => *slot,
            _ => {
                return Err(Fault::new(format!(
                    "selected Object root requires one {name} slot"
                )));
            }
        };
        if !owns_slot(module, slot)
            || !slot.instance
            || !slot.is_virtual
            || slot.is_abstract
            || slot.is_override
            || slot.visibility != Visibility::Public
            || slot.receiver_byref
            || slot.receiver_readonly
            || slot.no_result
            || slot.returns != result
            || !slot.generic_parameters.is_empty()
            || !slot.generic_constraints.is_empty()
            || !slot.interface_implementations.is_empty()
            || !slot.generic_arguments.is_empty()
            || !slot.out_parameters.is_empty()
            || !slot.out_when_true.is_empty()
            || !slot.readonly_parameters.is_empty()
        {
            return Err(Fault::new(format!(
                "incompatible selected Object root {name} slot"
            )));
        }
    }
    Ok(())
}
