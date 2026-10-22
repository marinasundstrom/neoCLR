//! Shared scalar storage mapping for private calls and C import adapters.
//! Pointer admission is opt-in and macOS ARM64-only; this is not managed layout.
use cranelift_codegen::ir::types;
use neoclr::metadata::Type;

pub(crate) fn scalar_type(t: &Type) -> Option<cranelift_codegen::ir::Type> {
    match t {
        Type::Int32 => Some(types::I32),
        Type::Ptr(inner) if matches!(**inner, Type::Void | Type::Byte | Type::Int32) => {
            Some(types::I64)
        }
        _ => None,
    }
}
