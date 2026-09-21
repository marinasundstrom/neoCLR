//! Typed default initialization for managed storage, independent of native layout.
use crate::{Fault, Module, Value, metadata::Type};

pub(crate) fn default_value(module: &Module, ty: &Type) -> Result<Value, Fault> {
    build_default(module, ty, false)
}

// A reference object can exist while its constructor assigns its fields.
// Types without a readable default retain checked, uninitialized storage instead.
pub(crate) fn object_field_value(module: &Module, ty: &Type) -> Result<Value, Fault> {
    build_default(module, ty, true)
}

fn build_default(module: &Module, ty: &Type, allow_uninitialized: bool) -> Result<Value, Fault> {
    fn build(
        module: &Module,
        ty: &Type,
        path: &mut Vec<Type>,
        remaining: &mut usize,
        allow_uninitialized: bool,
    ) -> Result<Value, Fault> {
        if *remaining == 0 || path.len() >= 64 {
            return Err(Fault::new(
                "default initialization depth or complexity limit exceeded",
            ));
        }
        *remaining -= 1;
        if module.is_object_reference_type(ty) {
            return Ok(Value::NullObjectReference(ty.clone()));
        }
        if allow_uninitialized
            && module.type_definition(ty).is_some_and(|definition| {
                definition.representation == crate::metadata::Representation::Delegate
            })
        {
            return Ok(Value::Uninitialized(ty.clone()));
        }
        Ok(match ty {
            Type::Void => Value::Void,
            Type::Boolean => Value::Boolean(false),
            Type::SByte => Value::SByte(0),
            Type::Byte => Value::Byte(0),
            Type::Int16 => Value::Int16(0),
            Type::UInt16 => Value::UInt16(0),
            Type::Char => Value::Char("\0".into()),
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
                crate::inheritance::require_concrete(module, ty)?;
                let definitions = crate::vm::record_fields(module, ty, 0)?;
                path.push(ty.clone());
                let fields = definitions
                    .iter()
                    .map(|field| build(module, &field.ty, path, remaining, allow_uninitialized))
                    .collect::<Result<Vec<_>, _>>()?;
                path.pop();
                if allow_uninitialized
                    && fields
                        .iter()
                        .any(|field| matches!(field, Value::Uninitialized(_)))
                {
                    return Ok(Value::Uninitialized(ty.clone()));
                }
                Value::Object {
                    ty: ty.clone(),
                    fields,
                }
            }
            _ if allow_uninitialized => Value::Uninitialized(ty.clone()),
            _ => {
                return Err(Fault::new(format!(
                    "managed default initialization is not defined for {ty:?}"
                )));
            }
        })
    }
    build(module, ty, &mut vec![], &mut 16_384, allow_uninitialized)
}
