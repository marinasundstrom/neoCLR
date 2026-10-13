//! Preserve the lexical caller through the verified RuntimeContext query facade.
//! The extra String argument is private AOT lowering, never source metadata or ABI.
use neoclr::metadata::{Function, FunctionRef, Instruction as Op, Type};
use neoclr::{Module, Value};
type Error = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct Query {
    pub getter: usize,
    pub assemblies: Vec<(String, Value)>,
}

fn targets(reference: &FunctionRef, function: &Function) -> bool {
    if reference.definition.is_some() {
        reference.definition == function.definition
    } else {
        reference.name == function.name
            && reference.owner == function.owner
            && reference.instance == function.instance
            && reference.parameters == function.parameters
    }
}

fn forwarding_target<'a>(module: &'a Module, function: &Function) -> Result<&'a Function, Error> {
    let [Op::Call(reference), Op::Return] = function.body.as_slice() else {
        return Err("ExecutingAssembly requires the exact source forwarding facade".into());
    };
    if !function.locals.is_empty()
        || function.impl_flags != 0
        || function.pinvoke.is_some()
        || !function.parameters.is_empty()
        || function.no_result
        || function.is_virtual
        || function.is_abstract
        || function.is_override
        || function.receiver_byref
        || function.receiver_readonly
        || !function.generic_parameters.is_empty()
        || !function.generic_arguments.is_empty()
        || !reference.parameters.is_empty()
        || reference.instance
        || !reference.generic_arguments.is_empty()
    {
        return Err(
            "ExecutingAssembly requires an ordinary parameterless forwarding facade".into(),
        );
    }
    let mut matches = module
        .functions
        .iter()
        .filter(|candidate| targets(reference, candidate));
    let target = matches
        .next()
        .ok_or("missing ExecutingAssembly facade target")?;
    if matches.next().is_some() || target.returns != function.returns {
        return Err("ambiguous or mismatched ExecutingAssembly facade target".into());
    }
    Ok(target)
}

pub fn has_queries(input: &Module, source: &Module) -> bool {
    input.functions.iter().any(|getter| {
        getter.origin.as_ref().is_some_and(|origin| origin.name == "get_ExecutingAssembly")
            && getter.owner.as_ref().and_then(|owner| source.type_definition(owner))
                .and_then(|owner| owner.origin.as_ref())
                .is_some_and(|origin| origin.name == "System.Runtime.RuntimeContext")
            && input.functions.iter().flat_map(|f| &f.body).any(|op|
                matches!(op, Op::Call(target) | Op::CallVirtual(target) | Op::BindFunction { target, .. }
                    if targets(target, getter)))
    })
}

pub fn prepare(input: &mut Module, source: &Module) -> Result<Vec<Query>, Error> {
    let getters: Vec<_> = input
        .functions
        .iter()
        .enumerate()
        .filter(|(_, function)| {
            function
                .origin
                .as_ref()
                .is_some_and(|origin| origin.name == "get_ExecutingAssembly")
                && function
                    .owner
                    .as_ref()
                    .and_then(|owner| source.type_definition(owner))
                    .and_then(|owner| owner.origin.as_ref())
                    .is_some_and(|origin| origin.name == "System.Runtime.RuntimeContext")
        })
        .map(|(index, _)| index)
        .collect();
    let mut result = vec![];
    for getter in getters {
        let original = input.functions[getter].clone();
        let owner = source
            .type_definition(original.owner.as_ref().unwrap())
            .unwrap();
        let Some(contract) = source
            .type_definition(&original.returns)
            .and_then(|ty| ty.origin.as_ref())
        else {
            continue;
        };
        let scope = owner.origin.as_ref().unwrap();
        if contract.name != "System.Introspection.AssemblyInfo"
            || contract.assembly != scope.assembly
            || contract.module != scope.module
        {
            continue; // A similarly named user type is not the runtime contract.
        }
        let wrapper = forwarding_target(input, &original)?;
        let wrapper_owner = wrapper
            .owner
            .as_ref()
            .and_then(|ty| source.type_definition(ty))
            .and_then(|ty| ty.origin.as_ref())
            .ok_or("missing ExecutingAssembly service owner")?;
        if !original.instance
            || !original.origin.as_ref().is_some_and(|origin| {
                origin.assembly == scope.assembly && origin.module == scope.module
            })
            || wrapper.instance
            || wrapper_owner.name != "System.Runtime.CompilerServices.IntrospectionRuntimeServices"
            || wrapper_owner.assembly != scope.assembly
            || wrapper_owner.module != scope.module
            || !wrapper.origin.as_ref().is_some_and(|origin| {
                origin.name == "ExecutingAssembly"
                    && origin.assembly == scope.assembly
                    && origin.module == scope.module
            })
        {
            return Err("ExecutingAssembly requires its scoped runtime service facade".into());
        }
        let service = forwarding_target(input, wrapper)?;
        if service.name != "neoCLR.Runtime.ExecutingAssembly"
            || service.owner.is_some()
            || service.instance
            || !service.parameters.is_empty()
            || service.impl_flags != 0x1000
            || !service.body.is_empty()
            || !service.locals.is_empty()
        {
            return Err("ExecutingAssembly requires the reserved InternalCall".into());
        }
        let mut assemblies = vec![];
        for index in 0..input.functions.len() {
            let function = &input.functions[index];
            if function.body.iter().any(
                |op| matches!(op, Op::BindFunction { target, .. } if targets(target, &original)),
            ) {
                return Err(
                    "native ExecutingAssembly requires a direct getter call, not a bound getter"
                        .into(),
                );
            }
            if !function.body.iter().any(|op| matches!(op, Op::Call(target) | Op::CallVirtual(target) if targets(target, &original))) {
                continue;
            }
            let recipe = neoclr::native_metadata::executing_assembly(
                source,
                &source.functions[getter],
                &source.functions[index],
            )
            .map_err(|error| error.to_string())?;
            let Value::Object { fields, .. } = &recipe else {
                return Err("invalid executing assembly recipe".into());
            };
            let [Value::String(identity)] = fields.as_slice() else {
                return Err("invalid executing assembly identity".into());
            };
            let identity = identity.to_string();
            if !assemblies.iter().any(|(known, _)| *known == identity) {
                assemblies.push((identity.clone(), recipe));
            }
            let function = &mut input.functions[index];
            let mut positions = Vec::with_capacity(function.body.len());
            let mut next = 0;
            for op in &function.body {
                positions.push(next);
                next += if matches!(op, Op::Call(target) | Op::CallVirtual(target) if targets(target, &original))
                {
                    2
                } else {
                    1
                };
            }
            let mut body = vec![];
            for op in &function.body {
                let mut op = op.clone();
                match &mut op {
                    Op::Call(target) | Op::CallVirtual(target) if targets(target, &original) => {
                        body.push(Op::String(identity.clone()));
                        target.parameters.push(Type::String);
                    }
                    Op::Branch(i)
                    | Op::BranchTrue(i)
                    | Op::BranchFalse(i)
                    | Op::BranchEqual(i)
                    | Op::BranchNotEqual(i)
                    | Op::BranchGreater(i)
                    | Op::BranchGreaterUnsigned(i)
                    | Op::BranchLess(i)
                    | Op::BranchLessUnsigned(i)
                    | Op::BranchGreaterEqual(i)
                    | Op::BranchGreaterEqualUnsigned(i)
                    | Op::BranchLessEqual(i)
                    | Op::BranchLessEqualUnsigned(i) => *i = positions[*i],
                    Op::Switch(targets) => {
                        for i in targets {
                            *i = positions[*i];
                        }
                    }
                    _ => {}
                }
                body.push(op);
            }
            for point in &mut function.sequence_points {
                point.instruction = positions[point.instruction];
            }
            function.body = body;
        }
        if !assemblies.is_empty() {
            input.functions[getter].parameters.push(Type::String);
            if let Some(origin) = &mut input.functions[getter].origin {
                origin.parameter_tokens.push(0); // Synthetic argument has no source metadata row.
                origin.nullable_annotations.clear();
            }
            input.functions[getter].parameter_names.push(None);
            result.push(Query { getter, assemblies });
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use neoclr::metadata::MemberId;
    use serde_json::json;

    fn fixture() -> Module {
        let mut source = neoclr::assemble(".module ContextTest\n.type class Context\n.end\n.type class Services\n.end\n.type class Assembly\n.end\n.function Stub() -> Int32\nldc.i4 0\nret\n.end\n").unwrap();
        for (ty, name) in source.types.iter_mut().zip([
            "System.Runtime.RuntimeContext",
            "System.Runtime.CompilerServices.IntrospectionRuntimeServices",
            "System.Introspection.AssemblyInfo",
        ]) {
            ty.origin = Some(
                serde_json::from_value(
                    json!({"assembly":"Runtime","module":"Runtime.dll","name":name,"token":1}),
                )
                .unwrap(),
            );
        }
        let seed = source.functions[0].clone();
        source.functions.clear();
        for (index, (name, owner, assembly)) in [
            ("get_ExecutingAssembly", Some("Context"), "Runtime"),
            ("ExecutingAssembly", Some("Services"), "Runtime"),
            ("neoCLR.Runtime.ExecutingAssembly", None, "Runtime"),
            ("AppCaller", None, "Application"),
            ("LibraryCaller", None, "Dependency"),
        ]
        .into_iter()
        .enumerate()
        {
            let mut function = seed.clone();
            function.name = name.into();
            function.owner = owner.map(Type::from_name);
            function.instance = index == 0;
            function.returns = Type::from_name("Assembly");
            function.definition = Some(MemberId {
                module: "ContextTest".into(),
                revision: source.revision.clone(),
                index: index as u32,
            });
            function.origin = Some(
                serde_json::from_value(
                    json!({"assembly":assembly,"module":"Runtime.dll","name":name,"token":index+1}),
                )
                .unwrap(),
            );
            source.functions.push(function);
        }
        let reference = |index: usize| {
            let f = &source.functions[index];
            FunctionRef {
                definition: f.definition.clone(),
                name: f.name.clone(),
                owner: f.owner.clone(),
                instance: f.instance,
                parameters: vec![],
                generic_arguments: vec![],
            }
        };
        let getter = reference(0);
        let wrapper = reference(1);
        let service = reference(2);
        source.functions[0].body = vec![Op::Call(wrapper), Op::Return];
        source.functions[1].body = vec![Op::Call(service), Op::Return];
        source.functions[2].body.clear();
        source.functions[2].impl_flags = 0x1000;
        source.functions[3].parameters = vec![Type::from_name("Context")];
        source.functions[3].body = vec![
            Op::Bool(false),
            Op::BranchFalse(5),
            Op::Arg(0),
            Op::CallVirtual(getter.clone()),
            Op::Return,
            Op::Arg(0),
            Op::Call(getter.clone()),
            Op::Return,
        ];
        source.functions[4].parameters = vec![Type::from_name("Context")];
        source.functions[4].body = vec![Op::Arg(0), Op::Call(getter), Op::Return];
        source.assemblies = serde_json::from_value(json!([
            {"name":"App","full_name":"Application","modules":["App.dll"],"references":[]},
            {"name":"Library","full_name":"Dependency","modules":["Library.dll"],"references":[]},
            {"name":"Runtime","full_name":"Runtime","modules":["Runtime.dll"],"references":[]}
        ]))
        .unwrap();
        source
    }

    #[test]
    fn callers_keep_their_own_assembly_and_virtual_receiver_checks() {
        let source = fixture();
        let mut input = source.clone();
        let queries = prepare(&mut input, &source).unwrap();
        assert_eq!(queries.len(), 1);
        assert_eq!(
            queries[0]
                .assemblies
                .iter()
                .map(|(identity, _)| identity.as_str())
                .collect::<Vec<_>>(),
            ["Application", "Dependency"]
        );
        assert_eq!(input.functions[0].parameters, [Type::String]);
        assert!(matches!(input.functions[3].body[1], Op::BranchFalse(6)));
        assert!(
            matches!(&input.functions[3].body[3], Op::String(identity) if identity == "Application")
        );
        assert!(
            matches!(&input.functions[3].body[4], Op::CallVirtual(target) if target.parameters == [Type::String])
        );
        assert!(
            matches!(&input.functions[4].body[1], Op::String(identity) if identity == "Dependency")
        );
        assert!(source.functions[0].parameters.is_empty());
    }

    #[test]
    fn facade_scope_preserves_a_foreign_same_named_caller() {
        let mut source = fixture();
        let mut foreign = source.types[0].clone();
        foreign.name = "ForeignContext".into();
        foreign.origin.as_mut().unwrap().assembly = "Dependency".into();
        foreign.origin.as_mut().unwrap().module = "Library.dll".into();
        source.types.push(foreign);
        let mut caller = source.functions[4].clone();
        caller.owner = Some(Type::from_name("ForeignContext"));
        caller.parameters.clear();
        let origin = caller.origin.as_mut().unwrap();
        origin.name = "get_ExecutingAssembly".into();
        origin.module = "Library.dll".into();
        let Value::Object { fields, .. } =
            neoclr::native_metadata::executing_assembly(&source, &source.functions[0], &caller)
                .unwrap()
        else {
            panic!("expected assembly descriptor");
        };
        assert_eq!(fields, [Value::String("Dependency".into())]);
    }

    #[test]
    fn changed_facades_and_indirect_getters_are_rejected() {
        let source = fixture();
        let mut changed = source.clone();
        changed.functions[0].body.insert(0, Op::Int(42));
        assert!(prepare(&mut changed, &source).is_err());
        let mut indirect = source.clone();
        let Op::Call(target) = indirect.functions[4].body[1].clone() else {
            panic!("missing getter call");
        };
        indirect.functions[4].body[1] = Op::BindFunction {
            function_type: Type::from_name("Callback"),
            target,
        };
        assert!(
            prepare(&mut indirect, &source)
                .unwrap_err()
                .to_string()
                .contains("bound getter")
        );
    }
}
