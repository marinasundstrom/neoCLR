//! Bind retained collection shapes to the VM's ordinary generic library plans.
use neoclr::metadata::{FunctionRef, Instruction as Op, Type, TypeDefId};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

pub fn bind(
    input: &mut neoclr::Module,
    source: &neoclr::Module,
    definitions: &[TypeDefId],
    retention: &mut Value,
) -> Result<(), Error> {
    let names = ["Kind", "Count", "Get", "Key", "Create"];
    let services: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            names
                .iter()
                .position(|n| f.name == format!("neoCLR.Runtime.ReflectionCollection{n}"))
                .map(|op| (i, op as u8))
        })
        .collect();
    if services.is_empty() {
        return Ok(());
    }
    fn concrete(t: &Type) -> bool {
        match t {
            Type::TypeParameter(_) | Type::MethodTypeParameter(_) => false,
            Type::Constructed { arguments, .. } => arguments.iter().all(concrete),
            Type::ArrayRef(t) => concrete(t),
            _ => true,
        }
    }
    fn constructor_metadata(t: &Type) -> bool {
        match t {
            // Callable type argument descriptors are not admitted by the native
            // metadata profile. Do not infer roots from unrelated queue storage.
            Type::Function(_) => false,
            Type::Constructed { arguments, .. } => arguments.iter().all(constructor_metadata),
            Type::ArrayRef(element) => constructor_metadata(element),
            _ => true,
        }
    }
    fn inventory(t: &Type, result: &mut Vec<Type>) {
        if result.contains(t) {
            return;
        }
        match t {
            Type::Constructed { arguments, .. } => {
                if concrete(t) {
                    result.push(t.clone());
                }
                for t in arguments {
                    inventory(t, result);
                }
            }
            Type::ArrayRef(t) => inventory(t, result),
            _ => {}
        }
    }
    let mut candidates = vec![];
    for op in input.functions.iter().flat_map(|f| &f.body) {
        match op {
            Op::LoadTypeToken(t) => inventory(t, &mut candidates),
            Op::Construct(t) => {
                if let Some(owner) = &t.owner {
                    if constructor_metadata(owner) {
                        inventory(owner, &mut candidates);
                    }
                }
                for t in &t.generic_arguments {
                    inventory(t, &mut candidates);
                }
            }
            Op::Call(t)
            | Op::CallVirtual(t)
            | Op::BindFunction { target: t, .. } => {
                for t in &t.generic_arguments {
                    inventory(t, &mut candidates);
                }
            }
            _ => {}
        }
    }
    for row in retention["types"].as_array().unwrap() {
        if row["properties"] != true {
            continue;
        }
        let id: TypeDefId = serde_json::from_value(row["definition"].clone())?;
        let index = definitions
            .iter()
            .position(|d| *d == id)
            .ok_or("missing collection property owner")?;
        for p in &source.types[index].properties {
            inventory(&p.ty, &mut candidates);
        }
    }
    let service = &source.functions[services[0].0];
    let mut shapes = vec![];
    for owner in candidates {
        if let Some((kind, element, concrete)) =
            neoclr::native_metadata::collection_shape(source, service, &owner)
                .map_err(|e| e.to_string())?
        {
            // Classify every shape, but do not select execution adapters whose
            // element boxing is outside the native profile. The mapper rejects
            // unsupported scalar elements during preflight.
            let executable = source.is_object_reference_type(&element)
                || matches!(element, Type::Int32 | Type::Boolean);
            for t in [owner, concrete] {
                if !shapes.iter().any(|(s, _, _)| *s == t) {
                    shapes.push((t, kind, executable));
                }
            }
        }
    }
    let providers: Vec<_> = input
        .types
        .iter()
        .filter(|t| {
            t.origin.as_ref().is_some_and(|o| {
                matches!(
                    o.name.as_str(),
                    "System.Introspection.RuntimeTypeInfo"
                        | "System.Introspection.RuntimeNominalTypeInfo"
                        | "System.Introspection.RuntimeFunctionTypeInfo"
                )
            })
        })
        .map(|t| Type::Named(t.name.clone()))
        .collect();
    let mut bindings = vec![];
    for (index, operation) in services {
        let service = &source.functions[index];
        neoclr::native_metadata::validate_collection_service(source, service, operation)
            .map_err(|e| e.to_string())?;
        let mut body = vec![];
        let mut entries = vec![];
        for provider in &providers {
            let getter = input
                .functions
                .iter()
                .find(|f| {
                    f.owner.as_ref() == Some(provider)
                        && f.origin
                            .as_ref()
                            .is_some_and(|o| o.name == "get_ExecutionHandle")
                })
                .ok_or("missing collection TypeInfo handle getter")?;
            let target = FunctionRef {
                definition: getter.definition.clone(),
                name: getter.name.clone(),
                owner: Some(provider.clone()),
                instance: true,
                generic_arguments: vec![],
                parameters: vec![],
            };
            let branch = body.len() + 3;
            body.extend([
                Op::Arg(0),
                Op::IsInstance(provider.clone()),
                Op::ReferenceIsNull,
                Op::BranchTrue(0),
                Op::Arg(0),
                Op::CastClass(provider.clone()),
                Op::Call(target),
                Op::Store(0),
                Op::Branch(0),
            ]);
            entries.push(body.len() - 1);
            body[branch] = Op::BranchTrue(body.len());
        }
        body.push(Op::Fault(
            "collection access requires runtime TypeInfo".into(),
        ));
        let dispatch = body.len();
        for entry in entries {
            body[entry] = Op::Branch(dispatch);
        }
        for (owner, kind, executable) in &shapes {
            if (operation != 0 && !executable) || (operation == 3 && *kind == 1) {
                continue;
            }
            let branch = body.len() + 3;
            body.extend([
                Op::Load(0),
                Op::LoadTypeToken(owner.clone()),
                Op::Equal,
                Op::BranchFalse(0),
            ]);
            let mut plan =
                neoclr::native_metadata::collection_plan(source, service, owner, operation)
                    .map_err(|e| e.to_string())?;
            for op in &mut plan.body {
                if let Op::Call(target) = op {
                    let f = input
                        .functions
                        .iter()
                        .find(|f| {
                            f.name == target.name
                                && f.owner == target.owner
                                && f.parameters == target.parameters
                        })
                        .ok_or("missing native collection target")?;
                    target.definition = f.definition.clone();
                }
            }
            body.extend(plan.body);
            body[branch] = Op::BranchFalse(body.len());
        }
        if operation == 0 {
            body.extend([Op::Int(0), Op::Return]);
        } else {
            body.push(Op::Fault(
                "native collection execution was not retained".into(),
            ));
        }
        let f = &mut input.functions[index];
        f.impl_flags = 0;
        f.locals = vec![Type::RuntimeTypeHandle];
        f.local_names = vec![None];
        f.body = body;
        bindings.push(json!({"definition":f.definition,"name":f.name,"implementation":"reflection-collection-retained-v1","shapeCount":shapes.len()}));
    }
    retention["collectionBindings"] = json!(bindings);
    Ok(())
}
