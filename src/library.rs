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
    crate::vm::validate_linked(library)?;
    let mut linked = application.clone();
    linked.types.extend(library.types.iter().cloned());
    linked.functions.extend(library.functions.iter().cloned());
    crate::vm::validate_linked(&linked)?;
    Ok(linked)
}
