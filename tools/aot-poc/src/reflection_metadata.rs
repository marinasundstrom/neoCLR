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
