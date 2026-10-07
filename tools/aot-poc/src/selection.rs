//! Explicit closed-world direct-call selection for reference-free applications.
//! This is bounded code selection, not a general reflection-aware trimmer.
use neoclr::metadata::{FunctionRef, Instruction as Op, MemberId, Type, TypeDefId};
use serde_json::{Value, json};
use std::collections::BTreeSet;
type Error = Box<dyn std::error::Error>;

/// Metadata-only owner of static methods; never an executable reference value.
pub(crate) fn static_owner(t: &neoclr::metadata::TypeDef) -> bool {
    t.is_reference_type
        && t.is_abstract
        && t.is_sealed
        && t.representation == neoclr::metadata::Representation::Record
        && t.fields.is_empty()
        && t.base.is_none()
        && t.enum_info.is_none()
        && t.generic_parameters.is_empty()
        && t.generic_constraints.is_empty()
        && t.packing.is_none()
        && t.minimum_size.is_none()
}

/// Shared preparation for native emission and read-only admission inspection.
pub fn prepare(input: &neoclr::Module, root: &str) -> Result<(neoclr::Module, Value), Error> {
    validate_source(input, true)?;
    if input.types.iter().any(|t| !t.generic_parameters.is_empty())
        || input
            .functions
            .iter()
            .any(|f| !f.generic_parameters.is_empty())
    {
        let (expanded, specialization) = super::specialization::expand(input, root)?;
        if specialization["methods"].as_array().unwrap().iter().any(|m| !m["arguments"].as_array().unwrap().is_empty())
            || specialization["types"].as_array().unwrap().iter().any(|t| t["expandedIndex"].as_u64().unwrap() as usize >= input.types.len()) {
            // Cloning assigns private origin tokens. Verify originals first so this
            // cannot repair invalid source metadata or bypass generic contracts.
            neoclr::LoadedProgram::new(input)
                .and_then(|p| p.verify())
                .map_err(|e| e.to_string())?;
        }
        let (selected, mut report) = select(&expanded, root)?;
        super::specialization::restore_methods(&mut report, &specialization);
        report["specialization"] = specialization;
        Ok((selected, report))
    } else {
        select(input, root)
    }
}

fn resolve(input: &neoclr::Module, target: &FunctionRef) -> Result<usize, Error> {
    if !target.generic_arguments.is_empty() {
        return Err("closed-world selection does not support generic calls".into());
    }
    let mut candidates = input.functions.iter().enumerate().filter(|(_, f)| {
        f.name == target.name
            && f.owner == target.owner
            && f.instance == target.instance
            && f.parameters == target.parameters
            && target
                .definition
                .as_ref()
                .is_none_or(|id| Some(id) == f.definition.as_ref())
    });
    let (index, _) = candidates.next().ok_or_else(|| {
        format!("closed-world call is external or has an invalid identity/signature: {target:?}")
    })?;
    if candidates.next().is_some() {
        return Err("ambiguous closed-world call".into());
    }
    Ok(index)
}

pub(super) fn interface_contract(input: &neoclr::Module, f: &neoclr::metadata::Function) -> bool {
    f.owner.as_ref().and_then(|t| input.type_definition(t))
        .is_some_and(|t| t.representation == neoclr::metadata::Representation::Interface)
}

/// Exact implicit implementations for classes constructed by the reachable program.
/// Original conformance is verified before private projection; this is not a binder.
pub(super) fn dispatch_targets(input: &neoclr::Module, contract: usize, reached: &BTreeSet<usize>) -> Result<Vec<(usize, usize)>, Error> {
    let f = &input.functions[contract];
    if !interface_contract(input, f) { return Ok(vec![]); }
    if !f.instance || f.receiver_byref || !f.body.is_empty() || !f.generic_parameters.is_empty() {
        return Err("interface dispatch requires a bodyless nongeneric instance contract".into());
    }
    let mut constructed = BTreeSet::new();
    for &i in reached {
        for op in &input.functions[i].body {
            if let Op::Construct(target) = op {
                if let Some(Type::Named(name)) = &target.owner { constructed.insert(name.as_str()); }
            }
        }
    }
    let member_name = |m: &neoclr::metadata::Function| m.origin.as_ref().map(|o| o.name.clone())
        .unwrap_or_else(|| m.name.rsplit('.').next().unwrap().to_owned());
    let name = member_name(f);
    let mut targets = vec![];
    for (ti, t) in input.types.iter().enumerate() {
        if !constructed.contains(t.name.as_str()) || !t.implements.iter().any(|v| Some(v) == f.owner.as_ref()) { continue; }
        if !t.is_reference_type || !t.generic_parameters.is_empty() || t.representation != neoclr::metadata::Representation::Record {
            return Err("interface dispatch requires nongeneric constructed classes".into());
        }
        let candidates: Vec<_> = input.functions.iter().enumerate().filter(|(_, m)|
            m.owner == Some(Type::Named(t.name.clone())) && member_name(m) == name
            && m.instance == f.instance && m.parameters == f.parameters && m.returns == f.returns
            && m.no_result == f.no_result && m.out_parameters == f.out_parameters
            && m.visibility == neoclr::metadata::Visibility::Public && m.interface_implementations.is_empty()
        ).map(|(i, _)| i).collect();
        let [target] = candidates.as_slice() else { return Err(format!("interface implementation is missing or ambiguous: {} on {}", f.name, t.name).into()); };
        targets.push((ti, *target));
    }
    Ok(targets)
}

pub(super) fn validate_source(input: &neoclr::Module, single_assembly: bool) -> Result<(), Error> {
    if input.functions.len() > 4096 || input.types.len() > 1024 {
        return Err("closed-world input inventory exceeds bounds".into());
    }
    if single_assembly && input.assemblies.len() > 1 {
        return Err("closed-world selection requires one source assembly".into());
    }
    for origin in input
        .functions
        .iter()
        .filter_map(|f| f.origin.as_ref())
        .chain(input.types.iter().filter_map(|t| t.origin.as_ref()))
    {
        if !input
            .assemblies
            .iter()
            .any(|a| a.full_name == origin.assembly && a.modules.contains(&origin.module))
        {
            return Err("closed-world origin must belong to the source assembly".into());
        }
    }
    // Never let projection repair a malformed supplied identity.
    for (i, f) in input.functions.iter().enumerate() {
        let expected = MemberId {
            module: input.name.clone(),
            revision: input.revision.clone(),
            index: i as u32,
        };
        if f.definition.as_ref().is_some_and(|id| *id != expected) {
            return Err("noncanonical source function identity".into());
        }
    }
    for (i, t) in input.types.iter().enumerate() {
        let expected = TypeDefId {
            module: input.name.clone(),
            revision: input.revision.clone(),
            index: i as u32,
        };
        if t.definition.as_ref().is_some_and(|id| *id != expected) {
            return Err("noncanonical source type identity".into());
        }
    }
    Ok(())
}

pub fn select(input: &neoclr::Module, root: &str) -> Result<(neoclr::Module, Value), Error> {
    select_inventory(input, root, true)
}

pub(super) fn select_inventory(
    input: &neoclr::Module,
    root: &str,
    single_assembly: bool,
) -> Result<(neoclr::Module, Value), Error> {
    validate_source(input, single_assembly)?;
    let roots: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter(|(_, f)| f.name == root)
        .map(|(i, _)| i)
        .collect();
    let [root_index] = roots.as_slice() else {
        return Err("closed-world root must have one exact name match".into());
    };
    let mut functions = BTreeSet::new();
    let mut pending = vec![*root_index];
    loop {
        while let Some(i) = pending.pop() {
            if !functions.insert(i) {
                continue;
            }
            if functions.len() > 128 {
                return Err("selected functions exceed the value profile limit".into());
            }
            for op in &input.functions[i].body {
                match op {
                    Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) => {
                        let callee = resolve(input, target)?;
                        let f = &input.functions[callee];
                        if matches!(op, Op::CallVirtual(_)) && !interface_contract(input, f) && (f.is_virtual || f.is_abstract || f.is_override) {
                            return Err("virtual calls requiring dispatch need a later selection profile".into());
                        }
                        pending.push(callee);
                    }
                    _ => (), // Unsupported selected opcodes still fail ordinary AOT admission.
                }
            }
        }
        for &contract in &functions {
            for (_, target) in dispatch_targets(input, contract, &functions)? {
                if !functions.contains(&target) { pending.push(target); }
            }
        }
        if pending.is_empty() { break; }
    }
    let mut types = BTreeSet::new();
    let mut pending_types = vec![];
    for &i in &functions {
        let f = &input.functions[i];
        pending_types.extend(f.owner.iter().cloned());
        pending_types.extend(f.parameters.iter().chain(&f.locals).cloned());
        pending_types.push(f.returns.clone());
        for op in &f.body {
            if let Op::New(t)
            | Op::InitializeObject(t)
            | Op::LoadObject(t)
            | Op::StoreObject(t)
            | Op::PackValue(t)
            | Op::IsValue(t)
            | Op::UnpackValue(t) | Op::IsInstance(t) | Op::CastClass(t)
            | Op::NewArray(t) | Op::ArrayElement(t) | Op::StoreArrayElement(t) | Op::ArrayAddress(t) = op
            {
                pending_types.push(t.clone());
            }
        }
    }
    while let Some(ty) = pending_types.pop() {
        match ty {
            Type::Named(name) => {
                let candidates: Vec<_> = input
                    .types
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.name == name)
                    .map(|(i, _)| i)
                    .collect();
                let [i] = candidates.as_slice() else {
                    return Err(
                        "closed-world type must resolve uniquely inside the input module".into(),
                    );
                };
                if !types.insert(*i) {
                    continue;
                }
                if types.len() > 64 {
                    return Err("selected types exceed the value profile limit".into());
                }
                let t = &input.types[*i];
                pending_types.extend(t.fields.iter().map(|f| f.ty.clone()));
                pending_types.extend(t.base.iter().chain(&t.implements).cloned());
                if let Some(owner) = &t.declaring_type {
                    if owner.module != input.name || owner.revision != input.revision {
                        return Err("external lexical owner is unsupported".into());
                    }
                    pending_types.push(Type::Named(
                        input
                            .types
                            .get(owner.index as usize)
                            .ok_or("invalid lexical owner")?
                            .name
                            .clone(),
                    ));
                }
            }
            Type::ByRef(t) => pending_types.push(*t),
            Type::ArrayRef(t) if *t == Type::Byte => (),
            Type::Int32 | Type::Byte | Type::SByte | Type::Int16 | Type::UInt16 | Type::Boolean | Type::Void | Type::Value | Type::String | Type::Char | Type::UInt32 | Type::Int64 | Type::UInt64 | Type::IntPtr | Type::UIntPtr => (),
            _ => {
                return Err(
                    "closed-world selection requires reference-free nongeneric value signatures"
                        .into(),
                );
            }
        }
    }
    let rows: Vec<_> = functions.iter().copied().collect();
    let type_rows: Vec<_> = types.iter().copied().collect();
    let mut projected = input.clone();
    projected.functions = rows.iter().map(|i| input.functions[*i].clone()).collect();
    projected.types = type_rows.iter().map(|i| input.types[*i].clone()).collect();
    // Compile a private, canonical verification projection. Semantic names, visibility,
    // signatures, storage and instruction bodies remain; non-executable descriptions
    // stay in the original artifact. Report the original-to-projection row map.
    for assembly in &mut projected.assemblies {
        assembly.references.clear();
        assembly.value_type_references.clear();
        assembly.native_module_bindings.clear();
        assembly.native_type_bindings.clear();
        assembly.array_backing = None;
    }
    projected.references = None;
    projected.entry = if input.functions[*root_index].parameters.is_empty() {
        root.to_owned()
    } else {
        String::new()
    };
    for (i, f) in projected.functions.iter_mut().enumerate() {
        f.custom_attributes.clear();
        f.definition = Some(MemberId {
            module: input.name.clone(),
            revision: input.revision.clone(),
            index: i as u32,
        });
        for op in &mut f.body {
            if let Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) = op {
                let old = resolve(input, target)?;
                let new = rows.binary_search(&old).expect("selected call");
                target.definition = Some(MemberId {
                    module: input.name.clone(),
                    revision: input.revision.clone(),
                    index: new as u32,
                });
            }
        }
    }
    for (i, t) in projected.types.iter_mut().enumerate() {
        if let Some(origin) = &mut t.origin {
            origin.property_tokens.clear();
        }
        t.custom_attributes.clear();
        t.properties.clear(); // Accessor bodies are selected through actual calls.
        t.definition = Some(TypeDefId {
            module: input.name.clone(),
            revision: input.revision.clone(),
            index: i as u32,
        });
        if let Some(owner) = &mut t.declaring_type {
            owner.index = type_rows
                .binary_search(&(owner.index as usize))
                .expect("selected lexical owner") as u32;
        }
    }
    let mut dispatch = vec![];
    for (compiled, source) in rows.iter().enumerate() {
        if interface_contract(input, &input.functions[*source]) {
            let targets = dispatch_targets(input, *source, &functions)?;
            dispatch.push(json!({"contractCompiledIndex":compiled,"contractSourceIndex":source,
                "targets":targets.iter().map(|(ty, method)| json!({"typeCompiledIndex":type_rows.binary_search(ty).unwrap(),"functionCompiledIndex":rows.binary_search(method).unwrap()})).collect::<Vec<_>>() }));
        }
    }
    let report = json!({"schema":"neoclr-aot-selection-v1", "module":input.name, "root":root,
        "policy":"explicit closed world with constructed-class implicit interface dispatch; every opcode of selected bodies retained; no reflection, dynamic loading or class virtual dispatch",
        "interfaceDispatch":dispatch,
        "metadataPolicy":"original artifact unchanged; private verification projection omits attributes/property descriptors, relocates definition rows; source origins retain access and readonly facts; external assembly bindings omitted",
        "functions": rows.iter().enumerate().map(|(new, old)| json!({"sourceIndex":old,"compiledIndex":new,"definition":input.functions[*old].definition,"name":input.functions[*old].name})).collect::<Vec<_>>(),
        "types": type_rows.iter().enumerate().map(|(new, old)| json!({"sourceIndex":old,"compiledIndex":new,"definition":input.types[*old].definition,"name":input.types[*old].name})).collect::<Vec<_>>(),
        "excludedFunctions": input.functions.iter().enumerate().filter(|(i,_)| !functions.contains(i)).map(|(i,f)| json!({"sourceIndex":i,"definition":f.definition,"name":f.name})).collect::<Vec<_>>(),
        "excludedTypes": input.types.iter().enumerate().filter(|(i,_)| !types.contains(i)).map(|(i,t)| json!({"sourceIndex":i,"definition":t.definition,"name":t.name})).collect::<Vec<_>>()});
    Ok((projected, report))
}
