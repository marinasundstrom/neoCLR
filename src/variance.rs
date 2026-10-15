//! Nominal interface variance. Structural signatures and writable slots stay invariant.
use crate::{
    Fault, Module,
    metadata::{Representation, Type, TypeDef, Variance},
};

// Closed native shapes retain source identity and argument variance even though
// their executable owners have private nongeneric names. This is compiler IR only.
#[derive(Debug, Clone)]
pub(crate) struct NativeInterface {
    pub source: crate::metadata::TypeDefId,
    pub arguments: Vec<Type>,
    pub annotations: Vec<Variance>,
}

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
            if base.contains_self() {
                return Err(Fault::new("variant interface bases cannot contain Self"));
            }
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

fn ordinary_reference(module: &Module, ty: &Type) -> bool {
    // Intrinsic text has a distinct payload representation; widening it to an
    // object/interface needs materialization, not just a reference view.
    if matches!(ty, Type::String) {
        return false;
    }
    matches!(ty, Type::ArrayRef(_))
        || module
            .type_definition(ty)
            .is_some_and(|d| d.is_reference_type || d.representation == Representation::Interface)
}

fn reference_conversion(module: &Module, source: &Type, target: &Type, depth: usize) -> bool {
    if source == target {
        return true;
    }
    if depth >= 64 || !ordinary_reference(module, source) || !ordinary_reference(module, target) {
        return false;
    }
    if *target == Type::from_name("System.Object") && module.is_reference_type(target) {
        return true;
    }
    if module.array_nominal(source).as_ref() == Some(target) {
        return true;
    }
    if module
        .type_definition(target)
        .is_some_and(|d| d.representation == Representation::Interface)
    {
        return crate::interfaces::closure(module, source).is_ok_and(|interfaces| {
            interfaces.contains(target)
                || interfaces
                    .iter()
                    .filter(|i| compatible_at(module, i, target, depth + 1))
                    .count()
                    == 1
        });
    }
    module.is_reference_type(source)
        && module.is_reference_type(target)
        && crate::inheritance::require_base(module, source, target).is_ok()
}

fn compatible_at(module: &Module, source: &Type, target: &Type, depth: usize) -> bool {
    if source == target {
        return true;
    }
    if depth >= 64 {
        return false;
    }
    let Some(def) = module.type_definition(source) else {
        return false;
    };
    let Some(destination) = module.type_definition(target) else {
        return false;
    };
    if def.representation != Representation::Interface
        || destination.representation != Representation::Interface
    {
        return false;
    }
    let (from, to, annotations) = match (&def.native_interface, &destination.native_interface) {
        (Some(a), Some(b)) if a.source == b.source => (
            a.arguments.as_slice(),
            b.arguments.as_slice(),
            a.annotations.as_slice(),
        ),
        (None, None) if std::ptr::eq(def, destination) => (
            source.generic_arguments(),
            target.generic_arguments(),
            def.generic_variance.as_slice(),
        ),
        _ => return false,
    };
    if from.len() != to.len() {
        return false;
    }
    from.iter().zip(to).enumerate().all(|(index, (a, b))| {
        if a == b {
            return true;
        }
        // Identity for value arguments remains legal; differing value arguments never box.
        if !ordinary_reference(module, a) || !ordinary_reference(module, b) {
            return false;
        }
        match annotations
            .get(index)
            .copied()
            .unwrap_or(Variance::Invariant)
        {
            Variance::Invariant => false,
            Variance::Covariant => reference_conversion(module, a, b, depth + 1),
            Variance::Contravariant => reference_conversion(module, b, a, depth + 1),
        }
    })
}

pub(crate) fn compatible(module: &Module, source: &Type, target: &Type) -> bool {
    compatible_at(module, source, target, 0)
}

/// Exact conformance wins. Multiple distinct variant views are deliberately ambiguous.
pub(crate) fn implemented_view(
    module: &Module,
    source: &Type,
    target: &Type,
) -> Result<Type, Fault> {
    let interfaces = crate::interfaces::closure(module, source)?;
    if interfaces.contains(target) {
        return Ok(target.clone());
    }
    // The caller controls the storage category. Ordinary assignment requires a
    // reference, while an existing boxed receiver may reuse its box here.
    let mut candidates = interfaces
        .into_iter()
        .filter(|i| compatible(module, i, target));
    let candidate = candidates
        .next()
        .ok_or_else(|| Fault::new("no compatible nominal interface view"))?;
    if candidates.next().is_some() {
        return Err(Fault::new("ambiguous variant interface view"));
    }
    Ok(candidate)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reference_relationships_compose_without_numeric_boxing_or_array_covariance() {
        let module = crate::assembler::parse_module(".module VarianceRelations\n.type class System.Object\n.end\n.type class Base\n.extends System.Object\n.end\n.type class Derived\n.extends Base\n.end\n.interface Producer<T>\n.variance out\n.end\n.interface Consumer<T>\n.variance in\n.end\n.interface Fixed<T>\n.end\n.interface System.Collections.Iterable<T>\n.end\n").unwrap();
        for (source, target, expected) in [
            ("Producer<Derived>", "Producer<Base>", true),
            ("Producer<Base>", "Producer<Derived>", false),
            ("Consumer<Base>", "Consumer<Derived>", true),
            (
                "Producer<Consumer<Base>>",
                "Producer<Consumer<Derived>>",
                true,
            ),
            (
                "Producer<Consumer<Derived>>",
                "Producer<Consumer<Base>>",
                false,
            ),
            (
                "Producer<arrayref<Int32>>",
                "Producer<System.Collections.Iterable<Int32>>",
                true,
            ),
            (
                "Producer<arrayref<Derived>>",
                "Producer<arrayref<Base>>",
                false,
            ),
            ("Producer<Int32>", "Producer<Int32>", true),
            ("Producer<Int32>", "Producer<System.Object>", false),
            ("Producer<String>", "Producer<System.Object>", false),
            ("Fixed<Derived>", "Fixed<Base>", false),
            ("Producer<Fixed<Derived>>", "Producer<Fixed<Base>>", false),
        ] {
            let source = crate::assembler::parse_type(source).unwrap();
            let target = crate::assembler::parse_type(target).unwrap();
            assert_eq!(
                module.reference_assignable(&source, &target),
                expected,
                "{source:?} -> {target:?}"
            );
            assert_eq!(
                crate::type_relationships::assignable(&module, &source, &target).unwrap(),
                expected
            );
            if source != target {
                assert!(!crate::type_relationships::subclass(&module, &source, &target).unwrap());
            }
        }
    }
}
