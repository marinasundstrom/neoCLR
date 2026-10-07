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
    for (i, function) in joined.functions.iter_mut().enumerate() {
        function.definition = Some(method_id(i));
        for op in &mut function.body {
            if let Op::Call(target) | Op::Construct(target) = op {
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
    // Conformance was verified in original scopes. Metadata-only relationships
    // must not consume executable specialization shapes (Option and Result share
    // Propagatable with different arguments, including metadata-only Void).
    let relationships: Vec<_> = joined
        .types
        .iter_mut()
        .map(|ty| std::mem::take(&mut ty.implements))
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
    let mut verified_relationships = vec![];
    for row in report["types"].as_array().unwrap() {
        let index = row["sourceIndex"].as_u64().unwrap() as usize;
        if relationships[index].is_empty() {
            continue;
        }
        let arguments: Vec<neoclr::metadata::Type> = report["specialization"]["types"]
            .as_array()
            .and_then(|rows| rows.iter().find(|r| r["sourceIndex"] == row["sourceIndex"]))
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
        "original load-set conformance verified; relationships omitted only from private direct-call projection; interface operations, storage and explicit implementations unsupported"
    );
    let bind_user_fault = context.is_some_and(|c| c.bind_user_fault);
    report["nativeBindings"] = if bind_user_fault {
        super::bindings::user_fault(&mut selected, &report)?
    } else { json!([]) };
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
            if let Op::Call(target) | Op::Construct(target) = op {
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
        "runtimeContext": {"system": system.name, "revision": system.revision, "explicit": context.is_some(), "objectRoot": context.and_then(|c| c.object_root.as_ref()), "compileSystem": compile_system, "bindUserFault": bind_user_fault, "scope": if compile_system { "explicit managed System body selection; native services still require bindings" } else { "validation only; System seed bodies are not compilation inputs" }},
        "limits": "one closed instantiation per local value definition; primitive static generic methods; one to eight explicit dependencies; no dynamic loading"});
    Ok((selected, report))
}
