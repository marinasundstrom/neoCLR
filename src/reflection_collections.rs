//! Private checked collection plans. Execution stays in ordinary library generic code.
use crate::{
    Fault, Module, Value,
    metadata::{Function, FunctionRef, Instruction as Op, Type},
};

/// Exact source-owned collection families; never classify an arbitrary user name.
pub(crate) fn shape(
    module: &Module,
    service: &Function,
    owner: &Type,
) -> Result<Option<(i32, Type, Type)>, Fault> {
    let parameter = service
        .parameters
        .first()
        .ok_or_else(|| Fault::new("collection service requires TypeInfo parameter"))?;
    let contract = module
        .type_definition(parameter)
        .and_then(|t| t.origin.as_ref())
        .ok_or_else(|| Fault::new("collection service requires scoped TypeInfo"))?;
    let Type::Constructed { arguments, .. } = owner else {
        return Ok(None);
    };
    let Some(origin) = module
        .type_definition(owner)
        .and_then(|t| t.origin.as_ref())
    else {
        return Ok(None);
    };
    if origin.assembly != contract.assembly || origin.module != contract.module {
        return Ok(None);
    }
    let name = origin.name.split('`').next().unwrap();
    let (kind, element, concrete_name) = match (name, arguments.as_slice()) {
        (
            "System.Collections.Sequence"
            | "System.Collections.List"
            | "System.Collections.ArrayList",
            [element],
        ) => (1, element.clone(), "System.Collections.ArrayList"),
        (
            "System.Collections.Map"
            | "System.Collections.MutableMap"
            | "System.Collections.HashMap",
            [Type::String, element],
        ) => (2, element.clone(), "System.Collections.HashMap"),
        _ => return Ok(None),
    };
    let candidates: Vec<_> = module
        .types
        .iter()
        .filter(|t| {
            t.origin.as_ref().is_some_and(|o| {
                o.assembly == contract.assembly
                    && o.module == contract.module
                    && o.name.split('`').next() == Some(concrete_name)
            })
        })
        .collect();
    let [concrete] = candidates.as_slice() else {
        return Err(Fault::new("ambiguous or missing collection implementation"));
    };
    let concrete = Type::Constructed {
        definition: concrete.name.clone(),
        arguments: if kind == 1 {
            vec![element.clone()]
        } else {
            vec![Type::String, element.clone()]
        },
    };
    Ok(Some((kind, element, concrete)))
}

pub(crate) fn validate_service(
    module: &Module,
    service: &Function,
    operation: u8,
) -> Result<(), Fault> {
    if !matches!(crate::native::bind_in(module, service)?, crate::native::Binding::ReflectionCollection(op) if op == operation)
        || service.impl_flags != 0x1000
        || !service.body.is_empty()
        || !service.locals.is_empty()
        || service.pinvoke.is_some()
        || service.is_virtual
        || service.is_abstract
        || service.is_override
        || service.receiver_byref
        || service.receiver_readonly
        || !service.generic_parameters.is_empty()
        || !service.generic_arguments.is_empty()
        || !service.generic_constraints.is_empty()
        || !service.interface_implementations.is_empty()
        || !service.out_parameters.is_empty()
        || !service.out_when_true.is_empty()
        || !service.readonly_parameters.is_empty()
    {
        return Err(Fault::new("invalid collection service contract"));
    }
    Ok(())
}

pub(crate) fn plan(
    module: &Module,
    service: &Function,
    owner: &Type,
    operation: u8,
) -> Result<Function, Fault> {
    validate_service(module, service, operation)?;
    let shape = shape(module, service, owner)?;
    let mut adapter = service.clone();
    adapter.impl_flags = 0;
    if operation == 0 {
        adapter.body = vec![Op::Int(shape.map_or(0, |s| s.0)), Op::Return];
        return Ok(adapter);
    }
    let (kind, element, _) = shape.ok_or_else(|| Fault::new("unsupported collection shape"))?;
    if kind == 1 && operation == 3 {
        return Err(Fault::new("sequence has no keys"));
    }
    let method = format!(
        "{}{}",
        if kind == 1 { "Sequence" } else { "Map" },
        match operation {
            1 => "Count",
            2 => "Get",
            3 => "Key",
            4 => "Create",
            _ => return Err(Fault::new("unknown collection operation")),
        }
    );
    let origin = service
        .origin
        .as_ref()
        .ok_or_else(|| Fault::new("missing collection service identity"))?;
    let helpers: Vec<_> = module
        .types
        .iter()
        .filter(|t| {
            t.origin.as_ref().is_some_and(|o| {
                o.assembly == origin.assembly
                    && o.module == origin.module
                    && o.name == "System.Data.Json.CollectionAdapters"
            })
        })
        .collect();
    let [helper] = helpers.as_slice() else {
        return Err(Fault::new("missing or ambiguous collection adapters"));
    };
    let candidates: Vec<_> = module
        .functions
        .iter()
        .filter(|f| {
            f.owner.as_ref() == Some(&Type::Named(helper.name.clone()))
                && f.origin.as_ref().is_some_and(|o| o.name == method)
        })
        .collect();
    let [target] = candidates.as_slice() else {
        return Err(Fault::new("missing or ambiguous collection adapter method"));
    };
    if target.instance
        || target.generic_parameters.len() != 1
        || target.is_internal_call()
        || target.pinvoke.is_some()
        || target.no_result
        || target.parameters != service.parameters[1..]
        || target.returns != service.returns
    {
        return Err(Fault::new("invalid collection adapter contract"));
    }
    let target = FunctionRef {
        definition: target.definition.clone(),
        name: target.name.clone(),
        owner: target.owner.clone(),
        instance: false,
        generic_arguments: vec![element],
        parameters: target.parameters.clone(),
    };
    adapter.body = (1..service.parameters.len()).map(Op::Arg).collect();
    adapter.body.extend([Op::Call(target), Op::Return]);
    Ok(adapter)
}

pub(crate) fn adapter(
    module: &Module,
    service: &Function,
    args: &[Value],
    operation: u8,
) -> Result<Function, Fault> {
    let Some(Value::ObjectReference(info)) = args.first() else {
        return Err(Fault::new("collection access requires runtime TypeInfo"));
    };
    if !matches!(
        crate::reflection_source::name(module, &info.concrete_type()),
        Some(
            "System.Introspection.RuntimeTypeInfo"
                | "System.Introspection.RuntimeNominalTypeInfo"
                | "System.Introspection.RuntimeFunctionTypeInfo"
        )
    ) {
        return Err(Fault::new("collection access requires runtime TypeInfo"));
    }
    let Value::Object { fields, .. } = info.reference.read()? else {
        return Err(Fault::new("invalid runtime TypeInfo"));
    };
    let handle = fields
        .first()
        .ok_or_else(|| Fault::new("missing runtime type handle"))?;
    let owner = crate::reflection_execution::bound_type(module, handle)
        .map_err(|_| Fault::new("collection access requires bound type"))?;
    plan(module, service, &owner, operation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> (Module, Function, Type) {
        let mut module = crate::assemble(".module Collections\n.references ()\n").unwrap();
        for (name, source) in [
            ("Info", "System.Introspection.TypeInfo"),
            ("Sequence", "System.Collections.Sequence`1"),
            ("List", "System.Collections.ArrayList`1"),
            ("Map", "System.Collections.Map`2"),
            ("HashMap", "System.Collections.HashMap`2"),
        ] {
            module.types.push(serde_json::from_value(json!({
                "name": name, "fields": [], "generic_parameters": if name == "Info" { vec![] } else if name == "Map" || name == "HashMap" { vec![Some("K"), Some("V")] } else { vec![Some("T")] },
                "origin": {"assembly":"System.Runtime","module":"System.Runtime.dll","name":source,"token":0}
            })).unwrap());
        }
        let service: Function = serde_json::from_value(json!({
            "name":"neoCLR.Runtime.ReflectionCollectionKind",
            "parameters":[Type::Named("Info".into())],"returns":Type::Int32,"impl_flags":4096
        }))
        .unwrap();
        let owner = Type::Constructed {
            definition: "Sequence".into(),
            arguments: vec![Type::Int32],
        };
        (module, service, owner)
    }

    #[test]
    fn collection_shapes_require_source_scope_and_string_keys() {
        let (mut module, service, owner) = fixture();
        assert_eq!(shape(&module, &service, &owner).unwrap().unwrap().0, 1);
        let map = Type::Constructed {
            definition: "Map".into(),
            arguments: vec![Type::String, owner.clone()],
        };
        assert_eq!(shape(&module, &service, &map).unwrap().unwrap().0, 2);
        let wrong = Type::Constructed {
            definition: "Map".into(),
            arguments: vec![Type::Int32, Type::Int32],
        };
        assert!(shape(&module, &service, &wrong).unwrap().is_none());
        module.types[1].origin.as_mut().unwrap().assembly = "Impostor".into();
        assert!(shape(&module, &service, &owner).unwrap().is_none());
        let mut malformed = service.clone();
        malformed.parameters.clear();
        assert!(shape(&module, &malformed, &owner).is_err());
    }

    #[test]
    fn collection_kind_validates_exact_service_before_emitting_plan() {
        let (module, service, owner) = fixture();
        assert!(matches!(
            plan(&module, &service, &owner, 0).unwrap().body.as_slice(),
            [Op::Int(1), Op::Return]
        ));
        for mutation in 0..5 {
            let mut bad = service.clone();
            match mutation {
                0 => bad.returns = Type::Boolean,
                1 => bad.body = vec![Op::Int(0), Op::Return],
                2 => bad.is_virtual = true,
                3 => bad.parameters.clear(),
                _ => bad.generic_parameters = vec![Some("T".into())],
            }
            assert!(plan(&module, &bad, &owner, 0).is_err());
        }
        assert!(plan(&module, &service, &owner, 1).is_err());
        assert!(adapter(&module, &service, &[], 0).is_err());
    }
}
