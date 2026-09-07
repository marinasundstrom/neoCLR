//! Owned host input schemas, independent of native memory layout.
use crate::{Fault, Module, Value, metadata::Type};

#[derive(Debug)]
pub(crate) enum Input {
    Primitive(Type),
    Record { ty: Type, fields: Vec<Input> },
}

pub(crate) fn resolve(module: &Module, parameters: &[Type]) -> Result<Vec<Input>, Fault> {
    fn build(
        module: &Module,
        ty: &Type,
        depth: usize,
        remaining: &mut usize,
        active: &mut Vec<Type>,
    ) -> Result<Input, Fault> {
        if depth > 64 || *remaining == 0 {
            return Err(Fault::new(
                "host input schema exceeds depth or complexity limit",
            ));
        }
        *remaining -= 1;
        if ty.is_primitive() {
            return Ok(Input::Primitive(ty.clone()));
        }
        if !matches!(ty, Type::Named(_) | Type::Constructed { .. }) {
            return Err(Fault::new(
                "host input requires primitives or pointer-free records; Ref and union inputs are not yet supported",
            ));
        }
        if active.contains(ty) {
            return Err(Fault::new("recursive by-value host input schema"));
        }
        active.push(ty.clone());
        let fields = crate::vm::record_fields(module, ty, 0)?
            .iter()
            .map(|field| build(module, &field.ty, depth + 1, remaining, active))
            .collect::<Result<Vec<_>, _>>()?;
        active.pop();
        Ok(Input::Record {
            ty: ty.clone(),
            fields,
        })
    }
    let mut remaining = 16_384;
    let mut active = Vec::new();
    parameters
        .iter()
        .map(|ty| build(module, ty, 0, &mut remaining, &mut active))
        .collect()
}

impl Input {
    pub(crate) fn import(&self, module: &Module, value: Value) -> Result<Value, Fault> {
        match self {
            Input::Primitive(expected) => {
                if matches!(
                    value,
                    Value::Object { .. }
                        | Value::Union { .. }
                        | Value::Pointer(_)
                        | Value::Reference { .. }
                ) {
                    return Err(Fault::new(format!("expected primitive {expected:?}")));
                }
                if value.ty() != *expected {
                    return Err(Fault::new(format!(
                        "expected {expected:?}, got {:?}",
                        value.ty()
                    )));
                }
                Ok(value)
            }
            Input::Record {
                ty: expected,
                fields: schema,
            } => {
                let Value::Object { ty, fields } = value else {
                    return Err(Fault::new(format!("expected record {expected:?}")));
                };
                let actual = crate::scope::normalize_type(module, &ty)?;
                if actual != *expected {
                    return Err(Fault::new(format!(
                        "expected record {expected:?}, got {actual:?}"
                    )));
                }
                if fields.len() != schema.len() {
                    return Err(Fault::new(format!(
                        "record {expected:?} requires {} fields, got {}",
                        schema.len(),
                        fields.len()
                    )));
                }
                let fields = fields
                    .into_iter()
                    .zip(schema)
                    .enumerate()
                    .map(|(index, (value, field))| {
                        field.import(module, value).map_err(|error| {
                            Fault::new(format!("field {index}: {}", error.message))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::Object {
                    ty: expected.clone(),
                    fields,
                })
            }
        }
    }
}
