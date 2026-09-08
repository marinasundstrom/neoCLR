//! Resolved closed signature keys, independent of an execution backend or layout.
use crate::{
    Fault, Module,
    metadata::{Type, TypeDefId},
};

/// An identity within the resolved modules of one build, not a cross-build cache key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeIdentity {
    Definition {
        definition: TypeDefId,
        arguments: Vec<TypeIdentity>,
    },
    ByRef(Box<TypeIdentity>),
    ReadOnlyByRef(Box<TypeIdentity>),
    Array(Box<TypeIdentity>),
    Ptr(Box<TypeIdentity>),
    InterfaceRef(Box<TypeIdentity>),
}

/// Read-only type information resolved within one loaded program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDescriptor {
    pub identity: TypeIdentity,
    pub name: String,
    pub generic_arguments: Vec<TypeDescriptor>,
    pub declaring_type: Option<TypeDefId>,
}

pub(crate) fn describe(module: &Module, ty: &Type) -> Result<TypeDescriptor, Fault> {
    let normalized = crate::scope::normalize_type(module, ty)?;
    resolve(module, &normalized)?;
    describe_loaded(module, &normalized)
}

// Introspection follows already validated metadata, including transitive signatures.
// It does not grant permission to name those types in executable instructions.
pub(crate) fn describe_loaded(module: &Module, normalized: &Type) -> Result<TypeDescriptor, Fault> {
    crate::vm::check_type(normalized, module)?;
    let identity = build(module, normalized)?;
    let name = match normalized {
        Type::ByRef(element) => reference_name(element, false)?,
        Type::ReadOnlyByRef(element) => reference_name(element, true)?,
        Type::Array(element) => format!("{}[]", signature_name(element)?),
        Type::Ptr(element) => format!("{}*", signature_name(element)?),
        Type::InterfaceRef(element) => format!("InterfaceRef<{}>", signature_name(element)?),
        _ => normalized
            .definition_name()
            .ok_or_else(|| Fault::new("type has no metadata name"))?
            .to_owned(),
    };
    let generic_arguments = match normalized {
        Type::Constructed { arguments, .. } => arguments
            .iter()
            .map(|argument| describe_loaded(module, argument))
            .collect::<Result<_, _>>()?,
        _ => vec![],
    };
    let declaring_type = module
        .type_definition(normalized)
        .and_then(|definition| definition.declaring_type.clone());
    Ok(TypeDescriptor {
        identity,
        name,
        generic_arguments,
        declaring_type,
    })
}

pub(crate) fn signature_name(ty: &Type) -> Result<String, Fault> {
    Ok(match ty {
        Type::ByRef(element) => reference_name(element, false)?,
        Type::ReadOnlyByRef(element) => reference_name(element, true)?,
        Type::Array(element) => format!("{}[]", signature_name(element)?),
        Type::Ptr(element) => format!("{}*", signature_name(element)?),
        Type::InterfaceRef(element) => format!("InterfaceRef<{}>", signature_name(element)?),
        Type::Constructed {
            definition,
            arguments,
        } => format!(
            "{}<{}>",
            definition,
            arguments
                .iter()
                .map(signature_name)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        ),
        _ => ty
            .definition_name()
            .ok_or_else(|| Fault::new("type has no metadata name"))?
            .to_owned(),
    })
}

/// Resolve a closed signature with the bundled System library, without executing IL.
pub fn resolve_type_identity(module: &Module, ty: &Type) -> Result<TypeIdentity, Fault> {
    crate::LoadedProgram::new(module)?.resolve_type_identity(ty)
}

/// Resolve using the same application/System linking rules as execution.
pub fn resolve_type_identity_with_library(
    module: &Module,
    library: &Module,
    ty: &Type,
) -> Result<TypeIdentity, Fault> {
    crate::LoadedProgram::with_library(module, library)?.resolve_type_identity(ty)
}

pub(crate) fn resolve(module: &Module, ty: &Type) -> Result<TypeIdentity, Fault> {
    let ty = crate::scope::normalize_type(module, ty)?;
    // Enforce canonical signatures, arity, closedness, and the shared nesting limit.
    crate::vm::check_type(&ty, module)?;
    crate::references::check_type(module, module, &ty)?;
    build(module, &ty)
}

fn build(module: &Module, ty: &Type) -> Result<TypeIdentity, Fault> {
    let nested = |ty: &Type| build(module, ty).map(Box::new);
    Ok(match ty {
        Type::ByRef(t) => TypeIdentity::ByRef(nested(t)?),
        Type::ReadOnlyByRef(t) => TypeIdentity::ReadOnlyByRef(nested(t)?),
        Type::Array(t) => TypeIdentity::Array(nested(t)?),
        Type::Ptr(t) => TypeIdentity::Ptr(nested(t)?),
        Type::InterfaceRef(t) => TypeIdentity::InterfaceRef(nested(t)?),
        _ => {
            let (name, arguments) = match ty {
                Type::Constructed {
                    definition,
                    arguments,
                } => (definition.as_str(), arguments.as_slice()),
                _ => (
                    ty.definition_name()
                        .ok_or_else(|| Fault::new("expected closed type signature"))?,
                    &[][..],
                ),
            };
            let definition = module
                .types
                .iter()
                .find(|d| d.name == name && d.generic_parameters.len() == arguments.len())
                .and_then(|d| d.definition.clone())
                .ok_or_else(|| {
                    Fault::new(format!("missing type definition identity for {name}"))
                })?;
            TypeIdentity::Definition {
                definition,
                arguments: arguments
                    .iter()
                    .map(|t| build(module, t))
                    .collect::<Result<_, _>>()?,
            }
        }
    })
}

fn reference_name(target: &Type, readonly: bool) -> Result<String, Fault> {
    let mut name = signature_name(target)?;
    if name.starts_with("readonly ") {
        name = format!("({name})");
    }
    Ok(format!(
        "{}{name}&",
        if readonly { "readonly " } else { "" }
    ))
}
