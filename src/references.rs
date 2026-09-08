//! Direct metadata-reference visibility, not a security or runtime value boundary.
use crate::{
    Fault, Module,
    metadata::{CustomAttribute, FunctionRef, Instruction, Type},
};

pub(crate) fn validate_list(source: &Module, supplied: &[&Module]) -> Result<(), Fault> {
    let Some(references) = &source.references else {
        return Ok(());
    };
    let mut seen = std::collections::HashSet::new();
    for reference in references {
        let name = reference.name();
        if name.is_empty() || name == source.name || !seen.insert(name) {
            return Err(Fault::new(format!(
                "invalid or duplicate module reference {name:?} in {}",
                source.name
            )));
        }
        let target = supplied.iter().find(|m| m.name == name).ok_or_else(|| {
            Fault::new(format!(
                "missing referenced module {name} for {}",
                source.name
            ))
        })?;
        match reference.revision() {
            Some(revision)
                if !crate::metadata::valid_revision(revision)
                    || target.revision.as_deref() != Some(revision) =>
            {
                return Err(Fault::new(format!(
                    "module revision mismatch for {name}: expected {revision}"
                )));
            }
            _ => (),
        }
    }
    Ok(())
}

fn check_module(source: &Module, target: &str) -> Result<(), Fault> {
    if source.references.as_ref().is_some_and(|references| {
        target != source.name
            && target != "System"
            && !references.iter().any(|r| r.name() == target)
    }) {
        return Err(Fault::new(format!(
            "module {} does not reference {target}",
            source.name
        )));
    }
    Ok(())
}

// Called after structural type validation has enforced the signature nesting limit.
pub(crate) fn check_type(linked: &Module, source: &Module, ty: &Type) -> Result<(), Fault> {
    if source.references.is_none() {
        return Ok(());
    }
    match ty {
        Type::Array(t)
        | Type::ByRef(t)
        | Type::ReadOnlyByRef(t)
        | Type::Ptr(t)
        | Type::InterfaceRef(t) => check_type(linked, source, t)?,
        Type::Constructed {
            definition,
            arguments,
        } => {
            check_named(linked, source, definition, arguments.len())?;
            for argument in arguments {
                check_type(linked, source, argument)?;
            }
        }
        Type::Named(name) => check_named(linked, source, name, 0)?,
        // Primitive signatures refer to implicit System; parameters are contextual.
        _ => (),
    }
    Ok(())
}

fn check_named(linked: &Module, source: &Module, name: &str, arity: usize) -> Result<(), Fault> {
    let owner = linked
        .types
        .iter()
        .find(|d| d.name == name && d.generic_parameters.len() == arity)
        .and_then(|d| d.definition.as_ref())
        .ok_or_else(|| Fault::new(format!("missing definition identity for {name}")))?;
    check_module(source, &owner.module)
}

pub(crate) fn check_call(
    linked: &Module,
    source: &Module,
    target: &FunctionRef,
) -> Result<(), Fault> {
    let function = crate::vm::resolve(linked, target)?;
    let definition = function
        .definition
        .ok_or_else(|| Fault::new("missing function identity"))?;
    check_module(source, &definition.module)?;
    if let Some(owner) = &target.owner {
        check_type(linked, source, owner)?;
    }
    for parameter in &target.parameters {
        check_type(linked, source, parameter)?;
    }
    Ok(())
}

fn check_attributes(
    linked: &Module,
    source: &Module,
    attributes: &[CustomAttribute],
) -> Result<(), Fault> {
    for attribute in attributes {
        check_call(linked, source, &attribute.constructor)?;
    }
    Ok(())
}

pub(crate) fn validate_uses(linked: &Module, source: &Module) -> Result<(), Fault> {
    if source.references.is_none() {
        return Ok(());
    }
    if !source.entry.is_empty() {
        let entry = linked
            .functions
            .iter()
            .find(|f| f.name == source.entry && f.parameters.is_empty() && !f.instance)
            .and_then(|f| f.definition.as_ref())
            .ok_or_else(|| Fault::new("missing entry definition"))?;
        check_module(source, &entry.module)?;
    }
    for definition in &source.types {
        for property in &definition.properties {
            property.clone().map_types(|ty| {
                check_type(linked, source, ty)?;
                Ok(ty.clone())
            })?;
            for accessor in property.getter.iter().chain(property.setter.iter()) {
                check_call(linked, source, accessor)?;
            }
        }
        if let Some(base) = &definition.base {
            check_type(linked, source, base)?;
        }
        for ty in &definition.implements {
            check_type(linked, source, ty)?;
        }
        for field in &definition.fields {
            check_type(linked, source, &field.ty)?;
        }
        check_attributes(linked, source, &definition.custom_attributes)?;
    }
    for function in &source.functions {
        function.map_types(|ty| {
            check_type(linked, source, ty)?;
            Ok(ty.clone())
        })?;
        for instruction in &function.body {
            if let Instruction::Call(target)
            | Instruction::CallVirtual(target)
            | Instruction::Construct(target) = instruction
            {
                check_call(linked, source, target)?;
            }
        }
        check_attributes(linked, source, &function.custom_attributes)?;
    }
    Ok(())
}
