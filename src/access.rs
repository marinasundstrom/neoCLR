//! Member access checks on resolved identities, not source names or call operands.
use crate::{
    Fault, Module,
    metadata::{Function, Type, TypeDefId, Visibility},
};

pub(crate) fn check_call(
    module: &Module,
    caller: Option<&Function>,
    callee: &Function,
) -> Result<(), Fault> {
    if callee.visibility == Visibility::Public {
        return Ok(());
    }
    let permitted = caller.is_some_and(|caller| {
        let same_module = caller
            .definition
            .as_ref()
            .zip(callee.definition.as_ref())
            .is_some_and(|(a, b)| a.module == b.module && a.revision == b.revision);
        if !same_module {
            return false;
        }
        match callee.visibility {
            Visibility::Public | Visibility::Internal => true,
            Visibility::Private => declaring_type(module, caller)
                .zip(declaring_type(module, callee))
                .is_some_and(|(a, b)| a == b),
        }
    });
    if permitted {
        Ok(())
    } else {
        Err(Fault::new(format!(
            "method access denied: {} is {:?}",
            callee.name, callee.visibility
        )))
    }
}

pub(crate) fn check_entry(module: &Module, entry: &Function) -> Result<(), Fault> {
    // Explicit entry selection is an execution root, not arbitrary host member invocation.
    if entry
        .definition
        .as_ref()
        .is_some_and(|id| id.module == module.name && id.revision == module.revision)
    {
        Ok(())
    } else {
        check_call(module, None, entry)
    }
}

fn declaring_type<'a>(module: &'a Module, function: &Function) -> Option<&'a TypeDefId> {
    let owner = function.owner.as_ref()?;
    let definition = match owner {
        Type::Constructed { definition, .. } => {
            module.types.iter().find(|ty| &ty.name == definition)
        }
        _ => module.type_definition(owner),
    }?;
    definition.definition.as_ref()
}

fn record_definition<'a>(
    module: &'a Module,
    owner: &Type,
) -> Result<&'a crate::metadata::TypeDef, Fault> {
    let definition = match owner {
        Type::Constructed { definition, .. } => {
            module.types.iter().find(|ty| &ty.name == definition)
        }
        _ => module.type_definition(owner),
    };
    definition.ok_or_else(|| Fault::new("field owner has no type definition"))
}

pub(crate) fn check_field(
    module: &Module,
    caller: &Function,
    owner: &Type,
    index: usize,
) -> Result<(), Fault> {
    let definition = record_definition(module, owner)?;
    let field = definition
        .fields
        .get(index)
        .ok_or_else(|| Fault::new("field index out of range"))?;
    let same_module = definition
        .definition
        .as_ref()
        .zip(caller.definition.as_ref())
        .is_some_and(|(ty, method)| ty.module == method.module && ty.revision == method.revision);
    let allowed = match field.visibility {
        Visibility::Public => true,
        Visibility::Internal => same_module,
        Visibility::Private => {
            same_module
                && declaring_type(module, caller)
                    .zip(definition.definition.as_ref())
                    .is_some_and(|(a, b)| a == b)
        }
    };
    if allowed {
        Ok(())
    } else {
        Err(Fault::new(format!(
            "field access denied: {}.{} is {:?}",
            definition.name, field.name, field.visibility
        )))
    }
}

pub(crate) fn check_construction(
    module: &Module,
    caller: &Function,
    owner: &Type,
) -> Result<(), Fault> {
    let definition = record_definition(module, owner)?;
    for index in 0..definition.fields.len() {
        check_field(module, caller, owner, index)?;
    }
    Ok(())
}
