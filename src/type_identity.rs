//! Resolved signature keys and descriptive generic definitions, independent of layout.
use crate::{
    Fault, Module,
    metadata::{Type, TypeDefId},
};

/// An identity within the resolved modules of one build, not a cross-build cache key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeIdentity {
    /// Ordered structural value identity, without a nominal definition.
    Tuple(Vec<TypeIdentity>),
    /// Experimental union identity; constituents retain resolved definition keys.
    Union(Vec<TypeIdentity>),
    /// Experimental intersection identity; distinct from nominal inheritance.
    Intersection(Vec<TypeIdentity>),
    Function {
        parameters: Vec<TypeIdentity>,
        returns: Box<TypeIdentity>,
        no_result: bool,
        out_parameters: Vec<usize>,
        out_when_true: Vec<usize>,
    },
    GenericParameter {
        definition: TypeDefId,
        index: u16,
    },
    Definition {
        definition: TypeDefId,
        arguments: Vec<TypeIdentity>,
    },
    ByRef(Box<TypeIdentity>),
    ReadOnlyByRef(Box<TypeIdentity>),
    Array(Box<TypeIdentity>),
    ArrayRef(Box<TypeIdentity>),
    Ptr(Box<TypeIdentity>),
    InterfaceRef(Box<TypeIdentity>),
}

/// Read-only type information resolved within one loaded program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDescriptor {
    pub identity: TypeIdentity,
    pub name: String,
    pub generic_arguments: Vec<TypeDescriptor>,
    /// Direct structural union alternatives or intersection bounds, in normalized
    /// signature order. Empty for nominal, Function and other non-composition types.
    pub constituents: Vec<TypeDescriptor>,
    /// Ordered tuple elements, preserving duplicates and nesting; empty otherwise.
    pub tuple_elements: Vec<TypeDescriptor>,
    /// Native element names at this signature use, excluded from type identity.
    pub tuple_element_names: Vec<Option<String>>,
    pub declaring_type: Option<TypeDefId>,
}

pub(crate) fn describe(module: &Module, ty: &Type) -> Result<TypeDescriptor, Fault> {
    let normalized = crate::scope::normalize_type(module, ty)?;
    resolve(module, &normalized)?;
    describe_loaded(module, &normalized)
}

/// Host inspection admits composition snapshots without creating guest TypeInfo.
pub(crate) fn describe_host(module: &Module, ty: &Type) -> Result<TypeDescriptor, Fault> {
    let normalized = crate::scope::normalize_type(module, ty)?;
    resolve(module, &normalized)?;
    describe_validated(module, &normalized)
}

// Introspection follows already validated metadata, including transitive signatures.
// It does not grant permission to name those types in executable instructions.
pub(crate) fn describe_loaded(module: &Module, normalized: &Type) -> Result<TypeDescriptor, Fault> {
    describe_validated(module, normalized)
}

fn describe_validated(module: &Module, normalized: &Type) -> Result<TypeDescriptor, Fault> {
    crate::vm::check_type(normalized, module)?;
    let identity = build(module, normalized)?;
    let name = match normalized {
        Type::Function(_) | Type::Union(_) | Type::Intersection(_) | Type::Tuple(_) => {
            signature_name(normalized)?
        }
        Type::ByRef(element) => reference_name(element, false)?,
        Type::ReadOnlyByRef(element) => reference_name(element, true)?,
        Type::Array(element) => format!("{}[]", signature_name(element)?),
        Type::ArrayRef(element) => format!("arrayref<{}>", signature_name(element)?),
        Type::Ptr(element) => format!("{}*", signature_name(element)?),
        Type::InterfaceRef(element) => format!("InterfaceRef<{}>", signature_name(element)?),
        _ => normalized
            .definition_name()
            .ok_or_else(|| Fault::new("type has no metadata name"))?
            .to_owned(),
    };
    let name = module
        .type_definition(normalized)
        .and_then(|d| d.origin.as_ref())
        .map_or(name, |o| o.name.clone());
    let generic_arguments = match normalized {
        Type::ArrayRef(element) if module.type_definition(normalized).is_some() => {
            vec![describe_validated(module, element)?]
        }
        Type::Constructed { arguments, .. } => arguments
            .iter()
            .map(|argument| describe_validated(module, argument))
            .collect::<Result<_, _>>()?,
        _ => vec![],
    };
    let constituents = match normalized {
        Type::Union(parts) | Type::Intersection(parts) => parts
            .iter()
            .map(|part| describe_validated(module, part))
            .collect::<Result<_, _>>()?,
        _ => vec![],
    };
    let tuple_elements = match normalized {
        Type::Tuple(parts) => parts
            .iter()
            .map(|part| describe_validated(module, part))
            .collect::<Result<_, _>>()?,
        _ => vec![],
    };
    let declaring_type = module
        .type_definition(normalized)
        .and_then(|definition| declaring_definition(module, definition))
        .and_then(|parent| parent.definition.clone());
    Ok(TypeDescriptor {
        identity,
        name,
        generic_arguments,
        constituents,
        tuple_elements,
        tuple_element_names: match normalized {
            Type::Tuple(parts) => (0..parts.len())
                .map(|i| parts.names().get(i).cloned().flatten())
                .collect(),
            _ => vec![],
        },
        declaring_type,
    })
}

pub(crate) fn signature_name(ty: &Type) -> Result<String, Fault> {
    Ok(match ty {
        Type::Union(parts) | Type::Intersection(parts) => format!(
            "{}<{}>",
            if matches!(ty, Type::Union(_)) {
                "union"
            } else {
                "intersection"
            },
            parts
                .iter()
                .map(signature_name)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        ),
        Type::Tuple(parts) => format!(
            "tuple<{}>",
            parts
                .iter()
                .map(signature_name)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        ),
        Type::SelfType => "Self".into(),
        Type::Function(shape) => {
            let mut parts = shape
                .parameters
                .iter()
                .enumerate()
                .map(|(i, ty)| {
                    let mode = if shape.out_parameters.contains(&i) {
                        "out "
                    } else if shape.out_when_true.contains(&i) {
                        "outtrue "
                    } else {
                        ""
                    };
                    Ok(format!("{mode}{}", signature_name(ty)?))
                })
                .collect::<Result<Vec<_>, Fault>>()?;
            parts.push(format!(
                "{}{}",
                if shape.no_result { "noresult " } else { "" },
                signature_name(&shape.returns)?
            ));
            format!("fn<{}>", parts.join(","))
        }
        Type::ByRef(element) => reference_name(element, false)?,
        Type::ReadOnlyByRef(element) => reference_name(element, true)?,
        Type::Array(element) => format!("{}[]", signature_name(element)?),
        Type::ArrayRef(element) => format!("arrayref<{}>", signature_name(element)?),
        Type::Ptr(element) => format!("{}*", signature_name(element)?),
        Type::InterfaceRef(element) => format!("InterfaceRef<{}>", signature_name(element)?),
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
            .ok_or_else(|| Fault::new("type has no metadata name"))?
            .to_owned(),
    })
}

/// Resolve a closed signature with the bundled System library, without executing IL.
pub fn resolve_type_identity(module: &Module, ty: &Type) -> Result<TypeIdentity, Fault> {
    crate::LoadedProgram::new(module)?.resolve_type_identity(ty)
}

/// Resolve using the same application/System linking rules as execution.
pub fn resolve_type_identity_with_library(
    module: &Module,
    library: &Module,
    ty: &Type,
) -> Result<TypeIdentity, Fault> {
    crate::LoadedProgram::with_library(module, library)?.resolve_type_identity(ty)
}

pub(crate) fn resolve(module: &Module, ty: &Type) -> Result<TypeIdentity, Fault> {
    let ty = crate::scope::normalize_type(module, ty)?;
    // Enforce canonical signatures, arity, closedness, and the shared nesting limit.
    crate::vm::check_type(&ty, module)?;
    crate::references::check_type(module, module, &ty)?;
    build(module, &ty)
}

fn build(module: &Module, ty: &Type) -> Result<TypeIdentity, Fault> {
    let nested = |ty: &Type| build(module, ty).map(Box::new);
    Ok(match ty {
        Type::Tuple(parts) => TypeIdentity::Tuple(
            parts
                .iter()
                .map(|t| build(module, t))
                .collect::<Result<_, _>>()?,
        ),
        Type::Union(parts) => TypeIdentity::Union(
            parts
                .iter()
                .map(|t| build(module, t))
                .collect::<Result<_, _>>()?,
        ),
        Type::Intersection(parts) => TypeIdentity::Intersection(
            parts
                .iter()
                .map(|t| build(module, t))
                .collect::<Result<_, _>>()?,
        ),
        Type::Function(shape) => TypeIdentity::Function {
            parameters: shape
                .parameters
                .iter()
                .map(|t| build(module, t))
                .collect::<Result<_, _>>()?,
            returns: nested(&shape.returns)?,
            no_result: shape.no_result,
            out_parameters: shape.out_parameters.clone(),
            out_when_true: shape.out_when_true.clone(),
        },
        Type::ByRef(t) => TypeIdentity::ByRef(nested(t)?),
        Type::ReadOnlyByRef(t) => TypeIdentity::ReadOnlyByRef(nested(t)?),
        Type::Array(t) => TypeIdentity::Array(nested(t)?),
        Type::ArrayRef(t) => TypeIdentity::ArrayRef(nested(t)?),
        Type::Ptr(t) => TypeIdentity::Ptr(nested(t)?),
        Type::InterfaceRef(t) => TypeIdentity::InterfaceRef(nested(t)?),
        _ => {
            let (name, arguments) = match ty {
                Type::Constructed {
                    definition,
                    arguments,
                } => (definition.as_str(), arguments.as_slice()),
                _ => (
                    ty.definition_name()
                        .ok_or_else(|| Fault::new("expected closed type signature"))?,
                    &[][..],
                ),
            };
            let definition = module
                .types
                .iter()
                .find(|d| d.name == name && d.generic_parameters.len() == arguments.len())
                .and_then(|d| d.definition.clone())
                .ok_or_else(|| {
                    Fault::new(format!("missing type definition identity for {name}"))
                })?;
            TypeIdentity::Definition {
                definition,
                arguments: arguments
                    .iter()
                    .map(|t| build(module, t))
                    .collect::<Result<_, _>>()?,
            }
        }
    })
}

fn reference_name(target: &Type, readonly: bool) -> Result<String, Fault> {
    let mut name = signature_name(target)?;
    if name.starts_with("readonly ") {
        name = format!("({name})");
    }
    Ok(format!(
        "{}{name}&",
        if readonly { "readonly " } else { "" }
    ))
}

/// A definition snapshot can describe open generic metadata without making it an
/// executable closed signature. Member substitution still requires closed types.
pub(crate) fn describe_definition(
    module: &Module,
    definition: &crate::metadata::TypeDef,
) -> Result<TypeDescriptor, Fault> {
    if definition.generic_parameters.is_empty() {
        return describe_loaded(module, &definition.open_type());
    }
    let id = definition
        .definition
        .clone()
        .ok_or_else(|| Fault::new("missing type definition identity"))?;
    let arguments = definition
        .generic_parameters
        .iter()
        .enumerate()
        .map(|(i, name)| {
            Ok(TypeDescriptor {
                identity: TypeIdentity::GenericParameter {
                    definition: id.clone(),
                    index: u16::try_from(i)
                        .map_err(|_| Fault::new("generic parameter index overflow"))?,
                },
                name: name.clone().unwrap_or_else(|| format!("T{i}")),
                generic_arguments: vec![],
                constituents: vec![],
                tuple_elements: vec![],
                tuple_element_names: vec![],
                declaring_type: None,
            })
        })
        .collect::<Result<Vec<_>, Fault>>()?;
    Ok(TypeDescriptor {
        identity: TypeIdentity::Definition {
            definition: id,
            arguments: arguments.iter().map(|a| a.identity.clone()).collect(),
        },
        name: definition
            .origin
            .as_ref()
            .map_or_else(|| definition.name.clone(), |o| o.name.clone()),
        generic_arguments: arguments,
        constituents: vec![],
        tuple_elements: vec![],
        tuple_element_names: vec![],
        declaring_type: declaring_definition(module, definition).and_then(|p| p.definition.clone()),
    })
}

pub(crate) fn declaring_definition<'a>(
    module: &'a Module,
    definition: &crate::metadata::TypeDef,
) -> Option<&'a crate::metadata::TypeDef> {
    if let Some(id) = &definition.declaring_type {
        return module
            .types
            .iter()
            .find(|d| d.definition.as_ref() == Some(id));
    }
    let origin = definition.origin.as_ref()?;
    let token = origin.declaring_type_token?;
    module.types.iter().find(|d| {
        d.origin.as_ref().is_some_and(|p| {
            p.assembly == origin.assembly && p.module == origin.module && p.token == token
        })
    })
}

#[cfg(test)]
mod composition_tests {
    use super::*;

    #[test]
    fn composition_keys_roundtrip_without_definition_tokens() {
        let module = crate::assemble(
            ".module Identities\n.revision first\n.interface Left\n.end\n.interface Right\n.end\n",
        )
        .unwrap();
        for ty in [
            Type::Union(vec![Type::from_name("Left"), Type::from_name("Right")]),
            Type::Intersection(vec![Type::from_name("Left"), Type::from_name("Right")]),
        ] {
            // Host and guest handles preserve the same structural snapshot.
            assert!(describe(&module, &ty).is_ok());
            assert!(describe_loaded(&module, &ty).is_ok());
            assert!(describe_host(&module, &ty).is_ok());
            let descriptor = describe_host(&module, &ty).unwrap();
            let result = crate::reflection::Query::Shape
                .invoke(
                    &module,
                    &[
                        crate::Value::RuntimeTypeHandle(Box::new(descriptor)),
                        crate::Value::Int32(14),
                    ],
                    &crate::Limits::default(),
                )
                .unwrap();
            assert_eq!(result, crate::Value::Boolean(matches!(ty, Type::Union(_))));
            let descriptor = describe(&module, &ty).unwrap();
            let args = [crate::Value::RuntimeTypeHandle(Box::new(
                descriptor.clone(),
            ))];
            let constituents = crate::reflection::Query::Constituents
                .invoke(&module, &args, &crate::Limits::default())
                .unwrap();
            let crate::Value::Array { elements, .. } = constituents else {
                panic!("missing constituents")
            };
            assert_eq!(elements.len(), 2);
            assert_eq!(descriptor.constituents.len(), 2);
            assert!(descriptor.generic_arguments.is_empty());
            for query in [
                crate::reflection::Query::Fields,
                crate::reflection::Query::Interfaces,
                crate::reflection::Query::BaseType,
            ] {
                assert!(
                    query
                        .invoke(&module, &args, &crate::Limits::default())
                        .unwrap_err()
                        .to_string()
                        .contains("composition member and hierarchy")
                );
            }
            let identity = resolve(&module, &ty).unwrap();
            assert_eq!(
                crate::reflection::from_identity(&module, &identity).unwrap(),
                ty
            );
            assert!(crate::metadata_tokens::definition(&module, &identity).is_none());
            assert_eq!(
                crate::metadata_tokens::type_token(&module, &identity).unwrap(),
                0
            );
            let other = crate::assemble(".module Identities\n.revision second\n.interface Left\n.end\n.interface Right\n.end\n").unwrap();
            assert!(crate::reflection::from_identity(&other, &identity).is_err());
        }
    }
}

#[cfg(test)]
mod tuple_tests {
    use super::*;

    #[test]
    fn tuple_elements_retain_nominal_module_revision_identity() {
        let make = |revision| {
            crate::assemble(&format!(".module TupleIdentity\n.revision {revision}\n.type class Cell\n.field Value Int32\n.end\n" )).unwrap()
        };
        let first = make("first");
        let second = make("second");
        let ty = Type::Tuple(vec![Type::from_name("Cell")].into());
        let first_identity = resolve(&first, &ty).unwrap();
        assert_ne!(first_identity, resolve(&second, &ty).unwrap());
        assert!(crate::reflection::from_identity(&second, &first_identity).is_err());
    }

    #[test]
    fn tuple_name_queries_preserve_signature_names_without_changing_identity() {
        let source = crate::assemble(
            ".module TupleNames\n.entry Main\n.function Main() -> Int32\nldc.i4 0\nret\n.end",
        )
        .unwrap();
        let program = crate::LoadedProgram::new(&source).unwrap();
        let module = crate::library::link(&source, crate::library::system().unwrap()).unwrap();
        let describe = |name: &str| {
            program
                .describe_type(&Type::Tuple(
                    crate::metadata::TupleSignature::new(
                        vec![Type::Int32, Type::Int32],
                        vec![Some(name.into()), None],
                    )
                    .unwrap(),
                ))
                .unwrap()
        };
        let left = describe("left");
        let renamed = describe("x");
        assert_eq!(left.identity, renamed.identity);
        for (descriptor, expected) in [(left, "left"), (renamed, "x")] {
            let args = [crate::Value::RuntimeTypeHandle(Box::new(descriptor))];
            for _ in 0..2 {
                let result = crate::reflection::Query::TupleNames
                    .invoke(&module, &args, &crate::Limits::default())
                    .unwrap();
                let crate::Value::Array { elements, .. } = result else {
                    panic!("missing tuple names")
                };
                assert_eq!(
                    elements,
                    vec![
                        crate::Value::String(expected.into()),
                        crate::Value::String("Item2".into())
                    ]
                );
            }
        }
        let nominal =
            crate::Value::RuntimeTypeHandle(Box::new(program.describe_type(&Type::Int32).unwrap()));
        assert!(
            crate::reflection::Query::TupleNames
                .invoke(&module, &[nominal], &crate::Limits::default())
                .unwrap_err()
                .to_string()
                .contains("requires a structural tuple")
        );
    }

    #[test]
    fn tuple_identity_preserves_elements_without_definition_tokens() {
        let source = crate::assemble(
            ".module TupleIdentity\n.entry Main\n.function Main() -> Int32\nldc.i4 0\nret\n.end",
        )
        .unwrap();
        let program = crate::LoadedProgram::new(&source).unwrap();
        let linked = crate::library::link(&source, crate::library::system().unwrap()).unwrap();
        let module = &linked;
        let ty = Type::Tuple(
            vec![
                Type::Int32,
                Type::String,
                Type::Tuple(vec![Type::Int32, Type::Int32].into()),
            ]
            .into(),
        );
        let descriptor = program.describe_type(&ty).unwrap();
        assert_eq!(
            descriptor.name,
            "tuple<System.Int32,System.String,tuple<System.Int32,System.Int32>>"
        );
        assert_eq!(descriptor.tuple_elements.len(), 3);
        let args = [crate::Value::RuntimeTypeHandle(Box::new(
            descriptor.clone(),
        ))];
        let elements = crate::reflection::Query::TupleElements
            .invoke(module, &args, &crate::Limits::default())
            .unwrap();
        let crate::Value::Array { elements, .. } = elements else {
            panic!("missing tuple elements")
        };
        assert_eq!(elements.len(), 3);
        assert_eq!(
            crate::reflection::Query::Shape
                .invoke(
                    module,
                    &[args[0].clone(), crate::Value::Int32(16)],
                    &crate::Limits::default()
                )
                .unwrap(),
            crate::Value::Boolean(true)
        );
        let nominal =
            crate::Value::RuntimeTypeHandle(Box::new(program.describe_type(&Type::Int32).unwrap()));
        assert!(
            crate::reflection::Query::TupleElements
                .invoke(module, &[nominal], &crate::Limits::default())
                .unwrap_err()
                .to_string()
                .contains("requires a structural tuple")
        );

        assert_eq!(descriptor.tuple_elements[2].tuple_elements.len(), 2);
        assert_eq!(
            descriptor.tuple_elements[2].tuple_elements[0],
            descriptor.tuple_elements[2].tuple_elements[1]
        );
        assert!(descriptor.generic_arguments.is_empty());
        assert!(descriptor.constituents.is_empty());
        assert!(descriptor.declaring_type.is_none());
        assert_eq!(
            descriptor.identity,
            program.resolve_type_identity(&ty).unwrap()
        );
        assert_eq!(
            crate::reflection::from_identity(module, &descriptor.identity).unwrap(),
            ty
        );
        assert!(crate::metadata_tokens::definition(module, &descriptor.identity).is_none());
        assert_eq!(
            crate::metadata_tokens::type_token(module, &descriptor.identity).unwrap(),
            0
        );
        for different in [
            Type::Tuple(
                vec![
                    Type::String,
                    Type::Int32,
                    Type::Tuple(vec![Type::Int32, Type::Int32].into()),
                ]
                .into(),
            ),
            Type::Tuple(vec![Type::Int32, Type::String, Type::Int32, Type::Int32].into()),
            Type::Tuple(
                vec![
                    Type::Int32,
                    Type::String,
                    Type::Tuple(vec![Type::Int32].into()),
                ]
                .into(),
            ),
        ] {
            assert_ne!(
                descriptor.identity,
                program.resolve_type_identity(&different).unwrap()
            );
        }
        assert_ne!(
            program
                .resolve_type_identity(&Type::Tuple(vec![Type::Int32].into()))
                .unwrap(),
            program.resolve_type_identity(&Type::Int32).unwrap()
        );
        assert!(
            program
                .describe_type(&Type::Int32)
                .unwrap()
                .tuple_elements
                .is_empty()
        );
    }
}
