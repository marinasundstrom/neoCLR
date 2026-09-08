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
    if let Some(owner) = &callee.owner {
        check_owner(module, caller.and_then(scope), owner)?;
    }
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
    let definition = module.type_definition(owner)?;
    definition.definition.as_ref()
}

fn record_definition<'a>(
    module: &'a Module,
    owner: &Type,
) -> Result<&'a crate::metadata::TypeDef, Fault> {
    let definition = module.type_definition(owner);
    definition.ok_or_else(|| Fault::new("field owner has no type definition"))
}

pub(crate) fn check_field(
    module: &Module,
    caller: &Function,
    owner: &Type,
    index: usize,
) -> Result<(), Fault> {
    check_owner(module, scope(caller), owner)?;
    let (declaring_owner, index) = crate::inheritance::field_owner(module, owner, index)?;
    let definition = record_definition(module, &declaring_owner)?;
    let field = definition
        .fields
        .get(index)
        .ok_or_else(|| Fault::new("field index out of range"))?;
    check_type(module, scope(caller), &field.ty)?;
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
    check_owner(module, scope(caller), owner)?;
    for index in 0..crate::inheritance::fields(module, owner)?.len() {
        check_field(module, caller, owner, index)?;
    }
    Ok(())
}

type Scope<'a> = Option<(&'a str, Option<&'a str>)>;
fn scope(function: &Function) -> Scope<'_> {
    function
        .definition
        .as_ref()
        .map(|id| (id.module.as_str(), id.revision.as_deref()))
}

fn check_type(module: &Module, source: Scope<'_>, ty: &Type) -> Result<(), Fault> {
    fn visit(module: &Module, source: Scope<'_>, ty: &Type, depth: usize) -> Result<(), Fault> {
        if depth > 32 {
            return Err(Fault::new("type nesting exceeds 32"));
        }
        let mut definition = module.type_definition(ty);
        let mut owners = 0;
        while let Some(def) = definition {
            if owners > 32 {
                return Err(Fault::new("type ownership nesting exceeds 32"));
            }
            if def.visibility != Visibility::Public
                && def
                    .definition
                    .as_ref()
                    .is_none_or(|id| source != Some((id.module.as_str(), id.revision.as_deref())))
            {
                return Err(Fault::new(format!(
                    "type access denied: {} is {:?}",
                    def.name, def.visibility
                )));
            }
            definition = def.declaring_type.as_ref().and_then(|owner| {
                module
                    .types
                    .iter()
                    .find(|candidate| candidate.definition.as_ref() == Some(owner))
            });
            owners += 1;
        }
        let nested = |ty| visit(module, source, ty, depth + 1);
        match ty {
            Type::Constructed { arguments, .. } => {
                for argument in arguments {
                    nested(argument)?;
                }
            }
            Type::Array(t)
            | Type::ByRef(t)
            | Type::ReadOnlyByRef(t)
            | Type::Ptr(t)
            | Type::InterfaceRef(t) => nested(t)?,
            Type::Scoped { .. } => {
                return Err(Fault::new("unresolved type scope during access checking"));
            }
            Type::TypeParameter(_) => {}
            _ => {}
        }
        Ok(())
    }
    visit(module, source, ty, 0)
}

fn check_owner(module: &Module, source: Scope<'_>, owner: &Type) -> Result<(), Fault> {
    // Runtime specialization does not revoke access to caller-supplied generic arguments.
    // Explicit signature/operand types were checked in their open declaring context.
    let definition = module
        .type_definition(owner)
        .ok_or_else(|| Fault::new("owner has no type definition"))?;
    check_type(module, source, &definition.open_type())
}

pub(crate) fn check_signature(
    module: &Module,
    source: Scope<'_>,
    function: &Function,
) -> Result<(), Fault> {
    for ty in function
        .owner
        .iter()
        .chain(&function.parameters)
        .chain([&function.returns])
    {
        check_type(module, source, ty)?;
    }
    Ok(())
}

pub(crate) fn validate_types(module: &Module) -> Result<(), Fault> {
    let attributes =
        |source, attributes: &[crate::metadata::CustomAttribute]| -> Result<(), Fault> {
            for attribute in attributes {
                for ty in attribute
                    .constructor
                    .owner
                    .iter()
                    .chain(&attribute.constructor.parameters)
                {
                    check_type(module, source, ty)?;
                }
            }
            Ok(())
        };
    for definition in &module.types {
        let source = definition
            .definition
            .as_ref()
            .map(|id| (id.module.as_str(), id.revision.as_deref()));
        if let Some(base) = &definition.base {
            check_type(module, source, base)?;
        }
        for ty in &definition.implements {
            check_type(module, source, ty)?;
        }
        for field in &definition.fields {
            check_type(module, source, &field.ty)?;
        }
        for property in &definition.properties {
            property.clone().map_types(|ty| {
                check_type(module, source, ty)?;
                Ok(ty.clone())
            })?;
        }
        attributes(source, &definition.custom_attributes)?;
    }
    for function in &module.functions {
        let source = scope(function);
        function.map_types(|ty| {
            check_type(module, source, ty)?;
            Ok(ty.clone())
        })?;
        attributes(source, &function.custom_attributes)?;
        // A free call can expose a type through its return without spelling it in IL.
        for op in &function.body {
            if let crate::metadata::Instruction::Call(target)
            | crate::metadata::Instruction::CallVirtual(target)
            | crate::metadata::Instruction::Construct(target) = op
            {
                check_signature(module, source, &crate::vm::resolve(module, target)?)?;
            }
        }
    }
    Ok(())
}
