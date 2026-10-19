//! Bounded persistent byte-memory lowering. Borrowed views remain inadmissible.
use neoclr::metadata::{MemoryViewKind, Representation, Type, TypeDef};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub(crate) fn byte_memory(t: &TypeDef) -> bool {
    matches!(
        t.memory_view,
        Some(MemoryViewKind::Memory | MemoryViewKind::ReadOnlyMemory)
    ) && neoclr::native_metadata::memory_view_element(t) == Some(&Type::Byte)
        && t.representation == Representation::Record
        && !t.is_reference_type
        && !t.is_abstract
        && t.generic_parameters.is_empty()
        && t.generic_constraints.is_empty()
        && t.generic_variance.is_empty()
        && t.fields.is_empty()
        && t.static_fields.is_empty()
        && t.literal_fields.is_empty()
        && t.implements.is_empty()
        && t.base.is_none()
        && t.packing.is_none()
        && t.minimum_size.is_none()
        && t.enum_info.is_none()
        && t.declaring_type.is_none()
}

pub(crate) fn bind(input: &mut neoclr::Module, report: &Value) -> Result<Vec<Value>, Error> {
    let mut result = vec![];
    for row in report["functions"]
        .as_array()
        .ok_or("missing selection inventory")?
    {
        let name = row["name"].as_str().ok_or("missing selected name")?;
        let expected_kind = match name {
            "neoCLR.Runtime.ViewCreateMemory" => Some(MemoryViewKind::Memory),
            "neoCLR.Runtime.ViewCreateReadOnlyMemory" => Some(MemoryViewKind::ReadOnlyMemory),
            "neoCLR.Runtime.ViewLength" | "neoCLR.Runtime.ViewSlice" => None,
            _ => continue,
        };
        let slicing = name == "neoCLR.Runtime.ViewSlice";
        let index = row["compiledIndex"]
            .as_u64()
            .ok_or("missing compiled index")? as usize;
        let f = &input.functions[index];
        let view_type = if expected_kind.is_some() {
            Some(&f.returns)
        } else {
            f.parameters.first()
        };
        let valid_view = view_type
            .and_then(|t| input.type_definition(t))
            .is_some_and(|t| {
                byte_memory(t) && expected_kind.is_none_or(|k| t.memory_view == Some(k))
            });
        let compiled_name = row["compiledName"].as_str().unwrap_or(name);
        if !valid_view
            || f.name != compiled_name
            || f.owner.is_some()
            || f.instance
            || f.receiver_byref
            || f.receiver_readonly
            || f.no_result
            || f.impl_flags != 0x1000
            || f.pinvoke.is_some()
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.is_virtual
            || f.is_override
            || f.is_abstract
            || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty()
            || !f.readonly_parameters.is_empty()
            || if expected_kind.is_some() {
                f.parameters != [Type::ArrayRef(Box::new(Type::Byte))]
            } else {
                if slicing {
                    f.parameters.len() != 3
                        || f.parameters[1..] != [Type::Int32, Type::Int32]
                        || f.parameters.first() != Some(&f.returns)
                } else {
                    f.parameters.len() != 1 || f.returns != Type::Int32
                }
            }
        {
            return Err("native memory binding requires exact closed persistent Byte create/length/slice InternalCall contracts".into());
        }
        let implementation = if expected_kind.is_some() {
            "byte-memory-create-v1"
        } else if slicing {
            "byte-memory-slice-v1"
        } else {
            "byte-memory-length-v1"
        };
        input.functions[index].impl_flags = 0;
        input.functions[index].body = vec![neoclr::metadata::Instruction::Branch(0)];
        result.push(json!({"definition":row["definition"], "name":name,
            "compiledIndex":index, "implementation":implementation}));
    }
    Ok(result)
}

/// Verification-only nominal shells for already admitted opaque storage. The VM
/// verifier checks call/field identity, initialization and address lifetimes; the
/// native profile has independently rejected raw construction/boxing and supplies
/// the actual three-lane layout. Never execute or serialize this temporary copy.
pub(crate) fn project_verification(input: &mut neoclr::Module) -> Result<(), Error> {
    let indices: Vec<_> = input
        .types
        .iter()
        .enumerate()
        .filter(|(_, t)| byte_memory(t))
        .map(|(i, _)| i)
        .collect();
    for index in indices {
        let old = input.types[index].name.clone();
        let mut name = format!("$verify_ByteMemory_{index}");
        while input.types.iter().any(|t| t.name == name) {
            name.push('_');
        }
        input.types[index].name = name.clone();
        input.types[index].memory_view = None;
        crate::linking::rename_nominal(input, &old, &name, "")?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn test_module(kind: MemoryViewKind, element: Type) -> neoclr::Module {
    let spelling = if kind == MemoryViewKind::Memory {
        "memory"
    } else {
        "read_only_memory"
    };
    let source = neoclr::assemble(&format!(
        ".module System\n.type {}<T>\n.memoryview {spelling}\n.end\n",
        kind.name()
    ))
    .unwrap();
    let mut closed = source.clone();
    neoclr::native_metadata::retain_memory_view_shape(&mut closed.types[0], &source, 0, &[element])
        .unwrap();
    closed.name = "NativeByteMemory".into();
    closed.types[0].name = "$closed_memory".into();
    closed.types[0].generic_parameters.clear();
    closed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistent_byte_memory_verification_copy_keeps_initialization_checks() {
        use neoclr::metadata::Instruction as Op;
        let mut input = test_module(MemoryViewKind::Memory, Type::Byte);
        input.types[0].definition = None;
        let ty = Type::Named(input.types[0].name.clone());
        input.functions.push(serde_json::from_value(json!({
            "name":"Keep", "parameters":[ty.clone()], "returns":ty.clone(),
            "locals":[ty.clone()], "body":[Op::LocalAddress(0), Op::InitializeObject(ty.clone()),
                Op::Arg(0), Op::Return]
        })).unwrap());
        let original = input.clone();
        project_verification(&mut input).unwrap();
        assert!(original.types[0].memory_view.is_some());
        assert!(input.types[0].memory_view.is_none());
        neoclr::verify(&input).unwrap();
        input.functions[0].body = vec![Op::Load(0), Op::Return];
        assert!(neoclr::verify(&input).is_err());
        let mut untrusted: neoclr::Module =
            serde_json::from_value(serde_json::to_value(&original).unwrap()).unwrap();
        project_verification(&mut untrusted).unwrap();
        assert!(untrusted.types[0].memory_view.is_some());
        assert!(neoclr::verify(&untrusted).is_err());
    }

    #[test]
    fn persistent_byte_memory_requires_retained_exact_shape() {
        for kind in [MemoryViewKind::Memory, MemoryViewKind::ReadOnlyMemory] {
            let module = test_module(kind, Type::Byte);
            assert!(byte_memory(&module.types[0]));
            let roundtrip: neoclr::Module =
                serde_json::from_value(serde_json::to_value(&module).unwrap()).unwrap();
            assert!(!byte_memory(&roundtrip.types[0]));
            assert!(!byte_memory(&test_module(kind, Type::Int32).types[0]));
            let mut bad = module.types[0].clone();
            bad.memory_view = Some(MemoryViewKind::Slice);
            assert!(!byte_memory(&bad));
            bad = module.types[0].clone();
            bad.is_reference_type = true;
            assert!(!byte_memory(&bad));
        }
    }

    #[test]
    fn byte_memory_length_and_slice_bind_only_exact_view_signatures() {
        for kind in [MemoryViewKind::Memory, MemoryViewKind::ReadOnlyMemory] {
            for slicing in [false, true] {
                let mut input = test_module(kind, Type::Byte);
                let ty = Type::Named(input.types[0].name.clone());
                let name = if slicing {
                    "neoCLR.Runtime.ViewSlice"
                } else {
                    "neoCLR.Runtime.ViewLength"
                };
                let parameters = if slicing {
                    vec![ty.clone(), Type::Int32, Type::Int32]
                } else {
                    vec![ty.clone()]
                };
                input.functions.push(
                    serde_json::from_value(json!({
                        "name":name, "parameters":parameters,
                        "returns":if slicing { ty.clone() } else { Type::Int32 }, "impl_flags":4096
                    }))
                    .unwrap(),
                );
                let report = json!({"functions":[{"name":name, "compiledIndex":0}]});
                let original = input.clone();
                assert_eq!(bind(&mut input, &report).unwrap().len(), 1);
                let mut bad = original.clone();
                bad.functions[0].parameters.push(Type::Int32);
                assert!(bind(&mut bad, &report).is_err());
                let mut bad = original.clone();
                bad.functions[0].returns = Type::Byte;
                assert!(bind(&mut bad, &report).is_err());
                let mut bad = original;
                bad.functions[0].parameters[0] = Type::ByRef(Box::new(ty));
                assert!(bind(&mut bad, &report).is_err());
            }
        }
    }

    #[test]
    fn byte_memory_bindings_reject_mutated_internalcall_contracts() {
        let mut input = test_module(MemoryViewKind::Memory, Type::Byte);
        input.functions.push(serde_json::from_value(json!({
            "name":"neoCLR.Runtime.ViewCreateMemory", "parameters":[Type::ArrayRef(Box::new(Type::Byte))],
            "returns":Type::Named(input.types[0].name.clone()), "impl_flags":4096
        })).unwrap());
        let report =
            json!({"functions":[{"name":"neoCLR.Runtime.ViewCreateMemory", "compiledIndex":0}]});
        let original = input.clone();
        assert_eq!(
            bind(&mut input, &report).unwrap()[0]["implementation"],
            "byte-memory-create-v1"
        );
        for variant in 0..6 {
            let mut altered = original.clone();
            match variant {
                0 => altered.functions[0].parameters[0] = Type::Array(Box::new(Type::Byte)),
                1 => altered.functions[0].parameters[0] = Type::ArrayRef(Box::new(Type::Int32)),
                2 => altered.functions[0].impl_flags = 0,
                3 => altered.functions[0].no_result = true,
                4 => altered.functions[0]
                    .generic_parameters
                    .push(Some("T".into())),
                _ => altered.types[0].memory_view = Some(MemoryViewKind::ReadOnlyMemory),
            }
            assert!(bind(&mut altered, &report).is_err(), "variant {variant}");
        }
    }
}
