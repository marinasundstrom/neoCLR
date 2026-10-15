//! Nominal interface variance. Structural signatures and writable slots stay invariant.
use crate::{
    Fault, Module,
    metadata::{Representation, Type, TypeDef, Variance},
};

pub(crate) fn parameter(def: &TypeDef, index: usize) -> Variance {
    def.generic_variance
        .get(index)
        .copied()
        .unwrap_or(Variance::Invariant)
}

// +1 produces, -1 consumes, 0 is an invariant occurrence.
fn position(module: &Module, def: &TypeDef, ty: &Type, polarity: i8) -> Result<(), Fault> {
    match ty {
        Type::TypeParameter(index) => {
            let allowed = match parameter(def, *index as usize) {
                Variance::Invariant => true,
                Variance::Covariant => polarity == 1,
                Variance::Contravariant => polarity == -1,
            };
            if !allowed {
                return Err(Fault::new("unsafe interface variance position"));
            }
        }
        Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } => {
            let nested = module.type_definition(ty);
            for (index, argument) in arguments.iter().enumerate() {
                let direction = nested
                    .map(|d| parameter(d, index))
                    .unwrap_or(Variance::Invariant);
                let polarity = match direction {
                    Variance::Invariant => 0,
                    Variance::Covariant => polarity,
                    Variance::Contravariant => -polarity,
                };
                position(module, def, argument, polarity)?;
            }
        }
        Type::ByRef(t)
        | Type::ReadOnlyByRef(t)
        | Type::Ptr(t)
        | Type::Array(t)
        | Type::ArrayRef(t)
        | Type::InterfaceRef(t) => position(module, def, t, 0)?,
        Type::Union(parts) | Type::Intersection(parts) => {
            for t in parts {
                position(module, def, t, 0)?;
            }
        }
        Type::Tuple(parts) => {
            for t in parts.iter() {
                position(module, def, t, 0)?;
            }
        }
        Type::Function(shape) => {
            for t in shape.parameters.iter().chain([&shape.returns]) {
                position(module, def, t, 0)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    for def in &module.types {
        if !def.generic_variance.is_empty()
            && def.generic_variance.len() != def.generic_parameters.len()
        {
            return Err(Fault::new("variance must align with generic parameters"));
        }
        if !def
            .generic_variance
            .iter()
            .any(|v| *v != Variance::Invariant)
        {
            continue;
        }
        if def.representation != Representation::Interface {
            return Err(Fault::new("variance requires an interface declaration"));
        }
        for base in &def.implements {
            position(module, def, base, 1)?;
        }
        // Bounds and storage are not variance-enabled positions in this first slice.
        for bound in crate::constraints::bounds(&def.generic_constraints) {
            position(module, def, bound, 0)?;
        }
        for field in &def.static_fields {
            position(module, def, &field.ty, 0)?;
        }
        for method in &module.functions {
            if !method
                .owner
                .as_ref()
                .and_then(|o| module.type_definition(o))
                .is_some_and(|d| std::ptr::eq(d, def))
            {
                continue;
            }
            // Do not admit default bodies, Self, method generics or static contracts yet:
            // their substitution/call-adapter obligations require separate qualification.
            if !method.body.is_empty()
                || !method.instance
                || !method.generic_parameters.is_empty()
                || crate::self_types::signature_has_self(method)
            {
                return Err(Fault::new(
                    "variant interfaces require bodyless instance contracts without Self or method generics",
                ));
            }
            for ty in &method.parameters {
                position(module, def, ty, -1)?;
            }
            position(module, def, &method.returns, 1)?;
        }
    }
    Ok(())
}
