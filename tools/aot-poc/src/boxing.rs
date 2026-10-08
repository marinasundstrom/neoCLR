//! Private lowering of verified empty value-record boxing. This is not a native
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
    let mut helpers = std::collections::BTreeMap::new();
    let mut rows = vec![];
    for (caller, pc, ty) in sites {
        let index = input.types.iter().position(|t| Type::Named(t.name.clone()) == ty)
            .ok_or("record boxing requires a local closed empty value record")?;
        let t = &input.types[index];
        if t.is_reference_type || t.representation != Representation::Record || !t.fields.is_empty()
            || t.base.is_some() || t.enum_info.is_some() || !t.generic_parameters.is_empty() {
            return Err("record boxing currently requires empty value records".into());
        }
        let helper = if let Some(&helper) = helpers.get(&index) { helper } else {
            if input.functions.len() >= 512 { return Err("boxing helpers exceed the selected function limit".into()); }
            let helper = input.functions.len();
            let mut name = format!("$aot_box_empty_{index}");
            while input.functions.iter().any(|f| f.name == name) { name.push('_'); }
            let definition = MemberId { module:input.name.clone(), revision:input.revision.clone(), index:helper as u32 };
            // Verifiable private placeholder. Native emission allocates a distinct
            // eight-byte object header tagged with the original record identity.
            let function: Function = serde_json::from_value(json!({"name":name,"definition":definition,
                "parameters":[ty],"returns":object,"locals":[object],
                "body":[Op::LocalAddress(0),Op::InitializeObject(object.clone()),Op::Load(0),Op::Return]}))?;
            input.functions.push(function);
            helpers.insert(index, helper);
            helper
        };
        let f = &input.functions[helper];
        input.functions[caller].body[pc] = Op::Call(FunctionRef { definition:f.definition.clone(),name:f.name.clone(),
            owner:None,instance:false,generic_arguments:vec![],parameters:vec![ty] });
        rows.push(json!({"functionCompiledIndex":caller,"instruction":pc,"helperCompiledIndex":helper,"typeCompiledIndex":index}));
    }
    report["emptyRecordBoxSites"] = json!(rows);
    report["emptyRecordBoxes"] = json!(helpers.into_iter().map(|(ty, helper)| json!({"typeCompiledIndex":ty,"helperCompiledIndex":helper})).collect::<Vec<_>>());
    Ok(())
}
