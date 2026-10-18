//! Explicit metadata admission for opaque interpreter memory views.
use crate::{
    Fault, Module,
    metadata::{MemoryViewKind, Representation, Type},
};

pub(crate) fn kind(module: &Module, ty: &Type) -> Option<MemoryViewKind> {
    module
        .type_definition(ty)
        .and_then(|definition| definition.memory_view)
}

pub(crate) fn contains_borrowed(module: &Module, ty: &Type) -> bool {
    if kind(module, ty).is_some_and(MemoryViewKind::is_borrowed) {
        return true;
    }
    match ty {
        Type::Array(t)
        | Type::ArrayRef(t)
        | Type::ByRef(t)
        | Type::ReadOnlyByRef(t)
        | Type::Ptr(t)
        | Type::InterfaceRef(t) => contains_borrowed(module, t),
        Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } => {
            arguments.iter().any(|t| contains_borrowed(module, t))
        }
        Type::Tuple(parts) => parts.iter().any(|t| contains_borrowed(module, t)),
        Type::Union(parts) | Type::Intersection(parts) => {
            parts.iter().any(|t| contains_borrowed(module, t))
        }
        Type::Function(shape) => shape
            .parameters
            .iter()
            .chain([&shape.returns])
            .any(|t| contains_borrowed(module, t)),
        _ => false,
    }
}

pub(crate) fn validate(module: &Module) -> Result<(), Fault> {
    for definition in &module.types {
        if let Some(kind) = definition.memory_view {
            if definition.name != kind.name()
                || !(definition
                    .definition
                    .as_ref()
                    .is_some_and(|id| id.module == "System")
                    || definition
                        .origin
                        .as_ref()
                        .is_some_and(|origin| origin.name == format!("{}`1", kind.name())))
                || definition.representation != Representation::Record
                || definition.is_reference_type
                || definition.is_abstract
                || definition.generic_parameters.len() != 1
                || !definition.fields.is_empty()
                || !definition.static_fields.is_empty()
                || !definition.literal_fields.is_empty()
                || !definition.implements.is_empty()
                || definition.base.is_some()
                || definition.packing.is_some()
                || definition.minimum_size.is_some()
                || definition.enum_info.is_some()
                || definition.declaring_type.is_some()
                || !definition.generic_constraints.is_empty()
                || !definition.generic_variance.is_empty()
            {
                return Err(Fault::new("invalid opaque memory-view declaration"));
            }
        } else if [
            MemoryViewKind::Slice,
            MemoryViewKind::ReadOnlySlice,
            MemoryViewKind::Memory,
            MemoryViewKind::ReadOnlyMemory,
        ]
        .iter()
        .any(|kind| definition.name == kind.name())
        {
            return Err(Fault::new(
                "memory-view type requires explicit memory_view metadata",
            ));
        }
        for ty in definition
            .fields
            .iter()
            .map(|field| &field.ty)
            .chain(definition.static_fields.iter().map(|field| &field.ty))
        {
            if contains_borrowed(module, ty) {
                return Err(Fault::new("borrowed memory-view fields are not supported"));
            }
        }
    }
    for function in &module.functions {
        if function.name.ends_with("..ctor")
            && function
                .owner
                .as_ref()
                .and_then(|ty| kind(module, ty))
                .is_some()
        {
            return Err(Fault::new(
                "opaque memory views cannot declare record constructors",
            ));
        }
    }
    Ok(())
}

/// Called at every signature node by the ordinary type validator.
pub(crate) fn signature(module: &Module, ty: &Type) -> Result<(), Fault> {
    let forbidden = match ty {
        Type::Array(t) | Type::ArrayRef(t) | Type::Ptr(t) => contains_borrowed(module, t),
        Type::Constructed { arguments, .. } => {
            arguments.iter().any(|t| contains_borrowed(module, t))
        }
        Type::Tuple(parts) => parts.iter().any(|t| contains_borrowed(module, t)),
        Type::Union(parts) | Type::Intersection(parts) => {
            parts.iter().any(|t| contains_borrowed(module, t))
        }
        Type::Function(_) => contains_borrowed(module, ty),
        _ => false,
    };
    if forbidden {
        return Err(Fault::new(
            "borrowed memory views cannot be array elements, generic arguments, compound values or Function signatures",
        ));
    }
    Ok(())
}
