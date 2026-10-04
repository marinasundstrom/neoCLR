//! Scheduling policy for the closed System.Tasks implementation. These regions
//! retain graph access across instruction quanta, not across user callbacks or I/O.
use super::Frame;
use crate::{metadata::Function, Module};

pub(super) fn queue_pump(module: &Module, function: &Function) -> bool {
    trusted_owner(module, function) == Some("System.Tasks.TaskQueue")
        && matches!(member(function), "Run" | "Drain")
}

fn trusted_owner<'a>(module: &'a Module, function: &'a Function) -> Option<&'a str> {
    let owner = function.owner.as_ref()?;
    if function.definition.as_ref()?.module == "System" {
        return owner.definition_name();
    }
    let definition = module.type_definition(owner)?;
    let origin = definition.origin.as_ref()?;
    if function.origin.as_ref()?.assembly != origin.assembly { return None; }
    match origin.name.as_str() {
        "System.Tasks.TaskQueue" => Some("System.Tasks.TaskQueue"),
        "System.Tasks.Promise`1" => Some("System.Tasks.Promise"),
        "System.Tasks.Task`1" => Some("System.Tasks.Task"),
        _ => None,
    }
}

fn member(function: &Function) -> &str {
    super::task_queue::member(function)
}

fn region(module: &Module, function: &Function) -> bool {
    match trusted_owner(module, function) {
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

pub(super) fn active(module: &Module, frames: &[Frame]) -> bool {
    for frame in frames.iter().rev() {
        if region(module, &frame.function) {
            return true;
        }
        if frame.queue_callback {
            return false;
        }
    }
    false
}
