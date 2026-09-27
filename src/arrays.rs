//! Owned managed-array payloads, independent of the native System.Array<T> descriptor.
use crate::{Fault, Limits, Value, metadata::Type};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Usage {
    elements: usize,
    bytes: usize,
}

impl Usage {
    pub(crate) fn is_empty(&self) -> bool {
        self.elements == 0 && self.bytes == 0
    }

    pub(crate) fn add(&mut self, other: Self, limits: &Limits) -> Result<(), Fault> {
        self.elements = self.elements.saturating_add(other.elements);
        self.bytes = self.bytes.saturating_add(other.bytes);
        self.check(limits)
    }

    fn check(&self, limits: &Limits) -> Result<(), Fault> {
        if self.elements > limits.array_elements || self.bytes > limits.array_bytes {
            return Err(Fault::coded(
                crate::FaultCode::ArrayLimitExceeded,
                "array payload budget exceeded",
            ));
        }
        Ok(())
    }
}

pub(crate) fn measure(value: &Value, usage: &mut Usage, limits: &Limits) -> Result<(), Fault> {
    // Walk borrowed sibling iterators rather than copying every child onto a
    // work list. Leaf values and flat arrays need no traversal allocation.
    let mut pending = Vec::new();
    let mut current = std::slice::from_ref(value).iter().rev();
    let mut inside = false;
    loop {
        let Some(value) = current.next() else {
            let Some((siblings, parent_inside)) = pending.pop() else {
                break;
            };
            current = siblings;
            inside = parent_inside;
            continue;
        };
        if inside {
            usage.bytes = usage.bytes.saturating_add(std::mem::size_of::<Value>());
            // Quotas charge logical text per occurrence, even when owners are shared.
            let bytes = match value {
                Value::String(s) => s.len(),
                Value::Char(s) => s.len(),
                _ => 0,
            };
            usage.bytes = usage.bytes.saturating_add(bytes);
        }
        let children = match value {
            Value::Array { elements, .. } => {
                usage.elements = usage.elements.saturating_add(elements.len());
                Some((elements.as_slice(), true))
            }
            Value::Object { fields, .. } => Some((fields.as_slice(), inside)),
            Value::Erased(v) => Some((std::slice::from_ref(v.as_ref()), inside)),
            _ => None,
        };
        usage.check(limits)?;
        if let Some((children, child_inside)) = children {
            if !children.is_empty() {
                // No need to retain a parent with no remaining siblings. This
                // also keeps single-child chains iterative without a work list.
                if current.len() != 0 {
                    pending.push((current, inside));
                }
                current = children.iter().rev();
                inside = child_inside;
            }
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
        return Err(Fault::coded(
            crate::FaultCode::ArrayLimitExceeded,
            "array length exceeds preview Int32 limit",
        ));
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
        return Err(Fault::coded(
            crate::FaultCode::ArrayLimitExceeded,
            "array payload budget exceeded",
        ));
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
        if matches!(new, Value::Uninitialized(_)) && !matches!(old, Value::Uninitialized(_)) {
            return Err(Fault::new(
                "initialized array element cannot become uninitialized",
            ));
        }
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
        Value::Int32(n) => usize::try_from(n).map_err(|_| {
            Fault::coded(
                crate::FaultCode::IndexOutOfRange,
                "array index out of range",
            )
        }),
        Value::IntPtr(n) => usize::try_from(n).map_err(|_| {
            Fault::coded(
                crate::FaultCode::IndexOutOfRange,
                "array index out of range",
            )
        }),
        Value::UIntPtr(n) => Ok(n),
        _ => Err(Fault::new("array index requires Int32 or native integer")),
    }
}

/// Mutable array views preserve their exact element type, including typed nulls.
pub(crate) fn check_cast(source: &Type, target: &Type) -> Result<(), Fault> {
    if matches!((source, target), (Type::ArrayRef(_), Type::ArrayRef(_))) && source != target {
        return Err(Fault::new(
            "mutable array casts require identical element types",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested_payload() -> Value {
        let text: crate::StringValue = "Café".into();
        Value::Object {
            ty: Type::from_name("Container"),
            fields: vec![
                // Text outside an array does not count toward its payload budget.
                Value::String("outside".into()),
                Value::Array {
                    element: Type::Value,
                    elements: vec![
                        Value::String(text.clone()),
                        Value::String(text),
                        Value::Object {
                            ty: Type::from_name("Nested"),
                            fields: vec![
                                Value::Char("e\u{301}".into()),
                                Value::Array {
                                    element: Type::Byte,
                                    elements: vec![Value::Byte(1), Value::Byte(2)],
                                },
                            ],
                        },
                        Value::Erased(Box::new(Value::String("x".into()))),
                    ],
                },
                Value::Array {
                    element: Type::Byte,
                    elements: vec![],
                },
            ],
        }
    }

    #[test]
    fn nested_array_budget_counts_wrappers_and_shared_text_per_occurrence() {
        // Six array elements; nine Value payloads including the nested object,
        // array and erasure wrappers; 5 + 5 + 3 + 1 UTF-8 bytes.
        let bytes = 9 * std::mem::size_of::<Value>() + 14;
        let limits = Limits {
            array_elements: 6,
            array_bytes: bytes,
            ..Limits::default()
        };
        let mut usage = Usage::default();
        measure(&nested_payload(), &mut usage, &limits).unwrap();
        assert_eq!((usage.elements, usage.bytes), (6, bytes));
        for reduced in [
            Limits {
                array_elements: 5,
                ..limits
            },
            Limits {
                array_bytes: bytes - 1,
                ..limits
            },
        ] {
            let error = measure(&nested_payload(), &mut Usage::default(), &reduced).unwrap_err();
            assert_eq!(error.code, crate::FaultCode::ArrayLimitExceeded);
        }
    }

    #[test]
    fn array_budget_accumulates_across_roots_and_keeps_outer_text_free() {
        let mut usage = Usage::default();
        let limits = Limits::default();
        measure(&nested_payload(), &mut usage, &limits).unwrap();
        measure(&nested_payload(), &mut usage, &limits).unwrap();
        measure(&Value::String("outside".into()), &mut usage, &limits).unwrap();
        assert_eq!(usage.elements, 12);
        assert_eq!(usage.bytes, 2 * (9 * std::mem::size_of::<Value>() + 14));
    }

    #[test]
    fn deeply_erased_array_measurement_stays_iterative() {
        let mut value = Value::Array {
            element: Type::Byte,
            elements: vec![Value::Byte(42)],
        };
        for _ in 0..4096 {
            value = Value::Erased(Box::new(value));
        }
        let mut usage = Usage::default();
        measure(&value, &mut usage, &Limits::default()).unwrap();
        assert_eq!(
            (usage.elements, usage.bytes),
            (1, std::mem::size_of::<Value>())
        );
        // Avoid recursive destruction of the deliberately deep host fixture.
        while let Value::Erased(inner) = value {
            value = *inner;
        }
    }
}
