//! Checked vector reflection over the closed vector token inventory.
use neoclr::metadata::{FunctionRef, Instruction as Op, Type};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub fn support_root(input: &neoclr::Module) -> Result<usize, Error> {
    let providers: Vec<_> = input
        .types
        .iter()
        .filter(|t| {
            t.origin
                .as_ref()
                .map_or(t.name.as_str(), |o| o.name.as_str())
                == "System.Introspection.RuntimeTypeInfo"
        })
        .collect();
    let [provider] = providers.as_slice() else {
        return Err("native array construction requires one RuntimeTypeInfo provider".into());
    };
    let owner = Type::Named(provider.name.clone());
    let candidates: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            f.owner.as_ref() == Some(&owner)
                && f.origin
                    .as_ref()
                    .map_or_else(|| f.name.rsplit('.').next().unwrap(), |o| o.name.as_str())
                    == "get_ExecutionHandle"
        })
        .collect();
    let [(index, f)] = candidates.as_slice() else {
        return Err("native array construction requires the RuntimeTypeInfo handle getter".into());
    };
    if !provider.is_reference_type
        || !f.instance
        || !f.parameters.is_empty()
        || f.returns != Type::RuntimeTypeHandle
        || f.no_result
        || f.receiver_byref
        || f.receiver_readonly
        || f.impl_flags != 0
        || f.is_virtual
        || f.is_abstract
        || f.is_override
    {
        return Err("unsupported RuntimeTypeInfo handle getter contract".into());
    }
    Ok(*index)
}

pub fn bind(input: &mut neoclr::Module, report: &mut Value) -> Result<(), Error> {
    let services: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            matches!(
                f.name.as_str(),
                "neoCLR.Runtime.ReflectionArrayLength"
                    | "neoCLR.Runtime.ReflectionArrayGet"
                    | "neoCLR.Runtime.ReflectionArrayCreate"
            )
        })
        .map(|(i, _)| i)
        .collect();
    if services.is_empty() {
        return Ok(());
    }
    let object_index = report["objectBaseProjection"]["compiledIndex"]
        .as_u64()
        .ok_or("array reflection requires Object projection")? as usize;
    let object = Type::Named(input.types[object_index].name.clone());
    let mut arrays = vec![];
    for f in &input.functions {
        for op in &f.body {
            if let Op::LoadTypeToken(t @ Type::ArrayRef(_)) = op {
                if !arrays.contains(t) {
                    arrays.push(t.clone());
                }
            }
        }
    }
    let mut nominal_tokens: Vec<_> = input
        .functions
        .iter()
        .flat_map(|f| &f.body)
        .filter_map(|op| {
            if let Op::LoadTypeToken(t) = op {
                (!matches!(t, Type::ArrayRef(_))).then_some(t.clone())
            } else {
                None
            }
        })
        .collect();
    nominal_tokens.sort_by_key(|t| format!("{t:?}"));
    nominal_tokens.dedup();
    let all_vectors: Vec<_> = report["referenceArrayBackingProjections"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| r["compiledIndex"].as_u64())
        .chain(report["arrayBackingProjection"]["compiledIndex"].as_u64())
        .filter_map(|i| input.types[i as usize].fields.first().map(|f| f.ty.clone()))
        .collect();
    let getter = report["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["definition"] == report["reflectionArrayHandleGetterDefinition"]
                && !r["definition"].is_null()
        })
        .map(|r| r["compiledIndex"].as_u64().unwrap() as usize);
    let getter = getter.map(|i| {
        let f = &input.functions[i];
        FunctionRef {
            definition: f.definition.clone(),
            name: f.name.clone(),
            owner: f.owner.clone(),
            instance: true,
            parameters: vec![],
            generic_arguments: vec![],
        }
    });
    for index in services {
        let name = input.functions[index].name.clone();
        let create = name.ends_with("Create");
        let get = name.ends_with("Get");
        let f = &input.functions[index];
        let parameters = if create {
            vec![
                f.parameters
                    .first()
                    .cloned()
                    .ok_or("invalid array creation signature")?,
                Type::ArrayRef(Box::new(object.clone())),
            ]
        } else if get {
            vec![object.clone(), Type::Int32]
        } else {
            vec![object.clone()]
        };
        let returns = if create || get {
            object.clone()
        } else {
            Type::Int32
        };
        if f.owner.is_some()
            || f.instance
            || f.parameters != parameters
            || f.returns != returns
            || f.no_result
            || f.impl_flags != 0x1000
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.pinvoke.is_some()
            || f.receiver_byref
            || f.receiver_readonly
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
        {
            return Err(
                "array reflection requires the exact reserved InternalCall contract".into(),
            );
        }
        let mut body = vec![];
        let mut locals = vec![];
        let mut instruction_map = vec![];
        if create {
            let getter = getter
                .as_ref()
                .ok_or("array construction handle getter was not retained")?;
            let provider = getter
                .owner
                .clone()
                .ok_or("missing array TypeInfo provider")?;
            if input
                .type_definition(&provider)
                .is_none_or(|t| !t.implements.contains(&parameters[0]))
            {
                return Err(
                    "array construction requires the exact TypeInfo interface parameter".into(),
                );
            }

            // This is the same provider guard used by the interpreter adapter.
            body.extend([
                Op::Arg(0),
                Op::IsInstance(provider.clone()),
                Op::ReferenceIsNull,
                Op::BranchFalse(5),
                Op::Fault("array construction requires runtime TypeInfo".into()),
                Op::Arg(0),
                Op::CastClass(provider),
                Op::Call(getter.clone()),
                Op::Store(0),
            ]);
            locals.push(Type::RuntimeTypeHandle);
        } else {
            body.extend([
                Op::Arg(0),
                Op::ReferenceIsNull,
                Op::BranchFalse(4),
                Op::Fault("array access requires a non-null array".into()),
            ]);
        }
        for array in &arrays {
            let Type::ArrayRef(element) = array else {
                unreachable!()
            };
            let reference = matches!(**element, Type::String)
                || input.type_definition(element).is_some_and(|t| {
                    t.is_reference_type
                        || t.representation == neoclr::metadata::Representation::Interface
                });
            if (get || create) && !reference && !matches!(**element, Type::Int32 | Type::Boolean) {
                return Err("native array reflection currently requires Int32, Boolean, String or reference elements".into());
            }
            let branch = body.len() + 3;
            if create {
                body.extend([
                    Op::Load(0),
                    Op::LoadTypeToken(array.clone()),
                    Op::Equal,
                    Op::BranchFalse(0),
                ]);
                let array_local = locals.len();
                locals.push(array.clone());
                let counter = locals.len();
                locals.push(Type::Int32);
                let start = body.len();
                let ops = vec![
                    Op::Arg(1),
                    Op::ArrayLength,
                    Op::ConvertInt32,
                    Op::ReserveArray((**element).clone()),
                    Op::Store(array_local),
                    Op::Int(0),
                    Op::Store(counter),
                    Op::Load(counter),
                    Op::Arg(1),
                    Op::ArrayLength,
                    Op::ConvertInt32,
                    Op::Less,
                    Op::BranchFalse(start + 25),
                    Op::Load(array_local),
                    Op::Load(counter),
                    Op::Arg(1),
                    Op::Load(counter),
                    Op::ArrayElement(object.clone()),
                    if reference {
                        Op::CastClass((**element).clone())
                    } else {
                        Op::UnboxAny((**element).clone())
                    },
                    Op::StoreArrayElement((**element).clone()),
                    Op::Load(counter),
                    Op::Int(1),
                    Op::Add,
                    Op::Store(counter),
                    Op::Branch(start + 7),
                    Op::Load(array_local),
                    Op::CastClass(object.clone()),
                    Op::Return,
                ];
                instruction_map.extend(
                    (0..ops.len())
                        .map(|pc| json!({"instruction":start+pc,"adapterInstruction":pc})),
                );
                body.extend(ops);
            } else {
                // An exact array view, not assignability of its element, selects the adapter.
                body.extend([
                    Op::Arg(0),
                    Op::IsInstance(array.clone()),
                    Op::ReferenceIsNull,
                    Op::BranchTrue(0),
                ]);
                let start = body.len();
                let mut ops = vec![Op::Arg(0), Op::CastClass(array.clone())];
                if get {
                    ops.extend([
                        Op::Arg(1),
                        Op::ArrayElement((**element).clone()),
                        if reference {
                            Op::CastClass(object.clone())
                        } else {
                            Op::BoxValue((**element).clone())
                        },
                        Op::Return,
                    ]);
                } else {
                    ops.extend([Op::ArrayLength, Op::ConvertInt32, Op::Return]);
                }
                instruction_map.extend(
                    (0..ops.len())
                        .map(|pc| json!({"instruction":start+pc,"adapterInstruction":pc})),
                );
                body.extend(ops);
            }
            body[branch] = if create {
                Op::BranchFalse(body.len())
            } else {
                Op::BranchTrue(body.len())
            };
        }
        if create {
            for token in &nominal_tokens {
                let next = body.len() + 5;
                body.extend([
                    Op::Load(0),
                    Op::LoadTypeToken(token.clone()),
                    Op::Equal,
                    Op::BranchFalse(next),
                    Op::Fault("array service requires a vector type".into()),
                ]);
            }
            body.push(Op::Fault(
                "native reflection array type metadata was not retained".into(),
            ));
        } else {
            for array in &all_vectors {
                if arrays.contains(array) {
                    continue;
                }
                let next = body.len() + 5;
                body.extend([
                    Op::Arg(0),
                    Op::IsInstance(array.clone()),
                    Op::ReferenceIsNull,
                    Op::BranchTrue(next),
                    Op::Fault("native reflection array type metadata was not retained".into()),
                ]);
            }
            body.push(Op::Fault("array service requires a vector type".into()));
        }
        let f = &mut input.functions[index];
        f.impl_flags = 0;
        f.body = body;
        f.locals = locals;
        f.local_names.clear();
        report["nativeBindings"].as_array_mut().unwrap().push(json!({"compiledIndex":index,"name":name,
            "implementation":"reflection-array-closed-v1","vectorTypes":arrays,"instructionMap":instruction_map,
            "policy":"closed vector token inventory; checked ordinary array adapters; unregistered vectors fail explicitly"}));
    }
    Ok(())
}
