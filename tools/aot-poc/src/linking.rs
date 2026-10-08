//! Bounded explicit library load sets. Verify original module scopes before any
//! canonical projection; never let relocation authorize cross-module access.
use neoclr::metadata::{Instruction as Op, MemberId, TypeDefId};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub struct RuntimeContext {
    pub system: neoclr::Module,
    pub object_root: Option<TypeDefId>,
    pub compile_system: bool,
    pub bind_user_fault: bool,
    pub bind_console_read_byte: bool,
    pub bind_console_write_line: bool,
    pub bind_console_stream_output: bool,
    pub bind_int32_to_string: bool,
    pub bind_utf8_text: bool,
    pub bind_character_text: bool,
    pub bind_integer_text: bool,
    pub bind_socket_listener: bool,
    pub bind_socket_accept: bool,
    pub reference_arena: bool,
}

pub fn prepare(
    app: &neoclr::Module,
    dependencies: &[neoclr::Module],
    root: &str,
    context: Option<&RuntimeContext>,
) -> Result<(neoclr::Module, Value), Error> {
    if dependencies.is_empty() || dependencies.len() > 8 {
        return Err("explicit load sets require one to eight dependencies".into());
    }
    if app.functions.iter().filter(|f| f.name == root).count() != 1 {
        return Err("load-set root must resolve uniquely inside the application".into());
    }
    let mut inputs: Vec<_> = std::iter::once(app).chain(dependencies).collect();
    if inputs.iter().map(|m| m.functions.len()).sum::<usize>() > 4096
        || inputs.iter().map(|m| m.types.len()).sum::<usize>() > 1024
    {
        return Err("load-set inventory exceeds AOT bounds".into());
    }
    for input in &inputs {
        super::selection::validate_source(input, true)?;
    }
    // This checks reference lists/revisions, source access, signatures, field access,
    // definite initialization and borrows in the ORIGINAL scopes, including all bodies.
    let system = if let Some(context) = context {
        super::selection::validate_source(&context.system, true)?;
        &context.system
    } else {
        neoclr::library::system().map_err(|e| e.to_string())?
    };
    let compile_system = context.is_some_and(|c| c.compile_system);
    if compile_system {
        inputs.push(system);
        if inputs.iter().map(|m| m.functions.len()).sum::<usize>() > 4096
            || inputs.iter().map(|m| m.types.len()).sum::<usize>() > 1024
        {
            return Err("load-set inventory including System exceeds AOT bounds".into());
        }
    }
    let program = if let Some(object_root) = context.and_then(|c| c.object_root.as_ref()) {
        neoclr::LoadedProgram::with_modules_and_object_root(app, system, dependencies, object_root)
    } else {
        neoclr::LoadedProgram::with_modules(app, system, dependencies)
    };
    program
        .and_then(|p| p.verify())
        .map_err(|e| e.to_string())?;

    let mut joined = app.clone();
    joined.types.clear();
    joined.functions.clear();
    joined.assemblies.clear();
    joined.references = None;
    let mut methods = vec![];
    let mut types = vec![];
    for input in &inputs {
        for (i, _) in input.functions.iter().enumerate() {
            methods.push(MemberId {
                module: input.name.clone(),
                revision: input.revision.clone(),
                index: i as u32,
            });
        }
        for (i, _) in input.types.iter().enumerate() {
            types.push(TypeDefId {
                module: input.name.clone(),
                revision: input.revision.clone(),
                index: i as u32,
            });
        }
        joined.functions.extend(input.functions.iter().cloned());
        joined.types.extend(input.types.iter().cloned());
        for assembly in &input.assemblies {
            if joined
                .assemblies
                .iter()
                .any(|a| a.full_name == assembly.full_name)
            {
                return Err("library profile requires distinct source assemblies".into());
            }
            joined.assemblies.push(assembly.clone());
        }
    }
    let method_id = |i| MemberId {
        module: app.name.clone(),
        revision: app.revision.clone(),
        index: i as u32,
    };
    let type_id = |i| TypeDefId {
        module: app.name.clone(),
        revision: app.revision.clone(),
        index: i as u32,
    };
    // LoadedProgram binds matching InternalCall declarations in the caller's
    // original module. Reproduce that narrow rule before flattening scopes;
    // otherwise a source-owned service and its seed declaration become ambiguous.
    // Original loading/verification above remains the authority for access/registry
    // checks. This does not grant general name-based calls a local preference.
    let local_services: Vec<_> = joined.functions.iter().enumerate()
        .filter(|(_, f)| f.is_internal_call())
        .map(|(i, f)| (i, f.clone())).collect();
    for (i, function) in joined.functions.iter_mut().enumerate() {
        function.definition = Some(method_id(i));
        for op in &mut function.body {
            if let Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) | Op::BindFunction { target, .. } = op {
                if target.definition.is_none() {
                    let matches: Vec<_> = local_services.iter().filter(|(index, f)| {
                        methods[*index].module == methods[i].module
                            && f.name == target.name && f.owner == target.owner
                            && f.instance == target.instance && f.parameters == target.parameters
                            && f.generic_parameters.is_empty() && target.generic_arguments.is_empty()
                    }).collect();
                    if let [candidate] = matches.as_slice() {
                        target.definition = Some(methods[candidate.0].clone());
                    }
                }
                if let Some(id) = &target.definition {
                    let i = methods
                        .iter()
                        .position(|candidate| candidate == id)
                        .ok_or("load-set call targets an unsupplied definition")?;
                    target.definition = Some(method_id(i));
                }
            }
        }
    }
    for (i, ty) in joined.types.iter_mut().enumerate() {
        ty.definition = Some(type_id(i));
        if let Some(owner) = &ty.declaring_type {
            let i = types
                .iter()
                .position(|candidate| candidate == owner)
                .ok_or("load-set type has an unsupplied lexical owner")?;
            ty.declaring_type = Some(type_id(i));
        }
    }
    // Retain the verified array-backing identity while canonicalizing the load set.
    for assembly in &mut joined.assemblies {
        if let Some(id) = &assembly.array_backing {
            let index = types.iter().position(|candidate| candidate == id)
                .ok_or("array backing targets an unsupplied definition")?;
            assembly.array_backing = Some(type_id(index));
        }
    }
    // Conformance was verified in original scopes. Metadata-only relationships
    // must not consume executable specialization shapes (Option and Result share
    // Propagatable with different arguments, including metadata-only Void).
    let source_conformance = joined.clone();
    let relationships: Vec<_> = joined
        .types
        .iter_mut()
        .map(|ty| {
            let relationships = ty.implements.clone();
            // Reference objects need their verified interface views at runtime.
            // Value-only conformance remains metadata-only in this profile.
            if !context.is_some_and(|c| c.reference_arena) || (!ty.is_reference_type && ty.name != "System.String" && ty.representation != neoclr::metadata::Representation::Interface) {
                ty.implements.clear();
            }
            relationships
        })
        .collect();
    let specialized = if joined
        .types
        .iter()
        .any(|t| !t.generic_parameters.is_empty())
        || joined
            .functions
            .iter()
            .any(|f| !f.generic_parameters.is_empty())
    {
        Some(super::specialization::expand(&joined, root)?)
    } else {
        None
    };
    let input = specialized.as_ref().map_or(&joined, |(module, _)| module);
    let (mut selected, mut report) = super::selection::select_inventory(input, root, false)?;
    // Retain original closed String conformance before private specialization
    // erases interface arguments and primitive declarations leave the inventory.
    let mut string_interfaces = vec![];
    for row in report["types"].as_array().unwrap() {
        let source = row["sourceIndex"].as_u64().unwrap() as usize;
        if input.types[source].representation != neoclr::metadata::Representation::Interface { continue; }
        let shape = specialized.as_ref().and_then(|(_, data)| data["types"].as_array())
            .and_then(|rows| rows.iter().find(|r| r["expandedIndex"] == source));
        let target = if let Some(shape) = shape {
            neoclr::metadata::Type::Constructed {
                definition: shape["name"].as_str().ok_or("invalid String interface shape")?.to_owned(),
                arguments: serde_json::from_value(shape["arguments"].clone())?,
            }
        } else { neoclr::metadata::Type::Named(input.types[source].name.clone()) };
        if super::selection::implements_interface(&source_conformance, &neoclr::metadata::Type::String, &target) {
            string_interfaces.push(row["compiledIndex"].clone());
        }
    }
    report["stringInterfaceViews"] = json!(string_interfaces);
    if let Some((_, mut specialization)) = specialized {
        for row in specialization["types"].as_array_mut().unwrap() {
            row["definition"] = json!(types[row["sourceIndex"].as_u64().unwrap() as usize]);
        }
        for row in specialization["methods"].as_array_mut().unwrap() {
            row["definition"] = json!(methods[row["sourceIndex"].as_u64().unwrap() as usize]);
        }
        super::specialization::restore_methods(&mut report, &specialization);
        report["specialization"] = specialization;
    }
    for key in ["functions", "excludedFunctions"] {
        for row in report[key].as_array_mut().unwrap() {
            row["definition"] = json!(methods[row["sourceIndex"].as_u64().unwrap() as usize]);
        }
    }
    for key in ["types", "excludedTypes"] {
        for row in report[key].as_array_mut().unwrap() {
            row["definition"] = json!(types[row["sourceIndex"].as_u64().unwrap() as usize]);
        }
    }
    if let Some(index) = report["arrayBackingProjection"]["compiledIndex"].as_u64() {
        if let Some(row) = report["types"].as_array().unwrap().iter().find(|r| r["compiledIndex"] == index).cloned() {
            report["arrayBackingProjection"]["definition"] = row["definition"].clone();
            report["arrayBackingProjection"]["sourceName"] = row["name"].clone();
        }
    }
    let mut verified_relationships = vec![];
    for row in report["types"].as_array().unwrap() {
        let index = row["sourceIndex"].as_u64().unwrap() as usize;
        if relationships[index].is_empty() {
            continue;
        }
        let arguments: Vec<neoclr::metadata::Type> = report["specialization"]["types"]
            .as_array()
            .and_then(|rows| rows.iter().find(|r| r["expandedIndex"] == row["expandedIndex"]))
            .map(|r| serde_json::from_value(r["arguments"].clone()))
            .transpose()?
            .unwrap_or_default();
        let interfaces = relationships[index]
            .iter()
            .map(|t| {
                t.substitute_type_parameters(&arguments)
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        verified_relationships.push(
            json!({"definition": types[index], "name": row["name"], "interfaces": interfaces}),
        );
    }
    report["verifiedInterfaceRelationships"] = json!(verified_relationships);
    report["interfacePolicy"] = json!(
        "original load-set conformance verified; value relationships omitted from private projection; reference-arena class interface views and constructed-class implicit dispatch retained; explicit implementations unsupported"
    );
    let bind_user_fault = context.is_some_and(|c| c.bind_user_fault);
    report["nativeBindings"] = if bind_user_fault {
        super::bindings::user_fault(&mut selected, &report)?
    } else { json!([]) };
    let bind_console_read_byte = context.is_some_and(|c| c.bind_console_read_byte);
    if bind_console_read_byte {
        let rows = super::bindings::console_read_byte(&mut selected, &report)?;
        report["nativeBindings"].as_array_mut().unwrap().extend(rows);
    }
    let bind_console_write_line = context.is_some_and(|c| c.bind_console_write_line);
    if bind_console_write_line {
        let rows = super::bindings::console_write_line(&mut selected, &report)?;
        report["nativeBindings"].as_array_mut().unwrap().extend(rows);
    }
    let bind_console_stream_output = context.is_some_and(|c| c.bind_console_stream_output);
    if bind_console_stream_output {
        let rows = super::bindings::console_stream_output(&mut selected, &report)?;
        report["nativeBindings"].as_array_mut().unwrap().extend(rows);
    }
    let bind_int32_to_string = context.is_some_and(|c| c.bind_int32_to_string);
    if bind_int32_to_string {
        let rows = super::bindings::int32_to_string(&mut selected, &report)?;
        report["nativeBindings"].as_array_mut().unwrap().extend(rows);
    }
    let bind_utf8_text = context.is_some_and(|c| c.bind_utf8_text);
    if bind_utf8_text {
        let rows = super::bindings::utf8_text(&mut selected, &report)?;
        report["nativeBindings"].as_array_mut().unwrap().extend(rows);
    }
    let bind_character_text = context.is_some_and(|c| c.bind_character_text);
    if bind_character_text {
        let rows = super::bindings::character_text(&mut selected, &report)?;
        report["nativeBindings"].as_array_mut().unwrap().extend(rows);
    }
    let bind_integer_text = context.is_some_and(|c| c.bind_integer_text);
    if bind_integer_text {
        let rows = super::bindings::integer_text(&mut selected, &report)?;
        report["nativeBindings"].as_array_mut().unwrap().extend(rows);
    }
    let bind_socket_listener = context.is_some_and(|c| c.bind_socket_listener);
    if bind_socket_listener {
        let rows = super::bindings::socket_listener(&mut selected, &report)?;
        report["nativeBindings"].as_array_mut().unwrap().extend(rows);
    }
    let bind_socket_accept = context.is_some_and(|c| c.bind_socket_accept);
    if bind_socket_accept {
        let rows = super::bindings::socket_accept(&mut selected, &report)?;
        report["nativeBindings"].as_array_mut().unwrap().extend(rows);
    }
    let reference_arena = context.is_some_and(|c| c.reference_arena);
    report["referenceArena"] = json!(reference_arena);
    if reference_arena {
        let rows = super::bindings::object_reference_equals(&mut selected, &report)?;
        report["nativeBindings"].as_array_mut().unwrap().extend(rows);
    }
    report["nativeAbi"] = if reference_arena || report["nativeBindings"].as_array().unwrap().iter()
        .any(|r| matches!(r["implementation"].as_str(), Some("int32-to-string-v1" | "int64-to-string-v1" | "uint64-to-string-v1"))) {
        json!("caller-owned-text-arena-v4")
    } else { json!("no-text-arena") };
    // Static primitive wrappers have no receiver/storage. Preserve their verified
    // source identity in the report, then use private free-function bodies: the
    // backend's bundled primitive declarations belong to a different verification module.
    let primitive_static: std::collections::BTreeSet<_> = selected.functions.iter().enumerate()
        .filter(|(_, f)| matches!(f.owner, Some(neoclr::metadata::Type::Char | neoclr::metadata::Type::String | neoclr::metadata::Type::Int32)) && !f.instance && !f.receiver_byref)
        .map(|(i, _)| i).collect();
    report["staticPrimitiveOwners"] = json!(primitive_static.iter().map(|i| json!({
        "compiledIndex": i, "owner": selected.functions[*i].owner, "lowering": "verified static wrapper to private free function"
    })).collect::<Vec<_>>());
    for (i, f) in selected.functions.iter_mut().enumerate() {
        if primitive_static.contains(&i) { f.owner = None; }
        for op in &mut f.body {
            if let Op::Call(target) = op {
                if target.definition.as_ref().is_some_and(|id| primitive_static.contains(&(id.index as usize))) {
                    target.owner = None;
                }
            }
        }
    }
    // Preserve borrowed intrinsic Int32 storage when moving verified instance
    // wrappers to private free functions. No receiver copy or formatting shortcut.
    let integer_members: std::collections::BTreeSet<_> = selected.functions.iter().enumerate()
        .filter(|(_, f)| f.owner == Some(neoclr::metadata::Type::Int32) && f.instance)
        .map(|(i, _)| i).collect();
    for &i in &integer_members {
        let f = &selected.functions[i];
        if !reference_arena || !f.receiver_byref || f.receiver_readonly || f.is_virtual || f.is_override || f.is_abstract
            || f.impl_flags != 0 || f.pinvoke.is_some() || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty() || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || f.visibility != neoclr::metadata::Visibility::Public || f.name.ends_with("..ctor") {
            return Err("Int32 instance projection requires public nonvirtual ordinary byref wrappers and --reference-arena".into());
        }
    }
    for (i, f) in selected.functions.iter_mut().enumerate() {
        if integer_members.contains(&i) {
            f.owner = None;
            f.instance = false;
            f.receiver_byref = false;
            f.parameters.insert(0, neoclr::metadata::Type::ByRef(Box::new(neoclr::metadata::Type::Int32)));
            if !f.parameter_names.is_empty() { f.parameter_names.insert(0, None); }
            for index in &mut f.out_parameters { *index += 1; }
            for index in &mut f.readonly_parameters { *index += 1; }
            for index in &mut f.out_when_true { *index += 1; }
            if let Some(origin) = &mut f.origin { origin.parameter_tokens.insert(0, 0); origin.nullable_annotations.clear(); }
        }
        for op in &mut f.body {
            if let Op::Call(target) = op {
                if target.definition.as_ref().is_some_and(|id| integer_members.contains(&(id.index as usize))) {
                    target.owner = None;
                    target.instance = false;
                    target.parameters.insert(0, neoclr::metadata::Type::ByRef(Box::new(neoclr::metadata::Type::Int32)));
                }
            }
        }
    }
    report["int32InstanceProjections"] = json!(integer_members.iter().map(|i| json!({"compiledIndex":i,
        "policy":"verified public nonvirtual Int32 wrapper; explicit borrowed receiver, unchanged CIL"})).collect::<Vec<_>>());
    // The original load set owns its String methods. The backend's bundled String
    // cannot be redeclared; lower verified public nonvirtual instance wrappers to
    // free functions with an explicit receiver, preserving the original CIL body.
    let string_members: std::collections::BTreeSet<_> = selected.functions.iter().enumerate()
        .filter(|(_, f)| f.owner == Some(neoclr::metadata::Type::String) && f.instance)
        .map(|(i, _)| i).collect();
    for &i in &string_members {
        let f = &selected.functions[i];
        if !reference_arena || f.receiver_byref || f.receiver_readonly || f.is_virtual || f.is_override
            || f.is_abstract || !f.generic_parameters.is_empty() || !f.interface_implementations.is_empty()
            || f.visibility != neoclr::metadata::Visibility::Public || f.name.ends_with("..ctor") {
            return Err("String instance projection requires public nonvirtual ordinary wrappers and --reference-arena".into());
        }
    }
    for (i, f) in selected.functions.iter_mut().enumerate() {
        if string_members.contains(&i) {
            f.owner = None;
            f.instance = false;
            f.parameters.insert(0, neoclr::metadata::Type::String);
            if !f.parameter_names.is_empty() { f.parameter_names.insert(0, None); }
            for index in &mut f.out_parameters { *index += 1; }
            for index in &mut f.readonly_parameters { *index += 1; }
            // These remain unsupported by the value profile; retain facts rather
            // than accidentally authorizing conditional-output methods.
            for contract in &mut f.out_when_true { *contract += 1; }
            if let Some(origin) = &mut f.origin { origin.parameter_tokens.insert(0, 0); origin.nullable_annotations.clear(); }
        }
        for op in &mut f.body {
            if let Op::Call(target) = op {
                if target.definition.as_ref().is_some_and(|id| string_members.contains(&(id.index as usize))) {
                    target.owner = None;
                    target.instance = false;
                    target.parameters.insert(0, neoclr::metadata::Type::String);
                }
            }
        }
    }
    report["stringInstanceProjections"] = json!(string_members.iter().map(|i| json!({"compiledIndex":i,
        "policy":"verified public nonvirtual String wrapper; explicit receiver, unchanged CIL argument indices"})).collect::<Vec<_>>());
    if reference_arena {
        if let Some(index) = selected.types.iter().position(|t| t.name == "System.Object") {
            let mut name = "$aot_ObjectBase".to_owned();
            while selected.types.iter().any(|t| t.name == name) { name.push('_'); }
            // Keep the ordinary empty base and constructor, but do not ask the
            // backend verifier to install a second runtime Object slot registry.
            // The complete original Object contract was verified above.
            selected.types[index].name = name.clone();
            rename_nominal(&mut selected, "System.Object", &name, root)?;
            report["objectBaseProjection"] = json!({"compiledIndex": index, "sourceName": "System.Object", "compiledName": name,
                "policy": "verified empty Object base/ordinary constructor; private nominal base, no virtual Object slots"});
            for row in report["types"].as_array_mut().unwrap() {
                if row["compiledIndex"] == index { row["compiledName"] = json!(name); }
            }
        }
    }
    if let Some(rows) = report["objectDisplayDispatch"].as_array() {
        if !rows.is_empty() && !reference_arena { return Err("Object display dispatch requires --reference-arena".into()); }
        for row in rows {
            let contract = row["contractCompiledIndex"].as_u64().ok_or("invalid Object display contract")? as usize;
            // The original virtual slot and overrides were verified in the full
            // load set. Keep instance calling conventions and callvirt null checks;
            // native dispatch replaces only this private placeholder body.
            let mut suffix = format!("$aot_ObjectDisplay_{contract}");
            while selected.functions.iter().any(|f| f.name.rsplit('.').next() == Some(suffix.as_str())) { suffix.push('_'); }
            let name = format!("{}.{}", selected.functions[contract].owner.as_ref().unwrap().definition_name().unwrap(), suffix);
            for f in &mut selected.functions {
                for op in &mut f.body {
                    if let Op::CallVirtual(target) = op {
                        if target.definition.as_ref().is_some_and(|id| id.index as usize == contract) { target.name = name.clone(); }
                    }
                }
            }
            let f = &mut selected.functions[contract];
            // A distinct private slot name avoids creating a nonvirtual hiding
            // relationship when the original owner is a declared Object base.
            f.name = name;
            f.is_virtual = false; f.is_override = false;
            f.body = vec![Op::String(String::new()), Op::Return];
            f.locals.clear(); f.local_names.clear();
            for target in row["targets"].as_array().ok_or("invalid Object display targets")? {
                let i = target["functionCompiledIndex"].as_u64().ok_or("invalid Object display target")? as usize;
                selected.functions[i].is_virtual = false;
                selected.functions[i].is_override = false;
            }
        }
    }
    // Value receivers have no derived allocation type. Original-scope verification
    // has checked their Object.ToString override contract; retain its borrowed body.
    let value_display: Vec<_> = selected.functions.iter().enumerate()
        .filter(|(_, f)| f.instance && f.receiver_byref && !f.receiver_readonly
            && f.is_virtual && f.is_override && !f.is_abstract && f.impl_flags == 0
            && f.name.rsplit('.').next() == Some("ToString") && f.parameters.is_empty()
            && f.returns == neoclr::metadata::Type::String && !f.no_result
            && f.generic_parameters.is_empty() && f.generic_arguments.is_empty()
            && f.generic_constraints.is_empty() && f.interface_implementations.is_empty()
            && f.owner.as_ref().and_then(|t| selected.type_definition(t)).is_some_and(|t|
                !t.is_reference_type && t.representation == neoclr::metadata::Representation::Record
                && t.base.is_none() && t.generic_parameters.is_empty()))
        .map(|(i, _)| i).collect();
    for &i in &value_display {
        selected.functions[i].is_virtual = false;
        selected.functions[i].is_override = false;
    }
    report["valueDisplayProjections"] = json!(value_display.iter().map(|i| json!({"compiledIndex":i,
        "policy":"verified value ToString override; preserve direct by-reference receiver and original CIL"})).collect::<Vec<_>>());
    let sealed_members: Vec<_> = selected.functions.iter().enumerate()
        .filter(|(_, f)| (f.is_virtual || f.is_override) && super::selection::sealed_member(&selected, f))
        .map(|(i, _)| i).collect();
    for &i in &sealed_members {
        selected.functions[i].is_virtual = false;
        selected.functions[i].is_override = false;
    }
    report["sealedMemberProjections"] = json!(sealed_members.iter().map(|i| json!({"compiledIndex":i,
        "policy":"verified sealed reference owner; retain callvirt null checks and ordinary method body"})).collect::<Vec<_>>());
    super::boxing::project(&mut selected, &mut report)?;
    let boxed_display = report["int32Boxes"].as_array().is_some_and(|r| !r.is_empty())
        && report["objectDisplayDispatch"].as_array().is_some_and(|r| !r.is_empty());
    if boxed_display && !bind_int32_to_string {
        return Err("boxed Int32 display requires --bind-int32-to-string".into());
    }
    report["boxedInt32Display"] = json!(boxed_display);
    let system_type_names: std::collections::BTreeSet<_> = neoclr::library::system()
        .map_err(|e| e.to_string())?.types.iter().map(|t| t.name.as_str()).collect();
    let mut static_projections = vec![];
    for index in 0..selected.types.len() {
        let ty = &selected.types[index];
        // The supplied compatibility seed represents Console as an empty value
        // owner rather than the source library's static class. Keep its shape.
        let seed_console = ty.name == "System.Console" && !ty.is_reference_type
            && ty.representation == neoclr::metadata::Representation::Record
            && ty.fields.is_empty() && ty.base.is_none() && ty.implements.is_empty()
            && ty.generic_parameters.is_empty() && ty.generic_constraints.is_empty()
            && ty.enum_info.is_none() && ty.packing.is_none() && ty.minimum_size.is_none();
        if !(super::selection::static_owner(ty) || seed_console) || !system_type_names.contains(ty.name.as_str()) { continue; }
        let source = selected.types[index].name.clone();
        let mut name = format!("$aot_StaticOwner_{index}");
        while selected.types.iter().any(|t| t.name == name) { name.push('_'); }
        selected.types[index].name = name.clone();
        rename_nominal(&mut selected, &source, &name, root)?;
        static_projections.push(json!({"compiledIndex":index,"sourceName":source,"compiledName":name,"policy":"verified static/empty seed Console owner; private nominal name, original shape retained"}));
        for row in report["types"].as_array_mut().unwrap() {
            if row["compiledIndex"] == index { row["compiledName"] = json!(name); }
        }
    }
    report["staticOwnerProjections"] = json!(static_projections);
    // The backend re-verifies the private module with its bundled System context.
    // Give selected seed/library members private names so originals such as
    // System.Fail cannot collide with that context. Exact definition IDs still bind calls.
    let system_names: std::collections::BTreeSet<_> = neoclr::library::system()
        .map_err(|e| e.to_string())?.functions.iter().map(|f| f.name.as_str()).collect();
    let private_names: Vec<_> = selected.functions.iter().enumerate().map(|(i, f)| {
        if f.name != root && system_names.contains(f.name.as_str()) {
            let mut name = format!("$aot_linked_{i}");
            while selected.functions.iter().any(|candidate| candidate.name == name) { name.push('_'); }
            name
        } else { f.name.clone() }
    }).collect();
    for (i, f) in selected.functions.iter_mut().enumerate() {
        f.name = private_names[i].clone();
        for op in &mut f.body {
            if let Op::Call(target) | Op::CallVirtual(target) | Op::Construct(target) | Op::BindFunction { target, .. } = op {
                if let Some(id) = &target.definition {
                    target.name = private_names[id.index as usize].clone();
                }
            }
        }
    }
    for row in report["functions"].as_array_mut().unwrap() {
        let i = row["compiledIndex"].as_u64().unwrap() as usize;
        if row["name"] != private_names[i] { row["compiledName"] = json!(private_names[i]); }
    }
    report["loadSet"] = json!({"modules": inputs.iter().map(|m| json!({"name": m.name, "revision": m.revision})).collect::<Vec<_>>(),
        "validation": "all original bodies verified with runtime binder before private canonical projection",
        "runtimeContext": {"system": system.name, "revision": system.revision, "explicit": context.is_some(), "objectRoot": context.and_then(|c| c.object_root.as_ref()), "compileSystem": compile_system, "bindUserFault": bind_user_fault, "bindConsoleReadByte": bind_console_read_byte, "bindConsoleWriteLine": bind_console_write_line, "bindConsoleStreamOutput": bind_console_stream_output, "bindInt32ToString": bind_int32_to_string, "bindUtf8Text": bind_utf8_text, "bindCharacterText": bind_character_text, "bindIntegerText": bind_integer_text, "bindSocketListener": bind_socket_listener, "bindSocketAccept": bind_socket_accept, "referenceArena": reference_arena, "scope": if compile_system { "explicit managed System body selection; native services still require bindings" } else { "validation only; System seed bodies are not compilation inputs" }},
        "limits": "up to 128 closed value/reference/interface shapes and 128 function clones; primitive static generic methods; one to eight explicit dependencies; no dynamic loading"});
    Ok((selected, report))
}

// Rename only structural nominal types and their owned method spellings. Source
// strings, origins and definition identities remain intact for diagnostics/access.
fn rename_nominal(module: &mut neoclr::Module, old: &str, new: &str, root: &str) -> Result<(), Error> {
    fn rename(value: &mut Value, old: &str, new: &str, root: &str) {
        match value {
            Value::Object(object) => {
                if object.get("owner") == Some(&json!({"Named":old})) {
                    if let Some(Value::String(member)) = object.get_mut("name") {
                        if member != root {
                            if let Some(suffix) = member.strip_prefix(&format!("{old}.")) { *member = format!("{new}.{suffix}"); }
                        }
                    }
                }
                if object.get("Named") == Some(&json!(old)) { object.insert("Named".into(), json!(new)); }
                for child in object.values_mut() { rename(child, old, new, root); }
            }
            Value::Array(values) => for child in values { rename(child, old, new, root); },
            _ => (),
        }
    }
    let mut encoded = serde_json::to_value(&*module)?;
    rename(&mut encoded, old, new, root);
    *module = serde_json::from_value(encoded)?;
    Ok(())
}
