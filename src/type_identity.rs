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
    Ptr(Box<TypeIdentity>),
    // Bootstrap signatures remain distinct until their library migration.
    Ref(Box<TypeIdentity>),
    Option(Box<TypeIdentity>),
    Result(Box<TypeIdentity>, Box<TypeIdentity>),
}

/// Resolve a closed signature with the bundled System library, without executing IL.
pub fn resolve_type_identity(module: &Module, ty: &Type) -> Result<TypeIdentity, Fault> {
    if module.name == "System" {
        crate::vm::validate(module)?;
        let mut normalized = module.clone();
        normalized.normalize_definition_ids()?;
        resolve(&normalized, ty)
    } else {
        resolve_type_identity_with_library(module, crate::library::system()?, ty)
    }
}

/// Resolve using the same application/System linking rules as execution.
pub fn resolve_type_identity_with_library(
    module: &Module,
    library: &Module,
    ty: &Type,
) -> Result<TypeIdentity, Fault> {
    resolve(&crate::library::link(module, library)?, ty)
}

fn resolve(module: &Module, ty: &Type) -> Result<TypeIdentity, Fault> {
    // Enforce canonical signatures, arity, closedness, and the shared nesting limit.
    crate::vm::check_type(ty, module)?;
    build(module, ty)
}

fn build(module: &Module, ty: &Type) -> Result<TypeIdentity, Fault> {
    let nested = |ty: &Type| build(module, ty).map(Box::new);
    Ok(match ty {
        Type::Ptr(t) => TypeIdentity::Ptr(nested(t)?),
        Type::Ref(t) => TypeIdentity::Ref(nested(t)?),
        Type::Option(t) => TypeIdentity::Option(nested(t)?),
        Type::Result(t, e) => TypeIdentity::Result(nested(t)?, nested(e)?),
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
                .find(|d| d.name == name)
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
