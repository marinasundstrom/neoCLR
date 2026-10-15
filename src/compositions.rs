//! Structural compositions. Value union alternatives use explicit boxing in the interpreter.
//! Intersections remain nominal reference views; direct managed parameter slots are supported.
use crate::{
    Fault, Module,
    metadata::{Representation, Type},
};

fn scalar_union_alternative(ty: &Type) -> bool {
    ty.is_primitive()
        && !matches!(
            ty,
            Type::Void | Type::Value | Type::RuntimeTypeHandle | Type::String
        )
}

pub(crate) fn is_compound(ty: &Type) -> bool {
    matches!(ty, Type::Union(_) | Type::Intersection(_))
}

pub(crate) fn contains(ty: &Type) -> bool {
    match ty {
        Type::Union(_) | Type::Intersection(_) => true,
        Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } => {
            arguments.iter().any(contains)
        }
        Type::Function(f) => f.parameters.iter().any(contains) || contains(&f.returns),
        Type::ByRef(t)
        | Type::ReadOnlyByRef(t)
        | Type::Ptr(t)
        | Type::Array(t)
        | Type::ArrayRef(t)
        | Type::InterfaceRef(t) => contains(t),
        _ => false,
    }
}

// Called before flattening so malformed/oversized source cannot normalize away.
pub(crate) fn check_shape(ty: &Type) -> Result<(), Fault> {
    if !cfg!(feature = "experimental-compositions") {
        return Err(Fault::new(
            "compound signatures require experimental-compositions",
        ));
    }
    // Generic argument syntax is distinct from a composition operand: value arguments
    // and scoped parameters are allowed inside nominal constructions. Compound
    // operands retain their own rules; open union/intersection operands stay unsupported.
    fn argument(ty: &Type, depth: usize, nodes: &mut usize) -> Result<(), Fault> {
        if depth > 32 || *nodes == 0 {
            return Err(Fault::new("compound signature resource limit"));
        }
        *nodes -= 1;
        match ty {
            Type::Union(_) | Type::Intersection(_) => visit(ty, depth, nodes),
            Type::Named(_) | Type::TypeParameter(_) | Type::MethodTypeParameter(_) => Ok(()),
            _ if scalar_union_alternative(ty) => Ok(()),
            Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } => {
                if arguments.len() > 256 {
                    return Err(Fault::new("too many compound generic arguments"));
                }
                for arg in arguments {
                    argument(arg, depth + 1, nodes)?;
                }
                Ok(())
            }
            Type::Function(shape) => {
                shape.validate()?;
                for parameter in &shape.parameters {
                    argument(parameter, depth + 1, nodes)?;
                }
                if !shape.no_result {
                    argument(&shape.returns, depth + 1, nodes)?;
                }
                Ok(())
            }
            Type::ArrayRef(element) => argument(element, depth + 1, nodes),
            _ if ty.is_primitive() => Ok(()),
            _ => Err(Fault::new(format!(
                "unsupported composition generic argument: {ty:?}"
            ))),
        }
    }
    fn visit(ty: &Type, depth: usize, nodes: &mut usize) -> Result<(), Fault> {
        if depth > 32 || *nodes == 0 {
            return Err(Fault::new("compound signature resource limit"));
        }
        *nodes -= 1;
        match ty {
            Type::Union(parts) | Type::Intersection(parts) => {
                if !(2..=256).contains(&parts.len()) {
                    return Err(Fault::new("compound signature requires 2 to 256 operands"));
                }
                for part in parts {
                    visit(part, depth + 1, nodes)?;
                }
                Ok(())
            }
            Type::Named(_) => Ok(()),
            _ if scalar_union_alternative(ty) => Ok(()),
            Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } => {
                if arguments.len() > 256 {
                    return Err(Fault::new("too many compound generic arguments"));
                }
                for arg in arguments {
                    argument(arg, depth + 1, nodes)?;
                }
                Ok(())
            }
            _ => Err(Fault::new(
                "compound constituents must be closed nominal or supported scalar types",
            )),
        }
    }
    visit(ty, 0, &mut 4096)
}

pub(crate) fn normalized(union: bool, parts: Vec<Type>) -> Type {
    let mut flat = Vec::new();
    for part in parts {
        match part {
            Type::Union(parts) if union => flat.extend(parts),
            Type::Intersection(parts) if !union => flat.extend(parts),
            other => flat.push(other),
        }
    }
    flat.sort_by_cached_key(|t| serde_json::to_string(t).expect("type serializes"));
    flat.dedup();
    if flat.len() == 1 {
        return flat.pop().unwrap();
    }
    if union {
        Type::Union(flat)
    } else {
        Type::Intersection(flat)
    }
}

pub(crate) fn resolved(module: &Module, ty: &Type) -> bool {
    match ty {
        Type::Union(parts) | Type::Intersection(parts) => parts.iter().all(|t| resolved(module, t)),
        Type::Named(_) | Type::Constructed { .. } => module.type_definition(ty).is_some(),
        _ => scalar_union_alternative(ty),
    }
}

pub(crate) fn validate(module: &Module, ty: &Type) -> Result<(), Fault> {
    check_shape(ty)?;
    fn visit(module: &Module, ty: &Type, intersection_bound: bool) -> Result<(), Fault> {
        match ty {
            Type::Union(parts) | Type::Intersection(parts) => {
                for part in parts {
                    visit(
                        module,
                        part,
                        intersection_bound || matches!(ty, Type::Intersection(_)),
                    )?;
                }
                Ok(())
            }
            _ if !intersection_bound && scalar_union_alternative(ty) => Ok(()),
            Type::Named(_) | Type::Constructed { .. } => {
                crate::vm::check_type(ty, module)?;
                let def = module
                    .type_definition(ty)
                    .ok_or_else(|| Fault::new("unknown compound constituent"))?;
                if intersection_bound && def.is_sealed {
                    return Err(Fault::new(
                        "intersection bounds must be interfaces or reference classes that permit subclassing",
                    ));
                }
                if def.representation == Representation::Interface
                    || (def.representation == Representation::Record
                        && (def.is_reference_type || !intersection_bound))
                {
                    Ok(())
                } else {
                    Err(Fault::new(
                        "compound constituents must be records or interfaces; intersections require references",
                    ))
                }
            }
            _ => Err(Fault::new("unresolved compound constituent")),
        }
    }
    visit(module, ty, false)
}

pub(crate) fn assignable(module: &Module, source: &Type, target: &Type) -> bool {
    if source == target {
        return true;
    }
    match (source, target) {
        (Type::Union(parts), _) => parts.iter().all(|p| assignable(module, p, target)),
        (_, Type::Intersection(parts)) => parts.iter().all(|p| assignable(module, source, p)),
        (_, Type::Union(parts)) => parts.iter().any(|p| assignable(module, source, p)),
        (Type::Intersection(parts), _) => parts.iter().any(|p| assignable(module, p, target)),
        _ => module.reference_assignable(source, target),
    }
}

pub(crate) fn matches_object(module: &Module, concrete: &Type, target: &Type) -> bool {
    if let Type::Union(parts) = target {
        return parts
            .iter()
            .any(|part| concrete == part || matches_object(module, concrete, part));
    }
    module
        .type_definition(concrete)
        .is_some_and(|d| d.is_reference_type && d.representation == Representation::Record)
        && assignable(module, concrete, target)
}

// Deliberately narrow interpreter surface. Inspect before execution, including dead
// instructions, so compiler omission is never the only enforcement boundary.
pub(crate) fn validate_surface(module: &Module) -> Result<(), Fault> {
    if !cfg!(feature = "experimental-compositions") {
        return Ok(());
    }
    use crate::metadata::Instruction as Op;
    for def in &module.types {
        if def.static_fields.iter().any(|f| contains(&f.ty))
            || def.base.iter().chain(&def.implements).any(contains)
        {
            return Err(Fault::new(
                "compound static fields and inheritance are not supported",
            ));
        }
        for field in &def.fields {
            if contains(&field.ty) {
                if matches!(&field.ty, Type::ByRef(_) | Type::ReadOnlyByRef(_)) {
                    return Err(Fault::new(
                        "compound managed-reference fields are not supported",
                    ));
                }
                if !def.is_reference_type || def.representation != Representation::Record {
                    return Err(Fault::new(
                        "compound fields require reference-class storage",
                    ));
                }
                crate::vm::check_type(&field.ty, module)?;
            }
        }
    }
    // Inspect closed bodies as well as source definitions: substituting T can
    // introduce compound addresses or unsupported instructions in an ordinary method.
    let mut pending: std::collections::VecDeque<_> = module
        .functions
        .iter()
        .map(|function| (std::borrow::Cow::Borrowed(function), None))
        .collect();
    let mut visited = std::collections::HashSet::new();
    while let Some((function, inherited_context)) = pending.pop_front() {
        let mut has_compound = false;
        // Bounds constrain instantiation; they do not introduce compound storage
        // into an otherwise ordinary generic method body.
        let mut executable = function.as_ref().clone();
        executable.generic_constraints.clear();
        executable.map_types(|ty| {
            has_compound |= contains(ty);
            Ok(ty.clone())
        })?;
        if !has_compound {
            continue;
        }
        if function.owner.as_ref().is_some_and(|owner| {
            inherited_context.is_none()
                && !module.type_definition(owner).is_some_and(|def| {
                    def.is_reference_type && def.representation == Representation::Record
                })
        }) || function.is_internal_call()
            || function.pinvoke.is_some()
        {
            return Err(Fault::new(
                "compound signatures require managed free functions or reference-class methods",
            ));
        }
        let context = inherited_context.unwrap_or(crate::vm::SignatureContext {
            types: function
                .owner
                .as_ref()
                .and_then(|owner| module.type_definition(owner))
                .map_or(0, |owner| owner.generic_parameters.len()),
            methods: function.generic_parameters.len(),
            allow_self: false,
        });
        let check = |ty: &Type| {
            crate::vm::check_type_context(ty, module, context, 0).map_err(|error| {
                Fault::new(format!(
                    "{} in structural body {} checking {ty:?}",
                    error, function.name
                ))
            })
        };
        if inherited_context.is_some() {
            function.map_types(|ty| {
                check(ty)?;
                Ok(ty.clone())
            })?;
        }
        for (index, op) in function.body.iter().enumerate() {
            match op {
                Op::LoadTypeToken(ty) if contains(ty) => {
                    check(ty)?;
                }
                Op::LocalAddress(i) if function.locals.get(*i).is_some_and(contains) => {
                    if function
                        .locals
                        .get(*i)
                        .is_some_and(|ty| is_compound(ty) || matches!(ty, Type::Constructed { .. }))
                    {
                        continue;
                    }
                    // Admit the compiler's immediate local-default idiom only.
                    // No instruction can retain or forward this transient address.
                    if !matches!(function.body.get(index + 1), Some(Op::InitializeObject(ty))
                        if function.locals.get(*i) == Some(ty))
                    {
                        return Err(Fault::new(
                            "compound local addresses require immediate initialization",
                        ));
                    }
                }
                Op::ArgumentAddress(i) if function.parameters.get(*i).is_some_and(contains) => {
                    return Err(Fault::new("compound argument addresses are not supported"));
                }
                Op::InitializeObject(ty) if contains(ty) => {
                    if !matches!(index.checked_sub(1).and_then(|i| function.body.get(i)),
                        Some(Op::LocalAddress(local)) if function.locals.get(*local) == Some(ty))
                    {
                        return Err(Fault::new(
                            "compound initialization requires an immediate local address",
                        ));
                    }
                    check(ty)?;
                    crate::initialization::default_value(module, ty)?;
                }
                Op::LoadObject(ty) | Op::StoreObject(ty)
                    if is_compound(ty) || matches!(ty, Type::Constructed { .. }) =>
                {
                    check(ty)?;
                }
                Op::CastClass(_) | Op::IsInstance(_) => (),
                Op::NewArray(ty)
                | Op::ReserveArray(ty)
                | Op::ArrayElement(ty)
                | Op::StoreArrayElement(ty)
                    if contains(ty) =>
                {
                    // Reference vectors reuse checked element storage and exact
                    // element identity. Element addresses/value arrays stay gated.
                    check(&Type::ArrayRef(Box::new(ty.clone())))?;
                }
                Op::BindFunction {
                    function_type,
                    target,
                } if contains(function_type)
                    || target.owner.as_ref().is_some_and(contains)
                    || target
                        .generic_arguments
                        .iter()
                        .chain(&target.parameters)
                        .any(contains) =>
                {
                    check(function_type)?;
                    if visited.insert(target.clone()) {
                        if visited.len() > 256 {
                            return Err(Fault::new("composition callback graph exceeds limit"));
                        }
                        let body = crate::vm::resolve(module, target)?;
                        pending.push_back((std::borrow::Cow::Owned(body), Some(context)));
                    }
                }
                Op::Call(target) | Op::CallVirtual(target)
                    if matches!(&target.owner, Some(Type::Function(_))) =>
                {
                    check(target.owner.as_ref().unwrap())?;
                }
                Op::New(ty) if matches!(ty, Type::Constructed { .. }) && contains(ty) => {
                    check(ty)?;
                }
                Op::Construct(target) | Op::Call(target) | Op::CallVirtual(target)
                    if target.owner.is_some()
                        && (target.owner.as_ref().is_some_and(contains)
                            || target.parameters.iter().any(contains)) =>
                {
                    let owner = target.owner.as_ref().unwrap();
                    check(owner)?;
                    if module.type_definition(owner).is_some_and(|definition| {
                        definition.representation == Representation::Interface
                    }) {
                        return Err(Fault::new(
                            "composition argument interface dispatch is not supported",
                        ));
                    }
                    if visited.insert(target.clone()) {
                        if visited.len() > 256 {
                            return Err(Fault::new("composition wrapper call graph exceeds limit"));
                        }
                        let body = if matches!(op, Op::Construct(_)) {
                            crate::vm::resolve_constructor(module, target)?
                        } else {
                            crate::vm::resolve(module, target)?
                        };
                        if body.is_virtual
                            || body.is_override
                            || body.is_abstract
                            || body.is_internal_call()
                            || body.pinvoke.is_some()
                        {
                            return Err(Fault::new(
                                "composition wrapper calls require nonvirtual managed bodies",
                            ));
                        }
                        pending.push_back((std::borrow::Cow::Owned(body), Some(context)));
                    }
                }
                Op::Call(target) if target.owner.is_none() => {
                    if !target.generic_arguments.is_empty() && visited.insert(target.clone()) {
                        if visited.len() > 256 {
                            return Err(Fault::new("composition call graph exceeds limit"));
                        }
                        pending.push_back((
                            std::borrow::Cow::Owned(crate::vm::resolve(module, target)?),
                            Some(context),
                        ));
                    }
                }
                _ => {
                    // Reuse the authoritative operand traversal. Only functions
                    // containing compounds enter this path; product code avoids it.
                    let mut operand = function.as_ref().clone();
                    operand.owner = None;
                    operand.parameters.clear();
                    operand.locals.clear();
                    operand.returns = Type::Void;
                    operand.generic_constraints.clear();
                    operand.interface_implementations.clear();
                    operand.body = vec![op.clone()];
                    operand.map_types(|ty| {
                        if contains(ty) {
                            return Err(Fault::new(format!(
                                "compound instruction operand is not supported in {}: {op:?}",
                                function.name,
                            )));
                        }
                        Ok(ty.clone())
                    })?;
                }
            }
        }
    }
    Ok(())
}
