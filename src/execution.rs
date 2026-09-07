//! Per-execution host controls, separate from guest memory and async semantics.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{Fault, Limits};

/// A shared, monotonic cancellation request. Create a new token for independent work.
/// Cancelling does not wait for execution to stop and cannot interrupt native code.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    /// Request cancellation for every execution using this token or one of its clones.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

/// Interpreter limits and optional cooperative cancellation for one execution.
/// Existing calls may pass Limits directly; this is an experimental Rust API.
#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    pub limits: Limits,
    pub cancellation: Option<CancellationToken>,
}

impl From<Limits> for ExecutionOptions {
    fn from(limits: Limits) -> Self {
        Self {
            limits,
            cancellation: None,
        }
    }
}

impl ExecutionOptions {
    pub(crate) fn check_cancellation(
        &self,
        function: &str,
        instruction: usize,
    ) -> Result<(), Fault> {
        if self
            .cancellation
            .as_ref()
            .is_some_and(CancellationToken::is_cancelled)
        {
            return Err(Fault {
                message: "execution cancelled".into(),
                function: Some(function.into()),
                instruction: Some(instruction),
                stack_trace: None,
            });
        }
        Ok(())
    }
}
