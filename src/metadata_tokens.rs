//! Definition tokens are scoped to their descriptive module, never provider objects.
use crate::metadata::{Function, TypeDef};
use crate::{Fault, Module, TypeIdentity, Value};

fn token(table: u32, row: usize) -> Result<i32, Fault> {
    if row == 0 || row > 0x00ff_ffff {
        return Err(Fault::new("metadata token row overflow"));
    }
    Ok(((table << 24) | row as u32) as i32)
}
pub(crate) fn definition<'a>(module: &'a Module, identity: &TypeIdentity) -> Option<&'a TypeDef> {
    let id = match identity {
        TypeIdentity::Definition { definition, .. }
        | TypeIdentity::GenericParameter { definition, .. } => definition,
        TypeIdentity::Array(t)
        | TypeIdentity::ArrayRef(t)
        | TypeIdentity::Ptr(t)
        | TypeIdentity::ByRef(t)
        | TypeIdentity::ReadOnlyByRef(t)
        | TypeIdentity::InterfaceRef(t) => return definition(module, t),
    };
    module
        .types
        .iter()
        .find(|d| d.definition.as_ref() == Some(id))
}
pub(crate) fn type_token(module: &Module, identity: &TypeIdentity) -> Result<i32, Fault> {
    if !matches!(identity, TypeIdentity::Definition { .. }) {
        return Ok(0);
    }
    let d =
        definition(module, identity).ok_or_else(|| Fault::new("missing type token definition"))?;
    if let Some(origin) = &d.origin {
        return Ok(origin.token as i32);
    }
    token(
        2,
        d.definition
            .as_ref()
            .ok_or_else(|| Fault::new("missing type identity"))?
            .index as usize
            + 1,
    )
}
pub(crate) fn member(
    module: &Module,
    owner: &TypeDef,
    index: usize,
    property: bool,
) -> Result<i32, Fault> {
    if let Some(origin) = &owner.origin {
        return (if property {
            &origin.property_tokens
        } else {
            &origin.field_tokens
        })
        .get(index)
        .copied()
        .map(|t| t as i32)
        .ok_or_else(|| Fault::new("missing source member token"));
    }
    let id = owner
        .definition
        .as_ref()
        .ok_or_else(|| Fault::new("missing declaring type identity"))?;
    let prior: usize = module
        .types
        .iter()
        .filter(|d| {
            d.definition
                .as_ref()
                .is_some_and(|other| other.module == id.module && other.index < id.index)
        })
        .map(|d| {
            if property {
                d.properties.len()
            } else {
                d.fields.len()
            }
        })
        .sum();
    token(if property { 0x17 } else { 4 }, prior + index + 1)
}
pub(crate) fn method(function: &Function) -> Result<i32, Fault> {
    if let Some(origin) = &function.origin {
        return Ok(origin.token as i32);
    }
    token(
        6,
        function
            .definition
            .as_ref()
            .ok_or_else(|| Fault::new("missing method identity"))?
            .index as usize
            + 1,
    )
}
pub(crate) fn parameter(module: &Module, function: &Function, index: usize) -> Result<i32, Fault> {
    if let Some(origin) = &function.origin {
        return origin
            .parameter_tokens
            .get(index)
            .copied()
            .map(|t| t as i32)
            .ok_or_else(|| Fault::new("missing parameter token"));
    }
    let id = function
        .definition
        .as_ref()
        .ok_or_else(|| Fault::new("missing method identity"))?;
    let prior: usize = module
        .functions
        .iter()
        .filter(|f| {
            f.definition
                .as_ref()
                .is_some_and(|other| other.module == id.module && other.index < id.index)
        })
        .map(|f| f.parameters.len())
        .sum();
    token(8, prior + index + 1)
}
pub(crate) fn module_value(
    module: &Module,
    origin: Option<&crate::metadata_origin::MetadataOrigin>,
    runtime_module: &str,
) -> Result<Value, Fault> {
    if let Some(origin) = origin {
        return Ok(crate::assembly_info::module_value(
            &origin.assembly,
            &origin.module,
        ));
    }
    let assembly = module
        .assemblies
        .iter()
        .find(|a| a.modules.iter().any(|m| m == runtime_module))
        .ok_or_else(|| Fault::new("missing module catalog entry"))?;
    Ok(crate::assembly_info::module_value(
        &assembly.full_name,
        runtime_module,
    ))
}
