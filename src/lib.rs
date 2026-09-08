mod arrays;
pub mod assembler;
pub use assembler::assemble;
mod access;
mod console;
pub mod debugger;
mod execution;
pub use console::{Console, StdioConsole};
mod delegates;
pub mod frontend;
mod gc;
pub use delegates::Delegate;
mod inheritance;
mod initialization;
mod input;
mod interfaces;
mod slots;
pub use execution::{CancellationToken, ExecutionOptions};
pub use gc::{CollectionEvent, CollectionReason, GcStatistics, ManagedHeap};
pub use slots::SlotReference;
mod char_categories;
mod file_io;
pub mod library;
mod math;
pub mod memory;
pub mod metadata;
mod native;
mod program;
mod reachability;
mod references;
mod reflection;
pub use reachability::{FunctionImplementation, Reachability, ReachableCall, ReachableFunction};
mod scope;
mod services;
mod stack_trace;
pub use services::{MissingService, RuntimeService, ServiceUse};
pub use stack_trace::{CodeLocation, StackFrame, StackTrace};
mod type_identity;
pub mod value;
mod vm;

pub use type_identity::{
    TypeDescriptor, TypeIdentity, resolve_type_identity, resolve_type_identity_with_library,
};

pub use metadata::Module;
pub use program::{LoadedFunction, LoadedProgram};
pub use value::Value;
pub use vm::{Execution, Limits, run, run_with_library, run_with_native};

/// A terminal runtime/loader failure. Guest code cannot catch a Fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    pub message: String,
    pub function: Option<String>,
    pub instruction: Option<usize>,
    /// Owned execution frames; absent for loading, verification, and host input faults.
    pub stack_trace: Option<StackTrace>,
}

impl Fault {
    pub(crate) fn with_stack_trace(mut self, trace: StackTrace) -> Self {
        if let Some(frame) = trace.frames.first() {
            self.function = Some(frame.function.name.clone());
            let CodeLocation::IlInstruction(index) = frame.location;
            self.instruction = Some(index);
        }
        self.stack_trace = Some(trace);
        self
    }

    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            function: None,
            instruction: None,
            stack_trace: None,
        }
    }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fault: {}", self.message)?;
        if let (Some(function), Some(instruction)) = (&self.function, self.instruction) {
            write!(f, " at {function}:{instruction}")?;
        }
        if let Some(trace) = &self.stack_trace {
            write!(f, "{trace}")?;
        }
        Ok(())
    }
}

pub fn load(source: &str) -> Result<Module, Fault> {
    let module = decode_module(source)?;
    vm::validate(&module)?;
    Ok(module)
}

pub(crate) fn decode_module(source: &str) -> Result<Module, Fault> {
    let value: serde_json::Value = serde_json::from_str(source)
        .map_err(|error| Fault::new(format!("invalid module: {error}")))?;
    if value.get("format").and_then(|v| v.as_u64()) != Some(5) {
        return Err(Fault::new(
            "unsupported module format (expected 5); reassemble source",
        ));
    }
    serde_json::from_value(value).map_err(|error| Fault::new(format!("invalid module: {error}")))
}

/// Load a JSON artifact set with bundled System. First artifact is the root;
/// remaining artifacts are dependencies. Returns validated source modules in order.
pub fn load_modules(sources: &[&str]) -> Result<Vec<Module>, Fault> {
    let inputs: Vec<_> = sources
        .iter()
        .map(|source| assembler::ModuleInput::Json(source))
        .collect();
    assembler::read_modules(&inputs, library::system()?)
}

mod numeric;

mod floating;

mod checked;

pub mod interop;

pub mod verifier;
pub use verifier::{Verification, verify, verify_with_library};

pub mod source;
