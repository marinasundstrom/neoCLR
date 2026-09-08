//! Metadata-only queries. Results own snapshots and never retain guest objects.
use crate::{
    Fault, Limits, Module, TypeDescriptor, TypeIdentity, Value,
    metadata::{Function, Representation, Type, Visibility},
};

#[derive(Clone, Copy)]
pub(crate) enum Query {
    Fields,
    Methods,
    Properties,
    Interfaces,
    GenericArguments,
    ElementType,
    BaseType,
    Shape,
    DisplayName,
}

impl Query {
    pub(crate) fn binding(name: &str) -> Option<(Self, bool, Type)> {
        let (query, integer, result) = match name {
            "neoCLR.Runtime.TypeFields" => (Self::Fields, true, "System.Reflection.FieldInfo[]"),
            "neoCLR.Runtime.TypeMethods" => (Self::Methods, true, "System.Reflection.MethodInfo[]"),
            "neoCLR.Runtime.TypeProperties" => {
                (Self::Properties, true, "System.Reflection.PropertyInfo[]")
            }
            "neoCLR.Runtime.TypeInterfaces" => (Self::Interfaces, false, "System.Type[]"),
            "neoCLR.Runtime.TypeGenericArguments" => {
                (Self::GenericArguments, false, "System.Type[]")
            }
            "neoCLR.Runtime.TypeBaseType" => (Self::BaseType, false, "System.Option<System.Type>"),
            "neoCLR.Runtime.TypeElementType" => {
                (Self::ElementType, false, "System.Option<System.Type>")
            }
            "neoCLR.Runtime.TypeShape" => (Self::Shape, true, "Boolean"),
            "neoCLR.Runtime.TypeDisplayName" => (Self::DisplayName, true, "String"),
            _ => return None,
        };
        Some((
            query,
            integer,
            crate::assembler::parse_type(result).expect("reflection signature"),
        ))
    }

    pub(crate) fn invoke(
        self,
        module: &Module,
        args: &[Value],
        limits: &Limits,
    ) -> Result<Value, Fault> {
        let Some(Value::RuntimeTypeHandle(handle)) = args.first() else {
            return Err(Fault::new("reflection requires type handle"));
        };
        let ty = from_identity(module, &handle.identity)?;
        let argument = match args.get(1) {
            Some(Value::Int32(n)) => *n,
            None => 0,
            _ => return Err(Fault::new("invalid reflection argument")),
        };
        let definition = module.type_definition(&ty);
        let arguments = type_arguments(&ty);
        match self {
            Self::Shape => Ok(Value::Boolean(match argument {
                0 => matches!(ty, Type::Array(_)),
                1 => matches!(ty, Type::ByRef(_) | Type::ReadOnlyByRef(_)),
                2 => matches!(ty, Type::Ptr(_)),
                4 => matches!(ty, Type::ReadOnlyByRef(_)),
                5 => definition.is_some_and(|d| {
                    d.is_abstract || d.representation == Representation::Interface
                }),
                3 => definition.is_some_and(|d| d.representation == Representation::Interface),
                _ => return Err(Fault::new("unknown type shape query")),
            })),
            Self::DisplayName => Ok(Value::String(match argument {
                0 => crate::type_identity::signature_name(&ty)?,
                1 => {
                    let mut outer = definition;
                    while let Some(parent) = outer.and_then(|d| d.declaring_type.as_ref()) {
                        outer = module
                            .types
                            .iter()
                            .find(|d| d.definition.as_ref() == Some(parent));
                    }
                    outer
                        .and_then(|d| d.name.rsplit_once('.'))
                        .map_or("", |(namespace, _)| namespace)
                        .to_owned()
                }
                _ => return Err(Fault::new("unknown type name query")),
            })),
            Self::BaseType => option(
                "System.Type",
                crate::inheritance::base(module, &ty)?
                    .as_ref()
                    .map(|base| type_value(module, base))
                    .transpose()?,
            ),
            Self::ElementType => option(
                "System.Type",
                match &ty {
                    Type::Array(t) | Type::ByRef(t) | Type::ReadOnlyByRef(t) | Type::Ptr(t) => {
                        Some(type_value(module, t)?)
                    }
                    _ => None,
                },
            ),
            Self::GenericArguments => array(
                "System.Type",
                handle
                    .generic_arguments
                    .iter()
                    .map(|d| Ok(wrap_type(d.clone()))),
                limits,
            ),
            Self::Interfaces => {
                let interfaces = if definition.is_some() {
                    crate::interfaces::closure(module, &ty)?
                } else {
                    Vec::new()
                };
                array(
                    "System.Type",
                    interfaces
                        .iter()
                        .filter(|t| **t != ty)
                        .map(|t| type_value(module, t)),
                    limits,
                )
            }
            Self::Fields => {
                validate_flags(argument)?;
                array(
                    "System.Reflection.FieldInfo",
                    definition
                        .into_iter()
                        .flat_map(|d| d.fields.iter().enumerate())
                        .filter(|(_, f)| selected(argument, f.visibility, false))
                        .map(|(index, f)| {
                            Ok(record(
                                "System.Reflection.FieldInfo",
                                vec![
                                    Value::String(f.name.clone()),
                                    wrap_type((**handle).clone()),
                                    type_value(
                                        module,
                                        &f.ty.substitute_type_parameters(arguments)?,
                                    )?,
                                    Value::Boolean(f.visibility == Visibility::Public),
                                    Value::Boolean(f.visibility == Visibility::Private),
                                    Value::Boolean(f.visibility == Visibility::Internal),
                                    Value::Boolean(false),
                                    index_value(index)?,
                                ],
                            ))
                        }),
                    limits,
                )
            }
            Self::Methods => {
                validate_flags(argument)?;
                let owner = definition.map(|d| d.open_type());
                array(
                    "System.Reflection.MethodInfo",
                    module
                        .functions
                        .iter()
                        .filter(|f| {
                            owner.is_some()
                                && f.owner == owner
                                && !f.name.ends_with("..ctor")
                                && selected(argument, f.visibility, !f.instance)
                        })
                        .map(|f| method(module, &ty, f, arguments, limits)),
                    limits,
                )
            }
            Self::Properties => {
                validate_flags(argument)?;
                array(
                    "System.Reflection.PropertyInfo",
                    definition
                        .into_iter()
                        .flat_map(|d| d.properties.iter().enumerate())
                        .filter_map(|(index, p)| {
                            let result = (|| {
                                let mut p = p.clone();
                                p.map_types(|t| t.substitute_type_parameters(arguments))?;
                                let getter = p
                                    .getter
                                    .as_ref()
                                    .map(|r| crate::vm::resolve(module, r))
                                    .transpose()?;
                                let setter = p
                                    .setter
                                    .as_ref()
                                    .map(|r| crate::vm::resolve(module, r))
                                    .transpose()?;
                                // A public accessor makes the property public; private accessors do not
                                // make a public property part of a NonPublic-only enumeration.
                                let public = getter
                                    .iter()
                                    .chain(&setter)
                                    .any(|f| f.visibility == Visibility::Public);
                                if !selected(
                                    argument,
                                    if public {
                                        Visibility::Public
                                    } else {
                                        Visibility::Private
                                    },
                                    !p.instance,
                                ) {
                                    return Ok(None);
                                }
                                let parameters =
                                    parameters(module, &p.parameters, &[], &[], &[], &[], limits)?;
                                let get = getter
                                    .as_ref()
                                    .map(|f| method(module, &ty, f, &[], limits))
                                    .transpose()?;
                                let set = setter
                                    .as_ref()
                                    .map(|f| method(module, &ty, f, &[], limits))
                                    .transpose()?;
                                Ok(Some(record(
                                    "System.Reflection.PropertyInfo",
                                    vec![
                                        Value::String(p.name),
                                        wrap_type((**handle).clone()),
                                        type_value(module, &p.ty)?,
                                        Value::Boolean(!p.instance),
                                        Value::Boolean(getter.is_some()),
                                        Value::Boolean(setter.is_some()),
                                        index_value(index)?,
                                        parameters,
                                        option("System.Reflection.MethodInfo", get)?,
                                        option("System.Reflection.MethodInfo", set)?,
                                    ],
                                )))
                            })();
                            match result {
                                Ok(None) => None,
                                Ok(Some(v)) => Some(Ok(v)),
                                Err(e) => Some(Err(e)),
                            }
                        }),
                    limits,
                )
            }
        }
    }
}

fn selected(flags: i32, visibility: Visibility, is_static: bool) -> bool {
    flags & if is_static { 8 } else { 4 } != 0
        && flags
            & if visibility == Visibility::Public {
                16
            } else {
                32
            }
            != 0
}
fn validate_flags(flags: i32) -> Result<(), Fault> {
    if flags & !62 != 0 {
        Err(Fault::new("unsupported BindingFlags bits"))
    } else {
        Ok(())
    }
}
fn type_arguments(ty: &Type) -> &[Type] {
    match ty {
        Type::Constructed { arguments, .. } => arguments,
        _ => &[],
    }
}
fn index_value(index: usize) -> Result<Value, Fault> {
    Ok(Value::Int32(
        i32::try_from(index).map_err(|_| Fault::new("metadata index exceeds Int32"))?,
    ))
}
fn record(name: &str, fields: Vec<Value>) -> Value {
    Value::Object {
        ty: Type::from_name(name),
        fields,
    }
}
fn wrap_type(descriptor: TypeDescriptor) -> Value {
    record(
        "System.Type",
        vec![Value::RuntimeTypeHandle(Box::new(descriptor))],
    )
}
fn type_value(module: &Module, ty: &Type) -> Result<Value, Fault> {
    Ok(wrap_type(crate::type_identity::describe_loaded(
        module, ty,
    )?))
}
fn option(name: &str, value: Option<Value>) -> Result<Value, Fault> {
    let element = crate::assembler::parse_type(name)?;
    let case = match value {
        Some(v) => Value::Object {
            ty: Type::Constructed {
                definition: "System.Option.Some".into(),
                arguments: vec![element.clone()],
            },
            fields: vec![v],
        },
        None => record("System.Option.None", vec![]),
    };
    Ok(Value::Object {
        ty: Type::Constructed {
            definition: "System.Option".into(),
            arguments: vec![element],
        },
        fields: vec![Value::Erased(Box::new(case))],
    })
}
fn array(
    name: &str,
    values: impl IntoIterator<Item = Result<Value, Fault>>,
    limits: &Limits,
) -> Result<Value, Fault> {
    let element = crate::assembler::parse_type(name)?;
    let mut elements = Vec::new();
    let mut usage = crate::arrays::Usage::default();
    for value in values {
        let mut unit = Value::Array {
            element: element.clone(),
            elements: vec![value?],
        };
        crate::arrays::measure(&unit, &mut usage, limits)?;
        let Value::Array { elements: one, .. } = &mut unit else {
            unreachable!()
        };
        elements.push(one.pop().unwrap());
    }
    Ok(Value::Array { element, elements })
}
fn parameters(
    module: &Module,
    types: &[Type],
    names: &[Option<String>],
    out: &[usize],
    conditional: &[usize],
    readonly: &[usize],
    limits: &Limits,
) -> Result<Value, Fault> {
    array(
        "System.Reflection.ParameterInfo",
        types.iter().enumerate().map(|(i, ty)| {
            let qualified = match ty {
                Type::ByRef(target) if readonly.contains(&i) => Type::ReadOnlyByRef(target.clone()),
                _ => ty.clone(),
            };
            Ok(record(
                "System.Reflection.ParameterInfo",
                vec![
                    Value::String(names.get(i).and_then(|n| n.clone()).unwrap_or_default()),
                    index_value(i)?,
                    type_value(module, &qualified)?,
                    Value::Boolean(out.contains(&i)),
                    Value::Boolean(conditional.contains(&i)),
                    Value::Boolean(readonly.contains(&i) || matches!(ty, Type::ReadOnlyByRef(_))),
                ],
            ))
        }),
        limits,
    )
}
fn method(
    module: &Module,
    owner: &Type,
    f: &Function,
    arguments: &[Type],
    limits: &Limits,
) -> Result<Value, Fault> {
    let parameter_types = f
        .parameters
        .iter()
        .map(|t| t.substitute_type_parameters(arguments))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(record(
        "System.Reflection.MethodInfo",
        vec![
            Value::String(
                f.name
                    .strip_prefix(&format!("{}.", owner.definition_name().unwrap_or("")))
                    .unwrap_or(&f.name)
                    .into(),
            ),
            type_value(module, owner)?,
            type_value(module, &f.returns.substitute_type_parameters(arguments)?)?,
            Value::Boolean(!f.instance),
            Value::Boolean(f.visibility == Visibility::Public),
            Value::Boolean(f.visibility == Visibility::Private),
            Value::Boolean(f.visibility == Visibility::Internal),
            Value::Boolean(f.receiver_byref),
            index_value(
                f.definition
                    .as_ref()
                    .ok_or_else(|| Fault::new("missing method identity"))?
                    .index as usize,
            )?,
            parameters(
                module,
                &parameter_types,
                &f.parameter_names,
                &f.out_parameters,
                &f.out_when_true,
                &f.readonly_parameters,
                limits,
            )?,
            Value::Boolean(f.receiver_readonly),
            Value::Boolean(f.is_virtual || crate::interfaces::is_contract(module, f)),
            Value::Boolean(f.is_override),
            Value::Boolean(f.is_abstract || crate::interfaces::is_contract(module, f)),
        ],
    ))
}

fn from_identity(module: &Module, identity: &TypeIdentity) -> Result<Type, Fault> {
    let nested = |id| from_identity(module, id).map(Box::new);
    Ok(match identity {
        TypeIdentity::ByRef(t) => Type::ByRef(nested(t)?),
        TypeIdentity::ReadOnlyByRef(t) => Type::ReadOnlyByRef(nested(t)?),
        TypeIdentity::Array(t) => Type::Array(nested(t)?),
        TypeIdentity::Ptr(t) => Type::Ptr(nested(t)?),
        TypeIdentity::InterfaceRef(t) => Type::InterfaceRef(nested(t)?),
        TypeIdentity::Definition {
            definition,
            arguments,
        } => {
            let d = module
                .types
                .iter()
                .find(|d| d.definition.as_ref() == Some(definition))
                .ok_or_else(|| Fault::new("type handle belongs to another load set"))?;
            let arguments = arguments
                .iter()
                .map(|id| from_identity(module, id))
                .collect::<Result<Vec<_>, _>>()?;
            d.open_type().substitute_type_parameters(&arguments)?
        }
    })
}
