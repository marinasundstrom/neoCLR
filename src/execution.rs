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
    parents: Option<Arc<[CancellationToken; 2]>>,
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
            || self
                .parents
                .as_ref()
                .is_some_and(|parents| parents.iter().any(Self::is_cancelled))
    }

    pub(crate) fn linked(left: Self, right: Self) -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            parents: Some(Arc::new([left, right])),
        }
    }
}

/// Interpreter limits, optional cooperative cancellation, and an optional host console.
/// Existing calls may pass Limits directly; this is an experimental Rust API.
#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    /// Guest-visible argv; CLI includes the input path first. Empty by default.
    /// String-array entry points receive elements after argv[0]; Environment retains all.
    pub arguments: Vec<String>,
    pub debugger: Option<crate::debugger::Debugger>,
    pub limits: Limits,
    pub cancellation: Option<CancellationToken>,
    /// None captures output and leaves input unavailable; Some performs live host I/O.
    pub console: Option<Arc<dyn crate::Console>>,
}

impl From<Limits> for ExecutionOptions {
    fn from(limits: Limits) -> Self {
        Self {
            arguments: Vec::new(),
            debugger: None,
            limits,
            cancellation: None,
            console: None,
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
                code: crate::FaultCode::ExecutionCancelled,
                message: "execution cancelled".into(),
                function: Some(function.into()),
                instruction: Some(instruction),
                stack_trace: None,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod linked_tests {
    use super::*;

    #[test]
    fn linked_cancellation_observes_either_parent_without_cancelling_them() {
        for index in 0..2 {
            let parents = [CancellationToken::new(), CancellationToken::new()];
            let linked = CancellationToken::linked(parents[0].clone(), parents[1].clone());
            assert!(!linked.is_cancelled());
            parents[index].cancel();
            assert!(linked.is_cancelled());
            assert!(!parents[1 - index].is_cancelled());
        }
        let left = CancellationToken::new();
        let right = CancellationToken::new();
        CancellationToken::linked(left.clone(), right.clone()).cancel();
        assert!(!left.is_cancelled() && !right.is_cancelled());
    }
}
