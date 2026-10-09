//! Private lowering of verified String reference views, empty value-record, Int32 and Boolean boxing. This is not a native
//! runtime-service binding: the source instruction retains its identity and fault site.
use neoclr::metadata::{Function, FunctionRef, Instruction as Op, MemberId, Representation, Type};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub fn project(input: &mut neoclr::Module, report: &mut Value) -> Result<(), Error> {
    let sites: Vec<_> = input.functions.iter().enumerate().flat_map(|(i, f)| {
        f.body.iter().enumerate().filter_map(move |(pc, op)| {
            if let Op::BoxValue(ty) = op { Some((i, pc, ty.clone())) } else { None }
        })
    }).collect();
    if sites.is_empty() { return Ok(()); }
    if report["referenceArena"] != true { return Err("record boxing requires --reference-arena".into()); }
    let root = report["objectBaseProjection"]["compiledIndex"].as_u64()
        .ok_or("record boxing requires a verified empty Object base")? as usize;
    let object = Type::Named(input.types[root].name.clone());
    let mut int32_index = None;
    let mut int32_helper = None;
    let mut bool_index = None;
    let mut bool_helper = None;
    let mut helpers = std::collections::BTreeMap::new();
    let mut rows = vec![];
    let mut int32_rows = vec![];
    let mut bool_rows = vec![];
    let mut string_rows = vec![];
    for (caller, pc, ty) in sites {
        // String is intrinsic reference storage. Match the interpreter's Object
        // view materialization instead of treating it as a value-record box.
        if ty == Type::String {
            input.functions[caller].body[pc] = Op::CastClass(object.clone());
            string_rows.push(json!({"functionCompiledIndex":caller,"instruction":pc}));
            continue;
        }
        let primitive = matches!(ty, Type::Int32 | Type::Boolean);
        let boolean = ty == Type::Boolean;
        let index = if primitive {
            if let Some(index) = if boolean { bool_index } else { int32_index } { index } else {
                if input.types.len() >= crate::limits::TYPES { return Err("boxing tags exceed the selected type limit".into()); }
                let index = input.types.len();
                let mut name = if boolean { "$aot_BoxedBoolean" } else { "$aot_BoxedInt32" }.to_owned();
                while input.types.iter().any(|t| t.name == name) { name.push('_'); }
                input.types.push(serde_json::from_value(json!({"name":name,"fields":[]}))?);
                if boolean { bool_index = Some(index); } else { int32_index = Some(index); }
                index
            }
        } else { input.types.iter().position(|t| Type::Named(t.name.clone()) == ty)
            .ok_or_else(|| format!("record boxing requires Int32, Boolean or a local closed empty value record; found {ty:?} in {} at instruction {pc}", input.functions[caller].name))? };
        let t = &input.types[index];
        if t.is_reference_type || t.representation != Representation::Record || !t.fields.is_empty()
            || t.base.is_some() || t.enum_info.is_some() || !t.generic_parameters.is_empty() {
            return Err("record boxing currently requires empty value records".into());
        }
        let helper = if let Some(&helper) = helpers.get(&index) { helper } else {
            if input.functions.len() >= crate::limits::FUNCTIONS { return Err("boxing helpers exceed the selected function limit".into()); }
            let helper = input.functions.len();
            let mut name = format!("$aot_box_{}_{index}", if boolean { "boolean" } else if primitive { "int32" } else { "empty" });
            while input.functions.iter().any(|f| f.name == name) { name.push('_'); }
            let definition = MemberId { module:input.name.clone(), revision:input.revision.clone(), index:helper as u32 };
            // Verifiable private placeholder. Native emission allocates a distinct
            // object header plus an Int32 snapshot when present; tags stay private.
            let function: Function = serde_json::from_value(json!({"name":name,"definition":definition,
                "parameters":[ty],"returns":object,"locals":[object],
                "body":[Op::LocalAddress(0),Op::InitializeObject(object.clone()),Op::Load(0),Op::Return]}))?;
            input.functions.push(function);
            if boolean { bool_helper = Some(helper); } else if primitive { int32_helper = Some(helper); }
            helpers.insert(index, helper);
            helper
        };
        let f = &input.functions[helper];
        input.functions[caller].body[pc] = Op::Call(FunctionRef { definition:f.definition.clone(),name:f.name.clone(),
            owner:None,instance:false,generic_arguments:vec![],parameters:vec![ty] });
        let sites = if boolean { &mut bool_rows } else if primitive { &mut int32_rows } else { &mut rows };
        sites.push(json!({"functionCompiledIndex":caller,"instruction":pc,"helperCompiledIndex":helper,"typeCompiledIndex":index}));
    }
    report["stringBoxSites"] = json!(string_rows);
    report["int32BoxSites"] = json!(int32_rows);
    report["booleanBoxSites"] = json!(bool_rows);
    report["emptyRecordBoxSites"] = json!(rows);
    report["emptyRecordBoxes"] = json!(helpers.into_iter().filter(|(ty, _)| Some(*ty) != int32_index && Some(*ty) != bool_index).map(|(ty, helper)| json!({"typeCompiledIndex":ty,"helperCompiledIndex":helper})).collect::<Vec<_>>());
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
    let sites: Vec<_> = input.functions.iter().enumerate().flat_map(|(i, f)| {
        f.body.iter().enumerate().filter_map(move |(pc, op)| {
            matches!(op, Op::IsInstance(Type::Char)).then_some((i, pc))
        })
    }).collect();
    if sites.is_empty() { return Ok(()); }
    if report["referenceArena"] != true { return Err("Char box tests require --reference-arena".into()); }
    if input.functions.iter().any(|f| f.body.iter().any(|op| matches!(op, Op::BoxValue(Type::Char)))) {
        return Err("Char box tests cannot project a Char box producer".into());
    }
    let root = report["objectBaseProjection"]["compiledIndex"].as_u64()
        .ok_or("Char box tests require a verified Object base")? as usize;
    let object = Type::Named(input.types[root].name.clone());
    if input.functions.len() >= crate::limits::FUNCTIONS { return Err("Char test helper exceeds the selected function limit".into()); }
    let helper = input.functions.len();
    let mut name = "$aot_nonmatching_char_box".to_owned();
    while input.functions.iter().any(|f| f.name == name) { name.push('_'); }
    let definition = MemberId { module: input.name.clone(), revision: input.revision.clone(), index: helper as u32 };
    input.functions.push(serde_json::from_value(json!({"name":name,"definition":definition,
        "parameters":[object],"returns":object,"locals":[object],
        "body":[Op::LocalAddress(0),Op::InitializeObject(object.clone()),Op::Load(0),Op::Return]}))?);
    for &(caller, pc) in &sites {
        input.functions[caller].body[pc] = Op::Call(FunctionRef { definition:Some(definition.clone()), name:name.clone(),
            owner:None, instance:false, generic_arguments:vec![], parameters:vec![object.clone()] });
    }
    report["nonmatchingCharBoxTests"] = json!({"helperCompiledIndex":helper,
        "sites":sites.into_iter().map(|(function, instruction)| json!({"functionCompiledIndex":function,"instruction":instruction})).collect::<Vec<_>>(),
        "policy":"closed profile rejects Char box producers; preserve null Object result; no matching Char box support"});
    Ok(())
}

/// Checked scalar box queries. A missing producer is a nonmatching type, never a
/// permission to reinterpret a different box payload.
pub fn project_scalar_queries(input: &mut neoclr::Module, report: &mut Value) -> Result<(), Error> {
    let sites: Vec<_> = input.functions.iter().enumerate().flat_map(|(i, f)| {
        f.body.iter().enumerate().filter_map(move |(pc, op)| match op {
            Op::IsInstance(t @ (Type::Int32 | Type::Boolean)) => Some((i, pc, t.clone(), false)),
            Op::UnboxAny(t @ (Type::Int32 | Type::Boolean)) => Some((i, pc, t.clone(), true)),
            _ => None,
        })
    }).collect();
    // Reference unboxing is the ordinary checked reference cast in the VM.
    let reference_types: Vec<_> = input.types.iter().filter(|t| t.is_reference_type).map(|t| Type::Named(t.name.clone())).collect();
    for f in &mut input.functions {
        for op in &mut f.body {
            if let Op::UnboxAny(t) = op {
                if *t == Type::String || reference_types.contains(t) { *op = Op::CastClass(t.clone()); }
            }
        }
    }
    if sites.is_empty() { return Ok(()); }
    let root = report["objectBaseProjection"]["compiledIndex"].as_u64().ok_or("scalar box queries require a verified Object base")? as usize;
    let object = Type::Named(input.types[root].name.clone());
    let mut helpers = std::collections::BTreeMap::new();
    let mut queries = vec![];
    for (caller, pc, ty, unbox) in sites {
        let boolean = ty == Type::Boolean;
        let key = (boolean, unbox);
        let helper = if let Some(&helper) = helpers.get(&key) { helper } else {
            if input.functions.len() >= crate::limits::FUNCTIONS { return Err("box query helpers exceed the selected function limit".into()); }
            let helper = input.functions.len();
            let mut name = format!("$aot_box_query_{helper}");
            while input.functions.iter().any(|f| f.name == name) { name.push('_'); }
            let returns = if unbox { ty.clone() } else { object.clone() };
            let body = if unbox {
                vec![Op::Fault("cannot unbox null".into()), Op::Fault("unbox.any requires the exact boxed value type".into())]
            } else { vec![Op::LocalAddress(0),Op::InitializeObject(object.clone()),Op::Load(0),Op::Return] };
            input.functions.push(serde_json::from_value(json!({"name":name,
                "definition":MemberId { module:input.name.clone(), revision:input.revision.clone(), index:helper as u32 },
                "parameters":[object],"returns":returns,"locals":[object],"body":body}))?);
            let box_key = if boolean { "booleanBoxes" } else { "int32Boxes" };
            let tag = report[box_key].as_array().and_then(|v| v.first()).map(|r| r["typeCompiledIndex"].clone()).unwrap_or(Value::Null);
            queries.push(json!({"helperCompiledIndex":helper,"typeCompiledIndex":tag,"unbox":unbox,"primitive":ty}));
            helpers.insert(key, helper);
            helper
        };
        let f = &input.functions[helper];
        input.functions[caller].body[pc] = Op::Call(FunctionRef { definition:f.definition.clone(),name:f.name.clone(),owner:None,
            instance:false,generic_arguments:vec![],parameters:vec![object.clone()] });
    }
    report["scalarBoxQueries"] = json!(queries);
    Ok(())
}
