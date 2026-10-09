//! Unstable build-time bridge for the in-repository native compiler experiment.
//! Produces trusted snapshot recipes through the same metadata queries as the VM.
//! It does not invoke user code or grant executable reflection rights.
use crate::{Fault, Limits, Module, Value, metadata::Type};

/// Return the logical property snapshot recipe, before runtime allocation.
pub fn properties(module: &Module, owner: &Type) -> Result<Value, Fault> {
    crate::reflection::Query::Properties.invoke_profile(
        module,
        &[
            Value::RuntimeTypeHandle(Box::new(crate::type_identity::describe_loaded(
                module, owner,
            )?)),
            Value::Int32(60),
        ],
        &Limits::default(),
        true,
    )
}

/// Return the optional logical element descriptor recipe.
pub fn element_type(module: &Module, owner: &Type) -> Result<Value, Fault> {
    crate::reflection::Query::ElementType.invoke_profile(
        module,
        &[Value::RuntimeTypeHandle(Box::new(
            crate::type_identity::describe_loaded(module, owner)?,
        ))],
        &Limits::default(),
        true,
    )
}

/// Resolve a recipe handle to its source signature, without publishing native ordinals.
pub fn handle_type(module: &Module, handle: &crate::TypeDescriptor) -> Result<Type, Fault> {
    crate::reflection::from_identity(module, &handle.identity)
}

/// Resolve the exact scoped runtime provider used by interpreter materialization.
pub fn provider(module: &Module, contract: &Type, fields: &[Value]) -> Result<Type, Fault> {
    crate::reflection_source::provider(module, contract, fields)
}

/// Resolve an inherited interface target using the VM's conformance anchor rules.
pub fn interface_implementation(
    module: &Module,
    concrete: &Type,
    contract: &crate::metadata::Function,
) -> Result<crate::metadata::Function, Fault> {
    let interface = contract
        .owner
        .as_ref()
        .ok_or_else(|| Fault::new("missing interface owner"))?;
    crate::interfaces::implementation(module, concrete, interface, contract)
}

/// Resolve closed generic argument handles using the interpreter's type identity.
pub fn generic_arguments(module: &Module, owner: &Type) -> Result<Vec<Type>, Fault> {
    crate::type_identity::describe_loaded(module, owner)?
        .generic_arguments.iter()
        .map(|handle| handle_type(module, handle)).collect()
}
