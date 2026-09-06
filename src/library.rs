//! Bootstrap linking. System is compiled by the same assembler as application code.
use crate::{Fault, Module};
use std::sync::OnceLock;

pub fn system() -> Result<&'static Module, Fault> {
    static SYSTEM: OnceLock<Result<Module, Fault>> = OnceLock::new();
    SYSTEM
        .get_or_init(|| crate::assembler::parse_module(include_str!("../runtime/System.neoil")))
        .as_ref()
        .map_err(Clone::clone)
}

pub(crate) fn link(application: &Module, library: &Module) -> Result<Module, Fault> {
    if library.name != "System" || !library.entry.is_empty() {
        return Err(Fault::new(
            "expected a System library module without an entry point",
        ));
    }
    let mut library = library.clone();
    library.normalize_member_ids()?;
    crate::vm::validate_linked(&library)?;
    let mut linked = application.clone();
    linked.normalize_member_ids()?;
    linked.types.extend(library.types.iter().cloned());
    linked.functions.extend(library.functions.iter().cloned());
    crate::vm::validate_linked(&linked)?;
    bind_member_references(&mut linked)?;
    Ok(linked)
}

// Bind symbolic references while their declaring generic context is still open.
// Specialization later substitutes signatures, but preserves the selected definition.
pub(crate) fn bind_member_references(module: &mut Module) -> Result<(), Fault> {
    let mut calls = Vec::new();
    for (function, definition) in module.functions.iter().enumerate() {
        for (pc, op) in definition.body.iter().enumerate() {
            if let crate::metadata::Instruction::Call(target) = op {
                let identity = crate::vm::resolve(module, target)?.definition;
                calls.push((function, pc, identity));
            }
        }
    }
    for (function, pc, identity) in calls {
        if let crate::metadata::Instruction::Call(target) = &mut module.functions[function].body[pc]
        {
            target.definition = identity;
        }
    }
    Ok(())
}
