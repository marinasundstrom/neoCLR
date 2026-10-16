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
        init_only: source.init_only,
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
                "genericParameters":f.generic_parameters,"genericConstraints":f.generic_constraints,"origin":f.origin,"customAttributes":f.custom_attributes}));
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

pub(super) fn semantic_token(
    input: &neoclr::Module,
    report: &Value,
    token: &Type,
) -> Result<Type, Error> {
    if token.is_primitive() {
        return Ok(token.clone());
    }
    if let Type::Function(function) = token {
        let mut function = function.clone();
        function.parameters = function
            .parameters
            .iter()
            .map(|p| semantic_token(input, report, p))
            .collect::<Result<_, _>>()?;
        function.returns = semantic_token(input, report, &function.returns)?;
        return Ok(Type::Function(function));
    }
    if let Type::ArrayRef(element) = token {
        return Ok(Type::ArrayRef(Box::new(semantic_token(
            input, report, element,
        )?)));
    }
    let Type::Named(name) = token else {
        return Err("native semantic token requires a closed nominal or vector type".into());
    };
    let index = input
        .types
        .iter()
        .position(|t| t.name == *name)
        .ok_or("missing token type")?;
    let row = report["sourceMetadata"]["types"]
        .as_array()
        .ok_or("missing source catalogue")?
        .iter()
        .find(|r| r["compiledTypeIndex"] == index)
        .ok_or("missing source token descriptor")?;
    let definition = row["name"]
        .as_str()
        .ok_or("missing source type name")?
        .to_owned();
    let arguments: Vec<Type> = serde_json::from_value(row["typeArguments"].clone())?;
    Ok(if arguments.is_empty() {
        Type::Named(definition)
    } else {
        Type::Constructed {
            definition,
            arguments,
        }
    })
}

fn native_token(input: &neoclr::Module, report: &Value, semantic: &Type) -> Result<Type, Error> {
    if semantic.is_primitive() {
        return Ok(semantic.clone());
    }
    if let Type::Function(function) = semantic {
        let mut function = function.clone();
        function.parameters = function
            .parameters
            .iter()
            .map(|p| native_token(input, report, p))
            .collect::<Result<_, _>>()?;
        function.returns = native_token(input, report, &function.returns)?;
        return Ok(Type::Function(function));
    }
    if let Type::ArrayRef(element) = semantic {
        return Ok(Type::ArrayRef(Box::new(native_token(
            input, report, element,
        )?)));
    }
    for t in &input.types {
        let token = Type::Named(t.name.clone());
        if semantic_token(input, report, &token).ok().as_ref() == Some(semantic) {
            return Ok(token);
        }
    }
    Err(format!("generic argument type metadata was not retained: {semantic:?}").into())
}

fn signature_name(ty: &Type) -> Result<String, Error> {
    Ok(match ty {
        Type::ArrayRef(element) => format!("arrayref<{}>", signature_name(element)?),
        Type::Constructed {
            definition,
            arguments,
        } => format!(
            "{}<{}>",
            definition,
            arguments
                .iter()
                .map(signature_name)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        ),
        _ => ty
            .definition_name()
            .ok_or("unsupported descriptor signature")?
            .to_owned(),
    })
}

fn shape(source: &neoclr::Module, ty: &Type) -> Result<[bool; 20], Error> {
    use neoclr::metadata::Representation;
    let Some(d) = source.type_definition(ty) else {
        if matches!(ty, Type::Function(_)) {
            let mut result = [false; 20];
            result[11] = visible(source, ty);
            result[13] = true;
            return Ok(result);
        }
        if matches!(ty, Type::ArrayRef(_)) {
            let mut result = [false; 20];
            result[0] = true;
            result[11] = visible(source, ty);
            return Ok(result);
        }
        return Err("native shape queries require source declaration".into());
    };
    let interface = d.representation == Representation::Interface;
    let mut result = [false; 20];
    result[0] = matches!(ty, Type::ArrayRef(_));
    result[3] = interface;
    result[5] = d.is_abstract || interface;
    result[6] = d.enum_info.is_some();
    result[7] = if matches!(ty, Type::ArrayRef(_)) {
        false
    } else if ty.is_primitive() {
        *ty != Type::String
    } else {
        !d.is_reference_type && !interface
    };
    result[8] = (d.is_reference_type || interface) && !d.is_sealed && !d.is_closed_hierarchy;
    result[9] = d.is_closed_hierarchy;
    result[10] = d.custom_attributes.iter().any(|a| {
        a.target_token.is_none()
            && a.constructor.owner.as_ref().and_then(|owner| {
                source
                    .type_definition(owner)
                    .and_then(|d| d.origin.as_ref())
                    .map(|origin| origin.name.as_str())
                    .or_else(|| owner.definition_name())
            }) == Some("System.Runtime.CompilerServices.UnionAttribute")
    });
    result[11] = visible(source, ty);
    result[12] = !matches!(ty, Type::ArrayRef(_));
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
                        | "neoCLR.Runtime.TypeDisplayName"
                        | "neoCLR.Runtime.TypeArgumentCount"
                        | "neoCLR.Runtime.TypeMetadataToken"
                        | "neoCLR.Runtime.TypeArgument"
                        | "neoCLR.Runtime.TypeShape"
                        | "neoCLR.Runtime.ReflectionAssignable"
                        | "neoCLR.Runtime.ReflectionSubclass"
                )
            )
        })
        .cloned()
        .collect();
    if queries.is_empty() {
        return Ok(vec![]);
    }
    let mut open_tokens = Vec::new();
    for op in input.functions.iter().flat_map(|f| &f.body) {
        if let Op::LoadMetadataToken(index) = op {
            if !open_tokens.contains(index) {
                open_tokens.push(*index);
            }
        }
    }
    // An open definition's argument handles are descriptive, never executable
    // specializations. Close this inventory before binding shape/name queries.
    if queries
        .iter()
        .any(|r| r["name"] == "neoCLR.Runtime.TypeArgument")
    {
        let mut next = 0;
        while next < open_tokens.len() {
            let descriptor = neoclr::native_metadata::descriptor(input, open_tokens[next])
                .map_err(|e| e.to_string())?
                .clone();
            for parameter in &descriptor.generic_arguments {
                let neoclr::TypeIdentity::GenericParameter { definition, index } =
                    &parameter.identity
                else {
                    return Err("open definition has a non-parameter argument".into());
                };
                let Op::LoadMetadataToken(index) =
                    neoclr::native_metadata::retain_definition_token(
                        input,
                        source,
                        definition,
                        Some(*index),
                    )
                    .map_err(|e| e.to_string())?
                else {
                    return Err("expected descriptive argument token".into());
                };
                if !open_tokens.contains(&index) {
                    open_tokens.push(index);
                }
            }
            next += 1;
        }
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
    // Argument queries produce more tokens; close this selected inventory before
    // generating any name/shape/argument dispatch. Unselected library bodies do
    // not contribute to retention.
    let arguments_needed = queries
        .iter()
        .any(|r| r["name"] == "neoCLR.Runtime.TypeArgument");
    let mut argument_rows = vec![];
    let mut next = 0;
    while next < tokens.len() {
        let token = tokens[next].clone();
        let mut arguments = vec![];
        if arguments_needed {
            let semantic = semantic_token(input, report, &token)?;
            for argument in neoclr::native_metadata::generic_arguments(source, &semantic)
                .map_err(|e| e.to_string())?
            {
                let lowered = native_token(input, report, &argument)?;
                if !tokens.contains(&lowered) {
                    tokens.push(lowered.clone());
                }
                arguments.push(lowered);
            }
        }
        argument_rows.push(arguments);
        next += 1;
    }
    let mut descriptors = vec![];
    let mut semantic_types = vec![];
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
        } else if matches!(token, Type::Function(_)) {
            let semantic = semantic_token(input, report, token)?;
            let neoclr::Value::String(name) =
                neoclr::native_metadata::display_name(source, &semantic)
                    .map_err(|e| e.to_string())?
            else {
                return Err("function display name was not a string".into());
            };
            (name.to_string(), 0, semantic)
        } else if matches!(token, Type::ArrayRef(_)) {
            let semantic = semantic_token(input, report, token)?;
            let declaration = source.type_definition(&semantic);
            let name = if let Some(origin) = declaration.and_then(|d| d.origin.as_ref()) {
                origin.name.clone()
            } else {
                signature_name(&semantic)?
            };
            (name, usize::from(declaration.is_some()), semantic)
        } else {
            return Err(format!("native type queries currently require primitive, closed nominal or vector tokens; found {token:?}").into());
        };
        let shapes = if queries
            .iter()
            .any(|r| r["name"] == "neoCLR.Runtime.TypeShape")
        {
            shape(source, &semantic_type)?
        } else {
            [false; 20]
        };
        let metadata_token = if queries
            .iter()
            .any(|r| r["name"] == "neoCLR.Runtime.TypeMetadataToken")
        {
            neoclr::native_metadata::type_token(source, &semantic_type)
                .map_err(|e| e.to_string())?
        } else {
            0
        };
        semantic_types.push(semantic_type);
        descriptors.push((token.clone(), name, count, shapes, metadata_token));
    }
    let mut bindings = vec![];
    for row in queries {
        let index = row["compiledIndex"].as_u64().ok_or("missing query index")? as usize;
        let assignable = row["name"] == "neoCLR.Runtime.ReflectionAssignable";
        let subclass = row["name"] == "neoCLR.Runtime.ReflectionSubclass";
        let relationship = assignable || subclass;
        let mut related = vec![Vec::new(); semantic_types.len()];
        if relationship {
            for (i, source_type) in semantic_types.iter().enumerate() {
                for (j, target_type) in semantic_types.iter().enumerate() {
                    let matches = if assignable {
                        neoclr::native_metadata::assignable(source, source_type, target_type)
                    } else {
                        neoclr::native_metadata::subclass(source, source_type, target_type)
                    }
                    .map_err(|e| e.to_string())?;
                    if matches {
                        related[i].push(j);
                    }
                }
            }
        }
        let prefix = open_query_prefix(input, source, &open_tokens, row["name"].as_str().unwrap())?;
        let f = &mut input.functions[index];
        let metadata_tokens = row["name"] == "neoCLR.Runtime.TypeMetadataToken";
        let names = row["name"] == "neoCLR.Runtime.TypeName";
        let display = row["name"] == "neoCLR.Runtime.TypeDisplayName";
        let shapes = row["name"] == "neoCLR.Runtime.TypeShape";
        let arguments = row["name"] == "neoCLR.Runtime.TypeArgument";
        let result = if names || display {
            Type::String
        } else if arguments {
            Type::RuntimeTypeHandle
        } else if shapes || relationship {
            Type::Boolean
        } else {
            Type::Int32
        };
        let parameters = if relationship {
            vec![Type::RuntimeTypeHandle, Type::RuntimeTypeHandle]
        } else if shapes || arguments || display {
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
            return Err("native descriptor query requires exact reserved TypeName/TypeDisplayName/TypeArgumentCount/TypeArgument/TypeShape/TypeMetadataToken/ReflectionAssignable/ReflectionSubclass InternalCall contract".into());
        }
        // Validate the selector once, then emit only true flags per descriptor.
        // Emitting all fourteen Boolean branches exceeded the bounded backend's
        // method budget when HTTP and JSON retained more than ninety types.
        let mut body = if shapes {
            vec![
                Op::Arg(1),
                Op::Int(0),
                Op::Less,
                Op::BranchFalse(5),
                Op::Fault("unknown type shape query".into()),
                Op::Arg(1),
                Op::Int(20),
                Op::Less,
                Op::BranchTrue(10),
                Op::Fault("unknown type shape query".into()),
            ]
        } else {
            vec![]
        };
        for (descriptor, (token, name, count, flags, metadata_token)) in
            descriptors.iter().enumerate()
        {
            let branch = body.len() + 3;
            body.extend([
                Op::Arg(0),
                Op::LoadTypeToken(token.clone()),
                Op::Equal,
                Op::BranchFalse(0),
            ]);
            if relationship {
                for target in &related[descriptor] {
                    let next = body.len() + 6;
                    body.extend([
                        Op::Arg(1),
                        Op::LoadTypeToken(descriptors[*target].0.clone()),
                        Op::Equal,
                        Op::BranchFalse(next),
                        Op::Bool(true),
                        Op::Return,
                    ]);
                }
                body.extend([Op::Bool(false), Op::Return]);
            } else if arguments {
                for (index, token) in argument_rows[descriptor].iter().enumerate() {
                    let next = body.len() + 6;
                    body.extend([
                        Op::Arg(1),
                        Op::Int(index as i32),
                        Op::Equal,
                        Op::BranchFalse(next),
                        Op::LoadTypeToken(token.clone()),
                        Op::Return,
                    ]);
                }
                body.push(Op::Fault("generic argument index out of range".into()));
            } else if display {
                for selector in 0..2 {
                    let name = neoclr::native_metadata::type_display_name(
                        source,
                        &semantic_types[descriptor],
                        selector,
                    )
                    .map_err(|e| e.to_string())?;
                    let next = body.len() + 6;
                    body.extend([
                        Op::Arg(1),
                        Op::Int(selector),
                        Op::Equal,
                        Op::BranchFalse(next),
                        Op::String(name),
                        Op::Return,
                    ]);
                }
                body.push(Op::Fault("unknown type name query".into()));
            } else if shapes {
                for (selector, value) in flags.iter().enumerate() {
                    if !value {
                        continue;
                    }
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
                body.extend([Op::Bool(false), Op::Return]);
            } else {
                body.extend([
                    if names {
                        Op::String(name.clone())
                    } else if metadata_tokens {
                        Op::Int(*metadata_token)
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
        if !prefix.is_empty() {
            let offset = prefix.len();
            for op in &mut body {
                match op {
                    Op::Branch(target) | Op::BranchTrue(target) | Op::BranchFalse(target) => {
                        *target += offset
                    }
                    _ => (),
                }
            }
            let mut combined = prefix;
            combined.extend(body);
            body = combined;
        }
        f.impl_flags = 0;
        f.body = body;
        bindings.push(json!({"definition":row["definition"],"name":row["name"],"compiledIndex":index,
            "implementation":if assignable { "type-assignable-closed-v1" } else if subclass { "type-subclass-closed-v1" } else if metadata_tokens { "type-metadata-token-closed-v1" } else if names { "type-name-closed-v1" } else if shapes { "type-shape-closed-v1" } else if arguments { "type-argument-closed-v1" } else { "type-argument-count-closed-v1" },
            "descriptorCount":descriptors.len(),"openDescriptorCount":open_tokens.len(),"policy":"closed tokens and retained descriptive definitions/parameters; semantic source names; no accessor reachability"}));
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
    fn catalogue_retains_generic_declarations_across_distinct_constructions() {
        use neoclr::metadata::{ConstraintKind, GenericConstraint, Variance};
        let mut source = neoclr::assemble(
            ".module GenericMetadata\n.interface Producer<T>\n.end\n.function Identity() -> Int32\nldc.i4 0\nret\n.end\n",
        ).unwrap();
        source.types[0].generic_variance = vec![Variance::Covariant];
        source.types[0].generic_constraints = vec![GenericConstraint {
            parameter: 0,
            kind: ConstraintKind::ReferenceType,
        }];
        let method = &mut source.functions[0];
        method.owner = Some(Type::Constructed {
            definition: "Producer".into(),
            arguments: vec![Type::TypeParameter(0)],
        });
        method.generic_parameters = vec![Some("U".into())];
        method.generic_constraints = vec![
            GenericConstraint {
                parameter: 0,
                kind: ConstraintKind::TypeBound(Type::TypeParameter(0)),
            },
            GenericConstraint {
                parameter: 0,
                kind: ConstraintKind::DefaultConstructor,
            },
        ];
        let definitions = vec![TypeDefId {
            module: "GenericMetadata".into(),
            revision: Some("r1".into()),
            index: 0,
        }];
        let methods = vec![MemberId {
            module: "GenericMetadata".into(),
            revision: Some("r1".into()),
            index: 0,
        }];
        let report = json!({"types":[
            {"sourceIndex":0,"compiledIndex":0,"typeArguments":[Type::String]},
            {"sourceIndex":0,"compiledIndex":1,"typeArguments":[Type::ArrayRef(Box::new(Type::Int32))]}
        ]});
        let result = catalogue(&source, &definitions, &methods, &report).unwrap();
        let rows = result["types"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_ne!(rows[0]["typeArguments"], rows[1]["typeArguments"]);
        assert_eq!(rows[0]["definition"], rows[1]["definition"]);
        for row in rows {
            let declaration: neoclr::metadata::TypeDef =
                serde_json::from_value(row["declaration"].clone()).unwrap();
            assert_eq!(declaration.generic_parameters, vec![Some("T".into())]);
            assert_eq!(declaration.generic_variance, vec![Variance::Covariant]);
            assert_eq!(
                declaration.generic_constraints,
                source.types[0].generic_constraints
            );
            let declared = &row["declaredMethods"][0];
            assert_eq!(declared["genericParameters"], json!(["U"]));
            let constraints: Vec<GenericConstraint> =
                serde_json::from_value(declared["genericConstraints"].clone()).unwrap();
            assert_eq!(constraints, source.functions[0].generic_constraints);
        }
    }

    #[test]
    fn relationship_queries_bind_identity_and_reject_inexact_contracts() {
        for (name, expected) in [
            ("ReflectionAssignable", true),
            ("ReflectionSubclass", false),
        ] {
            let (mut input, mut report) = query();
            input.functions[0].name = format!("neoCLR.Runtime.{name}");
            input.functions[0].parameters = vec![Type::RuntimeTypeHandle; 2];
            input.functions[0].returns = Type::Boolean;
            report["functions"][0]["name"] = json!(input.functions[0].name);
            let mut source = input.clone();
            source.types[0].name = "Original".into();
            let mut invalid = input.clone();
            invalid.functions[0].parameters[1] = Type::Int32;
            assert!(bind_queries(&mut invalid, &source, &report).is_err());
            bind_queries(&mut input, &source, &report).unwrap();
            assert_eq!(input.functions[0].impl_flags, 0);
            assert_eq!(
                input.functions[0]
                    .body
                    .iter()
                    .any(|op| matches!(op, Op::Bool(true))),
                expected
            );
            assert!(
                input.functions[0]
                    .body
                    .iter()
                    .any(|op| matches!(op, Op::Bool(false)))
            );
        }
    }

    #[test]
    fn function_generic_arguments_preserve_closed_identity_and_shape() {
        let (input, report) = query();
        let semantic = Type::Function(Box::new(neoclr::metadata::FunctionType {
            parameters: vec![Type::Named("Original".into()), Type::Int32],
            returns: Type::Void,
            no_result: true,
            out_parameters: vec![],
            out_when_true: vec![],
        }));
        let native = native_token(&input, &report, &semantic).unwrap();
        assert_ne!(native, semantic);
        assert_eq!(semantic_token(&input, &report, &native).unwrap(), semantic);
        let facts = shape(&input, &native).unwrap();
        assert!(facts[13]);
        assert!(!facts[7]);
        assert!(!facts[12]);
        let open = Type::Function(Box::new(neoclr::metadata::FunctionType {
            parameters: vec![Type::MethodTypeParameter(0)],
            returns: Type::Void,
            no_result: false,
            out_parameters: vec![],
            out_when_true: vec![],
        }));
        assert!(native_token(&input, &report, &open).is_err());
    }

    #[test]
    fn metadata_tokens_preserve_source_rows_instead_of_native_type_indices() {
        let (mut source, mut report) = query();
        source.types[0].name = "Original".into();
        source.types[0].origin = Some(serde_json::from_value(json!({
            "assembly":"OriginalAssembly","module":"Original.dll","name":"Original","token":0x0200002a
        })).unwrap());
        source.functions[0].name = "neoCLR.Runtime.TypeMetadataToken".into();
        source.functions[0].returns = Type::Int32;
        source.functions[1].body[0] = Op::LoadTypeToken(Type::Named("Original".into()));
        report["functions"][0]["name"] = json!("neoCLR.Runtime.TypeMetadataToken");
        let mut input = source.clone();
        bind_queries(&mut input, &source, &report).unwrap();
        assert!(
            input.functions[0]
                .body
                .iter()
                .any(|op| matches!(op, Op::Int(0x0200002a)))
        );
        input = source.clone();
        input.functions[0].returns = Type::RuntimeTypeHandle;
        assert!(bind_queries(&mut input, &source, &report).is_err());
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
        input.functions[1].body[0] = Op::LoadTypeToken(Type::ByRef(Box::new(Type::Int32)));
        assert!(
            bind_queries(&mut input, &source, &report)
                .unwrap_err()
                .to_string()
                .contains("primitive, closed nominal or vector")
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
        assert!(!flags[14], "nominal unions are not structural unions");
        source.types[0].origin = Some(serde_json::from_value(json!({"assembly":"Source","module":"Source.dll","name":"Outer/Choice","token":33554433})).unwrap());
        assert!(!shape(&source, &ty).unwrap()[11]);
        source.types[0].origin.as_mut().unwrap().publicly_visible = Some(true);
        assert!(shape(&source, &ty).unwrap()[11]);
        source.types[0].custom_attributes[0].target_token = Some(33554433);
        assert!(!shape(&source, &ty).unwrap()[10]);
        let mut marker = neoclr::assemble(".module Marker\n.type class InternalMarker\n.end\n")
            .unwrap()
            .types
            .remove(0);
        marker.origin = Some(serde_json::from_value(json!({"assembly":"Runtime","module":"Runtime.dll","name":"System.Runtime.CompilerServices.UnionAttribute","token":33554433})).unwrap());
        source.types.push(marker);
        source.types[0].custom_attributes[0].target_token = None;
        source.types[0].custom_attributes[0].constructor.owner =
            Some(Type::from_name("InternalMarker"));
        assert!(shape(&source, &ty).unwrap()[10]);
        assert!(!shape(&source, &ty).unwrap()[14]);
    }
}

/// Map private heap tags back to semantic tokens. Array backing records are not
/// source array types and must never escape as GetType results.
pub fn bind_object_type(input: &mut neoclr::Module, report: &mut Value) -> Result<(), Error> {
    use neoclr::metadata::{Instruction as Op, Representation};
    let rows: Vec<_> = report["functions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|r| r["name"] == "neoCLR.Runtime.ObjectTypeHandle")
        .cloned()
        .collect();
    if rows.is_empty() {
        return Ok(());
    }
    let root = report["objectBaseProjection"]["compiledIndex"]
        .as_u64()
        .ok_or("ObjectTypeHandle requires the verified Object base")? as usize;
    let object = Type::Named(input.types[root].name.clone());
    let backings: Vec<_> = report["referenceArrayBackingProjections"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| r["compiledIndex"].as_u64())
        .collect();
    let mut targets = vec![];
    for row in report["sourceMetadata"]["types"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let index = row["compiledTypeIndex"]
            .as_u64()
            .ok_or("missing object type index")? as usize;
        let t = &input.types[index];
        if t.is_reference_type
            && !t.is_abstract
            && t.representation == Representation::Record
            && !backings.contains(&(index as u64))
            && report["arrayBackingProjection"]["compiledIndex"].as_u64() != Some(index as u64)
        {
            targets.push(json!({"typeCompiledIndex":index,"token":Type::Named(t.name.clone())}));
        }
    }
    let vector_tokens: Vec<_> = input
        .functions
        .iter()
        .flat_map(|f| &f.body)
        .filter_map(|op| {
            if let Op::LoadTypeToken(t @ Type::ArrayRef(_)) = op {
                Some(t.clone())
            } else {
                None
            }
        })
        .collect();
    for index in backings
        .iter()
        .copied()
        .chain(report["arrayBackingProjection"]["compiledIndex"].as_u64())
    {
        let index = index as usize;
        if let Some(field) = input.types[index].fields.first() {
            if vector_tokens.contains(&field.ty) {
                targets.push(json!({"typeCompiledIndex":index,"token":field.ty}));
            }
        }
    }
    for (key, primitive) in [
        ("int64Boxes", Some(Type::Int64)),
        ("int32Boxes", Some(Type::Int32)),
        ("booleanBoxes", Some(Type::Boolean)),
        ("emptyRecordBoxes", None),
    ] {
        for row in report[key].as_array().into_iter().flatten() {
            let index = row["typeCompiledIndex"].as_u64().ok_or("missing box tag")? as usize;
            let token = primitive
                .clone()
                .unwrap_or_else(|| Type::Named(input.types[index].name.clone()));
            targets.push(json!({"typeCompiledIndex":index,"token":token}));
        }
    }
    let mut function_tokens = vec![];
    for function in &input.functions {
        for op in &function.body {
            if let Op::BindFunction { function_type, .. } = op {
                if !function_tokens.contains(function_type) {
                    function_tokens.push(function_type.clone());
                }
            }
        }
    }
    for row in rows {
        let index = row["compiledIndex"]
            .as_u64()
            .ok_or("missing object query index")? as usize;
        let f = &mut input.functions[index];
        if f.name != "neoCLR.Runtime.ObjectTypeHandle"
            || f.owner.is_some()
            || f.instance
            || f.parameters != vec![object.clone()]
            || f.returns != Type::RuntimeTypeHandle
            || f.no_result
            || f.impl_flags != 0x1000
            || f.pinvoke.is_some()
            || !f.body.is_empty()
            || !f.locals.is_empty()
            || f.receiver_byref
            || f.receiver_readonly
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
            return Err(
                "ObjectTypeHandle requires the exact reserved InternalCall contract".into(),
            );
        }
        f.impl_flags = 0;
        // Native body uses these diagnostic slots and enumerated token producers.
        f.body = vec![
            Op::Fault("GetType requires a non-null instance".into()),
            Op::Fault("native GetType does not yet admit this object shape".into()),
            Op::LoadTypeToken(Type::String),
            Op::Pop,
        ];
        for target in &targets {
            f.body.extend([
                Op::LoadTypeToken(serde_json::from_value(target["token"].clone())?),
                Op::Pop,
            ]);
        }
        for token in &function_tokens {
            f.body.extend([Op::LoadTypeToken(token.clone()), Op::Pop]);
        }
        f.body
            .push(Op::Fault("unreachable native object query body".into()));
        report["nativeBindings"].as_array_mut().unwrap().push(json!({"compiledIndex":index,
            "name":row["name"],"implementation":"object-type-closed-v1", "targets":targets, "functionTokens":function_tokens,
            "policy":"source class/box tags, selected Function bindings, intrinsic String and retained vector tokens; other compounds fail closed"}));
    }
    Ok(())
}

#[cfg(test)]
mod object_tests {
    use super::*;
    #[test]
    fn object_query_retains_exact_function_shapes_without_nominal_rows() {
        let mut source = neoclr::assemble(".module Test\n.type class abstract System.Object\n.end\n.function neoCLR.Runtime.ObjectTypeHandle(System.Object) -> RuntimeTypeHandle\n.methodimpl InternalCall\n.end\n.function Identity(Int32) -> Int32\nldarg 0\nret\n.end\n.function Main() -> Int32\nfunction.bind fn<Int32,Int32> = Identity(Int32)\npop\nfunction.bind fn<Int32,Int32> = Identity(Int32)\npop\nldc.i4 42\nret\n.end").unwrap();
        let shape = neoclr::assembler::parse_type("fn<Int32,Int32>").unwrap();
        let mut report = json!({"objectBaseProjection":{"compiledIndex":0},
            "nativeBindings":[], "functions":[{"compiledIndex":0,"name":"neoCLR.Runtime.ObjectTypeHandle"}],
            "sourceMetadata":{"types":[]}});
        bind_object_type(&mut source, &mut report).unwrap();
        assert_eq!(report["nativeBindings"][0]["functionTokens"], json!([shape.clone()]));
        assert_eq!(report["nativeBindings"][0]["targets"], json!([]));
        assert!(source.functions[0].body.iter().any(|op| matches!(op,
            neoclr::metadata::Instruction::LoadTypeToken(t) if t == &shape)));
    }

    #[test]
    fn object_query_requires_exact_contract_and_excludes_private_backings() {
        let original = neoclr::assemble(".module Test\n.type class abstract System.Object\n.end\n.type class Model\n.end\n.type class ArrayStorage\n.end\n.function neoCLR.Runtime.ObjectTypeHandle(System.Object) -> RuntimeTypeHandle\n.methodimpl InternalCall\n.end\n").unwrap();
        let report = json!({"objectBaseProjection":{"compiledIndex":0},
            "referenceArrayBackingProjections":[{"compiledIndex":2}],"nativeBindings":[],
            "functions":[{"compiledIndex":0,"name":"neoCLR.Runtime.ObjectTypeHandle"}],
            "sourceMetadata":{"types":[{"compiledTypeIndex":0},{"compiledTypeIndex":1},{"compiledTypeIndex":2}]}});
        let mut input = original.clone();
        let mut valid = report.clone();
        bind_object_type(&mut input, &mut valid).unwrap();
        assert_eq!(
            valid["nativeBindings"][0]["targets"],
            json!([{"typeCompiledIndex":1,"token":Type::Named("Model".into())}])
        );
        for change in 0..4 {
            let mut input = original.clone();
            let f = &mut input.functions[0];
            match change {
                0 => f.returns = Type::Int32,
                1 => f.impl_flags = 0,
                2 => f.locals.push(Type::Int32),
                _ => f
                    .body
                    .push(neoclr::metadata::Instruction::Fault("body".into())),
            }
            assert!(
                bind_object_type(&mut input, &mut report.clone())
                    .unwrap_err()
                    .to_string()
                    .contains("exact reserved")
            );
        }
    }
}

// Prefix source-owned open descriptor cases ahead of the established closed-token
// dispatcher. Unknown handles still reach its checked failure path.
fn open_query_prefix(
    input: &mut neoclr::Module,
    source: &neoclr::Module,
    tokens: &[u32],
    query: &str,
) -> Result<Vec<neoclr::metadata::Instruction>, Error> {
    use neoclr::metadata::Instruction as Op;
    let mut body = Vec::new();
    if query == "neoCLR.Runtime.TypeShape" && !tokens.is_empty() {
        body.extend([
            Op::Arg(1),
            Op::Int(0),
            Op::Less,
            Op::BranchFalse(5),
            Op::Fault("unknown type shape query".into()),
            Op::Arg(1),
            Op::Int(20),
            Op::Less,
            Op::BranchTrue(10),
            Op::Fault("unknown type shape query".into()),
        ]);
    }
    let relationship = matches!(
        query,
        "neoCLR.Runtime.ReflectionAssignable" | "neoCLR.Runtime.ReflectionSubclass"
    );
    for index in tokens {
        let descriptor = neoclr::native_metadata::descriptor(input, *index)
            .map_err(|e| e.to_string())?
            .clone();
        // Do not silently return false for a relationship whose open substitution
        // semantics have not been admitted, including an open target of a closed source.
        if relationship {
            for argument in [0, 1] {
                let skip = body.len() + 3;
                body.extend([
                    Op::Arg(argument),
                    Op::LoadMetadataToken(*index),
                    Op::Equal,
                    Op::BranchFalse(0),
                    Op::Fault("native open generic relationship queries are not supported".into()),
                ]);
                body[skip] = Op::BranchFalse(body.len());
            }
            continue;
        }
        let skip = body.len() + 3;
        body.extend([
            Op::Arg(0),
            Op::LoadMetadataToken(*index),
            Op::Equal,
            Op::BranchFalse(0),
        ]);
        match query {
            "neoCLR.Runtime.TypeName" => body.extend([Op::String(descriptor.name), Op::Return]),
            "neoCLR.Runtime.TypeDisplayName" => {
                for selector in 0..2 {
                    let name = neoclr::native_metadata::descriptor_display_name(
                        source,
                        &descriptor,
                        selector,
                    )
                    .map_err(|e| e.to_string())?;
                    let next = body.len() + 6;
                    body.extend([
                        Op::Arg(1),
                        Op::Int(selector),
                        Op::Equal,
                        Op::BranchFalse(next),
                        Op::String(name),
                        Op::Return,
                    ]);
                }
                body.push(Op::Fault("unknown type name query".into()));
            }
            "neoCLR.Runtime.TypeArgumentCount" => body.extend([
                Op::Int(descriptor.generic_arguments.len() as i32),
                Op::Return,
            ]),
            "neoCLR.Runtime.TypeMetadataToken" => body.extend([
                Op::Int(
                    neoclr::native_metadata::descriptor_metadata_token(source, &descriptor)
                        .map_err(|e| e.to_string())?,
                ),
                Op::Return,
            ]),
            "neoCLR.Runtime.TypeArgument" => {
                for (ordinal, parameter) in descriptor.generic_arguments.iter().enumerate() {
                    let neoclr::TypeIdentity::GenericParameter { definition, index } =
                        &parameter.identity
                    else {
                        return Err("open argument lacks declaring identity".into());
                    };
                    let token = neoclr::native_metadata::retain_definition_token(
                        input,
                        source,
                        definition,
                        Some(*index),
                    )
                    .map_err(|e| e.to_string())?;
                    let next = body.len() + 6;
                    body.extend([
                        Op::Arg(1),
                        Op::Int(ordinal as i32),
                        Op::Equal,
                        Op::BranchFalse(next),
                        token,
                        Op::Return,
                    ]);
                }
                body.push(Op::Fault("generic argument index out of range".into()));
            }
            "neoCLR.Runtime.TypeShape" => {
                for selector in 0..20 {
                    let fact =
                        neoclr::native_metadata::descriptor_shape(source, &descriptor, selector)
                            .map_err(|e| e.to_string())?;
                    if fact {
                        let next = body.len() + 6;
                        body.extend([
                            Op::Arg(1),
                            Op::Int(selector),
                            Op::Equal,
                            Op::BranchFalse(next),
                            Op::Bool(true),
                            Op::Return,
                        ]);
                    }
                }
                body.extend([Op::Bool(false), Op::Return]);
            }
            _ => return Err("unsupported open descriptor query".into()),
        }
        body[skip] = Op::BranchFalse(body.len());
    }
    Ok(body)
}

#[cfg(test)]
mod open_query_tests {
    use super::*;
    use neoclr::metadata::{FunctionRef, Instruction as Op};

    fn fixture() -> (neoclr::Module, neoclr::Module, Value) {
        let source = neoclr::assemble(".module OpenQueries\n.entry Main\n.interface Example.Producer<T>\n.variance out\n.end\n.interface Consumer<T>\n.variance in\n.end\n.interface Fixed<T>\n.end\n.function neoCLR.Runtime.TypeShape(RuntimeTypeHandle,Int32) -> Boolean\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.TypeArgument(RuntimeTypeHandle,Int32) -> RuntimeTypeHandle\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.TypeArgumentCount(RuntimeTypeHandle) -> Int32\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.TypeName(RuntimeTypeHandle) -> String\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.TypeMetadataToken(RuntimeTypeHandle) -> Int32\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.TypeDisplayName(RuntimeTypeHandle,Int32) -> String\n.methodimpl InternalCall\n.end\n.function Main() -> Int32\nldc.i4 42\nret\n.end\n").unwrap();
        let mut input = source.clone();
        let definition = source.types[0].definition.as_ref().unwrap();
        let open =
            neoclr::native_metadata::retain_definition_token(&mut input, &source, definition, None)
                .unwrap();
        let mut parameters = vec![];
        for ty in &source.types {
            parameters.push(
                neoclr::native_metadata::retain_definition_token(
                    &mut input,
                    &source,
                    ty.definition.as_ref().unwrap(),
                    Some(0),
                )
                .unwrap(),
            );
        }
        let call = |suffix: &str| {
            let f = source
                .functions
                .iter()
                .find(|f| f.name == format!("neoCLR.Runtime.{suffix}"))
                .unwrap();
            Op::Call(FunctionRef {
                definition: f.definition.clone(),
                name: f.name.clone(),
                owner: None,
                instance: false,
                parameters: f.parameters.clone(),
                generic_arguments: vec![],
            })
        };
        let mut body = vec![];
        let mut failures = vec![];
        let mut check = |code: Vec<Op>, expected: Op| {
            body.extend(code);
            body.extend([expected, Op::Equal]);
            failures.push(body.len());
            body.push(Op::BranchFalse(0));
        };
        check(
            vec![open.clone(), Op::Int(0), call("TypeDisplayName")],
            Op::String("Example.Producer".into()),
        );
        check(
            vec![open.clone(), Op::Int(1), call("TypeDisplayName")],
            Op::String("Example".into()),
        );
        check(
            vec![parameters[0].clone(), Op::Int(0), call("TypeDisplayName")],
            Op::String("T".into()),
        );
        check(vec![open.clone(), call("TypeArgumentCount")], Op::Int(1));
        check(
            vec![open.clone(), Op::Int(0), call("TypeArgument")],
            parameters[0].clone(),
        );
        check(
            vec![open.clone(), call("TypeName")],
            Op::String("Example.Producer".into()),
        );
        check(
            vec![open.clone(), call("TypeMetadataToken")],
            Op::Int(0x02000001),
        );
        for (index, parameter) in parameters.iter().enumerate() {
            check(
                vec![parameter.clone(), call("TypeArgumentCount")],
                Op::Int(0),
            );
            check(
                vec![parameter.clone(), call("TypeName")],
                Op::String("T".into()),
            );
            check(
                vec![parameter.clone(), call("TypeMetadataToken")],
                Op::Int(0),
            );
            for (selector, expected) in [(17, true), (18, index == 0), (19, index == 1)] {
                check(
                    vec![parameter.clone(), Op::Int(selector), call("TypeShape")],
                    Op::Bool(expected),
                );
            }
        }
        body.extend([Op::Int(42), Op::Return]);
        let fail = body.len();
        body.extend([Op::Int(1), Op::Return]);
        for i in failures {
            body[i] = Op::BranchFalse(fail);
        }
        input.functions.last_mut().unwrap().body = body;
        let report = json!({"functions":source.functions.iter().enumerate().take(6).map(|(i,f)| json!({"compiledIndex":i,"name":f.name,"definition":f.definition})).collect::<Vec<_>>()});
        (input, source, report)
    }

    // These have become ordinary generated bodies. Give the test copies local
    // names so interpreter loading does not collide with bundled System facades.
    fn localize_bound_queries(input: &mut neoclr::Module) {
        for function in &mut input.functions {
            function.name = function.name.replace("neoCLR.Runtime.", "BoundQuery.");
            for op in &mut function.body {
                if let Op::Call(target) = op {
                    target.name = target.name.replace("neoCLR.Runtime.", "BoundQuery.");
                }
            }
        }
    }

    #[test]
    fn open_queries_preserve_source_metadata_without_executable_types() {
        let (mut input, source, report) = fixture();
        bind_queries(&mut input, &source, &report).unwrap();
        localize_bound_queries(&mut input);
        let (selected, _) = crate::selection::prepare(&input, "Main").unwrap();
        assert!(selected.types.is_empty());
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
        assert!(
            !crate::compiler::compile(&selected, "Main", false, None)
                .unwrap()
                .is_empty()
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        crate::linking::descriptor_token_tests::assert_native_result(&selected, 42);
    }

    #[test]
    fn open_argument_results_extend_the_inventory_and_coexist_with_closed_tokens() {
        let (mut input, mut source, mut report) = fixture();
        let mut plain = neoclr::assemble(
            ".module PlainSource\n.type Example.Plain\n.field Value Int32\n.end\n",
        )
        .unwrap()
        .types
        .remove(0);
        plain.definition = Some(TypeDefId {
            module: source.name.clone(),
            revision: None,
            index: 3,
        });
        source.types.push(plain.clone());
        input.types.push(plain);
        report["sourceMetadata"] =
            json!({"types":[{"compiledTypeIndex":3,"name":"Example.Plain","typeArguments":[]}]});
        let token = neoclr::native_metadata::retain_definition_token(
            &mut input,
            &source,
            source.types[0].definition.as_ref().unwrap(),
            None,
        )
        .unwrap();
        let call = |name: &str| {
            let f = source
                .functions
                .iter()
                .find(|f| f.name == format!("neoCLR.Runtime.{name}"))
                .unwrap();
            Op::Call(FunctionRef {
                definition: f.definition.clone(),
                name: f.name.clone(),
                owner: None,
                instance: false,
                parameters: f.parameters.clone(),
                generic_arguments: vec![],
            })
        };
        // The parameter handle appears only as a query result, not as an input token.
        input.functions.last_mut().unwrap().body = vec![
            token,
            Op::Int(0),
            call("TypeArgument"),
            Op::Int(18),
            call("TypeShape"),
            Op::BranchFalse(20),
            Op::LoadTypeToken(Type::from_name("Example.Plain")),
            Op::Int(12),
            call("TypeShape"),
            Op::BranchFalse(20),
            Op::LoadTypeToken(Type::from_name("Example.Plain")),
            Op::Int(1),
            call("TypeDisplayName"),
            Op::String("Example".into()),
            Op::Equal,
            Op::BranchFalse(20),
            Op::Int(42),
            Op::Return,
            Op::Int(1),
            Op::Return,
            Op::Int(2),
            Op::Return,
        ];
        bind_queries(&mut input, &source, &report).unwrap();
        localize_bound_queries(&mut input);
        let (selected, _) = crate::selection::prepare(&input, "Main").unwrap();
        let program = neoclr::LoadedProgram::new(&selected).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(neoclr::Limits::default()).unwrap().value,
            neoclr::Value::Int32(42)
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        crate::linking::descriptor_token_tests::assert_native_result(&selected, 42);
    }

    #[test]
    fn open_queries_preserve_invalid_selector_and_argument_faults() {
        for (name, selector, expected) in [
            ("TypeShape", 20, "unknown type shape query"),
            ("TypeDisplayName", -1, "unknown type name query"),
            ("TypeDisplayName", 2, "unknown type name query"),
            ("TypeArgument", -1, "generic argument index out of range"),
        ] {
            let (mut input, source, report) = fixture();
            let token = neoclr::native_metadata::retain_definition_token(
                &mut input,
                &source,
                source.types[0].definition.as_ref().unwrap(),
                None,
            )
            .unwrap();
            let f = input
                .functions
                .iter()
                .find(|f| f.name == format!("neoCLR.Runtime.{name}"))
                .unwrap();
            let call = FunctionRef {
                definition: f.definition.clone(),
                name: f.name.clone(),
                owner: None,
                instance: false,
                parameters: f.parameters.clone(),
                generic_arguments: vec![],
            };
            input.functions.last_mut().unwrap().body = vec![
                token,
                Op::Int(selector),
                Op::Call(call),
                Op::Pop,
                Op::Int(42),
                Op::Return,
            ];
            bind_queries(&mut input, &source, &report).unwrap();
            localize_bound_queries(&mut input);
            let program = neoclr::LoadedProgram::new(&input).unwrap();
            program.verify().unwrap();
            assert!(
                program
                    .run(neoclr::Limits::default())
                    .unwrap_err()
                    .to_string()
                    .contains(expected)
            );
        }
    }
}
