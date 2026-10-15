//! Bounded scalar and mixed union storage. Tags are private backend values, not metadata tokens.
use neoclr::Module;
use neoclr::metadata::{Function, Instruction as Op, Representation, Type};

pub(crate) fn tag(ty: &Type) -> Option<i64> {
    match ty {
        Type::Int32 => Some(1),
        Type::Int64 => Some(2),
        Type::Boolean => Some(3),
        _ => None,
    }
}

// Floating alternatives retain their own numeric lanes rather than converting to
// the compact integer payload. Single and Double remain distinct alternatives.
pub(crate) fn bank_scalar(ty: &Type) -> bool {
    matches!(ty, Type::Single | Type::Double)
}

// Named alternatives are candidates here; Profile checks their exact storage
// family before admitting a mixed or record-union layout.
pub(crate) fn supported(ty: &Type) -> bool {
    matches!(ty, Type::Union(parts) if !parts.is_empty() && parts.iter().any(|t| tag(t).is_some())
        && parts.iter().all(|t| tag(t).is_some() || matches!(t, Type::Named(_))))
}

pub(crate) fn record(module: &Module, ty: &Type) -> bool {
    module.type_definition(ty).is_some_and(|d| {
        !d.is_reference_type && d.representation == Representation::Record && d.enum_info.is_none()
    })
}

// Records and callable descriptors use disjoint, independently traced payload regions.
pub(crate) fn payloads(module: &Module, ty: &Type) -> bool {
    matches!(ty, Type::Union(parts) if parts.iter().any(|t| record(module, t) || bank_scalar(t) || matches!(t, Type::Function(_) | Type::String))
        && parts.iter().all(|t| tag(t).is_some() || record(module, t) || bank_scalar(t) || matches!(t, Type::Function(_) | Type::String)
            || module.type_definition(t).is_some_and(|d| d.is_reference_type || d.representation == Representation::Interface)))
}

pub(crate) fn inline(module: &Module, ty: &Type) -> bool {
    supported(ty) || payloads(module, ty)
}

// Preserve both original instructions and their diagnostic positions. The native
// analyzer permits the pending scalar only as input to this following checked cast.
pub(crate) fn box_pair(module: &Module, f: &Function, pc: usize) -> bool {
    matches!((f.body.get(pc), f.body.get(pc + 1)),
        (Some(Op::BoxValue(t)), Some(Op::CastClass(union @ Type::Union(parts))))
        if (tag(t).is_some() || bank_scalar(t) || record(module, t)) && inline(module, union) && parts.contains(t))
}

// Only bypass nominal unboxing projection when the immediate producer declares
// the structural union. Other Object queries retain the existing boxed path.
pub(crate) fn query(module: &Module, f: &Function, pc: usize) -> bool {
    let Some(previous) = pc.checked_sub(1).and_then(|i| f.body.get(i)) else {
        return false;
    };
    match previous {
        Op::Load(i) => f.locals.get(*i).is_some_and(|t| inline(module, t)),
        Op::Arg(i) => f
            .argument_types()
            .get(*i)
            .is_some_and(|t| inline(module, t)),
        Op::CastClass(t) => inline(module, t),
        _ => false,
    }
}
