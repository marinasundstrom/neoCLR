//! Private execution planning for runtime-backed reflection. No public binder policy.
use crate::{
    Fault, Module, Value,
    metadata::{Function, FunctionRef, Instruction, Representation, Type},
};

/// Private bridge statuses; these are not public ReflectionError cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum ConstructionError {
    UnboundType = 1,
    UnsupportedType = 2,
    AccessDenied = 3,
    MissingConstructor = 4,
}

pub(crate) fn bound_type(module: &Module, handle: &Value) -> Result<Type, ConstructionError> {
    let Value::RuntimeTypeHandle(handle) = handle else {
        return Err(ConstructionError::UnboundType);
    };
    let owner = crate::reflection::from_identity(module, &handle.identity)
        .map_err(|_| ConstructionError::UnboundType)?;
    if crate::type_identity::resolve(module, &owner).map_err(|_| ConstructionError::UnboundType)?
        != handle.identity
    {
        return Err(ConstructionError::UnboundType);
    }
    Ok(owner)
}

pub(crate) fn constructor(
    module: &Module,
    handle: &Value,
) -> Result<FunctionRef, ConstructionError> {
    let owner = bound_type(module, handle)?;
    let definition = module
        .type_definition(&owner)
        .ok_or(ConstructionError::UnsupportedType)?;
    // First slice: concrete, nongeneric reference classes only. No bypass allocation.
    if !definition.is_reference_type
        || definition.representation != Representation::Record
        || definition.is_abstract
        || !definition.generic_parameters.is_empty()
        || !module.reference_assignable(&owner, &Type::from_name("System.Object"))
    {
        return Err(ConstructionError::UnsupportedType);
    }
    let mut target = FunctionRef {
        definition: None,
        name: format!("{}..ctor", definition.name),
        owner: Some(owner.clone()),
        instance: true,
        generic_arguments: vec![],
        parameters: vec![],
    };
    let constructor = crate::vm::resolve_constructor(module, &target)
        .map_err(|_| ConstructionError::MissingConstructor)?;
    if !crate::metadata_origin::reflection_public(module, &owner, &constructor) {
        return Err(ConstructionError::AccessDenied);
    }
    crate::access::check_call(module, None, &constructor)
        .map_err(|_| ConstructionError::AccessDenied)?;
    target.definition = constructor.definition;
    Ok(target)
}

pub(crate) fn check(module: &Module, handle: &Value) -> i32 {
    constructor(module, handle).map_or_else(|error| error as i32, |_| 0)
}

/// Execute through ordinary newobj/ret frames, retaining constructor checks and roots.
/// The public Result wrapper will check first; direct private-service misuse faults.
pub(crate) fn adapter(
    module: &Module,
    service: &Function,
    handle: &Value,
) -> Result<Function, Fault> {
    let target = constructor(module, handle)
        .map_err(|error| Fault::new(format!("reflection construction rejected: {error:?}")))?;
    let mut adapter = service.clone();
    adapter.impl_flags = 0;
    adapter.body = vec![Instruction::Construct(target), Instruction::Return];
    Ok(adapter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TypeIdentity;

    #[test]
    fn identity_binding_rejects_foreign_and_noncanonical_handles() {
        let mut module = crate::assemble(".module Here\n.type class abstract System.Object\n.end\n.type class Model\n.extends System.Object\n.method instance .ctor() -> noresult\nret\n.end\n.end").unwrap();
        module.normalize_definition_ids().unwrap();
        let handle = crate::type_identity::describe(&module, &Type::from_name("Model")).unwrap();
        let mut renamed = handle.clone();
        renamed.name = "Untrusted display name".into();
        assert_eq!(
            check(&module, &Value::RuntimeTypeHandle(Box::new(renamed))),
            0
        );
        let mut foreign = handle.clone();
        let TypeIdentity::Definition { definition, .. } = &mut foreign.identity else {
            panic!()
        };
        definition.module = "Elsewhere".into();
        assert_eq!(
            check(&module, &Value::RuntimeTypeHandle(Box::new(foreign))),
            1
        );
        let mut malformed = handle.clone();
        let TypeIdentity::Definition { arguments, .. } = &mut malformed.identity else {
            panic!()
        };
        arguments.push(handle.identity);
        assert_eq!(
            check(&module, &Value::RuntimeTypeHandle(Box::new(malformed))),
            1
        );
    }
}
