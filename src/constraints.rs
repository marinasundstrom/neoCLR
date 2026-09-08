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
                _ => {
                    return Err(Fault::new(
                        "unsupported generic constraint; expected notvoid or notreference",
                    ));
                }
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
    let mut seen = vec![0_u8; arity];
    for constraint in constraints {
        let bits = seen
            .get_mut(constraint.parameter as usize)
            .ok_or_else(|| Fault::new("generic constraint parameter outside declaring context"))?;
        let flag = match constraint.kind {
            ConstraintKind::NotVoid => 1,
            ConstraintKind::NotReference => 2,
        };
        if *bits & flag != 0 {
            return Err(Fault::new("duplicate generic constraint"));
        }
        *bits |= flag;
    }
    Ok(())
}

pub(crate) fn check(constraints: &[GenericConstraint], arguments: &[Type]) -> Result<(), Fault> {
    for constraint in constraints {
        let argument = arguments
            .get(constraint.parameter as usize)
            .ok_or_else(|| Fault::new("generic constraint argument missing"))?;
        // Symbolic forwarding is allowed; every concrete use must check again.
        let violates = match constraint.kind {
            ConstraintKind::NotVoid => matches!(argument, Type::Void),
            ConstraintKind::NotReference => matches!(
                argument,
                Type::ByRef(_) | Type::ReadOnlyByRef(_) | Type::InterfaceRef(_)
            ),
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

pub(crate) fn emit(constraints: &[GenericConstraint], names: &[String]) -> String {
    constraints
        .iter()
        .map(|constraint| {
            format!(
                ".constraint {} {}\n",
                names[constraint.parameter as usize],
                match constraint.kind {
                    ConstraintKind::NotVoid => "notvoid",
                    ConstraintKind::NotReference => "notreference",
                }
            )
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
                check(&definition.generic_constraints, arguments)?;
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
