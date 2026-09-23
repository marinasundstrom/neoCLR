//! Record ancestry, field layout and managed-receiver dispatch contracts.
use crate::{
    metadata::{Field, Representation, Type},
    Fault, Module,
};

pub(crate) fn base(module: &Module, ty: &Type) -> Result<Option<Type>, Fault> {
    let Some(definition) = module.type_definition(ty) else {
        return Ok(None);
    };
    let arguments = match ty {
        Type::Constructed { arguments, .. } => arguments.as_slice(),
        _ => &[],
    };
    definition
        .base
        .as_ref()
        .map(|base| base.substitute_type_parameters(arguments))
        .transpose()
}

/// Derived-first chain. Reject definition cycles even if generic arguments expand.
pub(crate) fn lineage(module: &Module, ty: &Type) -> Result<Vec<Type>, Fault> {
    let mut chain = Vec::new();
    let mut current = Some(ty.clone());
    while let Some(ty) = current {
        let definition = module
            .type_definition(&ty)
            .ok_or_else(|| Fault::new("unknown base type"))?;
        if definition.representation != Representation::Record {
            return Err(Fault::new("base inheritance requires record definitions"));
        }
        if chain.len() >= 64
            || chain
                .iter()
                .any(|t: &Type| t.definition_name() == ty.definition_name())
        {
            return Err(Fault::new("cyclic or excessively deep base inheritance"));
        }
        current = base(module, &ty)?;
        chain.push(ty);
    }
    Ok(chain)
}

pub(crate) fn fields(module: &Module, ty: &Type) -> Result<Vec<Field>, Fault> {
    let mut fields = Vec::new();
    for owner in lineage(module, ty)?.into_iter().rev() {
        let definition = module.type_definition(&owner).unwrap();
        let arguments = match &owner {
            Type::Constructed { arguments, .. } => arguments.as_slice(),
            _ => &[],
        };
        for field in &definition.fields {
            if fields.iter().any(|f: &Field| f.name == field.name) {
                return Err(Fault::new(
                    "inherited field names cannot be hidden in this preview",
                ));
            }
            fields.push(Field {
                visibility: field.visibility,
                name: field.name.clone(),
                ty: field.ty.substitute_type_parameters(arguments)?,
            });
        }
    }
    Ok(fields)
}

pub(crate) fn field_owner(
    module: &Module,
    ty: &Type,
    mut index: usize,
) -> Result<(Type, usize), Fault> {
    for owner in lineage(module, ty)?.into_iter().rev() {
        let count = module.type_definition(&owner).unwrap().fields.len();
        if index < count {
            return Ok((owner, index));
        }
        index -= count;
    }
    Err(Fault::new("field index out of range"))
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    for definition in &module.types {
        if definition.is_abstract && definition.representation != Representation::Record {
            return Err(Fault::new("abstract applies only to record definitions"));
        }
        if definition.base.is_none() {
            continue;
        }
        if !definition.is_reference_type
            && module.functions.iter().any(|f| {
                f.name.ends_with("..ctor")
                    && !f.receiver_byref
                    && f.owner
                        .as_ref()
                        .and_then(|t| module.type_definition(t))
                        .is_some_and(|d| std::ptr::eq(d, definition))
            })
        {
            return Err(Fault::new(
                "derived constructors require managed byref receivers",
            ));
        }
        let chain = lineage(module, &definition.open_type())?;
        fields(module, &definition.open_type())?;
        for base in chain.iter().skip(1) {
            let parent = module.type_definition(base).unwrap();
            if module.functions.iter().any(|f| {
                f.instance
                    && !f.receiver_byref
                    && !parent.is_reference_type
                    && f.owner
                        .as_ref()
                        .and_then(|t| module.type_definition(t))
                        .is_some_and(|d| std::ptr::eq(d, parent))
            }) {
                return Err(Fault::new(
                    "inherited value receivers are not supported yet",
                ));
            }
        }
    }
    validate_methods(module)?;
    for definition in &module.types {
        if definition.representation != Representation::Record || definition.is_abstract {
            continue;
        }
        let concrete = definition.open_type();
        for owner in lineage(module, &concrete)? {
            let args = match &owner {
                Type::Constructed { arguments, .. } => arguments.as_slice(),
                _ => &[],
            };
            for method in &module.functions {
                if method.is_abstract
                    && method
                        .owner
                        .as_ref()
                        .and_then(|t| module.type_definition(t))
                        .zip(module.type_definition(&owner))
                        .is_some_and(|(a, b)| std::ptr::eq(a, b))
                {
                    let contract = method.map_types(|t| t.substitute_type_parameters(args))?;
                    dispatch(module, &concrete, &contract)?;
                }
            }
        }
    }
    Ok(())
}

/// Only same-type or ancestor projections; no downcasts or value slicing.
pub(crate) fn require_base(module: &Module, from: &Type, to: &Type) -> Result<(), Fault> {
    if lineage(module, from)?.contains(to) {
        Ok(())
    } else {
        Err(Fault::new(
            "castclass requires the same record type or an ancestor",
        ))
    }
}

fn declared_method(
    module: &Module,
    owner: &Type,
    contract: &crate::metadata::Function,
) -> Result<Option<crate::metadata::Function>, Fault> {
    let definition = module
        .type_definition(owner)
        .ok_or_else(|| Fault::new("unknown method owner"))?;
    let arguments = match owner {
        Type::Constructed { arguments, .. } => arguments.as_slice(),
        _ => &[],
    };
    for method in &module.functions {
        if method.instance
            && method.interface_implementations.is_empty()
            && method
                .owner
                .as_ref()
                .and_then(|t| module.type_definition(t))
                .is_some_and(|d| std::ptr::eq(d, definition))
        {
            let candidate = method.map_types(|ty| ty.substitute_type_parameters(arguments))?;
            if candidate.name.rsplit('.').next() == contract.name.rsplit('.').next()
                && candidate.parameters == contract.parameters
            {
                return Ok(Some(candidate));
            }
        }
    }
    Ok(None)
}
// Value payloads have no Object storage ancestry. Only explicit overrides may
// adapt a System.Object slot to a managed reference into the box.
fn value_object_contract(
    module: &Module,
    owner: &Type,
    method: &crate::metadata::Function,
) -> Result<Option<crate::metadata::Function>, Fault> {
    if module.type_definition(owner).is_none_or(|d| {
        d.is_reference_type
            || d.representation != Representation::Record
            || d.base.is_some()
            || !d.generic_parameters.is_empty()
    }) {
        return Ok(None);
    }
    let object = Type::from_name("System.Object");
    if module.type_definition(&object).is_none() {
        return Ok(None);
    }
    Ok(declared_method(module, &object, method)?.filter(|f| {
        f.definition
            .as_ref()
            .is_some_and(|id| id.module == "System")
            && f.is_virtual
            && !f.receiver_byref
            && f.generic_parameters.is_empty()
    }))
}

fn validate_methods(module: &Module) -> Result<(), Fault> {
    fn flags(values: &[usize]) -> std::collections::BTreeSet<usize> {
        values.iter().copied().collect()
    }
    for method in &module.functions {
        if crate::interfaces::is_contract(module, method)
            || crate::delegates::is_contract(module, method)
        {
            continue;
        }
        if method.is_abstract
            && (!method.is_virtual
                || !method.body.is_empty()
                || !method.locals.is_empty()
                || method
                    .owner
                    .as_ref()
                    .and_then(|t| module.type_definition(t))
                    .is_none_or(|d| !d.is_abstract))
        {
            return Err(Fault::new(
                "abstract methods require an abstract record and no body or locals",
            ));
        }
        if (method.is_virtual || method.is_override)
            && (!method.is_virtual
                || !method.instance
                || (!method.receiver_byref
                    && !method
                        .owner
                        .as_ref()
                        .is_some_and(|owner| module.is_reference_type(owner)))
                || method.visibility != crate::metadata::Visibility::Public
                || method.name.ends_with("..ctor")
                || method.is_internal_call()
                || method.pinvoke.is_some()
                || method
                    .owner
                    .as_ref()
                    .and_then(|t| module.type_definition(t))
                    .is_none_or(|d| d.representation != Representation::Record))
        {
            return Err(Fault::new(
                "virtual and override require public IL record methods with managed receivers",
            ));
        }
        let Some(owner) = &method.owner else {
            continue;
        };
        if !method.instance
            || !method.interface_implementations.is_empty()
            || module
                .type_definition(owner)
                .is_none_or(|d| d.representation != Representation::Record)
        {
            continue;
        }
        let mut inherited = None;
        for base in lineage(module, owner)?.iter().skip(1) {
            if let Some(candidate) = declared_method(module, base, method)? {
                inherited = Some(candidate);
                break;
            }
        }
        let boxed_parent = if method.is_override && inherited.is_none() {
            value_object_contract(module, owner, method)?
        } else {
            None
        };
        let adapted_receiver = boxed_parent.is_some() && method.receiver_byref;
        if method.is_override {
            let parent = inherited
                .or(boxed_parent)
                .ok_or_else(|| Fault::new("override requires an inherited virtual method"))?;
            if !parent.is_virtual
                || (adapted_receiver && !method.generic_parameters.is_empty())
                || parent.returns != method.returns
                || parent.no_result != method.no_result
                || (!adapted_receiver && parent.receiver_readonly != method.receiver_readonly)
                || (!adapted_receiver && parent.receiver_byref != method.receiver_byref)
                || flags(&parent.out_parameters) != flags(&method.out_parameters)
                || flags(&parent.out_when_true) != flags(&method.out_when_true)
                || flags(&parent.readonly_parameters) != flags(&method.readonly_parameters)
            {
                return Err(Fault::new(
                    "override must preserve the exact inherited virtual contract",
                ));
            }
        } else if inherited.is_some() && !method.name.ends_with("..ctor") {
            return Err(Fault::new(
                "inherited method hiding is unsupported; use override for virtual methods",
            ));
        }
    }
    Ok(())
}

pub(crate) fn dispatch(
    module: &Module,
    concrete: &Type,
    contract: &crate::metadata::Function,
) -> Result<crate::metadata::Function, Fault> {
    if !contract.is_virtual {
        return Err(Fault::new("class callvirt requires a virtual method"));
    }
    let owner = contract
        .owner
        .as_ref()
        .ok_or_else(|| Fault::new("virtual method requires owner"))?;
    if owner == &Type::from_name("System.Object")
        && value_object_contract(module, concrete, contract)?.is_some()
    {
        if let Some(candidate) = declared_method(module, concrete, contract)? {
            if candidate.is_override && !candidate.is_abstract {
                return Ok(candidate);
            }
        }
        return Err(Fault::new("boxed value has no explicit Object override"));
    }
    // Object is an admitted common reference view even for a nominal class or
    // managed array without explicit Object ancestry. Such a view uses Object's
    // default slot; only declared ancestry can contribute an override.
    if owner == &Type::from_name("System.Object")
        && module.is_reference_type(concrete)
        && require_base(module, concrete, owner).is_err()
    {
        if contract.is_abstract {
            return Err(Fault::new(
                "concrete type has an unimplemented abstract method",
            ));
        }
        return Ok(contract.clone());
    }
    require_base(module, concrete, owner)?;
    for ty in lineage(module, concrete)? {
        if let Some(candidate) = declared_method(module, &ty, contract)? {
            if candidate.is_abstract {
                return Err(Fault::new(
                    "concrete type has an unimplemented abstract method",
                ));
            }
            return Ok(candidate);
        }
        if &ty == owner {
            break;
        }
    }
    Err(Fault::new("virtual implementation not found"))
}

pub(crate) fn dispatch_targets(
    module: &Module,
    contract: &crate::metadata::Function,
) -> Result<Vec<crate::metadata::Function>, Fault> {
    // Rootless classes and arrays can reach a concrete Object default even when
    // Object itself is abstract and no declared subclass is in this module.
    let mut targets = if contract.owner.as_ref() == Some(&Type::from_name("System.Object"))
        && !contract.is_abstract
    {
        vec![contract.clone()]
    } else {
        Vec::new()
    };
    for definition in &module.types {
        if definition.representation != Representation::Record || definition.is_abstract {
            continue;
        }
        let owner = definition.open_type();
        // Open generic class target inference is deliberately not advertised as closed.
        if !definition.generic_parameters.is_empty() {
            if lineage(module, &owner)?.iter().any(|t| {
                t.definition_name() == contract.owner.as_ref().and_then(Type::definition_name)
            }) {
                return Err(Fault::new(
                    "closed class dispatch graph cannot infer generic implementers yet",
                ));
            }
            continue;
        }
        let boxed_override = contract.owner.as_ref() == Some(&Type::from_name("System.Object"))
            && value_object_contract(module, &owner, contract)?.is_some()
            && declared_method(module, &owner, contract)?.is_some_and(|m| m.is_override);
        if boxed_override || require_base(module, &owner, contract.owner.as_ref().unwrap()).is_ok()
        {
            let target = dispatch(module, &owner, contract)?;
            if !targets.iter().any(|f: &crate::metadata::Function| {
                f.definition == target.definition && f.owner == target.owner
            }) {
                targets.push(target);
            }
        }
    }
    Ok(targets)
}

pub(crate) fn require_concrete(module: &Module, ty: &Type) -> Result<(), Fault> {
    if module.type_definition(ty).is_some_and(|d| d.is_abstract) {
        Err(Fault::new("cannot instantiate an abstract record"))
    } else {
        Ok(())
    }
}
