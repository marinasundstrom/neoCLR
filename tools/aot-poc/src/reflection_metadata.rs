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

// Keep this closed-token subset aligned with reflection::Query::Shape. Source
// metadata, never projected native layout, determines these language-visible facts.
fn visible(source: &neoclr::Module, ty: &Type) -> bool {
    match ty {
        Type::Function(function) => function
            .parameters
            .iter()
            .chain(std::iter::once(&function.returns))
            .all(|t| visible(source, t)),
        Type::Array(t)
        | Type::ArrayRef(t)
        | Type::ByRef(t)
        | Type::ReadOnlyByRef(t)
        | Type::Ptr(t)
        | Type::InterfaceRef(t) => visible(source, t),
        Type::TypeParameter(_) | Type::MethodTypeParameter(_) => true,
        _ => {
            source.type_definition(ty).is_none_or(|d| {
                d.visibility == neoclr::metadata::Visibility::Public
                    && d.origin
                        .as_ref()
                        .is_none_or(|o| o.publicly_visible == Some(true))
            }) && match ty {
                Type::Constructed { arguments, .. } => arguments.iter().all(|t| visible(source, t)),
                _ => true,
            }
        }
    }
}

fn shape(source: &neoclr::Module, ty: &Type) -> Result<[bool; 14], Error> {
    use neoclr::metadata::Representation;
    let d = source
        .type_definition(ty)
        .ok_or("native shape queries require source declaration")?;
    let interface = d.representation == Representation::Interface;
    let mut result = [false; 14];
    result[3] = interface;
    result[5] = d.is_abstract || interface;
    result[6] = d.enum_info.is_some();
    result[7] = if ty.is_primitive() {
        *ty != Type::String
    } else {
        !d.is_reference_type && !interface
    };
    result[8] = (d.is_reference_type || interface) && !d.is_sealed && !d.is_closed_hierarchy;
    result[9] = d.is_closed_hierarchy;
    result[10] = d.custom_attributes.iter().any(|a| {
        a.target_token.is_none()
            && a.constructor.owner.as_ref().and_then(Type::definition_name)
                == Some("System.Runtime.CompilerServices.UnionAttribute")
    });
    result[11] = visible(source, ty);
    result[12] = true;
    Ok(result)
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
                Some(
                    "neoCLR.Runtime.TypeName"
                        | "neoCLR.Runtime.TypeArgumentCount"
                        | "neoCLR.Runtime.TypeShape"
                )
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
        let (name, count, semantic_type) = if token.is_primitive() {
            let declaration = source
                .type_definition(token)
                .ok_or("native type queries require source primitive metadata")?;
            (
                declaration
                    .origin
                    .as_ref()
                    .map_or_else(|| declaration.name.clone(), |o| o.name.clone()),
                0,
                token.clone(),
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
            let arguments: Vec<Type> = serde_json::from_value(row["typeArguments"].clone())?;
            let definition = row["name"]
                .as_str()
                .ok_or("missing semantic type name")?
                .to_owned();
            let semantic = if arguments.is_empty() {
                Type::Named(definition)
            } else {
                Type::Constructed {
                    definition,
                    arguments: arguments.clone(),
                }
            };
            (name.to_owned(), arguments.len(), semantic)
        } else {
            return Err(format!("native type queries currently require primitive or closed nominal tokens; found {token:?}").into());
        };
        let shapes = if queries
            .iter()
            .any(|r| r["name"] == "neoCLR.Runtime.TypeShape")
        {
            shape(source, &semantic_type)?
        } else {
            [false; 14]
        };
        descriptors.push((token.clone(), name, count, shapes));
    }
    let mut bindings = vec![];
    for row in queries {
        let index = row["compiledIndex"].as_u64().ok_or("missing query index")? as usize;
        let f = &mut input.functions[index];
        let names = row["name"] == "neoCLR.Runtime.TypeName";
        let shapes = row["name"] == "neoCLR.Runtime.TypeShape";
        let result = if names {
            Type::String
        } else if shapes {
            Type::Boolean
        } else {
            Type::Int32
        };
        let parameters = if shapes {
            vec![Type::RuntimeTypeHandle, Type::Int32]
        } else {
            vec![Type::RuntimeTypeHandle]
        };
        if f.name != row["name"].as_str().unwrap()
            || f.owner.is_some()
            || f.instance
            || f.receiver_byref
            || f.receiver_readonly
            || f.parameters != parameters
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
            return Err("native descriptor query requires exact reserved TypeName/TypeArgumentCount/TypeShape InternalCall contract".into());
        }
        let mut body = vec![];
        for (token, name, count, flags) in &descriptors {
            let branch = body.len() + 3;
            body.extend([
                Op::Arg(0),
                Op::LoadTypeToken(token.clone()),
                Op::Equal,
                Op::BranchFalse(0),
            ]);
            if shapes {
                for (selector, value) in flags.iter().enumerate() {
                    let next = body.len() + 6;
                    body.extend([
                        Op::Arg(1),
                        Op::Int(selector as i32),
                        Op::Equal,
                        Op::BranchFalse(next),
                        Op::Bool(*value),
                        Op::Return,
                    ]);
                }
                body.push(Op::Fault("unknown type shape query".into()));
            } else {
                body.extend([
                    if names {
                        Op::String(name.clone())
                    } else {
                        Op::Int(*count as i32)
                    },
                    Op::Return,
                ]);
            }
            body[branch] = Op::BranchFalse(body.len());
        }
        // All guest token producers are enumerated above. Foreign/forged handles
        // are not admitted by this closed profile; never fabricate an empty name.
        body.push(Op::Fault("Unknown native type descriptor".into()));
        f.impl_flags = 0;
        f.body = body;
        bindings.push(json!({"definition":row["definition"],"name":row["name"],"compiledIndex":index,
            "implementation":if names { "type-name-closed-v1" } else if shapes { "type-shape-closed-v1" } else { "type-argument-count-closed-v1" },
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

#[cfg(test)]
mod shape_tests {
    use super::*;

    #[test]
    fn source_shape_facts_preserve_closed_union_and_origin_visibility() {
        let mut source = neoclr::assemble(".module Shapes\n.type class Choice\n.end\n").unwrap();
        let ty = Type::Named("Choice".into());
        source.types[0].is_closed_hierarchy = true;
        source.types[0].custom_attributes.push(serde_json::from_value(json!({"constructor": {
            "name":".ctor", "owner":{"Named":"System.Runtime.CompilerServices.UnionAttribute"}, "instance":true,"parameters":[]
        }})).unwrap());
        let flags = shape(&source, &ty).unwrap();
        assert!(!flags[8]);
        assert!(flags[9] && flags[10] && flags[11]);
        source.types[0].origin = Some(serde_json::from_value(json!({"assembly":"Source","module":"Source.dll","name":"Outer/Choice","token":33554433})).unwrap());
        assert!(!shape(&source, &ty).unwrap()[11]);
        source.types[0].origin.as_mut().unwrap().publicly_visible = Some(true);
        assert!(shape(&source, &ty).unwrap()[11]);
        source.types[0].custom_attributes[0].target_token = Some(33554433);
        assert!(!shape(&source, &ty).unwrap()[10]);
    }
}
