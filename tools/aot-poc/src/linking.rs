//! Bounded explicit library load sets. Verify original module scopes before any
//! canonical projection; never let relocation authorize cross-module access.
use neoclr::metadata::{Instruction as Op, MemberId, TypeDefId};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub fn prepare(
    app: &neoclr::Module,
    dependencies: &[neoclr::Module],
    root: &str,
) -> Result<(neoclr::Module, Value), Error> {
    if dependencies.is_empty() || dependencies.len() > 8 {
        return Err("explicit load sets require one to eight dependencies".into());
    }
    if app.functions.iter().filter(|f| f.name == root).count() != 1 {
        return Err("load-set root must resolve uniquely inside the application".into());
    }
    let inputs: Vec<_> = std::iter::once(app).chain(dependencies).collect();
    if inputs.iter().map(|m| m.functions.len()).sum::<usize>() > 4096
        || inputs.iter().map(|m| m.types.len()).sum::<usize>() > 1024
    {
        return Err("load-set inventory exceeds AOT bounds".into());
    }
    for input in &inputs {
        super::selection::validate_source(input, true)?;
        if input.types.iter().any(|t| !t.generic_parameters.is_empty())
            || input
                .functions
                .iter()
                .any(|f| !f.generic_parameters.is_empty())
        {
            return Err(
                "explicit library profile currently requires nongeneric declarations".into(),
            );
        }
    }
    // This checks reference lists/revisions, source access, signatures, field access,
    // definite initialization and borrows in the ORIGINAL scopes, including all bodies.
    neoclr::LoadedProgram::with_modules(
        app,
        neoclr::library::system().map_err(|e| e.to_string())?,
        dependencies,
    )
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
    let (selected, mut report) = super::selection::select_inventory(&joined, root, false)?;
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
    report["loadSet"] = json!({"modules": inputs.iter().map(|m| json!({"name": m.name, "revision": m.revision})).collect::<Vec<_>>(),
        "validation": "all original bodies verified with runtime binder before private canonical projection",
        "limits": "nongeneric declarations; one to eight explicit dependencies; bundled System; no dynamic loading"});
    Ok((selected, report))
}
