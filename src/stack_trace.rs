//! Owned logical guest frames. Source locations are optional; native unwinding remains separate.
use crate::metadata::{Function, FunctionRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeLocation {
    /// An index into the IL instruction vector, not a CLI byte offset.
    IlInstruction(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    pub source: Option<crate::metadata::SequencePoint>,
    pub function: FunctionRef,
    pub location: CodeLocation,
}

/// An owned snapshot, innermost frame first. Captures no arguments or local values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
    pub truncated: bool,
}

impl StackTrace {
    pub const MAX_FRAMES: usize = 64;

    pub(crate) fn capture<'a>(mut frames: impl Iterator<Item = (&'a Function, usize)>) -> Self {
        let mut trace = Self {
            frames: Vec::new(),
            truncated: false,
        };
        if trace.frames.try_reserve(Self::MAX_FRAMES).is_err() {
            trace.truncated = true;
            return trace;
        }
        for (function, instruction) in frames.by_ref().take(Self::MAX_FRAMES) {
            trace.frames.push(StackFrame {
                source: function
                    .sequence_points
                    .iter()
                    .rev()
                    .find(|p| p.instruction <= instruction)
                    .cloned(),
                function: FunctionRef {
                    definition: function.definition.clone(),
                    name: function.name.clone(),
                    owner: function.owner.clone(),
                    instance: function.instance,
                    generic_arguments: function.generic_arguments.clone(),
                    parameters: function.parameters.clone(),
                },
                location: CodeLocation::IlInstruction(instruction),
            });
        }
        trace.truncated = frames.next().is_some();
        trace
    }
}

impl std::fmt::Display for StackTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for frame in &self.frames {
            write!(f, "\n  at {}(", frame.function.name)?;
            for (index, parameter) in frame.function.parameters.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{parameter:?}")?;
            }
            write!(f, ")")?;
            if let Some(owner) = &frame.function.owner {
                write!(f, " owner={owner:?}")?;
            }
            if let Some(id) = &frame.function.definition {
                write!(f, " [{}", id.module)?;
                if let Some(revision) = &id.revision {
                    write!(f, "#{revision}")?;
                }
                write!(f, ":{}]", id.index)?;
            }
            let CodeLocation::IlInstruction(index) = frame.location;
            write!(f, " IL instruction {index}")?;
            if let Some(source) = &frame.source {
                write!(
                    f,
                    " at {}:{}:{}",
                    source.document, source.line, source.column
                )?;
            }
        }
        if self.truncated {
            write!(f, "\n  ... stack trace truncated")?;
        }
        Ok(())
    }
}

/// Diagnostic-only decoding of the experimental native projection's reserved
/// transport names. Malformed/unrecognized spellings remain unchanged. This does
/// not change binding identity, and source-origin names should take precedence.
pub fn diagnostic_function_name(name: &str) -> String {
    fn hex(text: &str) -> Option<String> {
        if text.len() % 2 != 0 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let bytes = (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
            .collect::<Option<Vec<_>>>()?;
        String::from_utf8(bytes)
            .ok()
            .filter(|s| !s.chars().any(char::is_control))
    }
    let decode = || -> Option<String> {
        let (module, member) = name.strip_prefix("NeoMetadata_")?.split_once('.')?;
        if module.len() != 64 || !module.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        if let Some(function) = member.strip_prefix("F_") {
            let function = hex(function)?;
            if let Some(encoded) = function.strip_prefix("<NeoFunction>") {
                let (namespace, member) = encoded.split_once('_')?;
                let namespace = hex(namespace)?;
                let member = hex(member)?;
                return Some(if namespace.is_empty() {
                    member
                } else {
                    format!("{namespace}.{member}")
                });
            }
            return Some(function);
        }
        let (owner, method) = member.strip_prefix("T_")?.split_once('.')?;
        let (namespace, owner) = owner.split_once('_')?;
        let namespace = hex(namespace)?;
        let owner = hex(owner)?;
        let method = if let Some(encoded) = method.strip_prefix("M_") {
            hex(encoded)?
        } else {
            method.to_owned()
        };
        Some(if namespace.is_empty() {
            format!("{owner}.{method}")
        } else {
            format!("{namespace}.{owner}.{method}")
        })
    };
    decode().unwrap_or_else(|| name.to_owned())
}
