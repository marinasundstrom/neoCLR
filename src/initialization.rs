//! Typed default initialization for managed storage, independent of native layout.
use crate::{Fault, Module, Value, metadata::Type};

pub(crate) fn default_value(module: &Module, ty: &Type) -> Result<Value, Fault> {
    fn build(
        module: &Module,
        ty: &Type,
        path: &mut Vec<Type>,
        remaining: &mut usize,
    ) -> Result<Value, Fault> {
        if *remaining == 0 || path.len() >= 64 {
            return Err(Fault::new(
                "default initialization depth or complexity limit exceeded",
            ));
        }
        *remaining -= 1;
        Ok(match ty {
            Type::Void => Value::Void,
            Type::Boolean => Value::Boolean(false),
            Type::SByte => Value::SByte(0),
            Type::Byte => Value::Byte(0),
            Type::Int16 => Value::Int16(0),
            Type::UInt16 => Value::UInt16(0),
            Type::Char => Value::Char(0),
            Type::Int32 => Value::Int32(0),
            Type::UInt32 => Value::UInt32(0),
            Type::Int64 => Value::Int64(0),
            Type::UInt64 => Value::UInt64(0),
            Type::IntPtr => Value::IntPtr(0),
            Type::UIntPtr => Value::UIntPtr(0),
            Type::Single => Value::Single(0.0),
            Type::Double => Value::Double(0.0),
            Type::Ptr(target) => Value::Pointer(crate::memory::Pointer::null((**target).clone())),
            Type::Named(_) | Type::Constructed { .. } => {
                if path.contains(ty) {
                    return Err(Fault::new("recursive default initialization"));
                }
                let definitions = crate::vm::record_fields(module, ty, 0)?;
                path.push(ty.clone());
                let fields = definitions
                    .iter()
                    .map(|field| build(module, &field.ty, path, remaining))
                    .collect::<Result<Vec<_>, _>>()?;
                path.pop();
                Value::Object {
                    ty: ty.clone(),
                    fields,
                }
            }
            _ => {
                return Err(Fault::new(format!(
                    "managed default initialization is not defined for {ty:?}"
                )));
            }
        })
    }
    build(module, ty, &mut vec![], &mut 16_384)
}
