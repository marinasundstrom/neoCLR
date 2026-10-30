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
        let normalized = match ty {
            Type::Tuple(parts) => {
                if parts.is_empty() || parts.len() > 256 {
                    return Err(Fault::new("tuple signature requires one to 256 elements"));
                }
                Type::Tuple(parts.map_types(nested)?)
            }
            Type::Union(parts) | Type::Intersection(parts) => {
                crate::compositions::check_shape(ty)?;
                // Validate constituents before singleton collapse can conceal a value type.
                let parts = parts.iter().map(nested).collect::<Result<Vec<_>, _>>()?;
                let candidate = if matches!(ty, Type::Union(_)) {
                    Type::Union(parts.clone())
                } else {
                    Type::Intersection(parts.clone())
                };
                // The assembler also binds field owners against a partial context.
                // Preserve the expression until library/dependency definitions exist;
                // otherwise singleton collapse could erase a later-invalid operand.
                if crate::compositions::resolved(context, &candidate) {
                    // Binding has not reached the owning signature yet. As with
                    // nominal generic construction, retain scoped arguments here;
                    // signature validation checks their actual owner arity later.
                    crate::compositions::validate_in_context(
                        context,
                        &candidate,
                        crate::vm::SignatureContext {
                            types: u16::MAX as usize + 1,
                            methods: u16::MAX as usize + 1,
                            allow_self: false,
                        },
                    )?;
                    crate::compositions::normalized(matches!(ty, Type::Union(_)), parts)
                } else {
                    candidate
                }
            }
            Type::FunctionReturn(operand) => Type::function_return(nested(operand)?)?,
            Type::Function(shape) => Type::Function(Box::new(shape.map_types(nested)?)),
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
                        .find(|d| {
                            &d.name == name
                                && (arguments.is_empty()
                                    || d.generic_parameters.len() == arguments.len())
                        })
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
            Type::ArrayRef(t) => Type::ArrayRef(Box::new(nested(t)?)),
            Type::Ptr(t) => Type::Ptr(Box::new(nested(t)?)),
            Type::InterfaceRef(t) => Type::InterfaceRef(Box::new(nested(t)?)),
            other => other.clone(),
        };
        canonical_array(context, normalized)
    }
    bind(context, ty, 0)
}

// Keep CLI array signatures and the generic library name on one storage identity.
fn canonical_array(context: &Module, ty: Type) -> Result<Type, Fault> {
    if let Type::Constructed {
        definition,
        arguments,
    } = &ty
    {
        if definition == "System.Array"
            && arguments.len() == 1
            && context
                .type_definition(&ty)
                .is_some_and(|d| d.is_reference_type)
        {
            return Ok(Type::ArrayRef(Box::new(arguments[0].clone())));
        }
    }
    Ok(ty)
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
    for assembly in &mut result.assemblies {
        attributes(context, &mut assembly.custom_attributes)?;
    }
    for definition in &mut result.types {
        crate::constraints::map_types(&mut definition.generic_constraints, |ty| {
            normalize_type(context, ty)
        })?;
        if let Some(base) = &mut definition.base {
            *base = normalize_type(context, base)?;
        }
        for ty in &mut definition.implements {
            *ty = normalize_type(context, ty)?;
        }
        for property in &mut definition.properties {
            property.map_types(|ty| normalize_type(context, ty))?;
        }
        for field in &mut definition.static_fields {
            field.ty = normalize_type(context, &field.ty)?;
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
