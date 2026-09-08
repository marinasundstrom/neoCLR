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

/// Transitive, substituted interface identities, with diamonds deduplicated.
pub(crate) fn closure(module: &Module, ty: &Type) -> Result<Vec<Type>, Fault> {
    fn visit(
        module: &Module,
        ty: &Type,
        path: &mut Vec<String>,
        result: &mut Vec<Type>,
    ) -> Result<(), Fault> {
        let definition = module
            .type_definition(ty)
            .ok_or_else(|| Fault::new("interface receiver requires a known implementing type"))?;
        let name = ty
            .definition_name()
            .ok_or_else(|| Fault::new("invalid interface owner"))?
            .to_string();
        if path.contains(&name) || path.len() >= 256 {
            return Err(Fault::new(
                "cyclic or excessively deep interface inheritance",
            ));
        }
        if definition.representation == Representation::Interface {
            if result.contains(ty) {
                return Ok(());
            }
            result.push(ty.clone());
        }
        path.push(name);
        for base in &definition.implements {
            let base = base.substitute_type_parameters(arguments(ty))?;
            interface_definition(module, &base)?;
            visit(module, &base, path, result)?;
        }
        if definition.representation == Representation::Record {
            if let Some(base) = crate::inheritance::base(module, ty)? {
                visit(module, &base, path, result)?;
            }
        }
        path.pop();
        Ok(())
    }
    let mut result = Vec::new();
    visit(module, ty, &mut Vec::new(), &mut result)?;
    Ok(result)
}

fn declared(module: &Module, concrete: &Type, interface: &Type) -> Result<(), Fault> {
    interface_definition(module, interface)?;
    if !closure(module, concrete)?.contains(interface) {
        return Err(Fault::new(
            "type must declare exactly one matching interface implementation (directly or transitively)",
        ));
    }
    Ok(())
}

fn member(module: &Module, concrete: &Type, contract: &Function) -> Result<Function, Fault> {
    // An inherited mapping is anchored at the class that declares conformance.
    // A repeated declaration remaps from that class, including inherited members.
    let chain = if module
        .type_definition(concrete)
        .is_some_and(|d| d.representation == Representation::Record)
    {
        crate::inheritance::lineage(module, concrete)?
    } else {
        vec![concrete.clone()]
    };
    let interface = contract
        .owner
        .as_ref()
        .ok_or_else(|| Fault::new("interface contract requires owner"))?;
    let mut anchor = None;
    for (index, owner) in chain.iter().enumerate() {
        let definition = module
            .type_definition(owner)
            .ok_or_else(|| Fault::new("unknown implementation owner"))?;
        for declared in &definition.implements {
            let declared = declared.substitute_type_parameters(arguments(owner))?;
            if closure(module, &declared)?.contains(interface) {
                anchor = Some(index);
                break;
            }
        }
        if anchor.is_some() {
            break;
        }
    }
    let anchor = anchor.ok_or_else(|| Fault::new("missing interface mapping declaration"))?;
    let name = contract
        .name
        .rsplit('.')
        .next()
        .ok_or_else(|| Fault::new("invalid interface member"))?;
    let mut selected = None;
    let mut explicit = false;
    for owner in &chain[anchor..] {
        for method in &module.functions {
            if method.interface_implementations.is_empty()
                || !method
                    .owner
                    .as_ref()
                    .and_then(|t| module.type_definition(t))
                    .zip(module.type_definition(owner))
                    .is_some_and(|(a, b)| std::ptr::eq(a, b))
            {
                continue;
            }
            let body = method.map_types(|ty| ty.substitute_type_parameters(arguments(owner)))?;
            for target in &body.interface_implementations {
                let declaration = crate::vm::resolve(module, target)?;
                if declaration.definition == contract.definition
                    && declaration.owner == contract.owner
                {
                    if explicit {
                        return Err(Fault::new(
                            "ambiguous explicit interface mapping after substitution",
                        ));
                    }
                    selected = Some(body.clone());
                    explicit = true;
                }
            }
        }
        if explicit {
            break;
        }
        let target = FunctionRef {
            definition: None,
            name: format!("{}.{name}", owner.definition_name().unwrap()),
            owner: Some(owner.clone()),
            instance: true,
            parameters: contract.parameters.clone(),
        };
        if let Ok(function) = crate::vm::resolve(module, &target) {
            if function.interface_implementations.is_empty()
                && function.visibility == Visibility::Public
            {
                selected = Some(function);
                break;
            }
        }
    }
    let implementation =
        selected.ok_or_else(|| Fault::new("interface implementation member not found"))?;
    check_signature(module, &implementation, contract, explicit)?;
    Ok(implementation)
}

fn check_signature(
    module: &Module,
    implementation: &Function,
    contract: &Function,
    explicit: bool,
) -> Result<(), Fault> {
    if implementation.parameters != contract.parameters {
        return Err(Fault::new(
            "interface implementation requires exact parameter types",
        ));
    }
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
        || (!explicit && implementation.visibility != Visibility::Public)
        || implementation.is_internal_call()
        || implementation.pinvoke.is_some()
        || is_contract(module, implementation)
    {
        return Err(Fault::new(
            "interface implementation requires matching IL receiver, parameter and return contracts",
        ));
    }
    Ok(())
}

pub(crate) fn ensure_implementation(
    module: &Module,
    concrete: &Type,
    interface: &Type,
) -> Result<(), Fault> {
    declared(module, concrete, interface)?;
    if interface_definition(module, concrete).is_ok() {
        return Ok(());
    }
    for inherited in closure(module, interface)? {
        let definition = interface_definition(module, &inherited)?;
        for method in &module.functions {
            if method
                .owner
                .as_ref()
                .and_then(|ty| module.type_definition(ty))
                .is_some_and(|d| std::ptr::eq(d, definition))
            {
                let contract =
                    method.map_types(|ty| ty.substitute_type_parameters(arguments(&inherited)))?;
                let implementation = member(module, concrete, &contract)?;
                if !module
                    .type_definition(concrete)
                    .is_some_and(|d| d.is_abstract)
                    && implementation.is_virtual
                {
                    crate::inheritance::dispatch(module, concrete, &implementation)?;
                }
            }
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
    let mapped = member(module, concrete, contract)?;
    if mapped.is_virtual {
        crate::inheritance::dispatch(module, concrete, &mapped)
    } else {
        Ok(mapped)
    }
}

fn flags<T: Ord + Copy>(values: &[T]) -> std::collections::BTreeSet<T> {
    values.iter().copied().collect()
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    let mut mappings = std::collections::HashSet::new();
    for body in &module.functions {
        if body.interface_implementations.is_empty() {
            continue;
        }
        let owner = body
            .owner
            .as_ref()
            .ok_or_else(|| Fault::new("explicit implementation requires a record owner"))?;
        let definition = module
            .type_definition(owner)
            .ok_or_else(|| Fault::new("unknown explicit implementation owner"))?;
        if definition.representation != Representation::Record
            || !body.instance
            || !body.receiver_byref
            || body.visibility != Visibility::Private
            || body.is_virtual
            || body.is_override
            || body.is_abstract
            || body.name.ends_with(".ctor")
            || body.is_internal_call()
            || body.pinvoke.is_some()
        {
            return Err(Fault::new(
                "explicit implementations require private concrete managed IL record methods",
            ));
        }
        for target in &body.interface_implementations {
            let contract = crate::vm::resolve(module, target)?;
            if !is_contract(module, &contract) {
                return Err(Fault::new(
                    "explicit implementation must map an interface declaration",
                ));
            }
            let interface = contract.owner.as_ref().unwrap();
            let mut declared_here = false;
            for declared in &definition.implements {
                if closure(module, declared)?.contains(interface) {
                    declared_here = true;
                }
            }
            if !declared_here {
                return Err(Fault::new(
                    "explicit implementation requires interface conformance declared on its owner",
                ));
            }
            if !mappings.insert((
                owner.clone(),
                contract.definition.clone(),
                interface.clone(),
            )) {
                return Err(Fault::new("duplicate explicit interface mapping"));
            }
            check_signature(module, body, &contract, true)?;
        }
    }
    for definition in &module.types {
        if definition.representation == Representation::Interface
            && (!definition.fields.is_empty()
                || definition.packing.is_some()
                || definition.minimum_size.is_some())
        {
            return Err(Fault::new("interfaces cannot declare storage or layout"));
        }
        let inherited = closure(module, &definition.open_type())?;
        if definition.representation == Representation::Interface {
            let mut contracts: Vec<Function> = Vec::new();
            for interface in inherited {
                let owner = interface_definition(module, &interface)?;
                for method in &module.functions {
                    if !method
                        .owner
                        .as_ref()
                        .and_then(|ty| module.type_definition(ty))
                        .is_some_and(|d| std::ptr::eq(d, owner))
                    {
                        continue;
                    }
                    let contract = method
                        .map_types(|ty| ty.substitute_type_parameters(arguments(&interface)))?;
                    for previous in &contracts {
                        if previous.name.rsplit('.').next() == contract.name.rsplit('.').next()
                            && previous.parameters == contract.parameters
                            && (previous.returns != contract.returns
                                || previous.receiver_byref != contract.receiver_byref
                                || previous.receiver_readonly != contract.receiver_readonly
                                || flags(&previous.out_parameters)
                                    != flags(&contract.out_parameters)
                                || flags(&previous.out_when_true) != flags(&contract.out_when_true)
                                || flags(&previous.readonly_parameters)
                                    != flags(&contract.readonly_parameters))
                        {
                            return Err(Fault::new("incompatible inherited interface contracts"));
                        }
                    }
                    contracts.push(contract);
                }
            }
        }
        let mut seen = std::collections::HashSet::new();
        for interface in &definition.implements {
            if !seen.insert(interface) {
                return Err(Fault::new("duplicate interface implementation"));
            }
        }
        for interface in closure(module, &definition.open_type())? {
            if interface != definition.open_type() {
                ensure_implementation(module, &definition.open_type(), &interface)?;
            }
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
            (Type::Ptr(a), Type::Ptr(b))
            | (Type::InterfaceRef(a), Type::InterfaceRef(b))
            | (Type::ByRef(a), Type::ByRef(b))
            | (Type::ReadOnlyByRef(a), Type::ReadOnlyByRef(b))
            | (Type::Array(a), Type::Array(b)) => infer(a, b, bindings),
            _ => pattern == actual,
        }
    }
    let interface = contract
        .owner
        .as_ref()
        .ok_or_else(|| Fault::new("interface call requires owner"))?;
    let mut targets = vec![];
    for definition in &module.types {
        if definition.representation == Representation::Interface || definition.is_abstract {
            continue;
        }
        for implemented in &closure(module, &definition.open_type())? {
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
