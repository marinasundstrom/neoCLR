//! Metadata-only queries. Results own snapshots and never retain guest objects.
use crate::{
    Fault, Limits, Module, TypeDescriptor, TypeIdentity, Value,
    metadata::{Function, Representation, Type, Visibility},
    metadata_origin::{SourceAccess, member_access},
};

#[derive(Clone, Copy)]
pub(crate) enum Query {
    CustomAttributes,
    DeclaringType,
    MetadataToken,
    Module,
    Fields,
    Methods,
    Constructors,
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
            "neoCLR.Runtime.MemberCustomAttributes" => (
                Self::CustomAttributes,
                true,
                "System.Introspection.CustomAttributeData[]",
            ),
            "neoCLR.Runtime.TypeDeclaringType" => (
                Self::DeclaringType,
                false,
                "System.Option<System.Introspection.TypeInfo>",
            ),
            "neoCLR.Runtime.TypeMetadataToken" => (Self::MetadataToken, false, "Int32"),
            "neoCLR.Runtime.TypeModule" => (Self::Module, false, "System.Introspection.ModuleInfo"),
            "neoCLR.Runtime.TypeFields" => (Self::Fields, true, "System.Introspection.FieldInfo[]"),
            "neoCLR.Runtime.TypeConstructors" => (
                Self::Constructors,
                true,
                "System.Introspection.ConstructorInfo[]",
            ),
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
        let snapshot;
        let descriptor = if matches!(self, Self::CustomAttributes) {
            let Some(Value::ObjectReference(reference)) = args.first() else {
                return Err(Fault::new("attribute query requires TypeInfo"));
            };
            snapshot = reference.reference.read()?;
            let Value::Object { ty, fields } = &snapshot else {
                return Err(Fault::new("invalid TypeInfo snapshot"));
            };
            if ty.definition_name() != Some("System.Introspection.RuntimeNominalTypeInfo")
                || fields.len() != 1
            {
                return Err(Fault::new("invalid TypeInfo provider"));
            }
            fields.first()
        } else {
            args.first()
        };
        let Some(Value::RuntimeTypeHandle(handle)) = descriptor else {
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
                Self::Fields
                    | Self::Methods
                    | Self::Constructors
                    | Self::Properties
                    | Self::Interfaces
                    | Self::BaseType
            )
        {
            return Err(Fault::new(
                "open generic member reflection requires a closed type",
            ));
        }
        let definition = module.type_definition(&ty);
        let arguments = type_arguments(&ty);
        match self {
            Self::CustomAttributes => {
                let mut attributes = Vec::new();
                if let Some(d) = definition {
                    let token = u32::try_from(argument)
                        .map_err(|_| Fault::new("invalid attribute target"))?;
                    if token != 0 {
                        let own =
                            crate::metadata_tokens::type_token(module, &handle.identity)? as u32;
                        attributes.extend(d.custom_attributes.iter().filter(|a| {
                            a.target_token == Some(token)
                                || a.target_token.is_none() && token == own
                        }));
                        for f in &module.functions {
                            if let (Some(origin), Some(owner)) = (&f.origin, &d.origin) {
                                if origin.assembly == owner.assembly
                                    && origin.module == owner.module
                                {
                                    attributes.extend(f.custom_attributes.iter().filter(|a| {
                                        a.target_token == Some(token)
                                            || a.target_token.is_none() && token == origin.token
                                    }));
                                }
                            }
                        }
                    }
                }
                array(
                    "System.Introspection.CustomAttributeData",
                    attributes
                        .into_iter()
                        .map(|a| attribute_data(module, a, limits)),
                    limits,
                )
            }
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
                    module,
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
                12 => matches!(handle.identity, TypeIdentity::Definition { .. }),
                8 => definition.is_some_and(|d| {
                    (d.is_reference_type || d.representation == Representation::Interface)
                        && !d.is_sealed
                        && !d.is_closed_hierarchy
                }),
                9 => definition.is_some_and(|d| d.is_closed_hierarchy),
                10 => definition.is_some_and(|d| {
                    d.custom_attributes.iter().any(|attribute| {
                        attribute.target_token.is_none()
                            && attribute
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
                    | Type::Function(_)
                    | Type::ByRef(_)
                    | Type::ReadOnlyByRef(_)
                    | Type::ArrayRef(_)
                    | Type::InterfaceRef(_)
                    | Type::Ptr(_)
                    | Type::TypeParameter(_)
                    | Type::MethodTypeParameter(_) => false,
                },
                11 => publicly_visible(module, &ty),
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
            Self::DisplayName => Ok(Value::String(
                (match argument {
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
                        while let Some(parent) = outer
                            .and_then(|d| crate::type_identity::declaring_definition(module, d))
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
                })
                .into(),
            )),
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
                module,
                type_contract(module),
                crate::inheritance::base(module, &ty)?
                    .as_ref()
                    .map(|base| type_value(module, base))
                    .transpose()?,
            ),
            Self::ElementType => option(
                module,
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
                        .filter(|(index, _)| {
                            selected(
                                argument,
                                if crate::metadata_origin::field_access(definition.unwrap(), *index)
                                    == SourceAccess::Public
                                {
                                    Visibility::Public
                                } else {
                                    Visibility::Private
                                },
                                false,
                            )
                        })
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
                                    Value::Boolean(
                                        crate::metadata_origin::field_access(
                                            definition.unwrap(),
                                            index,
                                        ) == SourceAccess::Public,
                                    ),
                                    Value::Boolean(
                                        crate::metadata_origin::field_access(
                                            definition.unwrap(),
                                            index,
                                        ) == SourceAccess::Private,
                                    ),
                                    Value::Boolean(
                                        crate::metadata_origin::field_access(
                                            definition.unwrap(),
                                            index,
                                        ) == SourceAccess::Assembly,
                                    ),
                                    Value::Boolean(false),
                                    index_value(index)?,
                                ],
                            ))
                        }),
                    limits,
                )
            }
            Self::Methods | Self::Constructors => {
                let constructors = matches!(self, Self::Constructors);
                validate_flags(argument)?;
                let owner = definition.map(|d| d.open_type());
                array(
                    if constructors {
                        "System.Introspection.ConstructorInfo"
                    } else {
                        "System.Introspection.MethodInfo"
                    },
                    module
                        .functions
                        .iter()
                        .filter(|f| {
                            owner.is_some()
                                && f.owner == owner
                                && f.name.ends_with("..ctor") == constructors
                                && selected(
                                    argument,
                                    if member_access(f) == SourceAccess::Public {
                                        Visibility::Public
                                    } else {
                                        Visibility::Private
                                    },
                                    !f.instance,
                                )
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
                                    .any(|f| member_access(f) == SourceAccess::Public);
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
                                        option(module, "System.Introspection.MethodInfo", get)?,
                                        option(module, "System.Introspection.MethodInfo", set)?,
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
fn attribute_data(
    module: &Module,
    attribute: &crate::metadata::CustomAttribute,
    limits: &Limits,
) -> Result<Value, Fault> {
    use crate::metadata::AttributeArgument as A;
    let owner = attribute
        .constructor
        .owner
        .as_ref()
        .ok_or_else(|| Fault::new("missing attribute type"))?;
    let constructor = crate::vm::resolve(module, &attribute.constructor)?;
    let arguments = array(
        "System.Introspection.CustomAttributeTypedArgument",
        attribute
            .arguments
            .iter()
            .zip(&attribute.constructor.parameters)
            .map(|(argument, ty)| {
                let value = match argument {
                    A::String(Some(value)) => {
                        Value::Erased(Box::new(Value::String(value.clone().into())))
                    }
                    A::String(None) => Value::NullObjectReference(Type::from_name("System.Object")),
                    A::Int32(value) => Value::Erased(Box::new(Value::Int32(*value))),
                    A::Boolean(value) => Value::Erased(Box::new(Value::Boolean(*value))),
                };
                Ok(record(
                    "System.Introspection.CustomAttributeTypedArgument",
                    vec![type_value(module, ty)?, value],
                ))
            }),
        limits,
    )?;
    Ok(record(
        "System.Introspection.CustomAttributeData",
        vec![
            type_value(module, owner)?,
            method(module, owner, &constructor, &[], limits)?,
            arguments,
        ],
    ))
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
fn option(module: &Module, name: &str, value: Option<Value>) -> Result<Value, Fault> {
    let element = crate::assembler::parse_type(name)?;
    let ty = Type::Constructed {
        definition: "System.Option".into(),
        arguments: vec![element.clone()],
    };
    let some_type = Type::Constructed {
        definition: "System.Option.Some".into(),
        arguments: vec![element],
    };
    let none_type = Type::from_name("System.Option.None");
    let layout = crate::inheritance::fields(module, &ty)?;
    let fields = match layout.as_slice() {
        [stored] if stored.ty == Type::Value => {
            // The separate Neo bootstrap profile retains its erased carrier ABI.
            let case = match value {
                Some(value) => Value::Object {
                    ty: some_type,
                    fields: vec![value],
                },
                None => record("System.Option.None", vec![]),
            };
            vec![Value::Erased(Box::new(case))]
        }
        [tag, some, none]
            if tag.ty == Type::Byte && some.ty == some_type && none.ty == none_type =>
        {
            // Raven's library profile stores a discriminator and both case records.
            // Validate the closed storage contract before publishing the snapshot.
            let (tag, payload) = match value {
                Some(value) => (
                    1,
                    Value::Object {
                        ty: some_type,
                        fields: vec![value],
                    },
                ),
                None => (2, crate::initialization::default_value(module, &some_type)?),
            };
            vec![
                Value::Byte(tag),
                payload,
                record("System.Option.None", vec![]),
            ]
        }
        _ => return Err(Fault::new("unsupported reflection Option storage contract")),
    };
    Ok(Value::Object { ty, fields })
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
        if f.name.ends_with("..ctor") {
            "System.Introspection.ConstructorInfo"
        } else {
            "System.Introspection.MethodInfo"
        },
        vec![
            Value::String(
                // Imported bodies may use escaped names. Preserve source names
                // for display without changing dispatch identities.
                f.origin
                    .as_ref()
                    .map(|origin| origin.name.as_str())
                    .unwrap_or_else(|| {
                        f.name
                            .strip_prefix(&format!("{}.", owner.definition_name().unwrap_or("")))
                            .unwrap_or(&f.name)
                    })
                    .into(),
            ),
            type_value(module, owner)?,
            type_value(module, &f.returns.substitute_type_parameters(arguments)?)?,
            Value::Boolean(!f.instance),
            Value::Boolean(member_access(f) == SourceAccess::Public),
            Value::Boolean(member_access(f) == SourceAccess::Private),
            Value::Boolean(member_access(f) == SourceAccess::Assembly),
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
            Value::Boolean(
                f.is_virtual
                    || (crate::interfaces::is_contract(module, f)
                        && !crate::interfaces::is_helper(f)),
            ),
            Value::Boolean(f.is_override),
            Value::Boolean(crate::interfaces::is_bodyless(module, f)),
        ],
    ))
}

pub(crate) fn from_identity(module: &Module, identity: &TypeIdentity) -> Result<Type, Fault> {
    let nested = |id| from_identity(module, id).map(Box::new);
    Ok(match identity {
        TypeIdentity::Function {
            parameters,
            returns,
            no_result,
            out_parameters,
            out_when_true,
        } => Type::Function(Box::new(crate::metadata::FunctionType {
            parameters: parameters
                .iter()
                .map(|t| from_identity(module, t))
                .collect::<Result<_, _>>()?,
            returns: *nested(returns)?,
            no_result: *no_result,
            out_parameters: out_parameters.clone(),
            out_when_true: out_when_true.clone(),
        })),
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
            Value::Erased(value)
                if matches!(
                    &*value,
                    Value::String(_) | Value::Int32(_) | Value::Boolean(_)
                ) =>
            {
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
                        "System.Introspection.TypeInfo" => {
                            if matches!(fields.first(), Some(Value::RuntimeTypeHandle(handle)) if matches!(handle.identity, TypeIdentity::Definition { .. }))
                            {
                                "System.Introspection.RuntimeNominalTypeInfo"
                            } else {
                                "System.Introspection.RuntimeTypeInfo"
                            }
                        }
                        "System.Introspection.ParameterInfo" => {
                            "System.Introspection.RuntimeParameterInfo"
                        }
                        "System.Introspection.FieldInfo" => "System.Introspection.RuntimeFieldInfo",
                        "System.Introspection.ConstructorInfo" => {
                            "System.Introspection.RuntimeConstructorInfo"
                        }
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
                let layout = crate::inheritance::fields(module, &ty)?;
                if fields.len() != layout.len() {
                    return Err(Fault::new("reflection snapshot field count mismatch"));
                }
                let fields = fields
                    .into_iter()
                    .zip(&layout)
                    .map(|(v, field)| {
                        let v = build(module, heap, limits, v, depth + 1)?;
                        if let (Value::Array { element, .. }, Type::ArrayRef(expected)) =
                            (&v, &field.ty)
                        {
                            if element != expected.as_ref() {
                                return Err(Fault::new("reflection array element mismatch"));
                            }
                            if heap.len() >= limits.heap_objects {
                                return Err(Fault::coded(
                                    crate::FaultCode::HeapLimitExceeded,
                                    "heap object limit exceeded",
                                ));
                            }
                            let index = heap.allocate(v)?;
                            return Ok(Value::ObjectReference(crate::value::ObjectReference {
                                reference: heap.address(index)?,
                                view: Some(field.ty.clone()),
                            }));
                        }
                        Ok(v)
                    })
                    .collect::<Result<Vec<_>, Fault>>()?;
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

// Source visibility already includes every enclosing type. Compound types must
// also expose only visible element/argument types; visibility grants no execution.
fn publicly_visible(module: &Module, ty: &Type) -> bool {
    match ty {
        Type::Function(shape) => shape
            .parameters
            .iter()
            .chain(std::iter::once(&shape.returns))
            .all(|t| publicly_visible(module, t)),
        Type::Array(t)
        | Type::ArrayRef(t)
        | Type::ByRef(t)
        | Type::ReadOnlyByRef(t)
        | Type::Ptr(t)
        | Type::InterfaceRef(t) => publicly_visible(module, t),
        Type::TypeParameter(_) | Type::MethodTypeParameter(_) => true,
        _ => {
            module.type_definition(ty).is_none_or(|d| {
                d.visibility == crate::metadata::Visibility::Public
                    && d.origin
                        .as_ref()
                        .is_none_or(|o| o.publicly_visible == Some(true))
            }) && ty
                .generic_arguments()
                .iter()
                .all(|t| publicly_visible(module, t))
        }
    }
}

#[cfg(test)]
mod visibility_tests {
    use super::*;

    #[test]
    fn visibility_respects_source_nesting_and_compound_types() {
        let mut module = crate::assemble(".module Visibility\n.type Public\n.end\n.type internal Hidden\n.end\n.type Box<T>\n.end\n").unwrap();
        for (name, expected) in [
            ("Public", true),
            ("Hidden", false),
            ("Int32", true),
            ("arrayref<Public>", true),
            ("arrayref<Hidden>", false),
            ("Box<Public>", true),
            ("Box<Hidden>", false),
            ("Hidden&", false),
            ("Public*", true),
        ] {
            assert_eq!(
                publicly_visible(&module, &crate::assembler::parse_type(name).unwrap()),
                expected,
                "{name}"
            );
        }
        let public = &mut module.types[0];
        public.origin = Some(serde_json::from_str(r#"{"assembly":"Source","module":"Source.dll","name":"Outer/Nested","token":33554433,"publicly_visible":false}"#).unwrap());
        assert!(!publicly_visible(&module, &Type::from_name("Public")));
        module.types[0].origin.as_mut().unwrap().publicly_visible = None;
        assert!(!publicly_visible(&module, &Type::from_name("Public")));
        module.types[0].origin.as_mut().unwrap().publicly_visible = Some(true);
        assert!(publicly_visible(&module, &Type::from_name("Public")));
    }
}
