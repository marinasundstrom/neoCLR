//! Materialize trusted metadata snapshots against the service's scoped result signature.
//! Logical snapshot names never enter the loaded type catalog or guest values.
use crate::metadata::{Representation, Type};
use crate::{Fault, Limits, ManagedHeap, Module, TypeIdentity, Value};

pub(crate) fn name<'a>(module: &'a Module, ty: &'a Type) -> Option<&'a str> {
    module
        .type_definition(ty)
        .and_then(|d| d.origin.as_ref())
        .map(|o| o.name.as_str())
        .or_else(|| ty.definition_name())
}

pub(crate) fn uses_source(module: &Module, ty: &Type) -> bool {
    match ty {
        Type::ArrayRef(_) => true,
        Type::Array(t) => uses_source(module, t),
        _ => module
            .type_definition(ty)
            .and_then(|d| d.origin.as_ref())
            .is_some_and(|o| {
                o.name.starts_with("System.Introspection.")
                    || o.name == "System.Option"
                    || o.name == "System.Option`1"
            }),
    }
}

fn allocate(
    heap: &mut ManagedHeap,
    limits: &Limits,
    value: Value,
    view: Type,
) -> Result<Value, Fault> {
    if heap.len() >= limits.heap_objects {
        return Err(Fault::coded(
            crate::FaultCode::HeapLimitExceeded,
            "heap object limit exceeded",
        ));
    }
    let index = heap.allocate(value)?;
    Ok(Value::ObjectReference(crate::value::ObjectReference {
        reference: heap.address(index)?,
        view: Some(view),
    }))
}

pub(crate) fn provider(module: &Module, contract: &Type, fields: &[Value]) -> Result<Type, Fault> {
    let definition = module
        .type_definition(contract)
        .ok_or_else(|| Fault::new("missing snapshot contract"))?;
    let origin = definition
        .origin
        .as_ref()
        .ok_or_else(|| Fault::new("missing snapshot source identity"))?;
    if definition.representation != Representation::Interface
        || !definition.generic_parameters.is_empty()
    {
        return Err(Fault::new("invalid source snapshot contract"));
    }
    if origin.name == "System.Introspection.ModuleInfo" {
        return crate::reflection::source_module_provider(module, contract);
    }
    let suffix = match origin.name.as_str() {
        "System.Introspection.TypeInfo" => match fields.first() {
            Some(Value::RuntimeTypeHandle(handle)) => match &handle.identity {
                TypeIdentity::Definition { .. } => "RuntimeNominalTypeInfo",
                TypeIdentity::Function { .. } => "RuntimeFunctionTypeInfo",
                _ => "RuntimeTypeInfo",
            },
            _ => return Err(Fault::new("missing source TypeInfo handle")),
        },
        "System.Introspection.AssemblyInfo" => "RuntimeAssemblyInfo",
        "System.Introspection.FieldInfo" => "RuntimeFieldInfo",
        "System.Introspection.MethodInfo" => "RuntimeMethodInfo",
        "System.Introspection.ConstructorInfo" => "RuntimeConstructorInfo",
        "System.Introspection.PropertyInfo" => "RuntimePropertyInfo",
        "System.Introspection.ParameterInfo" => "RuntimeParameterInfo",
        _ => return Err(Fault::new("unsupported source snapshot contract")),
    };
    let full_name = format!("System.Introspection.{suffix}");
    let mut matches = module.types.iter().filter(|d| {
        d.origin.as_ref().is_some_and(|o| {
            o.assembly == origin.assembly && o.module == origin.module && o.name == full_name
        })
    });
    let selected = matches
        .next()
        .ok_or_else(|| Fault::new("missing source snapshot provider"))?;
    if matches.next().is_some()
        || !selected.is_reference_type
        || selected.is_abstract
        || !selected.generic_parameters.is_empty()
        || selected.declaring_type.is_some()
        || !module.reference_assignable(&selected.open_type(), contract)
    {
        return Err(Fault::new("invalid or ambiguous source snapshot provider"));
    }
    Ok(selected.open_type())
}

pub(crate) fn materialize(
    module: &Module,
    heap: &mut ManagedHeap,
    limits: &Limits,
    value: Value,
    expected: &Type,
) -> Result<Value, Fault> {
    build(module, heap, limits, value, expected, 0)
}

fn build(
    module: &Module,
    heap: &mut ManagedHeap,
    limits: &Limits,
    value: Value,
    expected: &Type,
    depth: usize,
) -> Result<Value, Fault> {
    if depth > 64 {
        return Err(Fault::new("reflection snapshot nesting limit exceeded"));
    }
    match value {
        Value::Object { ty, mut fields } if ty == Type::from_name("$ReflectionSnapshot.Option") => {
            if !matches!(
                name(module, expected),
                Some("System.Option" | "System.Option`1")
            ) || expected.generic_arguments().len() != 1
            {
                return Err(Fault::new(
                    "source snapshot requires scoped Option signature",
                ));
            }
            let layout = crate::inheritance::fields(module, expected)?;
            let [tag, some, none] = layout.as_slice() else {
                return Err(Fault::new("source Option layout mismatch"));
            };
            let some_fields = crate::inheritance::fields(module, &some.ty)?;
            if tag.ty != Type::Byte
                || some_fields.len() != 1
                || some_fields[0].ty != expected.generic_arguments()[0]
                || !crate::inheritance::fields(module, &none.ty)?.is_empty()
            {
                return Err(Fault::new("source Option case layout mismatch"));
            }
            let present = match fields.first() {
                Some(Value::Boolean(value)) => *value,
                _ => return Err(Fault::new("invalid Option recipe")),
            };
            if fields.len() != if present { 2 } else { 1 } {
                return Err(Fault::new("invalid Option recipe payload"));
            }
            let some_value = if present {
                let value = build(
                    module,
                    heap,
                    limits,
                    fields.pop().unwrap(),
                    &some_fields[0].ty,
                    depth + 1,
                )?;
                Value::Object {
                    ty: some.ty.clone(),
                    fields: vec![value],
                }
            } else {
                crate::initialization::default_value(module, &some.ty)?
            };
            Ok(Value::Object {
                ty: expected.clone(),
                fields: vec![
                    Value::Byte(if present { 1 } else { 2 }),
                    some_value,
                    crate::initialization::default_value(module, &none.ty)?,
                ],
            })
        }
        Value::Array { elements, .. } => {
            if name(module, expected) == Some("System.Runtime.CompilerServices.ParameterSnapshot") {
                let definition = module
                    .type_definition(expected)
                    .ok_or_else(|| Fault::new("missing ParameterSnapshot"))?;
                let layout = crate::inheritance::fields(module, expected)?;
                if !definition.is_reference_type
                    || definition.is_abstract
                    || layout.len() != 1
                    || layout[0].name != "items"
                    || !matches!(&layout[0].ty, Type::ArrayRef(element) if name(module, element) == Some("System.Introspection.ParameterInfo"))
                {
                    return Err(Fault::new("source ParameterSnapshot layout mismatch"));
                }
                let array = build(
                    module,
                    heap,
                    limits,
                    Value::Array {
                        element: Type::Void,
                        elements,
                    },
                    &layout[0].ty,
                    depth + 1,
                )?;
                return allocate(
                    heap,
                    limits,
                    Value::Object {
                        ty: expected.clone(),
                        fields: vec![array],
                    },
                    expected.clone(),
                );
            }
            let element = match expected {
                Type::ArrayRef(element) | Type::Array(element) => element.as_ref(),
                _ => return Err(Fault::new("source snapshot array signature mismatch")),
            };
            let elements = elements
                .into_iter()
                .map(|v| build(module, heap, limits, v, element, depth + 1))
                .collect::<Result<Vec<_>, _>>()?;
            let array = Value::Array {
                element: element.clone(),
                elements,
            };
            let mut usage = crate::arrays::Usage::default();
            crate::arrays::measure(&array, &mut usage, limits)?;
            if matches!(expected, Type::ArrayRef(_)) {
                allocate(heap, limits, array, expected.clone())
            } else {
                Ok(array)
            }
        }
        Value::Object { ty, fields } => {
            let logical = ty
                .definition_name()
                .ok_or_else(|| Fault::new("missing snapshot name"))?;
            let expected_name =
                name(module, expected).ok_or_else(|| Fault::new("missing source snapshot name"))?;
            if logical != expected_name
                && !(logical == "System.Type" && expected_name == "System.Introspection.TypeInfo")
            {
                return Err(Fault::new(format!(
                    "snapshot contract mismatch: {logical} -> {expected_name}"
                )));
            }
            let owner = if module
                .type_definition(expected)
                .is_some_and(|d| d.representation == Representation::Interface)
            {
                provider(module, expected, &fields)?
            } else {
                expected.clone()
            };
            let layout = crate::inheritance::fields(module, &owner)?;
            if fields.len() != layout.len() {
                return Err(Fault::new(format!(
                    "source snapshot layout mismatch: {expected_name}: {} vs {}",
                    fields.len(),
                    layout.len()
                )));
            }
            let fields = fields
                .into_iter()
                .zip(layout)
                .map(|(value, field)| build(module, heap, limits, value, &field.ty, depth + 1))
                .collect::<Result<Vec<_>, _>>()?;
            let value = Value::Object {
                ty: owner.clone(),
                fields,
            };
            if module.is_reference_type(&owner) {
                allocate(heap, limits, value, expected.clone())
            } else {
                Ok(value)
            }
        }
        Value::NullObjectReference(_) if module.is_object_reference_type(expected) => {
            Ok(Value::NullObjectReference(expected.clone()))
        }
        other => {
            let result = crate::reflection::materialize(module, heap, limits, other)?;
            if result.ty() != *expected {
                return Err(Fault::new(format!(
                    "source snapshot scalar mismatch: {:?} -> {expected:?}",
                    result.ty()
                )));
            }
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Module {
        let mut module = crate::assemble(".module Source\n.interface Contract\n.end\n.type class Provider\n.implements Contract\n.field private StoredIdentity String\n.end\n").unwrap();
        for (index, logical) in [
            "System.Introspection.AssemblyInfo",
            "System.Introspection.RuntimeAssemblyInfo",
        ]
        .iter()
        .enumerate()
        {
            module.types[index].origin = Some(serde_json::from_value(serde_json::json!({
                "assembly": "Selected", "module": "Selected.dll", "name": logical, "token": 0x02000001u32 + index as u32
            })).unwrap());
        }
        module
    }
    fn snapshot() -> Value {
        Value::Object {
            ty: Type::from_name("System.Introspection.AssemblyInfo"),
            fields: vec![Value::String("Selected".into())],
        }
    }
    #[test]
    fn source_arrays_materialize_scoped_provider_objects() {
        let module = fixture();
        let mut heap = ManagedHeap::default();
        let contract = Type::from_name("Contract");
        let array = Type::ArrayRef(Box::new(contract.clone()));
        let value = materialize(
            &module,
            &mut heap,
            &Limits::default(),
            Value::Array {
                element: Type::from_name("System.Introspection.AssemblyInfo"),
                elements: vec![snapshot()],
            },
            &array,
        )
        .unwrap();
        assert_eq!(value.ty(), array);
        let Value::ObjectReference(reference) = value else {
            panic!("array not allocated");
        };
        let Value::Array { elements, element } = reference.reference.read().unwrap() else {
            panic!("wrong array storage");
        };
        assert_eq!(element, contract);
        let Value::ObjectReference(item) = &elements[0] else {
            panic!("provider not allocated");
        };
        assert_eq!(item.concrete_type(), Type::from_name("Provider"));
        assert_eq!(elements[0].ty(), contract);
    }
    #[test]
    fn source_provider_rejects_foreign_scope_and_wrong_layout() {
        let mut module = fixture();
        let mut heap = ManagedHeap::default();
        let contract = Type::from_name("Contract");
        module.types[1].origin.as_mut().unwrap().assembly = "Other".into();
        assert!(
            materialize(
                &module,
                &mut heap,
                &Limits::default(),
                snapshot(),
                &contract
            )
            .is_err()
        );
        module.types[1].origin.as_mut().unwrap().assembly = "Selected".into();
        module.types[1].fields[0].ty = Type::Int32;
        assert!(
            materialize(
                &module,
                &mut heap,
                &Limits::default(),
                snapshot(),
                &contract
            )
            .is_err()
        );
        assert_eq!(heap.len(), 0);
    }
    #[test]
    fn source_recipe_cannot_escape_as_an_unrelated_type() {
        let module = fixture();
        let mut heap = ManagedHeap::default();
        let recipe = Value::Object {
            ty: Type::from_name("$ReflectionSnapshot.Option"),
            fields: vec![Value::Boolean(false)],
        };
        assert!(
            materialize(
                &module,
                &mut heap,
                &Limits::default(),
                recipe,
                &Type::from_name("Contract")
            )
            .is_err()
        );
    }
}
