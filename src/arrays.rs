//! Owned managed-array payloads, independent of the native System.Array<T> descriptor.
use crate::{Fault, Limits, Value, metadata::Type};

#[derive(Default)]
pub(crate) struct Usage {
    elements: usize,
    bytes: usize,
}

pub(crate) fn measure(value: &Value, usage: &mut Usage, limits: &Limits) -> Result<(), Fault> {
    let mut pending = vec![(value, false)];
    while let Some((value, inside)) = pending.pop() {
        if inside {
            usage.bytes = usage.bytes.saturating_add(std::mem::size_of::<Value>());
            if let Value::String(s) | Value::Error(s) = value {
                usage.bytes = usage.bytes.saturating_add(s.len());
            }
        }
        match value {
            Value::Array { elements, .. } => {
                usage.elements = usage.elements.saturating_add(elements.len());
                pending.extend(elements.iter().map(|v| (v, true)));
            }
            Value::Object { fields, .. } => pending.extend(fields.iter().map(|v| (v, inside))),
            Value::Erased(v) => pending.push((v, inside)),
            _ => (),
        }
        if usage.elements > limits.array_elements || usage.bytes > limits.array_bytes {
            return Err(Fault::new("array payload budget exceeded"));
        }
    }
    Ok(())
}

pub(crate) fn create(
    element: Type,
    length: usize,
    initial: Value,
    limits: &Limits,
) -> Result<Value, Fault> {
    if length > i32::MAX as usize {
        return Err(Fault::new("array length exceeds preview Int32 limit"));
    }
    initial.ensure_heap_references()?;
    if length == 0 {
        return Ok(Value::Array {
            element,
            elements: vec![],
        });
    }
    // Measure one initialized element before multiplying or copying any payload.
    let mut unit = Usage::default();
    let one = Value::Array {
        element: element.clone(),
        elements: vec![initial],
    };
    measure(&one, &mut unit, limits)?;
    if length
        .checked_mul(unit.elements)
        .is_none_or(|n| n > limits.array_elements)
        || length
            .checked_mul(unit.bytes)
            .is_none_or(|n| n > limits.array_bytes)
    {
        return Err(Fault::new("array payload budget exceeded"));
    }
    let Value::Array { elements, .. } = one else {
        unreachable!()
    };
    Ok(Value::Array {
        element,
        elements: vec![elements[0].clone(); length],
    })
}

/// Fixed shape preserves all existing index/field reference paths during replacement.
pub(crate) fn check_replacement(old: &Value, new: &Value) -> Result<(), Fault> {
    let mut pending = vec![(old, new)];
    while let Some((old, new)) = pending.pop() {
        match (old, new) {
            (Value::Array { elements: a, .. }, Value::Array { elements: b, .. }) => {
                if a.len() != b.len() {
                    return Err(Fault::new("array replacement requires equal lengths"));
                }
                pending.extend(a.iter().zip(b));
            }
            (Value::Object { fields: a, .. }, Value::Object { fields: b, .. }) => {
                pending.extend(a.iter().zip(b))
            }
            _ => (),
        }
    }
    Ok(())
}

pub(crate) fn index(value: Value) -> Result<usize, Fault> {
    match value {
        Value::Int32(n) => usize::try_from(n).map_err(|_| Fault::new("array index out of range")),
        Value::IntPtr(n) => usize::try_from(n).map_err(|_| Fault::new("array index out of range")),
        Value::UIntPtr(n) => Ok(n),
        _ => Err(Fault::new("array index requires Int32 or native integer")),
    }
}
