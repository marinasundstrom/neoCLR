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
    if input.functions.iter().any(object_display_contract) {
        neoclr::LoadedProgram::new(input).and_then(|p| p.verify()).map_err(|e| e.to_string())?;
    }
    if input.types.iter().any(|t| !t.generic_parameters.is_empty())
        || input
            .functions
            .iter()
            .any(|f| !f.generic_parameters.is_empty())
    {
        let (expanded, specialization) = super::specialization::expand(input, root)?;
        if input.types.iter().any(|t| (t.is_reference_type || t.representation == neoclr::metadata::Representation::Interface) && !t.generic_parameters.is_empty())
            || specialization["methods"].as_array().unwrap().iter().any(|m| !m["arguments"].as_array().unwrap().is_empty())
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

pub(super) fn callable_invoke(target: &FunctionRef) -> Option<&neoclr::metadata::FunctionType> {
    match target.owner.as_ref() {
        Some(Type::Function(shape)) if target.name == "$Function.Invoke" && target.instance => Some(shape),
        _ => None,
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

/// Sealing is descriptive metadata today: also check the complete loaded type set.
/// Keep callvirt for its null check; only the private native projection loses slots.
pub(super) fn sealed_member(input: &neoclr::Module, f: &neoclr::metadata::Function) -> bool {
    f.instance && !f.is_abstract && !f.receiver_byref && f.impl_flags == 0
        && f.owner.as_ref().and_then(|t| input.type_definition(t))
            .is_some_and(|t| t.is_reference_type && t.is_sealed
                && t.representation != neoclr::metadata::Representation::Interface
                && !input.types.iter().any(|candidate| candidate.base.as_ref()
                    .and_then(Type::definition_name) == Some(t.name.as_str())))
}

/// Narrow class-virtual slice used by Console.WriteLine(Object). Original load-set
/// verification supplies slot ancestry; a same-named ordinary member is never enough.
pub(super) fn object_display_contract(f: &neoclr::metadata::Function) -> bool {
    f.name == "System.Object.ToString" && f.owner == Some(Type::Named("System.Object".into()))
        && f.instance && f.is_virtual && !f.is_override && !f.is_abstract
        && !f.receiver_byref && !f.receiver_readonly && f.parameters.is_empty()
        && f.returns == Type::String && !f.no_result && f.impl_flags == 0
        && f.generic_parameters.is_empty() && f.generic_constraints.is_empty()
}

pub(super) fn display_override(input: &neoclr::Module, concrete: &Type, contract: &neoclr::metadata::Function) -> Result<(usize, FunctionRef), Error> {
    let definition = input.type_definition(concrete).ok_or("Object display requires a local class")?;
    if !definition.is_reference_type || definition.base.as_ref().is_some_and(|base| *base != Type::Named("System.Object".into())) {
        return Err(format!("Object display requires a rootless or direct Object-derived class with a ToString override; default display and deeper inheritance are unsupported: {concrete:?}, base {:?}", definition.base).into());
    }
    // Object slots use native member names, unlike CLI-origin interface member
    // matching. An origin display name must never redirect a virtual slot.
    let arguments = match concrete { Type::Constructed { arguments, .. } => arguments.as_slice(), _ => &[] };
    let mut candidates = vec![];
    for (index, method) in input.functions.iter().enumerate() {
        if method.owner.as_ref().and_then(Type::definition_name) != concrete.definition_name()
            || method.name.rsplit('.').next() != contract.name.rsplit('.').next() { continue; }
        let method = closed_signature(method, arguments)?;
        if method.owner.as_ref() == Some(concrete) && method.instance && !method.receiver_byref
            && method.parameters == contract.parameters && method.returns == contract.returns
            && method.no_result == contract.no_result && method.generic_parameters.is_empty() {
            candidates.push((index, FunctionRef { definition:method.definition.clone(), name:method.name.clone(),
                owner:method.owner.clone(), instance:true, generic_arguments:vec![], parameters:method.parameters.clone() }));
        }
    }
    let [candidate] = candidates.as_slice() else {
        return Err("Object display requires an explicit ToString override; default display requires metadata support".into());
    };
    let (index, reference) = candidate.clone();
    if !input.functions[index].is_override || input.functions[index].is_abstract {
        return Err("Object display requires a verified concrete override".into());
    }
    Ok((index, reference))
}

/// Traverse verified interface inheritance with closed owner arguments. Class base
/// inheritance remains outside this profile; Object contributes no interfaces.
pub(super) fn implements_interface(input: &neoclr::Module, concrete: &Type, target: &Type) -> bool {
    let mut pending = vec![concrete.clone()];
    let mut seen = vec![];
    while let Some(current) = pending.pop() {
        if &current == target { return true; }
        if seen.contains(&current) { continue; }
        if seen.len() >= 1024 { return false; }
        seen.push(current.clone());
        let Some(definition) = input.type_definition(&current) else { continue; };
        let arguments = match &current { Type::Constructed { arguments, .. } => arguments.as_slice(), _ => &[] };
        for interface in &definition.implements {
            if let Ok(closed) = interface.substitute_type_parameters(arguments) { pending.push(closed); }
        }
    }
    false
}

pub(super) fn closed_signature(function: &neoclr::metadata::Function, arguments: &[Type]) -> Result<neoclr::metadata::Function, Error> {
    let mut result = function.clone();
    result.owner = function.owner.as_ref().map(|ty| ty.substitute_type_parameters(arguments)).transpose().map_err(|e| e.to_string())?;
    result.parameters = function.parameters.iter().map(|ty| ty.substitute_type_parameters(arguments)).collect::<Result<_, _>>().map_err(|e| e.to_string())?;
    result.returns = function.returns.substitute_type_parameters(arguments).map_err(|e| e.to_string())?;
    Ok(result)
}

pub(super) fn implicit_implementation(input: &neoclr::Module, concrete: &Type, contract: &neoclr::metadata::Function) -> Result<(usize, FunctionRef), Error> {
    let arguments = match concrete { Type::Constructed { arguments, .. } => arguments.as_slice(), _ => &[] };
    let member_name = |m: &neoclr::metadata::Function| m.origin.as_ref().map(|o| o.name.clone())
        .unwrap_or_else(|| m.name.rsplit('.').next().unwrap().to_owned());
    let name = member_name(contract);
    let mut candidates = vec![];
    for (index, method) in input.functions.iter().enumerate() {
        if method.owner.as_ref().and_then(Type::definition_name) != concrete.definition_name()
            || member_name(method) != name { continue; }
        let method = closed_signature(method, arguments)?;
        if method.owner.as_ref() == Some(concrete) && method.instance == contract.instance
            && method.parameters == contract.parameters && method.returns == contract.returns
            && method.no_result == contract.no_result && method.out_parameters == contract.out_parameters
            && method.visibility == neoclr::metadata::Visibility::Public && method.interface_implementations.is_empty()
            && method.generic_parameters.is_empty() {
            candidates.push((index, FunctionRef { definition: method.definition.clone(), name: method.name.clone(), owner: method.owner.clone(), instance: method.instance, generic_arguments: vec![], parameters: method.parameters.clone() }));
        }
    }
    let [target] = candidates.as_slice() else { return Err(format!("interface implementation is missing or ambiguous: {} on {concrete:?}", contract.name).into()); };
    Ok(target.clone())
}

/// The load set's verified nominal backing, closed over this profile's byte element.
/// During specialization the selected first shape retains the backing definition row.
pub(super) fn byte_array_owner(input: &neoclr::Module) -> Option<Type> {
    let id = input.assemblies.iter().find_map(|a| a.array_backing.as_ref())?;
    let t = input.types.iter().find(|t| t.definition.as_ref() == Some(id))?;
    match t.generic_parameters.len() {
        1 => Some(Type::Constructed { definition:t.name.clone(),arguments:vec![Type::Byte] }),
        0 => Some(Type::Named(t.name.clone())),
        _ => None,
    }
}

/// Intrinsic String dispatch uses verified conformance, not a nominal object tag.
fn string_dispatch_target(input: &neoclr::Module, contract: usize) -> Result<Option<usize>, Error> {
    let f = &input.functions[contract];
    if !interface_contract(input, f)
        || !implements_interface(input, &Type::String, f.owner.as_ref().unwrap()) { return Ok(None); }
    Ok(Some(implicit_implementation(input, &Type::String, f)?.0))
}

/// Exact implicit implementations for classes constructed by the reachable program.
/// Original conformance is verified before private projection; this is not a binder.
pub(super) fn dispatch_targets(input: &neoclr::Module, contract: usize, reached: &BTreeSet<usize>) -> Result<Vec<(usize, usize)>, Error> {
    dispatch_targets_with_array(input, contract, reached, None)
}

pub(super) fn dispatch_targets_with_array(input: &neoclr::Module, contract: usize, reached: &BTreeSet<usize>, array_backing: Option<usize>) -> Result<Vec<(usize, usize)>, Error> {
    let f = &input.functions[contract];
    let display = object_display_contract(f);
    if !interface_contract(input, f) && !display { return Ok(vec![]); }
    if !f.instance || f.receiver_byref || (!display && !f.body.is_empty()) || !f.generic_parameters.is_empty() {
        return Err("interface dispatch requires a bodyless nongeneric instance contract".into());
    }
    let mut constructed = BTreeSet::new();
    let array_owner = array_backing.map(|i| Type::Named(input.types[i].name.clone())).or_else(|| byte_array_owner(input));
    for &i in reached {
        for op in &input.functions[i].body {
            if display && (matches!(op, Op::BoxValue(t) if *t != Type::Int32) || matches!(op, Op::NewArray(_) | Op::ReserveArray(_))) {
                return Err("Object display with boxing or arrays requires a later receiver/metadata profile".into());
            }
            if matches!(op, Op::NewArray(Type::Byte) | Op::ReserveArray(Type::Byte)) {
                if let Some(Type::Named(name)) = &array_owner { constructed.insert(name.as_str()); }
            }
            if let Op::Construct(target) = op {
                if let Some(Type::Named(name)) = &target.owner { constructed.insert(name.as_str()); }
            }
        }
    }
    let mut targets = vec![];
    for (ti, t) in input.types.iter().enumerate() {
        if !constructed.contains(t.name.as_str()) || (!display && !implements_interface(input, &Type::Named(t.name.clone()), f.owner.as_ref().unwrap())) { continue; }
        if !t.is_reference_type || !t.generic_parameters.is_empty() || t.representation != neoclr::metadata::Representation::Record {
            return Err("interface dispatch requires nongeneric constructed classes".into());
        }
        let (target, _) = if display { display_override(input, &Type::Named(t.name.clone()), f)? }
            else { implicit_implementation(input, &Type::Named(t.name.clone()), f)? };
        targets.push((ti, target));
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
            if functions.len() > crate::limits::FUNCTIONS {
                return Err("selected functions exceed the value profile limit".into());
            }
            if object_display_contract(&input.functions[i]) { continue; }
            for op in &input.functions[i].body {
                match op {
                    Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) | Op::BindFunction { target, .. } => {
                        if callable_invoke(target).is_some() { continue; }
                        let callee = resolve(input, target)?;
                        let f = &input.functions[callee];
                        if matches!(op, Op::CallVirtual(_)) && !interface_contract(input, f) && !object_display_contract(f) && !sealed_member(input, f) && (f.is_virtual || f.is_abstract || f.is_override) {
                            return Err("virtual calls requiring dispatch need a later selection profile".into());
                        }
                        if matches!(op, Op::Call(_)) && object_display_contract(f) {
                            return Err("direct Object.ToString calls require default display metadata support".into());
                        }
                        pending.push(callee);
                    }
                    _ => (), // Unsupported selected opcodes still fail ordinary AOT admission.
                }
            }
        }
        for &contract in &functions {
            if let Some(target) = string_dispatch_target(input, contract)? {
                if !functions.contains(&target) { pending.push(target); }
            }
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
            if matches!(op, Op::BoxValue(_)) { pending_types.push(Type::Named("System.Object".into())); }
            if let Op::BindFunction { function_type, .. } = op { pending_types.push(function_type.clone()); }
            if let Op::New(t)
            | Op::InitializeObject(t)
            | Op::LoadObject(t)
            | Op::StoreObject(t)
            | Op::PackValue(t)
            | Op::IsValue(t)
            | Op::UnpackValue(t) | Op::BoxValue(t) | Op::IsInstance(t) | Op::CastClass(t)
            | Op::NewArray(t) | Op::ReserveArray(t) | Op::ArrayElement(t) | Op::StoreArrayElement(t) | Op::ArrayAddress(t) = op
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
                if types.len() > crate::limits::TYPES {
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
            Type::Function(shape) => { pending_types.extend(shape.parameters); pending_types.push(shape.returns); }
            Type::ByRef(t) => pending_types.push(*t),
            Type::Array(t) if *t == Type::Byte => (),
            Type::ArrayRef(t) if *t == Type::String => (),
            Type::ArrayRef(t) if matches!(*t, Type::Function(_) | Type::Named(_)) => pending_types.push(*t),
            Type::ArrayRef(t) if *t == Type::Byte => {
                if let Some(owner) = byte_array_owner(input) { pending_types.push(owner); }
            },
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
        if object_display_contract(f) {
            f.body = vec![Op::String(String::new()), Op::Return];
            f.locals.clear(); f.local_names.clear();
        }
        f.custom_attributes.clear();
        f.definition = Some(MemberId {
            module: input.name.clone(),
            revision: input.revision.clone(),
            index: i as u32,
        });
        for op in &mut f.body {
            if let Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) | Op::BindFunction { target, .. } = op {
                if callable_invoke(target).is_some() { continue; }
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
    let mut object_dispatch = vec![];
    let mut string_dispatch = vec![];
    for (compiled, source) in rows.iter().enumerate() {
        if let Some(target) = string_dispatch_target(input, *source)? {
            string_dispatch.push(json!({"contractCompiledIndex":compiled,"functionCompiledIndex":rows.binary_search(&target).unwrap()}));
        }
        let display = object_display_contract(&input.functions[*source]);
        if interface_contract(input, &input.functions[*source]) || display {
            let targets = dispatch_targets(input, *source, &functions)?;
            let inventory = if display { &mut object_dispatch } else { &mut dispatch };
            inventory.push(json!({"contractCompiledIndex":compiled,"contractSourceIndex":source,
                "targets":targets.iter().map(|(ty, method)| json!({"typeCompiledIndex":type_rows.binary_search(ty).unwrap(),"functionCompiledIndex":rows.binary_search(method).unwrap()})).collect::<Vec<_>>() }));
        }
    }
    let array_backing = byte_array_owner(input).and_then(|owner| input.types.iter().position(|t| owner == Type::Named(t.name.clone())))
        .and_then(|source| type_rows.binary_search(&source).ok().map(|compiled| json!({"sourceIndex":source,"compiledIndex":compiled,
            "policy":"verified nominal byte-array backing; identity-preserving views and intrinsic storage field"})));
    let report = json!({"schema":"neoclr-aot-selection-v1", "module":input.name, "root":root,
        "policy":"explicit closed world with constructed-class implicit interface dispatch; ordinary selected bodies retained; verified Object.ToString override dispatch replaces its private slot body; no reflection, dynamic loading or general class virtual dispatch",
        "stringInterfaceDispatch":string_dispatch, "interfaceDispatch":dispatch, "objectDisplayDispatch":object_dispatch, "arrayBackingProjection":array_backing,
        "metadataPolicy":"original artifact unchanged; private verification projection omits attributes/property descriptors, relocates definition rows; source origins retain access and readonly facts; external assembly bindings omitted",
        "functions": rows.iter().enumerate().map(|(new, old)| json!({"sourceIndex":old,"compiledIndex":new,"definition":input.functions[*old].definition,"name":input.functions[*old].name})).collect::<Vec<_>>(),
        "types": type_rows.iter().enumerate().map(|(new, old)| json!({"sourceIndex":old,"compiledIndex":new,"definition":input.types[*old].definition,"name":input.types[*old].name})).collect::<Vec<_>>(),
        "excludedFunctions": input.functions.iter().enumerate().filter(|(i,_)| !functions.contains(i)).map(|(i,f)| json!({"sourceIndex":i,"definition":f.definition,"name":f.name})).collect::<Vec<_>>(),
        "excludedTypes": input.types.iter().enumerate().filter(|(i,_)| !types.contains(i)).map(|(i,t)| json!({"sourceIndex":i,"definition":t.definition,"name":t.name})).collect::<Vec<_>>()});
    Ok((projected, report))
}
