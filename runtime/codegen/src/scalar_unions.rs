//! Bounded scalar union storage. Tags are private backend values, not metadata tokens.
use neoclr::metadata::{Function, Instruction as Op, Type};

pub(crate) fn tag(ty: &Type) -> Option<i64> {
    match ty {
        Type::Int32 => Some(1),
        Type::Int64 => Some(2),
        Type::Boolean => Some(3),
        _ => None,
    }
}

pub(crate) fn supported(ty: &Type) -> bool {
    matches!(ty, Type::Union(parts) if !parts.is_empty() && parts.iter().all(|t| tag(t).is_some()))
}

// Preserve both original instructions and their diagnostic positions. The native
// analyzer permits the pending scalar only as input to this following checked cast.
pub(crate) fn box_pair(f: &Function, pc: usize) -> bool {
    matches!((f.body.get(pc), f.body.get(pc + 1)),
        (Some(Op::BoxValue(t)), Some(Op::CastClass(union @ Type::Union(parts))))
        if tag(t).is_some() && supported(union) && parts.contains(t))
}

// Only bypass nominal unboxing projection when the immediate producer declares
// the structural union. Other Object queries retain the existing boxed path.
pub(crate) fn query(f: &Function, pc: usize) -> bool {
    let Some(previous) = pc.checked_sub(1).and_then(|i| f.body.get(i)) else {
        return false;
    };
    match previous {
        Op::Load(i) => f.locals.get(*i).is_some_and(supported),
        Op::Arg(i) => f.argument_types().get(*i).is_some_and(supported),
        Op::CastClass(t) => supported(t),
        _ => false,
    }
}
