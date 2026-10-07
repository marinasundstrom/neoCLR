//! Backend-neutral host presentation. Capture mechanisms remain backend-specific.
use crate::{CodeLocation, Fault, FaultCode, StackTrace};

/// Borrowed host view with a code-defined runtime message or the supplied user message.
///
/// Trace frames are innermost first. This view does not capture locals or arguments,
/// catch the fault, unwind guest cleanup, or terminate the host. `Display` prints a
/// shared compact format. Legacy `Fault::message` retains site-specific diagnostic
/// detail for compatibility; use this view for backend-independent presentation.
#[derive(Debug, Clone, Copy)]
pub struct FaultDiagnostic<'a> {
    pub code: FaultCode,
    pub message: &'a str,
    pub stack_trace: Option<&'a StackTrace>,
}
impl Fault {
    /// Get the common interpreter/native presentation contract without allocating.
    /// Runtime messages come from `FaultCode::standard_message`; UserFault preserves
    /// the supplied UTF-8 text, including empty text and embedded NUL bytes.
    pub fn diagnostic(&self) -> FaultDiagnostic<'_> {
        FaultDiagnostic {
            code: self.code,
            message: self.code.standard_message().unwrap_or(&self.message),
            stack_trace: self.stack_trace.as_ref(),
        }
    }
}
impl std::fmt::Display for FaultDiagnostic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", self.code, self.message)?;
        if let Some(trace) = self.stack_trace {
            for frame in &trace.frames {
                let CodeLocation::IlInstruction(pc) = frame.location;
                writeln!(f, "   at {} [instruction {pc}]", frame.function.name)?;
            }
            if trace.truncated {
                writeln!(f, "   ... stack trace truncated")?;
            }
        }
        Ok(())
    }
}
