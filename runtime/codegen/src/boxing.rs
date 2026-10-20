//! Private lowering of verified String reference views, closed value-record, Int32, Int64 and Boolean boxing. This is not a native
//! runtime-service binding: the source instruction retains its identity and fault site.
use neoclr::metadata::{Function, FunctionRef, Instruction as Op, MemberId, Representation, Type};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

fn scalar_box_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::SByte
            | Type::Byte
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64
            | Type::IntPtr
            | Type::UIntPtr
            | Type::Single
            | Type::Double
            | Type::Boolean
            | Type::Char
    )
}

fn persistent_memory(input: &neoclr::Module, ty: &Type) -> bool {
    input
        .type_definition(ty)
        .is_some_and(|t| t.memory_view.is_some_and(|kind| !kind.is_borrowed()))
}

pub fn project(input: &mut neoclr::Module, report: &mut Value) -> Result<(), Error> {
    let module: &neoclr::Module = input;
    let sites: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .flat_map(|(i, f)| {
            f.body.iter().enumerate().filter_map(move |(pc, op)| {
                if let Op::BoxValue(ty) = op {
                    if crate::scalar_unions::box_pair(module, f, pc) {
                        return None;
                    }
                    Some((i, pc, ty.clone()))
                } else {
                    None
                }
            })
        })
        .collect();
    if sites.iter().any(|(_, _, ty)| {
        input
            .type_definition(ty)
            .is_some_and(|t| t.memory_view.is_some_and(|kind| kind.is_borrowed()))
    }) {
        return Err("borrowed memory views cannot be boxed".into());
    }
    if sites.is_empty() {
        return Ok(());
    }
    if sites.iter().any(|(_, _, ty)| matches!(ty, Type::Union(_))) {
        return Err("native whole-union boxing requires structural boxed identity".into());
    }
    if report["referenceArena"] != true {
        return Err("record boxing requires --reference-arena".into());
    }
    let root = report["objectBaseProjection"]["compiledIndex"]
        .as_u64()
        .ok_or("record boxing requires a verified empty Object base")? as usize;
    let object = Type::Named(input.types[root].name.clone());
    let mut int64_index = None;
    let mut int64_helper = None;
    let mut int32_index = None;
    let mut int32_helper = None;
    let mut bool_index = None;
    let mut bool_helper = None;
    let mut extra_scalars = std::collections::BTreeMap::<String, (usize, Type)>::new();
    let mut helpers = std::collections::BTreeMap::new();
    let mut rows = vec![];
    let mut int64_rows = vec![];
    let mut int32_rows = vec![];
    let mut bool_rows = vec![];
    let mut string_rows = vec![];
    let mut reference_rows = vec![];
    for (caller, pc, ty) in sites {
        // String is intrinsic reference storage. Match the interpreter's Object
        // view materialization instead of treating it as a value-record box.
        if input.is_object_reference_type(&ty) {
            input.functions[caller].body[pc] = Op::CastClass(object.clone());
            if ty == Type::String {
                string_rows.push(json!({"functionCompiledIndex":caller,"instruction":pc}));
            } else {
                reference_rows.push(json!({"functionCompiledIndex":caller,"instruction":pc}));
            }
            continue;
        }
        let primitive = matches!(ty, Type::Int32 | Type::Int64 | Type::Boolean);
        let extra_scalar = scalar_box_type(&ty) && !primitive;
        let wide = ty == Type::Int64;
        let boolean = ty == Type::Boolean;
        let index = if primitive {
            if let Some(index) = if boolean {
                bool_index
            } else if wide {
                int64_index
            } else {
                int32_index
            } {
                index
            } else {
                if input.types.len() >= crate::limits::TYPES {
                    return Err("boxing tags exceed the selected type limit".into());
                }
                let index = input.types.len();
                let mut name = if boolean {
                    "$aot_BoxedBoolean"
                } else if wide {
                    "$aot_BoxedInt64"
                } else {
                    "$aot_BoxedInt32"
                }
                .to_owned();
                while input.types.iter().any(|t| t.name == name) {
                    name.push('_');
                }
                input
                    .types
                    .push(serde_json::from_value(json!({"name":name,"fields":[]}))?);
                if boolean {
                    bool_index = Some(index);
                } else if wide {
                    int64_index = Some(index);
                } else {
                    int32_index = Some(index);
                }
                index
            }
        } else if extra_scalar {
            let key = format!("{ty:?}");
            if let Some((index, _)) = extra_scalars.get(&key) {
                *index
            } else {
                if input.types.len() >= crate::limits::TYPES {
                    return Err("boxing tags exceed the selected type limit".into());
                }
                let index = input.types.len();
                let mut name = format!("$aot_Boxed_{key}");
                while input.types.iter().any(|t| t.name == name) {
                    name.push('_');
                }
                input.types.push(serde_json::from_value(json!({"name":name,
                    "fields":[{"name":"payload", "ty":ty, "visibility":"private"}]}))?);
                extra_scalars.insert(key, (index, ty.clone()));
                index
            }
        } else {
            input.types.iter().position(|t| Type::Named(t.name.clone()) == ty)
            .ok_or_else(|| format!("boxing requires a supported primitive or a local closed value record; found {ty:?} in {} at instruction {pc}", input.functions[caller].name))?
        };
        if primitive || extra_scalar {
            // A primitive box has the primitive's nominal ValueType ancestry,
            // while its private tag and scalar payload retain exact identity.
            if let Some(base) = input
                .types
                .iter()
                .find(|t| {
                    t.origin
                        .as_ref()
                        .map(|o| o.name.as_str())
                        .unwrap_or(&t.name)
                        == "System.ValueType"
                })
                .map(|t| Type::Named(t.name.clone()))
            {
                input.types[index].base = Some(base);
                if !neoclr::native_metadata::has_value_type_base(
                    input,
                    &Type::Named(input.types[index].name.clone()),
                ) {
                    return Err("primitive box requires the verified ValueType foundation".into());
                }
            }
        }
        let t = &input.types[index];
        if t.is_reference_type
            || t.representation != Representation::Record
            || (t.base.is_some()
                && !neoclr::native_metadata::has_value_type_base(
                    input,
                    &Type::Named(t.name.clone()),
                ))
            || !t.generic_parameters.is_empty()
        {
            return Err(format!("record boxing requires closed value records and enums; found {ty:?} in {} at instruction {pc}", input.functions[caller].name).into());
        }
        let helper = if let Some(&helper) = helpers.get(&index) {
            helper
        } else {
            if input.functions.len() >= crate::limits::FUNCTIONS {
                return Err("boxing helpers exceed the selected function limit".into());
            }
            let helper = input.functions.len();
            let mut name = format!(
                "$aot_box_{}_{index}",
                if boolean {
                    "boolean"
                } else if wide {
                    "int64"
                } else if primitive {
                    "int32"
                } else {
                    "empty"
                }
            );
            while input.functions.iter().any(|f| f.name == name) {
                name.push('_');
            }
            let definition = MemberId {
                module: input.name.clone(),
                revision: input.revision.clone(),
                index: helper as u32,
            };
            // Verifiable private placeholder. Native emission allocates a distinct
            // object header plus a scalar snapshot when present; tags stay private.
            let function: Function = serde_json::from_value(
                json!({"name":name,"definition":definition,
                "parameters":[ty],"returns":object,"locals":[object],
                "body":[Op::LocalAddress(0),Op::InitializeObject(object.clone()),Op::Load(0),Op::Return]}),
            )?;
            input.functions.push(function);
            if boolean {
                bool_helper = Some(helper);
            } else if wide {
                int64_helper = Some(helper);
            } else if primitive {
                int32_helper = Some(helper);
            }
            helpers.insert(index, helper);
            helper
        };
        let f = &input.functions[helper];
        input.functions[caller].body[pc] = Op::Call(FunctionRef {
            definition: f.definition.clone(),
            name: f.name.clone(),
            owner: None,
            instance: false,
            generic_arguments: vec![],
            parameters: vec![ty],
        });
        let sites = if boolean {
            &mut bool_rows
        } else if wide {
            &mut int64_rows
        } else if primitive {
            &mut int32_rows
        } else {
            &mut rows
        };
        sites.push(json!({"functionCompiledIndex":caller,"instruction":pc,"helperCompiledIndex":helper,"typeCompiledIndex":index}));
    }
    if extra_scalars
        .values()
        .any(|(_, ty)| matches!(ty, Type::Single | Type::Double))
        && report["objectDisplayDispatch"]
            .as_array()
            .is_some_and(|rows| rows.iter().any(|row| row["kind"] == "ToString"))
    {
        return Err("boxed floating-point ToString is not yet supported".into());
    }
    let char_targets = report["charObjectDispatch"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if let Some((index, _)) = extra_scalars.values().find(|(_, ty)| *ty == Type::Char) {
        for row in report["objectDisplayDispatch"]
            .as_array_mut()
            .into_iter()
            .flatten()
        {
            let target = char_targets
                .iter()
                .find(|t| t["contractCompiledIndex"] == row["contractCompiledIndex"])
                .ok_or("boxed Char Object slot has no selected source override")?;
            row["targets"].as_array_mut().ok_or("invalid Object dispatch inventory")?.push(json!({
                "typeCompiledIndex":index,"functionCompiledIndex":target["functionCompiledIndex"],"primitiveReceiver":"Char"
            }));
        }
    }
    report["stringBoxSites"] = json!(string_rows);
    report["referenceBoxSites"] = json!(reference_rows);
    report["int64BoxSites"] = json!(int64_rows);
    report["int32BoxSites"] = json!(int32_rows);
    report["booleanBoxSites"] = json!(bool_rows);
    report["emptyRecordBoxSites"] = json!(rows);
    report["emptyRecordBoxes"] = json!(
        helpers
            .into_iter()
            .filter(|(ty, _)| Some(*ty) != int32_index
                && Some(*ty) != int64_index
                && Some(*ty) != bool_index)
            .map(|(ty, helper)| {
                let primitive = extra_scalars
                    .values()
                    .find(|(index, _)| *index == ty)
                    .map(|(_, ty)| ty);
                json!({"typeCompiledIndex":ty,"helperCompiledIndex":helper,"primitive":primitive})
            })
            .collect::<Vec<_>>()
    );
    if let (Some(index), Some(helper)) = (int64_index, int64_helper) {
        report["int64Boxes"] = json!([{"typeCompiledIndex":index,"helperCompiledIndex":helper}]);
    }
    if let (Some(index), Some(helper)) = (int32_index, int32_helper) {
        report["int32Boxes"] = json!([{"typeCompiledIndex":index,"helperCompiledIndex":helper}]);
    }
    if let (Some(index), Some(helper)) = (bool_index, bool_helper) {
        report["booleanBoxes"] = json!([{"typeCompiledIndex":index,"helperCompiledIndex":helper}]);
    }
    Ok(())
}

/// After boxing admission, no Char box can exist in this closed profile. Preserve
/// the null Object result of isinst without making the backend's bundled Object
/// identity replace the selected source root. Do not call before project(): Char
/// box producers must reject, rather than turning a possible match into null.
pub fn project_char_tests(input: &mut neoclr::Module, report: &mut Value) -> Result<(), Error> {
    let sites: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .flat_map(|(i, f)| {
            f.body.iter().enumerate().filter_map(move |(pc, op)| {
                matches!(op, Op::IsInstance(Type::Char)).then_some((i, pc))
            })
        })
        .collect();
    if sites.is_empty() {
        return Ok(());
    }
    if report["referenceArena"] != true {
        return Err("Char box tests require --reference-arena".into());
    }
    if input.functions.iter().any(|f| {
        f.body
            .iter()
            .any(|op| matches!(op, Op::BoxValue(Type::Char)))
    }) {
        return Err("Char box tests cannot project a Char box producer".into());
    }
    let root = report["objectBaseProjection"]["compiledIndex"]
        .as_u64()
        .ok_or("Char box tests require a verified Object base")? as usize;
    let object = Type::Named(input.types[root].name.clone());
    if input.functions.len() >= crate::limits::FUNCTIONS {
        return Err("Char test helper exceeds the selected function limit".into());
    }
    let helper = input.functions.len();
    let mut name = "$aot_nonmatching_char_box".to_owned();
    while input.functions.iter().any(|f| f.name == name) {
        name.push('_');
    }
    let definition = MemberId {
        module: input.name.clone(),
        revision: input.revision.clone(),
        index: helper as u32,
    };
    input.functions.push(serde_json::from_value(
        json!({"name":name,"definition":definition,
        "parameters":[object],"returns":object,"locals":[object],
        "body":[Op::LocalAddress(0),Op::InitializeObject(object.clone()),Op::Load(0),Op::Return]}),
    )?);
    for &(caller, pc) in &sites {
        input.functions[caller].body[pc] = Op::Call(FunctionRef {
            definition: Some(definition.clone()),
            name: name.clone(),
            owner: None,
            instance: false,
            generic_arguments: vec![],
            parameters: vec![object.clone()],
        });
    }
    report["nonmatchingCharBoxTests"] = json!({"helperCompiledIndex":helper,
        "sites":sites.into_iter().map(|(function, instruction)| json!({"functionCompiledIndex":function,"instruction":instruction})).collect::<Vec<_>>(),
        "policy":"closed profile rejects Char box producers; preserve null Object result; no matching Char box support"});
    Ok(())
}

/// Checked scalar box queries. A missing producer is a nonmatching type, never a
/// permission to reinterpret a different box payload.
pub fn project_scalar_queries(input: &mut neoclr::Module, report: &mut Value) -> Result<(), Error> {
    let module: &neoclr::Module = input;
    let sites: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .flat_map(|(i, f)| {
            f.body
                .iter()
                .enumerate()
                .filter_map(move |(pc, op)| match op {
                    Op::IsInstance(t)
                        if (scalar_box_type(t)
                            || persistent_memory(module, t)
                            || crate::scalar_unions::record(module, t))
                            && !crate::scalar_unions::query(module, f, pc) =>
                    {
                        Some((i, pc, t.clone(), false))
                    }
                    Op::UnboxAny(t)
                        if (scalar_box_type(t)
                            || persistent_memory(module, t)
                            || crate::scalar_unions::record(module, t))
                            && !crate::scalar_unions::query(module, f, pc) =>
                    {
                        Some((i, pc, t.clone(), true))
                    }
                    _ => None,
                })
        })
        .collect();
    // Reference unboxing is the ordinary checked reference cast in the VM.
    let reference_types: Vec<_> = input
        .types
        .iter()
        .filter(|t| t.is_reference_type || t.representation == Representation::Interface)
        .map(|t| Type::Named(t.name.clone()))
        .collect();
    for f in &mut input.functions {
        for op in &mut f.body {
            if let Op::UnboxAny(t) = op {
                if *t == Type::String
                    || matches!(t, Type::ArrayRef(_))
                    || reference_types.contains(t)
                {
                    *op = Op::CastClass(t.clone());
                }
            }
        }
    }
    if sites.is_empty() {
        return Ok(());
    }
    let root = report["objectBaseProjection"]["compiledIndex"]
        .as_u64()
        .ok_or("scalar box queries require a verified Object base")? as usize;
    let object = Type::Named(input.types[root].name.clone());
    let mut helpers = std::collections::BTreeMap::new();
    let mut queries = vec![];
    for (caller, pc, ty, unbox) in sites {
        let boolean = ty == Type::Boolean;
        let wide = ty == Type::Int64;
        let key = (format!("{ty:?}"), unbox);
        let helper = if let Some(&helper) = helpers.get(&key) {
            helper
        } else {
            if input.functions.len() >= crate::limits::FUNCTIONS {
                return Err("box query helpers exceed the selected function limit".into());
            }
            let helper = input.functions.len();
            let mut name = format!("$aot_box_query_{helper}");
            while input.functions.iter().any(|f| f.name == name) {
                name.push('_');
            }
            let returns = if unbox { ty.clone() } else { object.clone() };
            let body = if unbox {
                vec![
                    Op::Fault("cannot unbox null".into()),
                    Op::Fault("unbox.any requires the exact boxed value type".into()),
                ]
            } else {
                vec![
                    Op::LocalAddress(0),
                    Op::InitializeObject(object.clone()),
                    Op::Load(0),
                    Op::Return,
                ]
            };
            input.functions.push(serde_json::from_value(json!({"name":name,
                "definition":MemberId { module:input.name.clone(), revision:input.revision.clone(), index:helper as u32 },
                "parameters":[object],"returns":returns,"locals":[object],"body":body}))?);
            let box_key = if boolean {
                "booleanBoxes"
            } else if wide {
                "int64Boxes"
            } else {
                "int32Boxes"
            };
            let record = input
                .types
                .iter()
                .position(|t| Type::Named(t.name.clone()) == ty);
            let tag = if scalar_box_type(&ty)
                && !matches!(ty, Type::Int32 | Type::Int64 | Type::Boolean)
            {
                report["emptyRecordBoxes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|r| r["primitive"] == json!(ty))
                    .map(|r| r["typeCompiledIndex"].clone())
                    .unwrap_or(Value::Null)
            } else if let Some(record) = record {
                report["emptyRecordBoxes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|r| r["typeCompiledIndex"].as_u64() == Some(record as u64))
                    .map(|r| r["typeCompiledIndex"].clone())
                    .unwrap_or(Value::Null)
            } else {
                report[box_key]
                    .as_array()
                    .and_then(|v| v.first())
                    .map(|r| r["typeCompiledIndex"].clone())
                    .unwrap_or(Value::Null)
            };
            queries.push(json!({"helperCompiledIndex":helper,"typeCompiledIndex":tag,"unbox":unbox,"primitive":ty}));
            helpers.insert(key, helper);
            helper
        };
        let f = &input.functions[helper];
        input.functions[caller].body[pc] = Op::Call(FunctionRef {
            definition: f.definition.clone(),
            name: f.name.clone(),
            owner: None,
            instance: false,
            generic_arguments: vec![],
            parameters: vec![object.clone()],
        });
    }
    report["scalarBoxQueries"] = json!(queries);
    Ok(())
}

#[cfg(test)]
mod structural_union_tests {
    use super::*;

    #[test]
    fn whole_union_box_is_not_projected_to_a_payload_reference_cast() {
        let mut input =
            neoclr::assemble(".module UnionBox\n.function Main() -> Int32\nldc.i4 42\nret\n.end")
                .unwrap();
        let op = Op::BoxValue(Type::Union(vec![
            Type::Named("Left".into()),
            Type::Named("Right".into()),
        ]));
        input.functions[0].body.insert(0, op.clone());
        let error = project(&mut input, &mut json!({"referenceArena":true})).unwrap_err();
        assert!(error.to_string().contains("structural boxed identity"));
        assert!(
            matches!(&input.functions[0].body[0], Op::BoxValue(Type::Union(parts)) if parts.len() == 2)
        );
    }
}
