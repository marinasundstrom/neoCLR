//! Shared scalar storage mapping for private calls and C import adapters.
//! Pointer admission is opt-in and macOS ARM64-only; this is not managed layout.
use cranelift_codegen::ir::types;
use neoclr::metadata::Type;

pub(crate) fn pointer_target(t: &Type) -> bool {
    matches!(t, Type::Void | Type::Byte | Type::Int32)
}

pub(crate) fn scalar_type(t: &Type) -> Option<cranelift_codegen::ir::Type> {
    match t {
        Type::Int32 => Some(types::I32),
        Type::Single => Some(types::F32),
        Type::Double => Some(types::F64),
        Type::Int64 | Type::UInt64 | Type::IntPtr | Type::UIntPtr => Some(types::I64),
        Type::Ptr(inner) if pointer_target(inner) => Some(types::I64),
        _ => None,
    }
}
