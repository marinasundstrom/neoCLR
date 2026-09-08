//! Preliminary aggregate inheritance: complete values, no base-reference views yet.
use crate::{
    Fault, Module,
    metadata::{Field, Representation, Type},
};

pub(crate) fn base(module: &Module, ty: &Type) -> Result<Option<Type>, Fault> {
    let Some(definition) = module.type_definition(ty) else {
        return Ok(None);
    };
    let arguments = match ty {
        Type::Constructed { arguments, .. } => arguments.as_slice(),
        _ => &[],
    };
    definition
        .base
        .as_ref()
        .map(|base| base.substitute_type_parameters(arguments))
        .transpose()
}

/// Derived-first chain. Reject definition cycles even if generic arguments expand.
pub(crate) fn lineage(module: &Module, ty: &Type) -> Result<Vec<Type>, Fault> {
    let mut chain = Vec::new();
    let mut current = Some(ty.clone());
    while let Some(ty) = current {
        let definition = module
            .type_definition(&ty)
            .ok_or_else(|| Fault::new("unknown base type"))?;
        if definition.representation != Representation::Record {
            return Err(Fault::new("base inheritance requires record definitions"));
        }
        if chain.len() >= 64
            || chain
                .iter()
                .any(|t: &Type| t.definition_name() == ty.definition_name())
        {
            return Err(Fault::new("cyclic or excessively deep base inheritance"));
        }
        current = base(module, &ty)?;
        chain.push(ty);
    }
    Ok(chain)
}

pub(crate) fn fields(module: &Module, ty: &Type) -> Result<Vec<Field>, Fault> {
    let mut fields = Vec::new();
    for owner in lineage(module, ty)?.into_iter().rev() {
        let definition = module.type_definition(&owner).unwrap();
        let arguments = match &owner {
            Type::Constructed { arguments, .. } => arguments.as_slice(),
            _ => &[],
        };
        for field in &definition.fields {
            if fields.iter().any(|f: &Field| f.name == field.name) {
                return Err(Fault::new(
                    "inherited field names cannot be hidden in this preview",
                ));
            }
            fields.push(Field {
                visibility: field.visibility,
                name: field.name.clone(),
                ty: field.ty.substitute_type_parameters(arguments)?,
            });
        }
    }
    Ok(fields)
}

pub(crate) fn field_owner(
    module: &Module,
    ty: &Type,
    mut index: usize,
) -> Result<(Type, usize), Fault> {
    for owner in lineage(module, ty)?.into_iter().rev() {
        let count = module.type_definition(&owner).unwrap().fields.len();
        if index < count {
            return Ok((owner, index));
        }
        index -= count;
    }
    Err(Fault::new("field index out of range"))
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    for definition in &module.types {
        if definition.base.is_none() {
            continue;
        }
        let chain = lineage(module, &definition.open_type())?;
        fields(module, &definition.open_type())?;
        for base in chain.iter().skip(1) {
            let parent = module.type_definition(base).unwrap();
            if !parent.implements.is_empty()
                || module.functions.iter().any(|f| {
                    f.instance
                        && f.owner
                            .as_ref()
                            .and_then(|t| module.type_definition(t))
                            .is_some_and(|d| std::ptr::eq(d, parent))
                })
            {
                return Err(Fault::new(
                    "base types with instance methods or interface implementations require the forthcoming base-reference dispatch model",
                ));
            }
        }
    }
    Ok(())
}
