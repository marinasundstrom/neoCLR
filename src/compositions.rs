//! Branch-only reference compositions. No value layout, byref or backend ABI.
use crate::{
    Fault, Module,
    metadata::{Representation, Type},
};

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
    // are allowed inside a reference construction. Compound arguments retain the
    // same reference-only operand rules; open parameters remain unsupported.
    fn argument(ty: &Type, depth: usize, nodes: &mut usize) -> Result<(), Fault> {
        if depth > 32 || *nodes == 0 {
            return Err(Fault::new("compound signature resource limit"));
        }
        *nodes -= 1;
        match ty {
            Type::Union(_) | Type::Intersection(_) => visit(ty, depth, nodes),
            Type::Named(_) => Ok(()),
            Type::Constructed { arguments, .. } | Type::Scoped { arguments, .. } => {
                if arguments.len() > 256 {
                    return Err(Fault::new("too many compound generic arguments"));
                }
                for arg in arguments {
                    argument(arg, depth + 1, nodes)?;
                }
                Ok(())
            }
            Type::ArrayRef(element) => argument(element, depth + 1, nodes),
            _ if ty.is_primitive() && !matches!(ty, Type::Void) => Ok(()),
            _ => Err(Fault::new(
                "composition generic arguments must be closed storage types",
            )),
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
                "compound constituents must be closed nominal references",
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
        _ => false,
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
                    || (def.representation == Representation::Record && def.is_reference_type)
                {
                    Ok(())
                } else {
                    Err(Fault::new(
                        "compound constituents must be class or interface references",
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
    if is_compound(target)
        && !is_compound(source)
        && !matches!(source, Type::Named(_) | Type::Constructed { .. })
    {
        return false;
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
            || crate::constraints::bounds(&def.generic_constraints).any(contains)
        {
            return Err(Fault::new(
                "compound static fields, inheritance and constraints are not supported",
            ));
        }
        for field in &def.fields {
            if contains(&field.ty) {
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
        .map(|function| (std::borrow::Cow::Borrowed(function), false))
        .collect();
    let mut visited = std::collections::HashSet::new();
    while let Some((function, instantiated)) = pending.pop_front() {
        let mut has_compound = false;
        function.map_types(|ty| {
            has_compound |= contains(ty);
            Ok(ty.clone())
        })?;
        if !has_compound {
            continue;
        }
        if function.owner.as_ref().is_some_and(|owner| {
            !instantiated
                && !module.type_definition(owner).is_some_and(|def| {
                    def.is_reference_type && def.representation == Representation::Record
                })
        }) || !function.generic_parameters.is_empty()
            || function.is_internal_call()
            || function.pinvoke.is_some()
        {
            return Err(Fault::new(
                "compound signatures require nongeneric managed free functions or reference-class methods",
            ));
        }
        if instantiated {
            function.map_types(|ty| {
                crate::vm::check_type(ty, module)?;
                Ok(ty.clone())
            })?;
        }
        for (index, op) in function.body.iter().enumerate() {
            match op {
                Op::LoadTypeToken(ty) if contains(ty) => {
                    crate::vm::check_type(ty, module)?;
                }
                Op::LocalAddress(i) if function.locals.get(*i).is_some_and(contains) => {
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
                    crate::vm::check_type(ty, module)?;
                    crate::initialization::default_value(module, ty)?;
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
                    crate::vm::check_type(&Type::ArrayRef(Box::new(ty.clone())), module)?;
                }
                Op::New(ty) if matches!(ty, Type::Constructed { .. }) && contains(ty) => {
                    crate::vm::check_type(ty, module)?;
                }
                Op::Construct(target) | Op::Call(target) | Op::CallVirtual(target)
                    if target.owner.is_some()
                        && (target.owner.as_ref().is_some_and(contains)
                            || target.parameters.iter().any(contains)) =>
                {
                    if !target.generic_arguments.is_empty()
                        || (!target.instance && target.owner.as_ref().is_some_and(contains))
                    {
                        return Err(Fault::new(format!(
                            "composition calls require nongeneric methods and instance wrapper receivers: {target:?}",
                        )));
                    }
                    let owner = target.owner.as_ref().unwrap();
                    crate::vm::check_type(owner, module)?;
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
                        pending.push_back((std::borrow::Cow::Owned(body), true));
                    }
                }
                Op::Call(target)
                    if target.owner.is_none() && target.generic_arguments.is_empty() =>
                {
                    ()
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
                            return Err(Fault::new(
                                "compound instruction operand is not supported",
                            ));
                        }
                        Ok(ty.clone())
                    })?;
                }
            }
        }
    }
    Ok(())
}
