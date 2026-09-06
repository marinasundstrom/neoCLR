pub mod assembler;
pub use assembler::assemble;
pub mod library;
pub mod memory;
pub mod metadata;
mod native;
mod program;
mod references;
mod type_identity;
pub mod value;
mod vm;

pub use type_identity::{TypeIdentity, resolve_type_identity, resolve_type_identity_with_library};

pub use metadata::Module;
pub use program::LoadedProgram;
pub use value::Value;
pub use vm::{Execution, Limits, run, run_with_library, run_with_native};

/// A terminal runtime/loader failure. Guest code cannot catch a Fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    pub message: String,
    pub function: Option<String>,
    pub instruction: Option<usize>,
}

impl Fault {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            function: None,
            instruction: None,
        }
    }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fault: {}", self.message)?;
        if let (Some(function), Some(instruction)) = (&self.function, self.instruction) {
            write!(f, " at {function}:{instruction}")?;
        }
        Ok(())
    }
}

pub fn load(source: &str) -> Result<Module, Fault> {
    let module: Module = serde_json::from_str(source)
        .map_err(|error| Fault::new(format!("invalid module: {error}")))?;
    vm::validate(&module)?;
    Ok(module)
}

/// Load a JSON artifact set with bundled System. First artifact is the root;
/// remaining artifacts are dependencies. Returns validated source modules in order.
pub fn load_modules(sources: &[&str]) -> Result<Vec<Module>, Fault> {
    let modules = sources
        .iter()
        .map(|source| {
            serde_json::from_str(source)
                .map_err(|error| Fault::new(format!("invalid module: {error}")))
        })
        .collect::<Result<Vec<Module>, _>>()?;
    let (root, dependencies) = modules
        .split_first()
        .ok_or_else(|| Fault::new("expected at least one module artifact"))?;
    library::link_modules(root, library::system()?, dependencies)?;
    Ok(modules)
}

mod numeric;

mod floating;

mod checked;

pub mod interop;

pub mod verifier;
pub use verifier::{Verification, verify, verify_with_library};
