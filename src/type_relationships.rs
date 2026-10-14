//! Descriptive assignment checks, shared by managed execution and native lowering.
use crate::{
    Fault, Module,
    metadata::{Representation, Type},
};

fn indirect(ty: &Type) -> bool {
    matches!(ty, Type::ByRef(_) | Type::ReadOnlyByRef(_) | Type::Ptr(_))
}

fn interface(module: &Module, ty: &Type) -> bool {
    module
        .type_definition(ty)
        .is_some_and(|d| d.representation == Representation::Interface)
}

fn object(module: &Module, ty: &Type) -> bool {
    let Some(definition) = module.type_definition(ty) else {
        return false;
    };
    if let Some(root) = &module.object_root {
        return definition.definition.as_ref() == Some(root);
    }
    // The verified closed-image source catalogue retains semantic names before
    // lowering; its Object declaration has already passed load-set validation.
    definition.name == "System.Object" && definition.is_reference_type
}

// Validate before identity/Object shortcuts: neither may admit malformed or disabled
// experimental signatures, including compositions nested in generic arguments.
fn validate_compositions(module: &Module, source: &Type, target: &Type) -> Result<(), Fault> {
    for ty in [source, target] {
        if crate::compositions::contains(ty) {
            crate::vm::check_type(ty, module)?;
        }
    }
    Ok(())
}

pub(crate) fn assignable(module: &Module, source: &Type, target: &Type) -> Result<bool, Fault> {
    validate_compositions(module, source, target)?;
    if crate::compositions::is_compound(source) || crate::compositions::is_compound(target) {
        return Ok(crate::compositions::assignable(module, source, target));
    }
    if source == target {
        return Ok(true);
    }
    if indirect(source) || indirect(target) {
        return Ok(false);
    }
    if object(module, target) {
        return Ok(true);
    }
    let nominal = module.array_nominal(source);
    let source = nominal.as_ref().unwrap_or(source);
    if source == target {
        return Ok(true);
    }
    if interface(module, target) {
        if module.type_definition(source).is_none() {
            return Ok(false);
        }
        return Ok(crate::interfaces::closure(module, source)?.contains(target));
    }
    subclass(module, source, target)
}

pub(crate) fn subclass(module: &Module, source: &Type, target: &Type) -> Result<bool, Fault> {
    validate_compositions(module, source, target)?;
    // A structural guarantee is an assignment relationship, not a declared base.
    if crate::compositions::is_compound(source) || crate::compositions::is_compound(target) {
        return Ok(false);
    }
    if source == target
        || indirect(source)
        || indirect(target)
        || interface(module, source)
        || interface(module, target)
    {
        return Ok(false);
    }
    // The runtime has an implicit Object root, including boxed value types.
    if object(module, target) {
        return Ok(true);
    }
    let mut current = crate::inheritance::base(module, source)?;
    let mut visited = Vec::new();
    while let Some(parent) = current {
        if &parent == target {
            return Ok(true);
        }
        if visited.len() >= 64 || visited.contains(&parent) {
            return Err(Fault::new("cyclic or excessively deep base inheritance"));
        }
        current = crate::inheritance::base(module, &parent)?;
        visited.push(parent);
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> Module {
        crate::assemble(".module Relations\n.type class abstract System.Object\n.end\n.interface Named\n.end\n.interface Tagged\n.implements Named\n.end\n.type class Parent\n.implements Tagged\n.end\n.type class Child\n.extends Parent\n.end\n.type class Box<T>\n.end\n").unwrap()
    }

    #[cfg(feature = "experimental-compositions")]
    #[test]
    fn compositions_use_assignment_guarantees_without_inventing_ancestry() {
        let m = model();
        let parent = Type::from_name("Parent");
        let child = Type::from_name("Child");
        let named = Type::from_name("Named");
        let tagged = Type::from_name("Tagged");
        let object = Type::from_name("System.Object");
        let union = Type::Union(vec![parent.clone(), child.clone()]);
        let intersection = Type::Intersection(vec![named.clone(), tagged.clone()]);
        let mixed = Type::Union(vec![intersection.clone(), child.clone()]);
        for (source, target, expected) in [
            (&child, &union, true),
            (&union, &parent, true),
            (&union, &child, false),
            (&union, &intersection, true),
            (&intersection, &named, true),
            (&named, &intersection, false),
            (&intersection, &union, false),
            (&mixed, &named, true),
            (&mixed, &parent, false),
            (&union, &object, true),
            (&intersection, &object, true),
        ] {
            assert_eq!(
                assignable(&m, source, target).unwrap(),
                expected,
                "{source:?} -> {target:?}"
            );
            assert!(!subclass(&m, source, target).unwrap());
        }
        let wrapper = |t| Type::Constructed {
            definition: "Box".into(),
            arguments: vec![t],
        };
        let wide = wrapper(union);
        let narrow = wrapper(child);
        assert!(assignable(&m, &wide, &wide).unwrap());
        assert!(!assignable(&m, &narrow, &wide).unwrap());
        assert!(!assignable(&m, &wide, &narrow).unwrap());
    }

    #[test]
    fn composition_validation_precedes_identity_and_object_shortcuts() {
        let m = model();
        let object = Type::from_name("System.Object");
        let invalid = Type::Intersection(vec![Type::Int32, Type::from_name("Named")]);
        let wrapper = Type::Constructed {
            definition: "Box".into(),
            arguments: vec![invalid.clone()],
        };
        for ty in [&invalid, &wrapper] {
            assert!(assignable(&m, ty, ty).is_err());
            assert!(assignable(&m, ty, &object).is_err());
            assert!(subclass(&m, ty, &object).is_err());
        }
        #[cfg(not(feature = "experimental-compositions"))]
        {
            let union = Type::Union(vec![Type::from_name("Parent"), Type::from_name("Child")]);
            assert!(assignable(&m, &union, &union).is_err());
            assert!(assignable(&m, &union, &object).is_err());
            assert!(subclass(&m, &union, &object).is_err());
        }
    }

    #[test]
    fn structural_functions_preserve_signature_identity() {
        let m = model();
        let function = Type::Function(Box::new(crate::metadata::FunctionType {
            parameters: vec![Type::Int32],
            returns: Type::Int32,
            no_result: false,
            out_parameters: vec![],
            out_when_true: vec![],
        }));
        let mut other = function.clone();
        if let Type::Function(shape) = &mut other {
            shape.parameters.clear();
        }
        assert!(assignable(&m, &function, &function).unwrap());
        assert!(!assignable(&m, &function, &other).unwrap());
        assert!(assignable(&m, &function, &Type::from_name("System.Object")).unwrap());
        assert!(!assignable(&m, &function, &Type::from_name("Named")).unwrap());
    }

    #[test]
    fn assignment_and_strict_ancestry_have_distinct_interface_and_identity_rules() {
        let m = model();
        let parent = Type::from_name("Parent");
        let child = Type::from_name("Child");
        let interface = Type::from_name("Named");
        let object = Type::from_name("System.Object");
        assert!(assignable(&m, &child, &parent).unwrap());
        assert!(!assignable(&m, &parent, &child).unwrap());
        assert!(assignable(&m, &child, &interface).unwrap());
        assert!(assignable(&m, &interface, &object).unwrap());
        assert!(assignable(&m, &Type::Int32, &object).unwrap());
        assert!(subclass(&m, &child, &object).unwrap());
        assert!(!subclass(&m, &child, &child).unwrap());
        assert!(!subclass(&m, &child, &interface).unwrap());
        assert!(!subclass(&m, &interface, &object).unwrap());
    }

    #[test]
    fn identity_does_not_imply_conversions_or_reference_covariance() {
        let m = model();
        let array = Type::ArrayRef(Box::new(Type::Int32));
        let other_array = Type::ArrayRef(Box::new(Type::String));
        let pointer = Type::Ptr(Box::new(Type::Int32));
        let borrowed = Type::ByRef(Box::new(Type::Int32));
        for ty in [&array, &pointer, &borrowed, &Type::Int32] {
            assert!(assignable(&m, ty, ty).unwrap());
            assert!(!subclass(&m, ty, ty).unwrap());
        }
        assert!(!assignable(&m, &array, &other_array).unwrap());
        assert!(!assignable(&m, &pointer, &Type::from_name("System.Object")).unwrap());
        assert!(!assignable(&m, &borrowed, &Type::from_name("System.Object")).unwrap());
        assert!(!assignable(&m, &Type::Int32, &Type::Int64).unwrap());
        let integer = Type::Constructed {
            definition: "Box".into(),
            arguments: vec![Type::Int32],
        };
        let text = Type::Constructed {
            definition: "Box".into(),
            arguments: vec![Type::String],
        };
        assert!(assignable(&m, &integer, &integer).unwrap());
        assert!(!assignable(&m, &integer, &text).unwrap());
    }
}
