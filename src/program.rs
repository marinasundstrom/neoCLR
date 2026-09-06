//! Immutable resolved metadata shared by execution and analysis.
use crate::{Execution, Fault, Limits, Module, TypeIdentity, Verification, metadata::Type};

/// A validated, linked metadata snapshot with calls bound before specialization.
///
/// Loading does not perform typed verification or execute code. Each execution owns
/// fresh frames, allocations, output, and native-library state. This Rust API is not
/// a stable native hosting ABI or a serialized artifact format.
#[derive(Debug)]
pub struct LoadedProgram {
    module: Module,
}

impl LoadedProgram {
    /// Prepare an application with bundled System, or prepare System alone for analysis.
    pub fn new(module: &Module) -> Result<Self, Fault> {
        if module.name == "System" {
            crate::vm::validate(module)?;
            let mut module = module.clone();
            module.normalize_definition_ids()?;
            crate::library::bind_member_references(&mut module)?;
            Ok(Self { module })
        } else {
            Self::with_library(module, crate::library::system()?)
        }
    }

    /// Prepare an application with an explicitly supplied System artifact.
    pub fn with_library(module: &Module, library: &Module) -> Result<Self, Fault> {
        Ok(Self {
            module: crate::library::link(module, library)?,
        })
    }

    /// Analyze the already-bound metadata without executing code.
    pub fn verify(&self) -> Result<Verification, Fault> {
        crate::verifier::analyze(&self.module)
    }

    /// Resolve a closed signature in this snapshot, without relinking.
    pub fn resolve_type_identity(&self, ty: &Type) -> Result<TypeIdentity, Fault> {
        crate::type_identity::resolve(&self.module, ty)
    }

    /// Execute the entry point with fresh state and native imports disabled.
    pub fn run(&self, limits: Limits) -> Result<Execution, Fault> {
        self.check_entry()?;
        crate::vm::interpret(&self.module, limits, None)
    }

    /// Execute the entry point with fresh state and native imports enabled.
    ///
    /// # Safety
    /// Every executed native declaration must match its exported C ABI signature.
    /// Native code and library initializers/destructors must uphold pointer validity,
    /// allocation lifetimes, and Rust's memory safety requirements. Guest metadata
    /// alone cannot establish these guarantees. The caller must trust the code.
    pub unsafe fn run_with_native(&self, limits: Limits) -> Result<Execution, Fault> {
        self.check_entry()?;
        crate::vm::interpret(
            &self.module,
            limits,
            Some(crate::interop::NativeLibraries::default()),
        )
    }

    fn check_entry(&self) -> Result<(), Fault> {
        if self.module.entry.is_empty() {
            return Err(Fault::new(
                "cannot execute a library without an entry point",
            ));
        }
        Ok(())
    }
}
