//! Method access checks on resolved identities, not source names or call operands.
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
