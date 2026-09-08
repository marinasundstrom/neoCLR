//! Check source module scopes before lowering to the prototype's unique-name keys.
use crate::{
    Fault, Module,
    metadata::{CustomAttribute, Type},
};

pub(crate) fn normalize_type(context: &Module, ty: &Type) -> Result<Type, Fault> {
    fn bind(context: &Module, ty: &Type, depth: usize) -> Result<Type, Fault> {
        if depth > 32 {
            return Err(Fault::new("type nesting exceeds 32"));
        }
        let nested = |ty: &Type| bind(context, ty, depth + 1);
        Ok(match ty {
            Type::Scoped {
                module,
                name,
                arguments,
            } => {
                if module.is_empty() || name.is_empty() {
                    return Err(Fault::new("empty scoped type name"));
                }
                let canonical = Type::from_name(name);
                if canonical.is_primitive() && canonical.definition_name() != Some(name.as_str()) {
                    return Err(Fault::new("scoped primitive requires canonical type name"));
                }
                let origin = if canonical.is_primitive() {
                    "System"
                } else {
                    context
                        .types
                        .iter()
                        .find(|d| &d.name == name && d.generic_parameters.len() == arguments.len())
                        .and_then(|d| d.definition.as_ref())
                        .map(|d| d.module.as_str())
                        .ok_or_else(|| {
                            Fault::new(format!("unknown scoped type [{module}]{name}"))
                        })?
                };
                if origin != module {
                    return Err(Fault::new(format!(
                        "type {name} belongs to {origin}, not {module}"
                    )));
                }
                if arguments.is_empty() {
                    canonical
                } else {
                    Type::Constructed {
                        definition: name.clone(),
                        arguments: arguments.iter().map(nested).collect::<Result<_, _>>()?,
                    }
                }
            }
            Type::Constructed {
                definition,
                arguments,
            } => Type::Constructed {
                definition: definition.clone(),
                arguments: arguments.iter().map(nested).collect::<Result<_, _>>()?,
            },
            Type::ByRef(t) => Type::ByRef(Box::new(nested(t)?)),
            Type::ReadOnlyByRef(t) => Type::ReadOnlyByRef(Box::new(nested(t)?)),
            Type::Array(t) => Type::Array(Box::new(nested(t)?)),
            Type::Ptr(t) => Type::Ptr(Box::new(nested(t)?)),
            Type::InterfaceRef(t) => Type::InterfaceRef(Box::new(nested(t)?)),
            other => other.clone(),
        })
    }
    bind(context, ty, 0)
}

fn attributes(context: &Module, attributes: &mut [CustomAttribute]) -> Result<(), Fault> {
    for attribute in attributes {
        let target = &mut attribute.constructor;
        if let Some(owner) = &mut target.owner {
            *owner = normalize_type(context, owner)?;
        }
        for ty in &mut target.parameters {
            *ty = normalize_type(context, ty)?;
        }
    }
    Ok(())
}

pub(crate) fn normalize_module(context: &Module, source: &Module) -> Result<Module, Fault> {
    let mut result = source.clone();
    for definition in &mut result.types {
        for ty in &mut definition.implements {
            *ty = normalize_type(context, ty)?;
        }
        for property in &mut definition.properties {
            property.map_types(|ty| normalize_type(context, ty))?;
        }
        for field in &mut definition.fields {
            field.ty = normalize_type(context, &field.ty)?;
        }
        attributes(context, &mut definition.custom_attributes)?;
    }
    for function in &mut result.functions {
        *function = function.map_types(|ty| normalize_type(context, ty))?;
        attributes(context, &mut function.custom_attributes)?;
    }
    Ok(result)
}
