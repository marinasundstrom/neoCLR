//! Closed-world roots and trigger identities for native type initialization.
use neoclr::{
    Module,
    metadata::{Function, Representation, Type, TypeDef},
};

pub(super) fn initializer(input: &Module, owner: &Type) -> Option<usize> {
    input
        .functions
        .iter()
        .position(|f| f.name.ends_with("..cctor") && f.owner.as_ref() == Some(owner))
}
pub(super) fn needs_storage(input: &Module, owner: &TypeDef) -> bool {
    !owner.static_fields.is_empty() || initializer(input, &owner_type(owner)).is_some()
}
pub(super) fn entry_owner<'a>(input: &Module, f: &'a Function) -> Option<&'a Type> {
    if f.name.ends_with("..cctor") {
        return None;
    }
    let owner = f.owner.as_ref()?;
    let t = input.type_definition(owner)?;
    (!t.before_field_init
        && (!f.instance
            || f.name.ends_with("..ctor")
            || t.representation == Representation::Record && !t.is_reference_type))
        .then_some(owner)
}

pub(super) fn owner_type(owner: &TypeDef) -> Type {
    if owner.generic_parameters.is_empty() {
        Type::from_name(&owner.name)
    } else {
        Type::Constructed {
            definition: owner.name.clone(),
            arguments: (0..owner.generic_parameters.len())
                .map(|i| Type::TypeParameter(i as u16))
                .collect(),
        }
    }
}
