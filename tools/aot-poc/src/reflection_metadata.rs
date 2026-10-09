//! Source metadata retained independently of executable lowering. This catalogue
//! is build-time evidence, not runtime reflection admission or invocation authority.
use neoclr::metadata::{FunctionRef, MemberId, Property, Type, TypeDefId};
use serde_json::{Value, json};
type Error = Box<dyn std::error::Error>;

fn close(ty: &Type, arguments: &[Type]) -> Result<Type, Error> {
    Ok(ty
        .substitute_type_parameters(arguments)
        .map_err(|e| e.to_string())?)
}

fn accessor(reference: &FunctionRef, arguments: &[Type]) -> Result<FunctionRef, Error> {
    let mut result = reference.clone();
    result.owner = reference
        .owner
        .as_ref()
        .map(|t| close(t, arguments))
        .transpose()?;
    result.parameters = reference
        .parameters
        .iter()
        .map(|t| close(t, arguments))
        .collect::<Result<_, _>>()?;
    result.generic_arguments = reference
        .generic_arguments
        .iter()
        .map(|t| close(t, arguments))
        .collect::<Result<_, _>>()?;
    Ok(result)
}

fn property(source: &Property, arguments: &[Type]) -> Result<Property, Error> {
    Ok(Property {
        name: source.name.clone(),
        instance: source.instance,
        ty: close(&source.ty, arguments)?,
        parameters: source
            .parameters
            .iter()
            .map(|t| close(t, arguments))
            .collect::<Result<_, _>>()?,
        getter: source
            .getter
            .as_ref()
            .map(|r| accessor(r, arguments))
            .transpose()?,
        setter: source
            .setter
            .as_ref()
            .map(|r| accessor(r, arguments))
            .transpose()?,
    })
}

/// Called after report identities are restored, using a snapshot taken BEFORE
/// canonicalization. Never consult lowered properties (which may be empty).
pub fn catalogue(
    source: &neoclr::Module,
    definitions: &[TypeDefId],
    methods: &[MemberId],
    report: &Value,
) -> Result<Value, Error> {
    let rows = report["types"]
        .as_array()
        .ok_or("missing selected type inventory")?;
    let mut types = Vec::with_capacity(rows.len());
    for row in rows {
        let index = row["sourceIndex"]
            .as_u64()
            .ok_or("missing source type index")? as usize;
        let declaration = source
            .types
            .get(index)
            .ok_or("invalid source metadata type index")?;
        let definition = definitions
            .get(index)
            .ok_or("missing source metadata identity")?;
        let arguments: Vec<Type> = if row["typeArguments"].is_null() {
            vec![]
        } else {
            serde_json::from_value(row["typeArguments"].clone())?
        };
        if arguments.len() != declaration.generic_parameters.len() {
            return Err("source metadata requires the selected type's closed arguments".into());
        }
        let open_owner = if arguments.is_empty() {
            Type::Named(declaration.name.clone())
        } else {
            Type::Constructed {
                definition: declaration.name.clone(),
                arguments: (0..arguments.len())
                    .map(|i| Type::TypeParameter(i as u16))
                    .collect(),
            }
        };
        let mut members = vec![];
        for (i, f) in source.functions.iter().enumerate() {
            if f.owner.as_ref() != Some(&open_owner) {
                continue;
            }
            // Owner names alone must not merge methods from different source scopes.
            if methods[i].module != definition.module || methods[i].revision != definition.revision
            {
                continue;
            }
            members.push(json!({"definition":methods[i],"name":f.name,"visibility":f.visibility,
                "instance":f.instance,"virtual":f.is_virtual,"override":f.is_override,"abstract":f.is_abstract,
                "noResult":f.no_result,"receiverByRef":f.receiver_byref,"receiverReadonly":f.receiver_readonly,
                "parameters":f.parameters.iter().map(|t| close(t, &arguments)).collect::<Result<Vec<_>, _>>()?,
                "returns":close(&f.returns, &arguments)?,
                "genericParameters":f.generic_parameters,"origin":f.origin}));
        }
        let properties = declaration
            .properties
            .iter()
            .map(|p| property(p, &arguments))
            .collect::<Result<Vec<_>, _>>()?;
        types.push(json!({"definition":definition,"compiledTypeIndex":row["compiledIndex"],
            "declaration":declaration,"name":declaration.name,"typeArguments":arguments,"visibility":declaration.visibility,
            "declaringType":declaration.declaring_type,"origin":declaration.origin,
            "base":declaration.base.as_ref().map(|t| close(t, &arguments)).transpose()?,
            "interfaces":declaration.implements.iter().map(|t| close(t, &arguments)).collect::<Result<Vec<_>, _>>()?,
            "properties":properties,"declaredMethods":members}));
    }
    Ok(
        json!({"schemaVersion":1,"policy":"build-time source catalogue for selected nominal types; no additional executable roots, runtime tables or invocation admission","types":types}),
    )
}

/// Bind descriptor queries over the admitted token producers. The catalogue is
/// authoritative for semantic names; private specialized names are never exposed.
pub fn bind_queries(
    input: &mut neoclr::Module,
    source: &neoclr::Module,
    report: &Value,
) -> Result<Vec<Value>, Error> {
    use neoclr::metadata::Instruction as Op;
    let queries: Vec<_> = report["functions"]
        .as_array()
        .ok_or("missing function inventory")?
        .iter()
        .filter(|r| {
            matches!(
                r["name"].as_str(),
                Some("neoCLR.Runtime.TypeName" | "neoCLR.Runtime.TypeArgumentCount")
            )
        })
        .cloned()
        .collect();
    if queries.is_empty() {
        return Ok(vec![]);
    }
    let mut tokens = vec![];
    for f in &input.functions {
        for op in &f.body {
            if let Op::LoadTypeToken(t) = op {
                if !tokens.contains(t) {
                    tokens.push(t.clone());
                }
            }
        }
    }
    let mut descriptors = vec![];
    for token in &tokens {
        let (name, count) = if token.is_primitive() {
            let declaration = source
                .type_definition(token)
                .ok_or("native type queries require source primitive metadata")?;
            (
                declaration
                    .origin
                    .as_ref()
                    .map_or_else(|| declaration.name.clone(), |o| o.name.clone()),
                0,
            )
        } else if let Type::Named(name) = token {
            let index = input
                .types
                .iter()
                .position(|t| &t.name == name)
                .ok_or("type query token has no native shape")?;
            let row = report["sourceMetadata"]["types"]
                .as_array()
                .ok_or("type queries require source metadata catalogue")?
                .iter()
                .find(|r| r["compiledTypeIndex"] == index)
                .ok_or("type query token has no source descriptor")?;
            let name = row["origin"]["name"]
                .as_str()
                .or_else(|| row["name"].as_str())
                .ok_or("source descriptor has no name")?;
            (
                name.to_owned(),
                row["typeArguments"]
                    .as_array()
                    .ok_or("source descriptor has no type arguments")?
                    .len(),
            )
        } else {
            return Err(format!("native type queries currently require primitive or closed nominal tokens; found {token:?}").into());
        };
        descriptors.push((token.clone(), name, count));
    }
    let mut bindings = vec![];
    for row in queries {
        let index = row["compiledIndex"].as_u64().ok_or("missing query index")? as usize;
        let f = &mut input.functions[index];
        let names = row["name"] == "neoCLR.Runtime.TypeName";
        let result = if names { Type::String } else { Type::Int32 };
        if f.name != row["name"].as_str().unwrap()
            || f.owner.is_some()
            || f.instance
            || f.receiver_byref
            || f.receiver_readonly
            || f.parameters != [Type::RuntimeTypeHandle]
            || f.returns != result
            || f.no_result
            || f.impl_flags != 0x1000
            || f.pinvoke.is_some()
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.is_virtual
            || f.is_override
            || f.is_abstract
            || !f.generic_parameters.is_empty()
            || !f.generic_arguments.is_empty()
            || !f.generic_constraints.is_empty()
            || !f.interface_implementations.is_empty()
            || !f.out_parameters.is_empty()
            || !f.out_when_true.is_empty()
            || !f.readonly_parameters.is_empty()
        {
            return Err("native descriptor query requires exact reserved TypeName/TypeArgumentCount InternalCall contract".into());
        }
        let mut body = vec![];
        for (token, name, count) in &descriptors {
            let next = body.len() + 6;
            body.extend([
                Op::Arg(0),
                Op::LoadTypeToken(token.clone()),
                Op::Equal,
                Op::BranchFalse(next),
                if names {
                    Op::String(name.clone())
                } else {
                    Op::Int(*count as i32)
                },
                Op::Return,
            ]);
        }
        // All guest token producers are enumerated above. Foreign/forged handles
        // are not admitted by this closed profile; never fabricate an empty name.
        body.push(Op::Fault("Unknown native type descriptor".into()));
        f.impl_flags = 0;
        f.body = body;
        bindings.push(json!({"definition":row["definition"],"name":row["name"],"compiledIndex":index,
            "implementation":if names { "type-name-closed-v1" } else { "type-argument-count-closed-v1" },
            "descriptorCount":descriptors.len(),"policy":"closed token producers; semantic source names; no accessor reachability"}));
    }
    Ok(bindings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use neoclr::metadata::Instruction as Op;

    fn query() -> (neoclr::Module, Value) {
        let module = neoclr::assemble(".module Queries\n.type HiddenBackendName\n.end\n.function neoCLR.Runtime.TypeName(RuntimeTypeHandle) -> String\n.methodimpl InternalCall\n.end\n.function Main() -> RuntimeTypeHandle\nldtoken HiddenBackendName\nret\n.end\n").unwrap();
        let report = json!({"functions":[{"name":"neoCLR.Runtime.TypeName","compiledIndex":0}],
            "sourceMetadata":{"types":[{"compiledTypeIndex":0,"name":"Original","origin":{"name":"Café"},"typeArguments":[]}]}});
        (module, report)
    }

    #[test]
    fn descriptor_names_use_source_origin_and_require_exact_contract() {
        let (source, report) = query();
        let mut input = source.clone();
        bind_queries(&mut input, &source, &report).unwrap();
        assert!(
            input.functions[0]
                .body
                .iter()
                .any(|op| matches!(op, Op::String(s) if s == "Café"))
        );
        assert!(
            !input.functions[0]
                .body
                .iter()
                .any(|op| matches!(op, Op::String(s) if s == "HiddenBackendName"))
        );
        for change in 0..5 {
            let mut input = source.clone();
            match change {
                0 => input.functions[0].parameters[0] = Type::UInt64,
                1 => input.functions[0].returns = Type::Int32,
                2 => input.functions[0].impl_flags = 0,
                3 => input.functions[0].body = vec![Op::String("spoof".into()), Op::Return],
                _ => input.functions[0].out_parameters = vec![0],
            }
            assert!(bind_queries(&mut input, &source, &report).is_err());
        }
    }

    #[test]
    fn descriptor_queries_reject_missing_metadata_and_unadmitted_shapes() {
        let (source, mut report) = query();
        report["sourceMetadata"]["types"] = json!([]);
        assert!(
            bind_queries(&mut source.clone(), &source, &report)
                .unwrap_err()
                .to_string()
                .contains("no source descriptor")
        );
        let (mut input, report) = query();
        input.functions[1].body[0] = Op::LoadTypeToken(Type::ArrayRef(Box::new(Type::Int32)));
        assert!(
            bind_queries(&mut input, &source, &report)
                .unwrap_err()
                .to_string()
                .contains("primitive or closed nominal")
        );
    }
}
