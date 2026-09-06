pub mod assembler;
pub use assembler::assemble;
pub mod library;
pub mod memory;
pub mod metadata;
mod native;
pub mod value;
mod vm;

pub use metadata::Module;
pub use value::Value;
pub use vm::{Execution, Limits, run, run_with_library};

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
