//! Bootstrap linking. System is compiled by the same assembler as application code.
use crate::{Fault, Module};
use std::sync::OnceLock;

/// Expanded System source, built from the same manifest used by the CLI.
pub fn system_source() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/System.neoil"))
}

pub fn system() -> Result<&'static Module, Fault> {
    static SYSTEM: OnceLock<Result<Module, Fault>> = OnceLock::new();
    SYSTEM
        .get_or_init(|| crate::assembler::parse_module(system_source()))
        .as_ref()
        .map_err(Clone::clone)
}

pub(crate) fn link(application: &Module, library: &Module) -> Result<Module, Fault> {
    link_modules(application, library, &[])
}

pub(crate) fn link_modules(
    application: &Module,
    library: &Module,
    dependencies: &[Module],
) -> Result<Module, Fault> {
    if library.name != "System" || !library.entry.is_empty() {
        return Err(Fault::new(
            "expected a System library module without an entry point",
        ));
    }
    let mut library = library.clone();
    library.normalize_definition_ids()?;
    let library = crate::scope::normalize_module(&library, &library)?;
    crate::vm::validate_linked(&library)?;
    crate::references::validate_list(&library, &[&library])?;
    let mut names = std::collections::HashSet::from([library.name.as_str()]);
    for (index, module) in std::iter::once(application).chain(dependencies).enumerate() {
        if module.name.is_empty() || !names.insert(module.name.as_str()) {
            return Err(Fault::new("empty or duplicate module name in load set"));
        }
        if module.format != 4 {
            return Err(Fault::new("unsupported module format (expected 4)"));
        }
        if index > 0 && !module.entry.is_empty() {
            return Err(Fault::new(
                "dependency module must not declare an entry point",
            ));
        }
    }
    let supplied: Vec<_> = std::iter::once(application)
        .chain(std::iter::once(&library))
        .chain(dependencies)
        .collect();
    for module in std::iter::once(application).chain(dependencies) {
        crate::references::validate_list(module, &supplied)?;
    }
    let mut linked = application.clone();
    linked.normalize_definition_ids()?;
    linked.types.extend(library.types.iter().cloned());
    linked.functions.extend(library.functions.iter().cloned());
    for dependency in dependencies {
        let mut dependency = dependency.clone();
        dependency.normalize_definition_ids()?;
        linked.types.extend(dependency.types);
        linked.functions.extend(dependency.functions);
    }
    let mut linked = crate::scope::normalize_module(&linked, &linked)?;
    crate::vm::validate_linked(&linked)?;
    bind_member_references(&mut linked)?;
    for source in std::iter::once(application).chain(dependencies) {
        let normalized = crate::scope::normalize_module(&linked, source)?;
        crate::references::validate_uses(&linked, &normalized)?;
    }
    Ok(linked)
}

// Bind symbolic references while their declaring generic context is still open.
// Specialization later substitutes signatures, but preserves the selected definition.
pub(crate) fn bind_member_references(module: &mut Module) -> Result<(), Fault> {
    let mut properties = Vec::new();
    for (index, definition) in module.types.iter().enumerate() {
        for (row, property) in definition.properties.iter().enumerate() {
            let mut property = property.clone();
            for accessor in property.getter.iter_mut().chain(property.setter.iter_mut()) {
                accessor.definition = crate::vm::resolve(module, accessor)?.definition;
            }
            properties.push((index, row, property));
        }
    }
    for (index, row, property) in properties {
        module.types[index].properties[row] = property;
    }
    let mut calls = Vec::new();
    for (function, definition) in module.functions.iter().enumerate() {
        for (pc, op) in definition.body.iter().enumerate() {
            if let crate::metadata::Instruction::Call(target)
            | crate::metadata::Instruction::CallVirtual(target)
            | crate::metadata::Instruction::Construct(target) = op
            {
                let identity = crate::vm::resolve(module, target)?.definition;
                calls.push((function, pc, identity));
            }
        }
    }
    for (function, pc, identity) in calls {
        if let crate::metadata::Instruction::Call(target)
        | crate::metadata::Instruction::CallVirtual(target)
        | crate::metadata::Instruction::Construct(target) =
            &mut module.functions[function].body[pc]
        {
            target.definition = identity;
        }
    }
    Ok(())
}
