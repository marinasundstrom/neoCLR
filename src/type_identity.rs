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
    FunctionReturn(Box<TypeIdentity>),
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
    /// A method parameter retains both its source member and declaring type view.
    MethodGenericParameter {
        definition: crate::metadata::MemberId,
        declaring_type: Option<Box<TypeIdentity>>,
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

impl TypeIdentity {
    pub(crate) fn contains_generic_parameters(&self) -> bool {
        match self {
            Self::GenericParameter { .. } | Self::MethodGenericParameter { .. } => true,
            Self::Definition { arguments, .. }
            | Self::Tuple(arguments)
            | Self::Union(arguments)
            | Self::Intersection(arguments) => {
                arguments.iter().any(Self::contains_generic_parameters)
            }
            Self::Function {
                parameters,
                returns,
                ..
            } => {
                parameters.iter().any(Self::contains_generic_parameters)
                    || returns.contains_generic_parameters()
            }
            Self::FunctionReturn(element)
            | Self::ByRef(element)
            | Self::ReadOnlyByRef(element)
            | Self::Array(element)
            | Self::ArrayRef(element)
            | Self::Ptr(element)
            | Self::InterfaceRef(element) => element.contains_generic_parameters(),
        }
    }

    pub(crate) fn is_generic_definition(&self) -> bool {
        matches!(self, Self::Definition { definition, arguments }
            if !arguments.is_empty() && arguments.iter().enumerate().all(|(position, argument)|
                matches!(argument, Self::GenericParameter { definition: owner, index }
                    if owner == definition && usize::from(*index) == position)))
    }
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

/// Recover the exact declaration identity of a nominal generic view.
pub(crate) fn generic_definition(
    module: &Module,
    descriptor: &TypeDescriptor,
) -> Result<TypeDescriptor, Fault> {
    let TypeIdentity::Definition {
        definition,
        arguments,
    } = &descriptor.identity
    else {
        return Err(Fault::new(
            "generic definition requires a nominal generic type",
        ));
    };
    if arguments.is_empty() {
        return Err(Fault::new("generic definition requires a generic type"));
    }
    let declaration = module
        .types
        .iter()
        .find(|d| d.definition.as_ref() == Some(definition))
        .ok_or_else(|| Fault::new("generic declaration is not loaded"))?;
    describe_definition(module, declaration)
}

/// Apply one argument to an open generic identity. Executable constraints are
/// checked only once the identity is fully closed.
pub(crate) fn apply_generic_argument(
    module: &Module,
    owner: &TypeDescriptor,
    index: i32,
    argument: &TypeDescriptor,
) -> Result<TypeDescriptor, Fault> {
    let TypeIdentity::Definition {
        definition,
        arguments,
    } = &owner.identity
    else {
        return Err(Fault::new(
            "generic construction requires a nominal definition",
        ));
    };
    let index = usize::try_from(index).map_err(|_| Fault::new("invalid generic argument index"))?;
    if !matches!(arguments.get(index), Some(TypeIdentity::GenericParameter { definition: declaring, index: ordinal }) if declaring == definition && usize::from(*ordinal) == index)
    {
        return Err(Fault::new("generic argument position is not unbound"));
    }
    if argument.identity.contains_generic_parameters() {
        return Err(Fault::new(
            "generic construction currently requires closed arguments",
        ));
    }
    let value = crate::reflection::from_identity(module, &argument.identity)?;
    if matches!(
        value,
        Type::Void
            | Type::ByRef(_)
            | Type::ReadOnlyByRef(_)
            | Type::Ptr(_)
            | Type::RuntimeTypeHandle
    ) {
        return Err(Fault::new("invalid generic type argument"));
    }
    let mut bound = arguments.clone();
    bound[index] = argument.identity.clone();
    let identity = TypeIdentity::Definition {
        definition: definition.clone(),
        arguments: bound,
    };
    if !identity.contains_generic_parameters() {
        let declaration = module
            .types
            .iter()
            .find(|d| d.definition.as_ref() == Some(definition))
            .ok_or_else(|| Fault::new("generic definition is not loaded"))?;
        let Type::Constructed { arguments, .. } =
            crate::reflection::from_identity(module, &identity)?
        else {
            return Err(Fault::new("invalid constructed identity"));
        };
        crate::constraints::check(module, &declaration.generic_constraints, &arguments)?;
    }
    describe_identity(module, &identity)
}

/// ldtoken may name a generic declaration without creating executable storage.
pub(crate) fn describe_token(module: &Module, ty: &Type) -> Result<TypeDescriptor, Fault> {
    let normalized = crate::scope::normalize_type(module, ty)?;
    if let Type::Named(name) = &normalized {
        if let Some(declaration) = module.types.iter().find(|d| &d.name == name) {
            if !declaration.generic_parameters.is_empty() {
                return describe_definition(module, declaration);
            }
        }
    }
    // The operand was checked in its declaring module, not the load-set root.
    describe_loaded(module, &normalized)
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

/// Describe a verified member signature without treating its open parameters as
/// executable types. Parameter identity includes the declaration, not just its name.
pub(crate) fn describe_member(
    module: &Module,
    ty: &Type,
    owner: Option<&TypeDescriptor>,
    method: Option<&crate::metadata::Function>,
) -> Result<TypeDescriptor, Fault> {
    if let Type::TypeParameter(index) = ty {
        return owner
            .and_then(|o| o.generic_arguments.get(usize::from(*index)))
            .cloned()
            .ok_or_else(|| Fault::new("missing declaring type parameter"));
    }
    if let Type::MethodTypeParameter(index) = ty {
        let method = method.ok_or_else(|| Fault::new("missing method parameter context"))?;
        if let Some(argument) = method.generic_arguments.get(usize::from(*index)) {
            // Constructed arguments are in the caller's scope. Do not recursively
            // substitute the same method parameter with itself.
            return describe_member(module, argument, owner, None);
        }
        let name = method
            .generic_parameters
            .get(usize::from(*index))
            .ok_or_else(|| Fault::new("missing declaring method parameter"))?
            .clone()
            .unwrap_or_else(|| format!("T{index}"));
        return Ok(TypeDescriptor {
            identity: TypeIdentity::MethodGenericParameter {
                definition: method
                    .definition
                    .clone()
                    .ok_or_else(|| Fault::new("missing method identity"))?,
                declaring_type: owner.map(|o| Box::new(o.identity.clone())),
                index: *index,
            },
            name,
            generic_arguments: vec![],
            constituents: vec![],
            tuple_elements: vec![],
            tuple_element_names: vec![],
            declaring_type: owner.and_then(|o| match &o.identity {
                TypeIdentity::Definition { definition, .. } => Some(definition.clone()),
                _ => None,
            }),
        });
    }
    if let Ok(closed) = describe_loaded(module, ty) {
        return Ok(closed);
    }
    let nested = |ty| describe_member(module, ty, owner, method);
    let many = |types: &[Type]| {
        types
            .iter()
            .map(|ty| describe_member(module, ty, owner, method))
            .collect::<Result<Vec<_>, _>>()
    };
    let mut result = TypeDescriptor {
        identity: TypeIdentity::Tuple(vec![]),
        name: String::new(),
        generic_arguments: vec![],
        constituents: vec![],
        tuple_elements: vec![],
        tuple_element_names: vec![],
        declaring_type: None,
    };
    match ty {
        Type::Constructed { arguments, .. } => {
            let declaration = module
                .type_definition(ty)
                .ok_or_else(|| Fault::new("missing open signature declaration"))?;
            if arguments.len() != declaration.generic_parameters.len() {
                return Err(Fault::new("open signature generic arity mismatch"));
            }
            result = describe_definition(module, declaration)?;
            result.generic_arguments = many(arguments)?;
            result.identity = TypeIdentity::Definition {
                definition: declaration
                    .definition
                    .clone()
                    .ok_or_else(|| Fault::new("missing type identity"))?,
                arguments: result
                    .generic_arguments
                    .iter()
                    .map(|a| a.identity.clone())
                    .collect(),
            };
        }
        Type::ByRef(t)
        | Type::ReadOnlyByRef(t)
        | Type::Array(t)
        | Type::ArrayRef(t)
        | Type::Ptr(t)
        | Type::InterfaceRef(t) => {
            let element = nested(t)?;
            let identity = Box::new(element.identity.clone());
            let (kind, name) = match ty {
                Type::ByRef(_) => (TypeIdentity::ByRef(identity), format!("{}&", element.name)),
                Type::ReadOnlyByRef(_) => (
                    TypeIdentity::ReadOnlyByRef(identity),
                    format!("readonly {}&", element.name),
                ),
                Type::Array(_) => (TypeIdentity::Array(identity), format!("{}[]", element.name)),
                Type::ArrayRef(_) => (
                    TypeIdentity::ArrayRef(identity),
                    format!("arrayref<{}>", element.name),
                ),
                Type::Ptr(_) => (TypeIdentity::Ptr(identity), format!("{}*", element.name)),
                _ => (
                    TypeIdentity::InterfaceRef(identity),
                    format!("InterfaceRef<{}>", element.name),
                ),
            };
            result.identity = kind;
            result.name = name;
            if matches!(ty, Type::ArrayRef(_)) && module.type_definition(ty).is_some() {
                result.generic_arguments.push(element);
            }
        }
        Type::Tuple(parts) => {
            result.tuple_elements = many(parts)?;
            result.identity = TypeIdentity::Tuple(
                result
                    .tuple_elements
                    .iter()
                    .map(|p| p.identity.clone())
                    .collect(),
            );
            result.name = format!(
                "tuple<{}>",
                result
                    .tuple_elements
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            result.tuple_element_names = (0..parts.len())
                .map(|i| parts.names().get(i).cloned().flatten())
                .collect();
        }
        Type::Union(parts) | Type::Intersection(parts) => {
            result.constituents = many(parts)?;
            let identities = result
                .constituents
                .iter()
                .map(|p| p.identity.clone())
                .collect();
            let union = matches!(ty, Type::Union(_));
            result.identity = if union {
                TypeIdentity::Union(identities)
            } else {
                TypeIdentity::Intersection(identities)
            };
            result.name = format!(
                "{}<{}>",
                if union { "union" } else { "intersection" },
                result
                    .constituents
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        Type::FunctionReturn(operand) => {
            let operand = nested(operand)?;
            result.name = format!("ReturnType<{}>", operand.name);
            result.identity = TypeIdentity::FunctionReturn(Box::new(operand.identity));
        }
        Type::Function(shape) => {
            let parameters = many(&shape.parameters)?;
            let returns = nested(&shape.returns)?;
            let mut names = parameters
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    format!(
                        "{}{}",
                        if shape.out_parameters.contains(&i) {
                            "out "
                        } else if shape.out_when_true.contains(&i) {
                            "outtrue "
                        } else {
                            ""
                        },
                        p.name
                    )
                })
                .collect::<Vec<_>>();
            names.push(if shape.no_result {
                "void".into()
            } else {
                returns.name
            });
            result.name = format!("fn<{}>", names.join(","));
            result.identity = TypeIdentity::Function {
                parameters: parameters.into_iter().map(|p| p.identity).collect(),
                returns: Box::new(returns.identity),
                no_result: shape.no_result,
                out_parameters: shape.out_parameters.clone(),
                out_when_true: shape.out_when_true.clone(),
            };
        }
        _ => return describe_loaded(module, ty),
    }
    Ok(result)
}

/// Reconstruct authenticated descriptive metadata from its source-owned identity.
/// This is not executable type construction and never relaxes signature admission.
pub(crate) fn describe_identity(
    module: &Module,
    identity: &TypeIdentity,
) -> Result<TypeDescriptor, Fault> {
    fn children(identity: &TypeIdentity) -> Vec<&TypeIdentity> {
        match identity {
            TypeIdentity::Definition { arguments, .. }
            | TypeIdentity::Tuple(arguments)
            | TypeIdentity::Union(arguments)
            | TypeIdentity::Intersection(arguments) => arguments.iter().collect(),
            TypeIdentity::Function {
                parameters,
                returns,
                ..
            } => parameters
                .iter()
                .chain(std::iter::once(returns.as_ref()))
                .collect(),
            TypeIdentity::Array(t)
            | TypeIdentity::ArrayRef(t)
            | TypeIdentity::ByRef(t)
            | TypeIdentity::ReadOnlyByRef(t)
            | TypeIdentity::Ptr(t)
            | TypeIdentity::InterfaceRef(t) => vec![t],
            _ => vec![],
        }
    }
    fn parameter(identity: &TypeIdentity, method: bool) -> Option<&TypeIdentity> {
        if matches!(
            (identity, method),
            (TypeIdentity::MethodGenericParameter { .. }, true)
                | (TypeIdentity::GenericParameter { .. }, false)
        ) {
            return Some(identity);
        }
        children(identity)
            .into_iter()
            .find_map(|child| parameter(child, method))
    }
    let ty = crate::reflection::from_identity(module, identity)?;
    if !identity.contains_generic_parameters() {
        return describe_loaded(module, &ty);
    }
    if let Some(TypeIdentity::MethodGenericParameter {
        definition,
        declaring_type,
        ..
    }) = parameter(identity, true)
    {
        let method = module
            .functions
            .iter()
            .find(|f| f.definition.as_ref() == Some(definition))
            .ok_or_else(|| Fault::new("missing method descriptor declaration"))?;
        let owner = declaring_type
            .as_deref()
            .map(|id| describe_identity(module, id))
            .transpose()?;
        let declared_owner = method
            .owner
            .as_ref()
            .and_then(|ty| module.type_definition(ty))
            .and_then(|definition| definition.definition.as_ref());
        let supplied_owner = owner
            .as_ref()
            .and_then(|descriptor| match &descriptor.identity {
                TypeIdentity::Definition { definition, .. } => Some(definition),
                _ => None,
            });
        if declared_owner != supplied_owner || (method.owner.is_none() && owner.is_some()) {
            return Err(Fault::new(
                "method parameter has an unrelated declaring type",
            ));
        }
        return describe_member(module, &ty, owner.as_ref(), Some(method));
    }
    let Some(TypeIdentity::GenericParameter { definition, .. }) = parameter(identity, false) else {
        return Err(Fault::new("missing generic descriptor context"));
    };
    let owner = module
        .types
        .iter()
        .find(|d| d.definition.as_ref() == Some(definition))
        .ok_or_else(|| Fault::new("missing generic descriptor declaration"))?;
    let owner = describe_definition(module, owner)?;
    describe_member(module, &ty, Some(&owner), None)
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
        Type::FunctionReturn(operand) => format!("ReturnType<{}>", signature_name(operand)?),
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
            parts.push(if shape.no_result {
                "void".into()
            } else {
                signature_name(&shape.returns)?
            });
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

// Execution already validated each instruction against its declaring module. Runtime
// storage may belong to a transitive dependency that the root cannot name directly.
// Keep host resolution above strict; only normalized execution types use this path.
pub(crate) fn resolve_loaded(module: &Module, ty: &Type) -> Result<TypeIdentity, Fault> {
    crate::vm::check_type(ty, module)?;
    build(module, ty)
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

// The table is compiler-owned and absent from serialized programs. Checking it
// here keeps ordinary interpreter builds independent of the native bridge feature.
pub(crate) fn retained_descriptor(module: &Module, index: u32) -> Result<&TypeDescriptor, Fault> {
    module
        .native_descriptors
        .get(index as usize)
        .ok_or_else(|| Fault::new("metadata token has no retained descriptor"))
}

#[cfg(test)]
mod generic_state_tests {
    use super::*;

    #[test]
    fn array_construction_preserves_elements_and_rejects_invalid_storage() {
        let module = crate::assemble(".module Arrays\n").unwrap();
        let module =
            crate::library::link_modules(&module, crate::library::system().unwrap(), &[]).unwrap();
        for element in [
            Type::Int32,
            Type::String,
            Type::ArrayRef(Box::new(Type::Int32)),
        ] {
            let handle = describe(&module, &element).unwrap();
            let result = crate::reflection::Query::MakeArray
                .invoke(
                    &module,
                    &[crate::Value::RuntimeTypeHandle(Box::new(handle))],
                    &crate::Limits::default(),
                )
                .unwrap();
            let crate::Value::RuntimeTypeHandle(array) = result else {
                panic!("array handle")
            };
            assert_eq!(
                *array,
                describe(&module, &Type::ArrayRef(Box::new(element))).unwrap()
            );
        }
        for element in [
            Type::Void,
            Type::ByRef(Box::new(Type::Int32)),
            Type::RuntimeTypeHandle,
        ] {
            let handle = describe(&module, &element).unwrap();
            assert!(
                crate::reflection::Query::MakeArray
                    .invoke(
                        &module,
                        &[crate::Value::RuntimeTypeHandle(Box::new(handle))],
                        &crate::Limits::default()
                    )
                    .is_err()
            );
        }
    }

    #[test]
    fn definition_tokens_are_descriptive_not_open_storage() {
        let mut module = crate::assemble(".module Definitions\n.entry Main\n.type Box<T>\n.end\n.function Main() -> RuntimeTypeHandle\ncall Definition()\nret\n.end\n.function Definition() -> RuntimeTypeHandle\nldtoken Box\nret\n.end\n").unwrap();
        module.references = Some(vec![]);
        let program = crate::LoadedProgram::new(&module).unwrap();
        program.verify().unwrap();
        let crate::Value::RuntimeTypeHandle(handle) =
            program.run(crate::Limits::default()).unwrap().value
        else {
            panic!("type handle");
        };
        assert!(handle.identity.is_generic_definition());
        assert!(resolve(&module, &Type::from_name("Box")).is_err());
    }

    #[test]
    fn generic_construction_rejects_open_arguments_and_enforces_constraints() {
        let module = crate::assemble(".module Definitions\n.type ValueOnly<T>\n.constraint T value\n.end\n.type Atom\n.end\n").unwrap();
        let linked =
            crate::library::link_modules(&module, crate::library::system().unwrap(), &[]).unwrap();
        let module = &linked;
        let definition = describe_token(module, &Type::from_name("ValueOnly")).unwrap();
        let integer = describe(module, &Type::Int32).unwrap();
        let closed = apply_generic_argument(module, &definition, 0, &integer).unwrap();
        assert!(!closed.identity.contains_generic_parameters());
        assert!(apply_generic_argument(module, &definition, 1, &integer).is_err());
        assert!(apply_generic_argument(module, &closed, 0, &integer).is_err());
        assert!(apply_generic_argument(module, &definition, 0, &definition).is_err());
        let reference = describe(module, &Type::String).unwrap();
        assert!(apply_generic_argument(module, &definition, 0, &reference).is_err());
    }

    #[test]
    fn generic_definition_preserves_declaration_identity() {
        let module = crate::assemble(
            ".module Definitions\n.type Box<T>\n.end\n.type Other<T>\n.end\n.type Atom\n.end\n",
        )
        .unwrap();
        let closed = describe_loaded(
            &module,
            &Type::Constructed {
                definition: "Box".into(),
                arguments: vec![Type::from_name("Atom")],
            },
        )
        .unwrap();
        let definition = generic_definition(&module, &closed).unwrap();
        assert!(definition.identity.is_generic_definition());
        assert_eq!(
            generic_definition(&module, &definition).unwrap(),
            definition
        );
        assert_ne!(
            definition,
            describe_definition(&module, &module.types[1]).unwrap()
        );
        assert!(
            generic_definition(
                &module,
                &describe_loaded(&module, &Type::from_name("Atom")).unwrap()
            )
            .is_err()
        );
        assert!(generic_definition(&module, &definition.generic_arguments[0]).is_err());
    }

    #[test]
    fn generic_state_uses_owner_identity_and_recurses_through_structural_types() {
        let owner = TypeDefId {
            module: "M".into(),
            revision: Some("1".into()),
            index: 0,
        };
        let parameter = TypeIdentity::GenericParameter {
            definition: owner.clone(),
            index: 0,
        };
        let open = TypeIdentity::Definition {
            definition: owner.clone(),
            arguments: vec![parameter.clone()],
        };
        assert!(open.is_generic_definition());
        assert!(open.contains_generic_parameters());
        let closed = TypeIdentity::Definition {
            definition: owner.clone(),
            arguments: vec![],
        };
        assert!(!closed.is_generic_definition());
        assert!(!closed.contains_generic_parameters());
        let other = TypeDefId {
            index: 1,
            ..owner.clone()
        };
        let constructed = TypeIdentity::Definition {
            definition: other,
            arguments: vec![parameter.clone()],
        };
        assert!(!constructed.is_generic_definition());
        assert!(constructed.contains_generic_parameters());
        for wrapper in [
            TypeIdentity::ArrayRef(Box::new(parameter.clone())),
            TypeIdentity::ByRef(Box::new(parameter.clone())),
            TypeIdentity::ReadOnlyByRef(Box::new(parameter.clone())),
            TypeIdentity::Ptr(Box::new(parameter.clone())),
            TypeIdentity::Tuple(vec![closed.clone(), parameter.clone()]),
            TypeIdentity::Union(vec![closed.clone(), parameter.clone()]),
            TypeIdentity::Intersection(vec![closed.clone(), parameter.clone()]),
            TypeIdentity::Function {
                parameters: vec![closed],
                returns: Box::new(parameter),
                no_result: false,
                out_parameters: vec![],
                out_when_true: vec![],
            },
        ] {
            assert!(wrapper.contains_generic_parameters(), "{wrapper:?}");
            assert!(!wrapper.is_generic_definition(), "{wrapper:?}");
        }
    }
}
