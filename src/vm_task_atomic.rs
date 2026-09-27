//! Scheduling policy for the closed System.Tasks implementation. These regions
//! retain graph access across instruction quanta, not across user callbacks or I/O.
use super::Frame;
use crate::metadata::Function;

pub(super) fn queue_pump(function: &Function) -> bool {
    trusted_owner(function) == Some("System.Tasks.TaskQueue")
        && matches!(member(function), "Run" | "Drain")
}

fn trusted_owner(function: &Function) -> Option<&str> {
    if function.definition.as_ref()?.module != "System" {
        return None;
    }
    function.owner.as_ref()?.definition_name()
}

fn member(function: &Function) -> &str {
    function.name.rsplit('.').next().unwrap_or("")
}

fn region(function: &Function) -> bool {
    match trusted_owner(function) {
        Some("System.Tasks.Promise") => matches!(
            member(function),
            "Complete" | "Cancel" | "Register" | "Read" | "Completed" | "Cancelled"
        ),
        Some("System.Tasks.Task") => matches!(member(function), "get_State" | "get_Outcome"),
        Some("System.Tasks.TaskQueue") => {
            matches!(member(function), "Post" | "Run" | "Drain" | "get_Default")
        }
        _ => false,
    }
}

pub(super) fn active(frames: &[Frame]) -> bool {
    for frame in frames.iter().rev() {
        if region(&frame.function) {
            return true;
        }
        if frame.queue_callback {
            return false;
        }
    }
    false
}
