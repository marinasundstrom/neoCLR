//! Stable symbolic classifications for terminal faults, independent of diagnostics.
use serde::{Serialize, Serializer};

/// A host-visible terminal failure category, not a guest exception or recoverable Result.
///
/// Guest code cannot choose this value. Explicit guest faults always use `UserFault`.
/// Match with a fallback: future runtime releases may add categories. Use `as_str()`
/// or serialization for stable identifiers rather than an enum ordinal.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaultCode {
    /// A runtime/host failure without a more specific classification yet.
    RuntimeError,
    /// An explicit guest `fault` instruction or `System.Fault(message)` call.
    UserFault,
    /// Invalid program metadata or verified instruction behavior.
    InvalidProgram,
    /// The interpreter's configured call-frame limit was reached.
    StackOverflow,
    /// The interpreter's configured operand-stack limit was reached.
    EvaluationStackOverflow,
    /// The invocation exhausted its instruction budget.
    InstructionLimitExceeded,
    /// The host cancelled or stopped execution (not guest Task cancellation).
    ExecutionCancelled,
    /// An operation dereferenced a null managed object, array or interface.
    NullReference,
    /// A boxed value did not have the exact requested unboxing type.
    InvalidCast,
    /// An operation dereferenced a null native pointer.
    NullPointer,
    /// A managed array index was outside its valid range.
    IndexOutOfRange,
    /// Checked integer arithmetic or conversion overflowed.
    ArithmeticOverflow,
    /// Integer division or remainder used a zero divisor.
    DivideByZero,
    /// The managed object or identity budget was exhausted.
    HeapLimitExceeded,
    /// Managed array payload or supported-length limits were exceeded.
    ArrayLimitExceeded,
    /// The native pointer heap byte or allocation budget was exceeded.
    NativeMemoryLimitExceeded,
    /// The execution's String intern entry or UTF-8 payload budget was exceeded.
    InternPoolLimitExceeded,
}
impl FaultCode {
    /// Stable, case-sensitive identifier for logs and machine-readable diagnostics.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RuntimeError => "RuntimeError",
            Self::UserFault => "UserFault",
            Self::InvalidProgram => "InvalidProgram",
            Self::StackOverflow => "StackOverflow",
            Self::EvaluationStackOverflow => "EvaluationStackOverflow",
            Self::InstructionLimitExceeded => "InstructionLimitExceeded",
            Self::ExecutionCancelled => "ExecutionCancelled",
            Self::NullReference => "NullReference",
            Self::InvalidCast => "InvalidCast",
            Self::NullPointer => "NullPointer",
            Self::IndexOutOfRange => "IndexOutOfRange",
            Self::ArithmeticOverflow => "ArithmeticOverflow",
            Self::DivideByZero => "DivideByZero",
            Self::HeapLimitExceeded => "HeapLimitExceeded",
            Self::ArrayLimitExceeded => "ArrayLimitExceeded",
            Self::NativeMemoryLimitExceeded => "NativeMemoryLimitExceeded",
            Self::InternPoolLimitExceeded => "InternPoolLimitExceeded",
        }
    }
}
impl std::fmt::Display for FaultCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
impl Serialize for FaultCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
