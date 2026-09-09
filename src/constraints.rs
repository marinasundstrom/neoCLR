//! Generic contracts are checked again after concrete substitution, including IL calls.
use crate::{
    Fault,
    metadata::{ConstraintKind, GenericConstraint, Type},
};

pub(crate) fn parse(text: &str, names: &[Option<String>]) -> Result<Vec<GenericConstraint>, Fault> {
    let mut words = text.split_whitespace();
    let parameter = words
        .next()
        .ok_or_else(|| Fault::new("constraint requires a parameter"))?;
    let parameter = names
        .iter()
        .position(|name| name.as_deref() == Some(parameter))
        .or_else(|| {
            parameter
                .parse::<usize>()
                .ok()
                .filter(|index| *index < names.len())
        })
        .ok_or_else(|| Fault::new("unknown constraint parameter"))? as u16;
    let constraints = words
        .map(|word| {
            let kind = match word {
                "notvoid" => ConstraintKind::NotVoid,
                "notreference" => ConstraintKind::NotReference,
                _ => ConstraintKind::TypeBound(crate::assembler::parse_type(word)?),
            };
            Ok(GenericConstraint { parameter, kind })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if constraints.is_empty() {
        return Err(Fault::new("constraint requires a restriction"));
    }
    Ok(constraints)
}

pub(crate) fn validate(constraints: &[GenericConstraint], arity: usize) -> Result<(), Fault> {
    for (index, constraint) in constraints.iter().enumerate() {
        if constraint.parameter as usize >= arity {
            return Err(Fault::new(
                "generic constraint parameter outside declaring context",
            ));
        }
        if constraints[..index].contains(constraint) {
            return Err(Fault::new("duplicate generic constraint"));
        }
    }
    Ok(())
}

pub(crate) fn map_types(
    constraints: &mut [GenericConstraint],
    mut map: impl FnMut(&Type) -> Result<Type, Fault>,
) -> Result<(), Fault> {
    for constraint in constraints {
        if let ConstraintKind::TypeBound(bound) = &mut constraint.kind {
            *bound = map(bound)?;
        }
    }
    Ok(())
}

pub(crate) fn bounds(constraints: &[GenericConstraint]) -> impl Iterator<Item = &Type> {
    constraints.iter().filter_map(|c| match &c.kind {
        ConstraintKind::TypeBound(bound) => Some(bound),
        _ => None,
    })
}

fn symbolic(ty: &Type) -> bool {
    match ty {
        Type::TypeParameter(_) | Type::MethodTypeParameter(_) => true,
        Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } => {
            arguments.iter().any(symbolic)
        }
        Type::ByRef(t)
        | Type::ReadOnlyByRef(t)
        | Type::InterfaceRef(t)
        | Type::Array(t)
        | Type::Ptr(t) => symbolic(t),
        _ => false,
    }
}

pub(crate) fn validate_bounds(
    module: &crate::Module,
    constraints: &[GenericConstraint],
) -> Result<(), Fault> {
    let mut bases = std::collections::HashSet::new();
    for constraint in constraints {
        if let ConstraintKind::TypeBound(bound) = &constraint.kind {
            let definition = module
                .type_definition(bound)
                .filter(|_| matches!(bound, Type::Named(_) | Type::Constructed { .. }))
                .ok_or_else(|| {
                    Fault::new("generic bound requires a nominal record or interface")
                })?;
            match definition.representation {
                crate::metadata::Representation::Interface => (),
                crate::metadata::Representation::Record => {
                    if !bases.insert(constraint.parameter) {
                        return Err(Fault::new(
                            "generic parameter permits at most one base bound",
                        ));
                    }
                }
                _ => {
                    return Err(Fault::new(
                        "generic bound requires a nominal record or interface",
                    ));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn check(
    module: &crate::Module,
    constraints: &[GenericConstraint],
    arguments: &[Type],
) -> Result<(), Fault> {
    for constraint in constraints {
        let argument = arguments
            .get(constraint.parameter as usize)
            .ok_or_else(|| Fault::new("generic constraint argument missing"))?;
        let violates = match &constraint.kind {
            ConstraintKind::NotVoid => matches!(argument, Type::Void),
            ConstraintKind::NotReference => matches!(
                argument,
                Type::ByRef(_) | Type::ReadOnlyByRef(_) | Type::InterfaceRef(_)
            ),
            ConstraintKind::TypeBound(bound) => {
                // Independent of addressing mode. A bound never performs a conversion.
                let concrete = match argument {
                    Type::ByRef(t) | Type::ReadOnlyByRef(t) | Type::InterfaceRef(t) => t.as_ref(),
                    _ => argument,
                };
                if symbolic(concrete) || symbolic(bound) {
                    continue;
                }
                // Partial frontend probes cannot decide application-type conformance.
                // Complete linking validates all referenced types and repeats these checks.
                let Some(definition) = module.type_definition(bound) else {
                    continue;
                };
                if module.type_definition(concrete).is_none()
                    && matches!(
                        concrete,
                        Type::Named(_) | Type::Constructed { .. } | Type::Scoped { .. }
                    )
                {
                    continue;
                }
                if definition.representation == crate::metadata::Representation::Interface {
                    !crate::interfaces::closure(module, concrete)
                        .is_ok_and(|types| types.contains(bound))
                } else {
                    crate::inheritance::require_base(module, concrete, bound).is_err()
                }
            }
        };
        if violates {
            return Err(Fault::new(format!(
                "generic argument {argument:?} violates {:?} constraint on parameter {}",
                constraint.kind, constraint.parameter
            )));
        }
    }
    Ok(())
}

pub(crate) fn check_type_arguments(
    module: &crate::Module,
    constraints: &[GenericConstraint],
    arguments: &[Type],
) -> Result<(), Fault> {
    let mut constraints = constraints.to_vec();
    map_types(&mut constraints, |ty| {
        ty.substitute_type_parameters(arguments)
    })?;
    check(module, &constraints, arguments)
}

pub(crate) fn emit(constraints: &[GenericConstraint], names: &[String]) -> Result<String, Fault> {
    constraints
        .iter()
        .map(|constraint| {
            Ok(format!(
                ".constraint {} {}\n",
                names[constraint.parameter as usize],
                match &constraint.kind {
                    ConstraintKind::NotVoid => "notvoid".to_string(),
                    ConstraintKind::NotReference => "notreference".to_string(),
                    ConstraintKind::TypeBound(bound) =>
                        crate::type_identity::signature_name(bound)?,
                }
            ))
        })
        .collect()
}

// Frontend library probes and field introspection may use partial module contexts.
// Check available contracts here; linking/verification checks complete type existence.
pub(crate) fn check_known_type(
    module: &crate::Module,
    ty: &Type,
    depth: usize,
) -> Result<(), Fault> {
    if depth > 32 {
        return Err(Fault::new("type nesting exceeds 32"));
    }
    match ty {
        Type::Constructed { arguments, .. } => {
            if let Some(definition) = module.type_definition(ty) {
                check_type_arguments(module, &definition.generic_constraints, arguments)?;
                for inherited in definition.base.iter().chain(&definition.implements) {
                    check_known_type(
                        module,
                        &inherited.substitute_type_parameters(arguments)?,
                        depth + 1,
                    )?;
                }
            }
            for argument in arguments {
                check_known_type(module, argument, depth + 1)?;
            }
        }
        Type::ByRef(ty)
        | Type::ReadOnlyByRef(ty)
        | Type::InterfaceRef(ty)
        | Type::Array(ty)
        | Type::Ptr(ty) => check_known_type(module, ty, depth + 1)?,
        _ => (),
    }
    Ok(())
}

// The verifier may use a declared bound to authorize a view of an open parameter.
// Execution still substitutes T and checks the concrete receiver at the view opcode.
pub(crate) fn permits_view(
    module: &crate::Module,
    function: &crate::metadata::Function,
    source: &Type,
    target: &Type,
    interface: bool,
) -> bool {
    let (index, constraints) = match source {
        Type::MethodTypeParameter(index) => (*index, &function.generic_constraints),
        Type::TypeParameter(index) => {
            let Some(owner) = function
                .owner
                .as_ref()
                .and_then(|ty| module.type_definition(ty))
            else {
                return false;
            };
            (*index, &owner.generic_constraints)
        }
        _ => return false,
    };
    constraints.iter().any(|constraint| {
        if constraint.parameter != index {
            return false;
        }
        let ConstraintKind::TypeBound(bound) = &constraint.kind else {
            return false;
        };
        if interface {
            crate::interfaces::closure(module, bound).is_ok_and(|types| types.contains(target))
        } else {
            crate::inheritance::require_base(module, bound, target).is_ok()
        }
    })
}
