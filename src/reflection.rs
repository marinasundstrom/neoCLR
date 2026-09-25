//! Metadata-only queries. Results own snapshots and never retain guest objects.
use crate::{
    Fault, Limits, Module, TypeDescriptor, TypeIdentity, Value,
    metadata::{Function, Representation, Type, Visibility},
};

#[derive(Clone, Copy)]
pub(crate) enum Query {
    DeclaringType,
    MetadataToken,
    Module,
    Fields,
    Methods,
    Properties,
    Interfaces,
    GenericArguments,
    ElementType,
    BaseType,
    Shape,
    DisplayName,
    EnumNames,
    EnumValues,
    EnumFormat,
    EnumUnderlying,
}

impl Query {
    pub(crate) fn binding(name: &str) -> Option<(Self, bool, Type)> {
        let (query, integer, result) = match name {
            "neoCLR.Runtime.TypeDeclaringType" => (
                Self::DeclaringType,
                false,
                "System.Option<System.Introspection.TypeInfo>",
            ),
            "neoCLR.Runtime.TypeMetadataToken" => (Self::MetadataToken, false, "Int32"),
            "neoCLR.Runtime.TypeModule" => (Self::Module, false, "System.Introspection.ModuleInfo"),
            "neoCLR.Runtime.TypeFields" => (Self::Fields, true, "System.Introspection.FieldInfo[]"),
            "neoCLR.Runtime.TypeMethods" => {
                (Self::Methods, true, "System.Introspection.MethodInfo[]")
            }
            "neoCLR.Runtime.TypeProperties" => (
                Self::Properties,
                true,
                "System.Introspection.PropertyInfo[]",
            ),
            "neoCLR.Runtime.TypeInterfaces" => (Self::Interfaces, false, "System.Type[]"),
            "neoCLR.Runtime.TypeGenericArguments" => {
                (Self::GenericArguments, false, "System.Type[]")
            }
            "neoCLR.Runtime.TypeBaseType" => (Self::BaseType, false, "System.Option<System.Type>"),
            "neoCLR.Runtime.TypeElementType" => {
                (Self::ElementType, false, "System.Option<System.Type>")
            }
            "neoCLR.Runtime.TypeEnumNames" => (Self::EnumNames, false, "String[]"),
            "neoCLR.Runtime.TypeEnumValues" => (Self::EnumValues, false, "System.Object[]"),
            "neoCLR.Runtime.EnumFormat" => (Self::EnumFormat, true, "String"),
            "neoCLR.Runtime.TypeEnumUnderlying" => (Self::EnumUnderlying, false, "System.Type"),
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
        // Definition discovery admits open generics, but member substitution still
        // needs a closed type in this preview.
        if handle
            .generic_arguments
            .iter()
            .any(|a| matches!(a.identity, TypeIdentity::GenericParameter { .. }))
            && matches!(
                self,
                Self::Fields | Self::Methods | Self::Properties | Self::Interfaces | Self::BaseType
            )
        {
            return Err(Fault::new(
                "open generic member reflection requires a closed type",
            ));
        }
        let definition = module.type_definition(&ty);
        let arguments = type_arguments(&ty);
        match self {
            Self::DeclaringType => {
                let parent = match &handle.identity {
                    TypeIdentity::Definition { .. } => {
                        handle.declaring_type.as_ref().and_then(|id| {
                            module
                                .types
                                .iter()
                                .find(|d| d.definition.as_ref() == Some(id))
                        })
                    }
                    TypeIdentity::GenericParameter { definition, .. } => module
                        .types
                        .iter()
                        .find(|d| d.definition.as_ref() == Some(definition)),
                    _ => None,
                };
                option(
                    type_contract(module),
                    parent
                        .map(|d| {
                            crate::type_identity::describe_definition(module, d)
                                .map(|v| wrap_type(module, v))
                        })
                        .transpose()?,
                )
            }
            Self::MetadataToken => Ok(Value::Int32(crate::metadata_tokens::type_token(
                module,
                &handle.identity,
            )?)),
            Self::Module => {
                let d = crate::metadata_tokens::definition(module, &handle.identity)
                    .ok_or_else(|| Fault::new("missing type module"))?;
                crate::metadata_tokens::module_value(
                    module,
                    d.origin.as_ref(),
                    &d.definition
                        .as_ref()
                        .ok_or_else(|| Fault::new("missing type identity"))?
                        .module,
                )
            }
            Self::Shape => Ok(Value::Boolean(match argument {
                8 => definition.is_some_and(|d| {
                    (d.is_reference_type || d.representation == Representation::Interface)
                        && !d.is_sealed
                        && !d.is_closed_hierarchy
                }),
                9 => definition.is_some_and(|d| d.is_closed_hierarchy),
                10 => definition.is_some_and(|d| {
                    d.custom_attributes.iter().any(|attribute| {
                        attribute
                            .constructor
                            .owner
                            .as_ref()
                            .and_then(Type::definition_name)
                            == Some("System.Runtime.CompilerServices.UnionAttribute")
                    })
                }),
                6 => definition.is_some_and(|d| d.enum_info.is_some()),
                7 => match &ty {
                    Type::Void
                    | Type::Single
                    | Type::Double
                    | Type::Int32
                    | Type::SByte
                    | Type::Byte
                    | Type::Int16
                    | Type::UInt16
                    | Type::Char
                    | Type::UInt32
                    | Type::Int64
                    | Type::UInt64
                    | Type::IntPtr
                    | Type::UIntPtr
                    | Type::Boolean
                    | Type::Value
                    | Type::RuntimeTypeHandle
                    | Type::Array(_) => true,
                    Type::Named(_) | Type::Constructed { .. } | Type::Scoped { .. } => definition
                        .is_some_and(|d| {
                            !d.is_reference_type && d.representation != Representation::Interface
                        }),
                    Type::String
                    | Type::ByRef(_)
                    | Type::ReadOnlyByRef(_)
                    | Type::ArrayRef(_)
                    | Type::InterfaceRef(_)
                    | Type::Ptr(_)
                    | Type::TypeParameter(_)
                    | Type::MethodTypeParameter(_) => false,
                },
                0 => matches!(ty, Type::Array(_) | Type::ArrayRef(_)),
                1 => matches!(ty, Type::ByRef(_) | Type::ReadOnlyByRef(_)),
                2 => matches!(ty, Type::Ptr(_)),
                4 => matches!(ty, Type::ReadOnlyByRef(_)),
                5 => definition.is_some_and(|d| {
                    d.is_abstract || d.representation == Representation::Interface
                }),
                3 => definition.is_some_and(|d| d.representation == Representation::Interface),
                _ => return Err(Fault::new("unknown type shape query")),
            })),
            Self::DisplayName => Ok(Value::String((match argument {
                0 if matches!(handle.identity, TypeIdentity::GenericParameter { .. }) => {
                    handle.name.clone()
                }
                0 if definition.is_some_and(|d| {
                    d.origin.is_some()
                        || !d.generic_parameters.is_empty()
                            && handle.generic_arguments.iter().any(|a| {
                                matches!(a.identity, TypeIdentity::GenericParameter { .. })
                            })
                }) =>
                {
                    handle.name.clone()
                }
                0 => crate::type_identity::signature_name(&ty)?,
                1 => {
                    let mut outer = definition;
                    while let Some(parent) =
                        outer.and_then(|d| crate::type_identity::declaring_definition(module, d))
                    {
                        outer = Some(parent);
                    }
                    outer
                        .and_then(|d| {
                            d.origin
                                .as_ref()
                                .map_or(d.name.as_str(), |o| o.name.as_str())
                                .rsplit_once('.')
                        })
                        .map_or("", |(namespace, _)| namespace)
                        .to_owned()
                }
                _ => return Err(Fault::new("unknown type name query")),
            }).into())),
            Self::EnumNames | Self::EnumValues | Self::EnumFormat | Self::EnumUnderlying => {
                let info = definition
                    .and_then(|d| d.enum_info.as_ref())
                    .ok_or_else(|| Fault::new("enum reflection requires an enum type"))?;
                if matches!(self, Self::EnumUnderlying) {
                    return type_value(module, &info.underlying);
                }
                if matches!(self, Self::EnumFormat) {
                    return Ok(Value::String(crate::enums::format(info, argument).into()));
                }
                let members = crate::enums::members(info);
                if matches!(self, Self::EnumValues) {
                    return array(
                        "System.Object",
                        members.into_iter().map(|member| {
                            // Internal snapshot request: materialization creates an
                            // exact enum box, not a box of its underlying integer.
                            Ok(Value::Erased(Box::new(Value::Object {
                                ty: ty.clone(),
                                fields: vec![Value::Int32(member.value)],
                            })))
                        }),
                        limits,
                    );
                }
                array(
                    "String",
                    members
                        .into_iter()
                        .map(|m| Ok(Value::String(m.name.clone().into()))),
                    limits,
                )
            }
            Self::BaseType => option(
                type_contract(module),
                crate::inheritance::base(module, &ty)?
                    .as_ref()
                    .map(|base| type_value(module, base))
                    .transpose()?,
            ),
            Self::ElementType => option(
                type_contract(module),
                match &ty {
                    Type::Array(t)
                    | Type::ArrayRef(t)
                    | Type::ByRef(t)
                    | Type::ReadOnlyByRef(t)
                    | Type::Ptr(t) => Some(type_value(module, t)?),
                    _ => None,
                },
            ),
            Self::GenericArguments => array(
                type_contract(module),
                handle
                    .generic_arguments
                    .iter()
                    .map(|d| Ok(wrap_type(module, d.clone()))),
                limits,
            ),
            Self::Interfaces => {
                let interfaces = if definition.is_some() {
                    crate::interfaces::closure(module, &ty)?
                } else {
                    Vec::new()
                };
                array(
                    type_contract(module),
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
                    "System.Introspection.FieldInfo",
                    definition
                        .into_iter()
                        .flat_map(|d| d.fields.iter().enumerate())
                        .filter(|(_, f)| selected(argument, f.visibility, false))
                        .map(|(index, f)| {
                            Ok(member_record(
                                module,
                                crate::metadata_tokens::member(
                                    module,
                                    definition.unwrap(),
                                    index,
                                    false,
                                )?,
                                "System.Introspection.FieldInfo",
                                vec![
                                    Value::String(f.name.clone().into()),
                                    wrap_type(module, (**handle).clone()),
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
                    "System.Introspection.MethodInfo",
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
                    "System.Introspection.PropertyInfo",
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
                                let parameters = parameters(
                                    module,
                                    getter.as_ref().or(setter.as_ref()),
                                    &ty,
                                    3, // Property identity, distinct from its accessor method.
                                    index,
                                    &p.parameters,
                                    &[],
                                    &[],
                                    &[],
                                    &[],
                                    limits,
                                )?;
                                let get = getter
                                    .as_ref()
                                    .map(|f| method(module, &ty, f, &[], limits))
                                    .transpose()?;
                                let set = setter
                                    .as_ref()
                                    .map(|f| method(module, &ty, f, &[], limits))
                                    .transpose()?;
                                Ok(Some(member_record(
                                    module,
                                    crate::metadata_tokens::member(
                                        module,
                                        definition.unwrap(),
                                        index,
                                        true,
                                    )?,
                                    "System.Introspection.PropertyInfo",
                                    vec![
                                        Value::String(p.name.into()),
                                        wrap_type(module, (**handle).clone()),
                                        type_value(module, &p.ty)?,
                                        Value::Boolean(!p.instance),
                                        Value::Boolean(getter.is_some()),
                                        Value::Boolean(setter.is_some()),
                                        index_value(index)?,
                                        parameters,
                                        option("System.Introspection.MethodInfo", get)?,
                                        option("System.Introspection.MethodInfo", set)?,
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
    ty.generic_arguments()
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
fn member_record(module: &Module, token: i32, name: &str, mut fields: Vec<Value>) -> Value {
    if type_contract(module) == "System.Introspection.TypeInfo" {
        fields.insert(2, Value::Int32(token));
    }
    record(name, fields)
}
fn type_contract(module: &Module) -> &'static str {
    if module
        .type_definition(&Type::from_name("System.Introspection.TypeInfo"))
        .is_some_and(|d| d.representation == Representation::Interface)
    {
        "System.Introspection.TypeInfo"
    } else {
        "System.Type"
    }
}
pub(crate) fn wrap_type(module: &Module, descriptor: TypeDescriptor) -> Value {
    record(
        type_contract(module),
        vec![Value::RuntimeTypeHandle(Box::new(descriptor))],
    )
}
fn type_value(module: &Module, ty: &Type) -> Result<Value, Fault> {
    Ok(wrap_type(
        module,
        crate::type_identity::describe_loaded(module, ty)?,
    ))
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
pub(crate) fn array(
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
// Keep the independent CLI parameter tables explicit at this metadata boundary.
#[allow(clippy::too_many_arguments)]
fn parameters(
    module: &Module,
    function: Option<&Function>,
    declaring_type: &Type,
    member_kind: i32,
    member_index: usize,
    types: &[Type],
    names: &[Option<String>],
    out: &[usize],
    conditional: &[usize],
    readonly: &[usize],
    limits: &Limits,
) -> Result<Value, Fault> {
    array(
        "System.Introspection.ParameterInfo",
        types.iter().enumerate().map(|(i, ty)| {
            let qualified = match ty {
                Type::ByRef(target) if readonly.contains(&i) => Type::ReadOnlyByRef(target.clone()),
                _ => ty.clone(),
            };
            let mut fields = vec![
                Value::String(
                    names
                        .get(i)
                        .and_then(|n| n.clone())
                        .unwrap_or_default()
                        .into(),
                ),
                index_value(i)?,
                type_value(module, &qualified)?,
                Value::Boolean(out.contains(&i)),
                Value::Boolean(conditional.contains(&i)),
                Value::Boolean(readonly.contains(&i) || matches!(ty, Type::ReadOnlyByRef(_))),
            ];
            if type_contract(module) == "System.Introspection.TypeInfo" {
                let f = function
                    .ok_or_else(|| Fault::new("parameter snapshot requires declaring member"))?;
                fields.push(Value::Int32(crate::metadata_tokens::parameter(
                    module, f, i,
                )?));
                fields.push(crate::metadata_tokens::module_value(
                    module,
                    f.origin.as_ref(),
                    &f.definition
                        .as_ref()
                        .ok_or_else(|| Fault::new("missing method identity"))?
                        .module,
                )?);
                // Retain a compact owner key, not a member snapshot containing
                // this parameter list. Equality does not depend on token presence.
                fields.push(type_value(module, declaring_type)?);
                fields.push(Value::Int32(member_kind));
                fields.push(index_value(member_index)?);
            }
            Ok(record("System.Introspection.ParameterInfo", fields))
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
    if !f.generic_parameters.is_empty() {
        return Err(Fault::new(
            "generic method definition reflection is not yet supported",
        ));
    }
    let parameter_types = f
        .parameters
        .iter()
        .map(|t| t.substitute_type_parameters(arguments))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(member_record(
        module,
        crate::metadata_tokens::method(f)?,
        "System.Introspection.MethodInfo",
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
                Some(f),
                owner,
                2, // Method identity; keep in step with the descriptor hash kinds.
                f.definition
                    .as_ref()
                    .ok_or_else(|| Fault::new("missing method identity"))?
                    .index as usize,
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
            Value::Boolean(crate::interfaces::is_bodyless(module, f)),
        ],
    ))
}

pub(crate) fn from_identity(module: &Module, identity: &TypeIdentity) -> Result<Type, Fault> {
    let nested = |id| from_identity(module, id).map(Box::new);
    Ok(match identity {
        TypeIdentity::GenericParameter { index, .. } => Type::TypeParameter(*index),
        TypeIdentity::ByRef(t) => Type::ByRef(nested(t)?),
        TypeIdentity::ReadOnlyByRef(t) => Type::ReadOnlyByRef(nested(t)?),
        TypeIdentity::Array(t) => Type::Array(nested(t)?),
        TypeIdentity::ArrayRef(t) => Type::ArrayRef(nested(t)?),
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

/// Project trusted metadata snapshots into the library's declared storage categories.
/// This is deliberately confined to reflection results, never arbitrary host inputs.
pub(crate) fn materialize(
    module: &Module,
    heap: &mut crate::ManagedHeap,
    limits: &Limits,
    value: Value,
) -> Result<Value, Fault> {
    fn build(
        module: &Module,
        heap: &mut crate::ManagedHeap,
        limits: &Limits,
        value: Value,
        depth: usize,
    ) -> Result<Value, Fault> {
        if depth > 64 {
            return Err(Fault::new("reflection snapshot nesting limit exceeded"));
        }
        match value {
            Value::Erased(value) if matches!(&*value, Value::Object { ty, .. } if module.type_definition(ty).is_some_and(|definition| definition.enum_info.is_some())) =>
            {
                let Value::Object { ref ty, ref fields } = *value else {
                    return Err(Fault::new("invalid boxed enum snapshot"));
                };
                if module
                    .type_definition(ty)
                    .is_none_or(|d| d.enum_info.is_none())
                    || !matches!(fields.as_slice(), [Value::Int32(_)])
                {
                    return Err(Fault::new("invalid boxed enum snapshot"));
                }
                if heap.len() >= limits.heap_objects {
                    return Err(Fault::coded(
                        crate::FaultCode::HeapLimitExceeded,
                        "heap object limit exceeded",
                    ));
                }
                let index = heap.allocate(*value)?;
                Ok(Value::ObjectReference(crate::value::ObjectReference {
                    reference: heap.address(index)?,
                    view: Some(Type::from_name("System.Object")),
                }))
            }
            Value::Object { ty, fields } => {
                // Native snapshots name the descriptive contract. The Raven profile
                // realizes it with an internal provider; the legacy value profile
                // continues to use its declared record representation.
                let view = module
                    .type_definition(&ty)
                    .is_some_and(|definition| {
                        definition.representation == Representation::Interface
                    })
                    .then(|| ty.clone());
                let ty = if view.is_some() {
                    let name = ty
                        .definition_name()
                        .ok_or_else(|| Fault::new("missing snapshot contract"))?;
                    let provider = match name {
                        "System.Introspection.AssemblyInfo" => {
                            "System.Introspection.RuntimeAssemblyInfo"
                        }
                        "System.Introspection.ModuleInfo" => {
                            "System.Introspection.RuntimeModuleInfo"
                        }
                        "System.Introspection.TypeInfo" => "System.Introspection.RuntimeTypeInfo",
                        "System.Introspection.ParameterInfo" => {
                            "System.Introspection.RuntimeParameterInfo"
                        }
                        "System.Introspection.FieldInfo" => "System.Introspection.RuntimeFieldInfo",
                        "System.Introspection.MethodInfo" => {
                            "System.Introspection.RuntimeMethodInfo"
                        }
                        "System.Introspection.PropertyInfo" => {
                            "System.Introspection.RuntimePropertyInfo"
                        }
                        _ => return Err(Fault::new("unsupported runtime snapshot contract")),
                    };
                    let provider = Type::from_name(provider);
                    if !module.is_reference_type(&provider)
                        || !module.reference_assignable(&provider, &ty)
                    {
                        return Err(Fault::new("missing runtime snapshot provider"));
                    }
                    provider
                } else {
                    ty
                };
                let fields = fields
                    .into_iter()
                    .map(|v| build(module, heap, limits, v, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?;
                let value = Value::Object {
                    ty: ty.clone(),
                    fields,
                };
                if module.is_reference_type(&ty) {
                    if heap.len() >= limits.heap_objects {
                        return Err(Fault::coded(
                            crate::FaultCode::HeapLimitExceeded,
                            "heap object limit exceeded",
                        ));
                    }
                    let index = heap.allocate(value)?;
                    Ok(Value::ObjectReference(crate::value::ObjectReference {
                        reference: heap.address(index)?,
                        view,
                    }))
                } else {
                    Ok(value)
                }
            }
            Value::Array { element, elements } => Ok(Value::Array {
                element,
                elements: elements
                    .into_iter()
                    .map(|v| build(module, heap, limits, v, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            Value::Erased(value) => Ok(Value::Erased(Box::new(build(
                module,
                heap,
                limits,
                *value,
                depth + 1,
            )?))),
            value => Ok(value),
        }
    }
    build(module, heap, limits, value, 0)
}
