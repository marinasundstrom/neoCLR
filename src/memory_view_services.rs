//! Exact InternalCall signatures for the interpreter memory-view foundation.
use crate::memory_views::{Memory, Slice, Utf8Slice};
use crate::{
    Fault, ManagedHeap, MemoryView, Value,
    metadata::{Function, MemoryViewKind as Kind, Type},
};

#[derive(Clone, Debug)]
pub(crate) enum Operation {
    BoundsFault,
    Create(Kind, Type),
    Length,
    Get,
    Set,
    Slice,
    ReadOnly,
    AsSlice,
    Utf8,
}

fn ty(kind: Kind, element: &Type) -> Type {
    Type::Constructed {
        definition: kind.name().into(),
        arguments: vec![element.clone()],
    }
}

fn view(ty: &Type, element: &Type) -> Option<Kind> {
    [
        Kind::Slice,
        Kind::ReadOnlySlice,
        Kind::Memory,
        Kind::ReadOnlyMemory,
    ]
    .into_iter()
    .find(|kind| *ty == self::ty(*kind, element))
}

pub(crate) fn bind(function: &Function) -> Option<Result<Operation, Fault>> {
    let name = function.name.strip_prefix("neoCLR.Runtime.View")?;
    Some((|| {
        let invalid = || Fault::new("invalid memory-view InternalCall signature");
        if name == "BoundsFault" {
            if !function.parameters.is_empty()
                || function.returns != Type::Void
                || !function.no_result
                || !function.generic_parameters.is_empty()
                || !function.generic_arguments.is_empty()
                || !function.generic_constraints.is_empty()
                || function.receiver_byref
                || function.receiver_readonly
                || !function.body.is_empty()
                || !function.locals.is_empty()
                || function.pinvoke.is_some()
                || function.is_virtual
                || function.is_override
                || function.is_abstract
                || !function.interface_implementations.is_empty()
                || !function.out_parameters.is_empty()
                || !function.out_when_true.is_empty()
                || !function.readonly_parameters.is_empty()
            {
                return Err(invalid());
            }
            return Ok(Operation::BoundsFault);
        }
        if name == "Utf8" {
            if !function.generic_parameters.is_empty()
                || !function.generic_arguments.is_empty()
                || function.parameters != [Type::String]
                || function.returns != ty(Kind::ReadOnlySlice, &Type::Byte)
                || function.no_result
            {
                return Err(invalid());
            }
            return Ok(Operation::Utf8);
        }
        if function.generic_parameters.len() != 1
            || !function.generic_constraints.is_empty()
            || function.no_result && name != "Set"
        {
            return Err(invalid());
        }
        let element = match function.generic_arguments.as_slice() {
            [] => Type::MethodTypeParameter(0),
            [element] => element.clone(),
            _ => return Err(invalid()),
        };
        let parameters = &function.parameters;
        if let Some(kind) = match name {
            "CreateSlice" => Some(Kind::Slice),
            "CreateReadOnlySlice" => Some(Kind::ReadOnlySlice),
            "CreateMemory" => Some(Kind::Memory),
            "CreateReadOnlyMemory" => Some(Kind::ReadOnlyMemory),
            _ => None,
        } {
            let array = Type::Array(Box::new(element.clone()));
            let readonly = matches!(kind, Kind::ReadOnlySlice | Kind::ReadOnlyMemory);
            let input = parameters.first().ok_or_else(invalid)?;
            if !(*input == Type::ArrayRef(Box::new(element.clone()))
                || *input == Type::ByRef(Box::new(array.clone()))
                || readonly && *input == Type::ReadOnlyByRef(Box::new(array)))
                || !(parameters.len() == 1
                    || parameters.len() == 3 && parameters[1..] == [Type::Int32, Type::Int32])
                || function.returns != ty(kind, &element)
            {
                return Err(invalid());
            }
            return Ok(Operation::Create(kind, element));
        }
        let kind = parameters
            .first()
            .and_then(|input| view(input, &element))
            .ok_or_else(invalid)?;
        let tail = &parameters[1..];
        let (operation, expected) = match name {
            "Length" if tail.is_empty() => (Operation::Length, Type::Int32),
            "Get" if kind.is_borrowed() && tail == [Type::Int32] => {
                (Operation::Get, element.clone())
            }
            "Set" if kind == Kind::Slice && tail == [Type::Int32, element.clone()] => {
                (Operation::Set, Type::Void)
            }
            "Slice" if tail == [Type::Int32] || tail == [Type::Int32, Type::Int32] => {
                (Operation::Slice, ty(kind, &element))
            }
            "AsReadOnly" if tail.is_empty() => (
                Operation::ReadOnly,
                ty(
                    if kind.is_borrowed() {
                        Kind::ReadOnlySlice
                    } else {
                        Kind::ReadOnlyMemory
                    },
                    &element,
                ),
            ),
            "AsSlice" if !kind.is_borrowed() && tail.is_empty() => (
                Operation::AsSlice,
                ty(
                    if kind == Kind::Memory {
                        Kind::Slice
                    } else {
                        Kind::ReadOnlySlice
                    },
                    &element,
                ),
            ),
            _ => return Err(invalid()),
        };
        if function.returns != expected {
            return Err(invalid());
        }
        Ok(operation)
    })())
}

impl Operation {
    pub(crate) fn invoke(&self, args: Vec<Value>, heap: &ManagedHeap) -> Result<Value, Fault> {
        let result = match (self, args.as_slice()) {
            (Self::BoundsFault, []) => return Err(out_of_range()),
            (Self::Utf8, [Value::String(text)]) => MemoryView::utf8(Utf8Slice::new(text.clone())),
            (Self::Create(kind, element), [array, bounds @ ..]) => {
                let reference = match array {
                    Value::ObjectReference(object) => Some(object.reference.clone()),
                    Value::SlotReference(reference) => Some(reference.clone()),
                    Value::NullObjectReference(ty)
                        if *ty == Type::ArrayRef(Box::new(element.clone())) =>
                    {
                        None
                    }
                    _ => {
                        return Err(Fault::new(
                            "memory-view factory requires an array reference",
                        ));
                    }
                };
                let total = reference
                    .as_ref()
                    .map(|reference| reference.array_length())
                    .transpose()?
                    .unwrap_or(0);
                let (start, count) = match bounds {
                    [] => (0, total),
                    [start, count] => (index(start)?, index(count)?),
                    _ => return Err(Fault::new("invalid memory-view factory bounds")),
                };
                if start > total || count > total - start {
                    return Err(out_of_range());
                }
                if let Some(mut reference) = reference {
                    if reference.target() != &Type::Array(Box::new(element.clone())) {
                        return Err(Fault::new("memory-view element type mismatch"));
                    }
                    if reference.allocation_id().is_some() {
                        heap.check_reference_owner(&reference)?;
                    }
                    if matches!(kind, Kind::ReadOnlySlice | Kind::ReadOnlyMemory) {
                        reference.restrict_readonly();
                    } else {
                        reference.require_writable()?;
                    }
                    if kind.is_borrowed() {
                        MemoryView::borrowed(Slice::new(reference, start, count)?)
                    } else {
                        MemoryView::persistent(Memory::new(heap, reference, start, count)?)
                    }
                } else {
                    MemoryView::empty(*kind, element.clone())
                }
            }
            (operation, [Value::MemoryView(view), tail @ ..]) => {
                view.assigned()?;
                match (operation, tail) {
                    (Self::Length, []) => return Ok(Value::Int32(view.length() as i32)),
                    (Self::Get, [position]) => return view.get(index(position)?),
                    (Self::Set, [position, value]) => {
                        view.set(index(position)?, value.clone())?;
                        return Ok(Value::Void);
                    }
                    (Self::Slice, [start]) => {
                        let start = index(start)?;
                        let count = view.length().checked_sub(start).ok_or_else(out_of_range)?;
                        view.sliced(start, count)?
                    }
                    (Self::Slice, [start, count]) => view.sliced(index(start)?, index(count)?)?,
                    (Self::ReadOnly, []) => view.readonly(),
                    (Self::AsSlice, []) => view.as_slice(),
                    _ => return Err(Fault::new("invalid memory-view operation arguments")),
                }
            }
            _ => return Err(Fault::new("invalid memory-view operation arguments")),
        };
        Ok(Value::MemoryView(result))
    }
}

fn out_of_range() -> Fault {
    Fault::coded(
        crate::FaultCode::IndexOutOfRange,
        "memory view bounds out of range",
    )
}

fn index(value: &Value) -> Result<usize, Fault> {
    match value {
        Value::Int32(value) => usize::try_from(*value).map_err(|_| out_of_range()),
        _ => Err(Fault::new("memory view index requires Int32")),
    }
}

#[cfg(test)]
mod bounds_tests {
    use super::*;
    #[test]
    fn bounds_fault_requires_exact_nonreturning_service_contract() {
        let f: Function = serde_json::from_value(serde_json::json!({
            "name":"neoCLR.Runtime.ViewBoundsFault", "parameters":[],
            "returns":Type::Void, "no_result":true, "impl_flags":4096
        }))
        .unwrap();
        assert!(matches!(bind(&f).unwrap().unwrap(), Operation::BoundsFault));
        for variant in 0..4 {
            let mut bad = f.clone();
            match variant {
                0 => bad.no_result = false,
                1 => bad.parameters.push(Type::Int32),
                2 => bad.returns = Type::Int32,
                _ => bad.generic_parameters.push(Some("T".into())),
            }
            assert!(bind(&bad).unwrap().is_err());
        }
    }
}
