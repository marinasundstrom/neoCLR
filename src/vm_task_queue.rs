//! Typed runtime services keep queue ownership in the source assembly.
use super::*;
use crate::metadata::Function;

pub(super) fn validate(module: &Module, ty: &Type) -> Result<(), Fault> {
    let definition = module
        .type_definition(ty)
        .ok_or_else(|| Fault::new("TaskQueue service requires a defined queue type"))?;
    let legacy = definition.name == "System.Tasks.TaskQueue"
        && definition
            .definition
            .as_ref()
            .is_some_and(|id| id.module == "System");
    let native = definition
        .origin
        .as_ref()
        .is_some_and(|origin| origin.name == "System.Tasks.TaskQueue");
    if !definition.is_reference_type
        || !definition.generic_parameters.is_empty()
        || !(legacy || native)
    {
        return Err(Fault::new(
            "TaskQueue service requires the source-owned TaskQueue contract",
        ));
    }
    method(module, ty, "Drain", &[])?;
    Ok(())
}

pub(super) fn member(function: &Function) -> &str {
    function.origin.as_ref().map_or_else(
        || function.name.rsplit('.').next().unwrap_or(""),
        |origin| origin.name.as_str(),
    )
}

pub(super) fn method<'a>(
    module: &'a Module,
    ty: &Type,
    name: &str,
    parameters: &[Type],
) -> Result<&'a Function, Fault> {
    let mut candidates = module.functions.iter().filter(|function| {
        function.owner.as_ref() == Some(ty)
            && function.instance
            && function.generic_parameters.is_empty()
            && function.parameters == parameters
            && function.no_result
            && member(function) == name
    });
    let candidate = candidates
        .next()
        .ok_or_else(|| Fault::new("Missing source TaskQueue method contract"))?;
    if candidates.next().is_some() {
        return Err(Fault::new("Ambiguous source TaskQueue method contract"));
    }
    Ok(candidate)
}
