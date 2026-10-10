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

/// Return a retained nominal type's logical module descriptor recipe.
/// This unstable backend bridge adds no loading or callable execution capability.
pub fn type_module(module: &Module, owner: &Type) -> Result<Value, Fault> {
    crate::reflection::Query::Module.invoke_profile(
        module,
        &[Value::RuntimeTypeHandle(Box::new(crate::type_identity::describe_loaded(module, owner)?))],
        &Limits::default(),
        true,
    )
}

/// Return the owning assembly recipe after validating exact logical module ownership.
pub fn module_assembly(module: &Module, identity: &str, name: &str) -> Result<Value, Fault> {
    crate::assembly_info::Query::ModuleAssembly.invoke(
        module,
        &[Value::String(identity.into()), Value::String(name.into())],
        &Limits::default(),
    )
}

/// Return the simple name of an assembly in the loaded descriptive catalog.
pub fn assembly_name(module: &Module, identity: &str) -> Result<Value, Fault> {
    crate::assembly_info::Query::Name.invoke(
        module,
        &[Value::String(identity.into())],
        &Limits::default(),
    )
}

/// Return a complete explicit logical module catalog, including empty declarations.
/// Legacy namespace projections cannot establish completeness and are rejected.
pub fn assembly_modules(module: &Module, identity: &str) -> Result<Value, Fault> {
    if crate::assembly_info::lookup(module, identity)?.declaration_modules.is_none() {
        return Err(Fault::new("native module catalog requires explicit declaration metadata"));
    }
    crate::assembly_info::Query::Modules.invoke(
        module,
        &[Value::String(identity.into())],
        &Limits::default(),
    )
}

/// Select free-function definition identities in metadata order, without invocation.
/// Requires an explicit assembly-local module catalog, not a legacy projection.
/// This unstable backend bridge grants no AOT body or attribute retention rights.
pub fn module_function_definitions(
    module: &Module,
    identity: &str,
    name: &str,
) -> Result<Vec<crate::metadata::MemberId>, Fault> {
    if crate::assembly_info::lookup(module, identity)?.declaration_modules.is_none() {
        return Err(Fault::new("native module function discovery requires explicit declaration metadata"));
    }
    crate::assembly_info::module_functions(module, identity, name)?
        .into_iter()
        .map(|function| function.definition.clone()
            .ok_or_else(|| Fault::new("module function requires definition identity")))
        .collect()
}

/// Describe the assembly of a verified lexical caller, using the VM facade rules.
pub fn executing_assembly(
    module: &Module,
    facade: &crate::metadata::Function,
    caller: &crate::metadata::Function,
) -> Result<Value, Fault> {
    let identity = crate::assembly_info::executing_assembly(module, [facade, caller].into_iter())
        .ok_or_else(|| Fault::new("ExecutingAssembly requires source caller metadata"))?;
    crate::assembly_info::assembly_value(module, &identity)
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
        .generic_arguments
        .iter()
        .map(|handle| handle_type(module, handle))
        .collect()
}

/// Classify a scoped built-in collection and its ordinary concrete implementation.
pub fn collection_shape(
    module: &Module,
    service: &crate::metadata::Function,
    owner: &Type,
) -> Result<Option<(i32, Type, Type)>, Fault> {
    crate::reflection_collections::shape(module, service, owner)
}

/// Plan a call to source-owned generic collection code without executing it.
pub fn collection_plan(
    module: &Module,
    service: &crate::metadata::Function,
    owner: &Type,
    operation: u8,
) -> Result<crate::metadata::Function, Fault> {
    crate::reflection_collections::plan(module, service, owner, operation)
}

/// Validate the private collection service before generating any native dispatch.
pub fn validate_collection_service(
    module: &crate::Module,
    service: &crate::metadata::Function,
    operation: u8,
) -> Result<(), crate::Fault> {
    crate::reflection_collections::validate_service(module, service, operation)
}

/// Return declared custom-attribute data without constructing attribute instances.
pub fn custom_attributes(module: &Module, owner: &Type, token: i32) -> Result<Value, Fault> {
    crate::reflection::custom_attribute_snapshot(module, owner, token)
}

/// Return the original module-scoped metadata token, not a native layout ordinal.
pub fn type_token(module: &Module, owner: &Type) -> Result<i32, Fault> {
    let handle = crate::type_identity::describe_loaded(module, owner)?;
    crate::metadata_tokens::type_token(module, &handle.identity)
}
