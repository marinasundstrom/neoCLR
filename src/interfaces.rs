//! Explicit borrowed interface dispatch. No boxing, ownership or receiver writeback.
use crate::{
    Fault, Module,
    metadata::{Function, FunctionRef, Representation, Type, TypeDef, Visibility},
};

pub(crate) fn interface_definition<'a>(
    module: &'a Module,
    interface: &Type,
) -> Result<&'a TypeDef, Fault> {
    module
        .type_definition(interface)
        .filter(|d| d.representation == Representation::Interface)
        .ok_or_else(|| Fault::new("expected interface type"))
}

pub(crate) fn is_contract(module: &Module, function: &Function) -> bool {
    function
        .owner
        .as_ref()
        .and_then(|owner| module.type_definition(owner))
        .is_some_and(|d| d.representation == Representation::Interface)
}

pub(crate) fn validate_contract(function: &Function) -> Result<(), Fault> {
    if !function.instance
        || function.visibility != Visibility::Public
        || function.name.ends_with(".ctor")
        || !function.body.is_empty()
        || !function.locals.is_empty()
        || function.impl_flags != 0
        || function.pinvoke.is_some()
    {
        return Err(Fault::new(
            "interface members must be public instance declarations without bodies, locals or native bindings",
        ));
    }
    Ok(())
}

fn arguments(ty: &Type) -> &[Type] {
    if let Type::Constructed { arguments, .. } = ty {
        arguments
    } else {
        &[]
    }
}

fn declared(module: &Module, concrete: &Type, interface: &Type) -> Result<(), Fault> {
    interface_definition(module, interface)?;
    let definition = module
        .type_definition(concrete)
        .ok_or_else(|| Fault::new("interface receiver requires a known implementing type"))?;
    let interfaces = definition
        .implements
        .iter()
        .map(|ty| ty.substitute_type_parameters(arguments(concrete)))
        .collect::<Result<Vec<_>, _>>()?;
    if interfaces.iter().filter(|ty| *ty == interface).count() != 1 {
        return Err(Fault::new(
            "type must declare exactly one matching interface implementation",
        ));
    }
    Ok(())
}

fn member(module: &Module, concrete: &Type, contract: &Function) -> Result<Function, Fault> {
    let owner = concrete
        .definition_name()
        .ok_or_else(|| Fault::new("invalid interface implementation owner"))?;
    let name = contract
        .name
        .rsplit('.')
        .next()
        .ok_or_else(|| Fault::new("invalid interface member"))?;
    let target = FunctionRef {
        definition: None,
        name: format!("{owner}.{name}"),
        owner: Some(concrete.clone()),
        instance: true,
        parameters: contract.parameters.clone(),
    };
    let implementation = crate::vm::resolve(module, &target)?;
    if implementation
        .out_when_true
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        != contract
            .out_when_true
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
        || implementation
            .readonly_parameters
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            != contract
                .readonly_parameters
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
        || implementation.receiver_byref != contract.receiver_byref
        || implementation.receiver_readonly != contract.receiver_readonly
        || implementation
            .out_parameters
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            != contract
                .out_parameters
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
        || implementation.returns != contract.returns
        || implementation.visibility != Visibility::Public
        || implementation.is_internal_call()
        || implementation.pinvoke.is_some()
        || is_contract(module, &implementation)
    {
        return Err(Fault::new(
            "interface implementation requires a public IL instance method with the exact return type",
        ));
    }
    Ok(implementation)
}

pub(crate) fn ensure_implementation(
    module: &Module,
    concrete: &Type,
    interface: &Type,
) -> Result<(), Fault> {
    declared(module, concrete, interface)?;
    let definition = interface_definition(module, interface)?;
    for method in &module.functions {
        if method
            .owner
            .as_ref()
            .and_then(|ty| module.type_definition(ty))
            .is_some_and(|d| std::ptr::eq(d, definition))
        {
            let contract =
                method.map_types(|ty| ty.substitute_type_parameters(arguments(interface)))?;
            member(module, concrete, &contract)?;
        }
    }
    Ok(())
}

pub(crate) fn implementation(
    module: &Module,
    concrete: &Type,
    interface: &Type,
    contract: &Function,
) -> Result<Function, Fault> {
    declared(module, concrete, interface)?;
    if !is_contract(module, contract) || contract.owner.as_ref() != Some(interface) {
        return Err(Fault::new("callvirt requires an interface member"));
    }
    member(module, concrete, contract)
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    for definition in &module.types {
        if definition.representation == Representation::Interface
            && (!definition.fields.is_empty()
                || !definition.implements.is_empty()
                || definition.packing.is_some()
                || definition.minimum_size.is_some())
        {
            return Err(Fault::new(
                "interfaces cannot declare storage, layout or inherited interfaces in this preview",
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for interface in &definition.implements {
            if !seen.insert(interface) {
                return Err(Fault::new("duplicate interface implementation"));
            }
            ensure_implementation(module, &definition.open_type(), interface)?;
        }
    }
    Ok(())
}

/// Conservative targets in the loaded module set. Never silently omit unresolved
/// generic implementations from a graph advertised as closed.
pub(crate) fn dispatch_targets(
    module: &Module,
    contract: &Function,
) -> Result<Vec<Function>, Fault> {
    fn infer(pattern: &Type, actual: &Type, bindings: &mut [Option<Type>]) -> bool {
        match (pattern, actual) {
            (Type::TypeParameter(index), actual) => {
                let Some(slot) = bindings.get_mut(*index as usize) else {
                    return false;
                };
                match slot {
                    Some(bound) => bound == actual,
                    None => {
                        *slot = Some(actual.clone());
                        true
                    }
                }
            }
            (
                Type::Constructed {
                    definition: left,
                    arguments: a,
                },
                Type::Constructed {
                    definition: right,
                    arguments: b,
                },
            ) => {
                left == right
                    && a.len() == b.len()
                    && a.iter().zip(b).all(|(a, b)| infer(a, b, bindings))
            }
            (Type::Ptr(a), Type::Ptr(b)) | (Type::InterfaceRef(a), Type::InterfaceRef(b)) => {
                infer(a, b, bindings)
            }
            _ => pattern == actual,
        }
    }
    let interface = contract
        .owner
        .as_ref()
        .ok_or_else(|| Fault::new("interface call requires owner"))?;
    let mut targets = vec![];
    for definition in &module.types {
        for implemented in &definition.implements {
            let mut bindings = vec![None; definition.generic_parameters.len()];
            if !infer(implemented, interface, &mut bindings) {
                continue;
            }
            let arguments = bindings
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| {
                    Fault::new(
                        "closed interface call graph cannot infer all implementing type arguments",
                    )
                })?;
            let concrete = definition
                .open_type()
                .substitute_type_parameters(&arguments)?;
            let target = implementation(module, &concrete, interface, contract)?;
            if !targets
                .iter()
                .any(|f: &Function| f.definition == target.definition && f.owner == target.owner)
            {
                targets.push(target);
            }
        }
    }
    Ok(targets)
}
