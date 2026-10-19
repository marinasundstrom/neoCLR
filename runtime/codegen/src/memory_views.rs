//! Bounded byte-view lowering, including immutable UTF-8 string windows. Source verification enforces borrowing.
use neoclr::metadata::{MemoryViewKind, Representation, Type, TypeDef};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub(crate) fn byte_memory(t: &TypeDef) -> bool {
    matches!(
        t.memory_view,
        Some(MemoryViewKind::Memory | MemoryViewKind::ReadOnlyMemory | MemoryViewKind::Slice | MemoryViewKind::ReadOnlySlice)
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

/// Only access restriction or persistent-to-borrowed projection is admitted.
pub(crate) fn conversion(input: &neoclr::Module, source: &Type, target: &Type, borrowing: bool) -> bool {
    let (Some(source), Some(target)) = (input.type_definition(source), input.type_definition(target)) else {
        return false;
    };
    byte_memory(source) && byte_memory(target) && matches!(
        (source.memory_view, target.memory_view, borrowing),
        (Some(MemoryViewKind::Memory), Some(MemoryViewKind::ReadOnlyMemory), false)
        | (Some(MemoryViewKind::Slice), Some(MemoryViewKind::ReadOnlySlice), false)
        | (Some(MemoryViewKind::Memory), Some(MemoryViewKind::Slice), true)
        | (Some(MemoryViewKind::ReadOnlyMemory), Some(MemoryViewKind::ReadOnlySlice), true)
    )
}

pub(crate) fn bind(
    input: &mut neoclr::Module,
    report: &Value,
    reference_arena: bool,
) -> Result<Vec<Value>, Error> {
    let mut result = vec![];
    for row in report["functions"]
        .as_array()
        .ok_or("missing selection inventory")?
    {
        let name = row["name"].as_str().ok_or("missing selected name")?;
        if !reference_arena && name != "neoCLR.Runtime.ViewBoundsFault" {
            continue;
        }
        let expected_kind = match name {
            "neoCLR.Runtime.ViewUtf8" => Some(MemoryViewKind::ReadOnlySlice),
            "neoCLR.Runtime.ViewCreateSlice" => Some(MemoryViewKind::Slice),
            "neoCLR.Runtime.ViewCreateReadOnlySlice" => Some(MemoryViewKind::ReadOnlySlice),
            "neoCLR.Runtime.ViewCreateMemory" => Some(MemoryViewKind::Memory),
            "neoCLR.Runtime.ViewCreateReadOnlyMemory" => Some(MemoryViewKind::ReadOnlyMemory),
            "neoCLR.Runtime.ViewLength"
            | "neoCLR.Runtime.ViewSlice"
            | "neoCLR.Runtime.ViewAsSlice"
            | "neoCLR.Runtime.ViewGet"
            | "neoCLR.Runtime.ViewSet"
            | "neoCLR.Runtime.ViewAsReadOnly"
            | "neoCLR.Runtime.ViewBoundsFault" => None,
            _ => continue,
        };
        let utf8 = name == "neoCLR.Runtime.ViewUtf8";
        let slicing = name == "neoCLR.Runtime.ViewSlice";
        let readonly = name == "neoCLR.Runtime.ViewAsReadOnly";
        let borrowing = name == "neoCLR.Runtime.ViewAsSlice";
        let get = name == "neoCLR.Runtime.ViewGet";
        let set = name == "neoCLR.Runtime.ViewSet";
        let bounds_fault = name == "neoCLR.Runtime.ViewBoundsFault";
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
        if !valid_view && !bounds_fault
            || f.name != compiled_name
            || f.owner.is_some()
            || f.instance
            || f.receiver_byref
            || f.receiver_readonly
            || f.no_result != (bounds_fault || set)
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
            || if bounds_fault {
                !f.parameters.is_empty() || f.returns != Type::Void
            } else if utf8 {
                f.parameters != [Type::String]
            } else if expected_kind.is_some() {
                !(f.parameters == [Type::ArrayRef(Box::new(Type::Byte))]
                    || expected_kind.is_some_and(|k| k.is_borrowed())
                        && f.parameters == [Type::ArrayRef(Box::new(Type::Byte)), Type::Int32, Type::Int32])
            } else {
                if readonly || borrowing {
                    f.parameters.len() != 1 || !conversion(input, &f.parameters[0], &f.returns, borrowing)
                } else if get || set {
                    !input.type_definition(&f.parameters[0]).is_some_and(|t| {
                        t.memory_view.is_some_and(|k| k.is_borrowed())
                            && (!set || t.memory_view == Some(MemoryViewKind::Slice))
                    }) || if set {
                        f.parameters != [f.parameters[0].clone(), Type::Int32, Type::Byte]
                            || f.returns != Type::Void
                    } else {
                        f.parameters != [f.parameters[0].clone(), Type::Int32]
                            || f.returns != Type::Byte
                    }
                } else if slicing {
                    !matches!(f.parameters.len(), 2 | 3)
                        || f.parameters[1..].iter().any(|t| *t != Type::Int32)
                        || f.parameters.first() != Some(&f.returns)
                } else {
                    f.parameters.len() != 1 || f.returns != Type::Int32
                }
            }
        {
            return Err(format!("{name}: native memory binding requires exact closed Byte view InternalCall contracts").into());
        }
        let implementation = if bounds_fault {
            "memory-view-bounds-fault-v1"
        } else if utf8 {
            "byte-view-utf8-v1"
        } else if expected_kind.is_some() {
            "byte-memory-create-v1"
        } else if get {
            "byte-view-get-v1"
        } else if set {
            "byte-view-set-v1"
        } else if readonly || borrowing {
            "byte-memory-readonly-v1"
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
    let spelling = match kind {
        MemoryViewKind::Memory => "memory",
        MemoryViewKind::ReadOnlyMemory => "read_only_memory",
        MemoryViewKind::Slice => "slice",
        MemoryViewKind::ReadOnlySlice => "read_only_slice",
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
    fn utf8_view_requires_exact_string_readonly_slice_contract() {
        let mut input = test_module(MemoryViewKind::ReadOnlySlice, Type::Byte);
        let ty = Type::Named(input.types[0].name.clone());
        input.functions = vec![serde_json::from_value(json!({
            "name":"neoCLR.Runtime.ViewUtf8", "parameters":[Type::String],
            "returns":ty, "impl_flags":4096, "body":[]
        })).unwrap()];
        input.functions[0].impl_flags = 0x1000;
        let report = json!({"functions":[{"name":"neoCLR.Runtime.ViewUtf8", "compiledIndex":0}]});
        assert_eq!(bind(&mut input.clone(), &report, true).unwrap()[0]["implementation"], "byte-view-utf8-v1");
        for change in 0..5 {
            let mut bad = input.clone();
            match change {
                0 => bad.functions[0].parameters = vec![Type::ArrayRef(Box::new(Type::Byte))],
                1 => bad.types[0].memory_view = Some(MemoryViewKind::Slice),
                2 => bad.types[0].memory_view = Some(MemoryViewKind::ReadOnlyMemory),
                3 => bad.functions[0].impl_flags = 0,
                _ => bad.functions[0].body = vec![neoclr::metadata::Instruction::Branch(0)],
            }
            assert!(bind(&mut bad, &report, true).is_err());
        }
    }


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
        for kind in [MemoryViewKind::Memory, MemoryViewKind::ReadOnlyMemory, MemoryViewKind::Slice, MemoryViewKind::ReadOnlySlice] {
            let module = test_module(kind, Type::Byte);
            assert!(byte_memory(&module.types[0]));
            let roundtrip: neoclr::Module =
                serde_json::from_value(serde_json::to_value(&module).unwrap()).unwrap();
            assert!(!byte_memory(&roundtrip.types[0]));
            assert!(!byte_memory(&test_module(kind, Type::Int32).types[0]));
            let mut bad = module.types[0].clone();
            bad.is_reference_type = true;
            assert!(!byte_memory(&bad));
        }
    }

    #[test]
    fn byte_memory_length_and_slice_bind_only_exact_view_signatures() {
        for kind in [MemoryViewKind::Memory, MemoryViewKind::ReadOnlyMemory, MemoryViewKind::Slice, MemoryViewKind::ReadOnlySlice] {
            for count in [0, 1, 2] {
                let slicing = count != 0;
                let mut input = test_module(kind, Type::Byte);
                let ty = Type::Named(input.types[0].name.clone());
                let name = if slicing {
                    "neoCLR.Runtime.ViewSlice"
                } else {
                    "neoCLR.Runtime.ViewLength"
                };
                let parameters = if slicing {
                    let mut parameters = vec![ty.clone()];
                    parameters.extend(vec![Type::Int32; count]);
                    parameters
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
                assert_eq!(bind(&mut input, &report, true).unwrap().len(), 1);
                let mut bad = original.clone();
                bad.functions[0].parameters.push(Type::Byte);
                assert!(bind(&mut bad, &report, true).is_err());
                let mut bad = original.clone();
                bad.functions[0].parameters =
                    vec![ty.clone(), Type::Int32, Type::Int32, Type::Int32];
                assert!(bind(&mut bad, &report, true).is_err());
                let mut bad = original.clone();
                bad.functions[0].returns = Type::Byte;
                assert!(bind(&mut bad, &report, true).is_err());
                let mut bad = original;
                bad.functions[0].parameters[0] = Type::ByRef(Box::new(ty));
                assert!(bind(&mut bad, &report, true).is_err());
            }
        }
    }

    #[test]
    fn byte_view_conversions_follow_capabilities_not_layout() {
        use MemoryViewKind::*;
        for source in [Slice, ReadOnlySlice, Memory, ReadOnlyMemory] {
            for target in [Slice, ReadOnlySlice, Memory, ReadOnlyMemory] {
                let mut input = test_module(source, Type::Byte);
                let mut dest = test_module(target, Type::Byte).types.remove(0);
                dest.name = "$target".into();
                input.types.push(dest);
                for borrowing in [false, true] {
                    let expected = matches!((source, target, borrowing),
                        (Slice, ReadOnlySlice, false) | (Memory, ReadOnlyMemory, false)
                        | (Memory, Slice, true) | (ReadOnlyMemory, ReadOnlySlice, true));
                    assert_eq!(conversion(&input, &Type::Named(input.types[0].name.clone()),
                        &Type::Named("$target".into()), borrowing), expected);
                }
            }
        }
    }

    #[test]
    fn byte_view_access_requires_borrowed_and_writable_setter() {
        use MemoryViewKind::*;
        for kind in [Slice, ReadOnlySlice, Memory, ReadOnlyMemory] {
            for set in [false, true] {
                let mut input = test_module(kind, Type::Byte);
                let name = if set { "neoCLR.Runtime.ViewSet" } else { "neoCLR.Runtime.ViewGet" };
                let mut parameters = vec![Type::Named(input.types[0].name.clone()), Type::Int32];
                if set { parameters.push(Type::Byte); }
                input.functions.push(serde_json::from_value(json!({
                    "name":name, "parameters":parameters, "returns":if set {Type::Void} else {Type::Byte}, "no_result":set, "impl_flags":4096
                })).unwrap());
                let report = json!({"functions":[{"name":name, "compiledIndex":0}]});
                assert_eq!(bind(&mut input, &report, true).is_ok(), kind.is_borrowed() && (!set || kind == Slice));
            }
        }
    }

    #[test]
    fn view_bounds_fault_binds_only_exact_void_contract() {
        let mut input = neoclr::assemble(".module Test\n").unwrap();
        let name = "neoCLR.Runtime.ViewBoundsFault";
        input.functions.push(serde_json::from_value(json!({
            "name":name, "parameters":[], "returns":Type::Void, "no_result":true, "impl_flags":4096
        })).unwrap());
        let report = json!({"functions":[{"name":name,"compiledIndex":0}]});
        let original = input.clone();
        assert_eq!(
            bind(&mut input, &report, true).unwrap()[0]["implementation"],
            "memory-view-bounds-fault-v1"
        );
        let mut without_arena = original.clone();
        assert_eq!(bind(&mut without_arena, &report, false).unwrap().len(), 1);
        for variant in 0..4 {
            let mut bad = original.clone();
            match variant {
                0 => bad.functions[0].no_result = false,
                1 => bad.functions[0].parameters.push(Type::Int32),
                2 => bad.functions[0].returns = Type::Int32,
                _ => bad.functions[0].impl_flags = 0,
            }
            assert!(bind(&mut bad, &report, true).is_err());
        }
    }

    #[test]
    fn byte_memory_readonly_binding_only_restricts_exact_persistent_byte_views() {
        let mut input = test_module(MemoryViewKind::Memory, Type::Byte);
        let mut readonly = test_module(MemoryViewKind::ReadOnlyMemory, Type::Byte)
            .types
            .remove(0);
        readonly.name = "$closed_readonly_memory".into();
        input.types.push(readonly);
        let source = Type::Named(input.types[0].name.clone());
        let target = Type::Named(input.types[1].name.clone());
        let name = "neoCLR.Runtime.ViewAsReadOnly";
        input.functions.push(serde_json::from_value(json!({
            "name":name, "parameters":[source.clone()], "returns":target.clone(), "impl_flags":4096
        })).unwrap());
        let report = json!({"functions":[{"name":name, "compiledIndex":0}]});
        let original = input.clone();
        assert_eq!(
            bind(&mut input, &report, true).unwrap()[0]["implementation"],
            "byte-memory-readonly-v1"
        );
        for variant in 0..9 {
            let mut bad = original.clone();
            match variant {
                0 => {
                    bad.functions[0].parameters[0] = target.clone();
                    bad.functions[0].returns = source.clone();
                }
                1 => bad.functions[0].returns = source.clone(),
                2 => bad.functions[0].parameters[0] = Type::ByRef(Box::new(source.clone())),
                3 => bad.functions[0].parameters.push(Type::Int32),
                4 => bad.types[0].memory_view = Some(MemoryViewKind::Slice),
                5 => bad.types[1].memory_view = Some(MemoryViewKind::ReadOnlySlice),
                6 => {
                    let mut wrong = test_module(MemoryViewKind::ReadOnlyMemory, Type::Int32)
                        .types
                        .remove(0);
                    wrong.name = bad.types[1].name.clone();
                    bad.types[1] = wrong;
                }
                7 => bad.functions[0].impl_flags = 0,
                _ => bad.functions[0].parameters.clear(),
            }
            assert!(bind(&mut bad, &report, true).is_err(), "variant {variant}");
        }
        let mut untrusted: neoclr::Module =
            serde_json::from_value(serde_json::to_value(original).unwrap()).unwrap();
        assert!(bind(&mut untrusted, &report, true).is_err());
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
            bind(&mut input, &report, true).unwrap()[0]["implementation"],
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
            assert!(
                bind(&mut altered, &report, true).is_err(),
                "variant {variant}"
            );
        }
    }
}
