//! Owned host input schemas, independent of native memory layout.
use crate::{Fault, Module, Value, metadata::Type};

#[derive(Debug)]
pub(crate) enum Input {
    Primitive(Type),
    Erased,
    Record { ty: Type, fields: Vec<Input> },
}

pub(crate) fn resolve(module: &Module, parameters: &[Type]) -> Result<Vec<Input>, Fault> {
    resolve_with_budget(module, parameters, &mut 16_384)
}

fn resolve_with_budget(
    module: &Module,
    parameters: &[Type],
    remaining: &mut usize,
) -> Result<Vec<Input>, Fault> {
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
        if *ty == Type::RuntimeTypeHandle {
            return Err(Fault::new(
                "runtime type handles cannot be imported from the host",
            ));
        }
        if *ty == Type::Value {
            return Ok(Input::Erased);
        }
        if ty.is_primitive() {
            return Ok(Input::Primitive(ty.clone()));
        }
        if active.contains(ty) {
            return Err(Fault::new("recursive by-value host input schema"));
        }
        active.push(ty.clone());
        let schema = match ty {
            Type::Named(_) | Type::Constructed { .. } => Input::Record {
                ty: ty.clone(),
                fields: crate::vm::record_fields(module, ty, 0)?
                    .iter()
                    .map(|field| build(module, &field.ty, depth + 1, remaining, active))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            _ => {
                return Err(Fault::new(
                    "host input requires owned primitives or records; pointer and Ref inputs are not supported",
                ));
            }
        };
        active.pop();
        Ok(schema)
    }
    let mut active = Vec::new();
    parameters
        .iter()
        .map(|ty| build(module, ty, 0, remaining, &mut active))
        .collect()
}

struct ImportBudget {
    values: usize,
    schemas: usize,
}

impl Input {
    pub(crate) fn import(&self, module: &Module, value: Value) -> Result<Value, Fault> {
        self.import_checked(
            module,
            value,
            &mut ImportBudget {
                values: 16_384,
                schemas: 16_384,
            },
            0,
        )
    }

    fn import_checked(
        &self,
        module: &Module,
        value: Value,
        budget: &mut ImportBudget,
        depth: usize,
    ) -> Result<Value, Fault> {
        if depth > 64 || budget.values == 0 {
            return Err(Fault::new(
                "host input value exceeds depth or complexity limit",
            ));
        }
        budget.values -= 1;
        match self {
            Input::Erased => {
                let Value::Erased(payload) = value else {
                    return Err(Fault::new("expected explicit System.Value payload"));
                };
                let ty = match payload.as_ref() {
                    Value::Object { ty, .. } => crate::scope::normalize_type(module, ty)?,
                    Value::Pointer(_) | Value::Reference { .. } => {
                        return Err(Fault::new(
                            "pointer and Ref host payloads are not supported",
                        ));
                    }
                    other => other.ty(),
                };
                crate::vm::check_type(&ty, module)?;
                let schema = resolve_with_budget(module, &[ty], &mut budget.schemas)?.remove(0);
                let payload = schema.import_checked(module, *payload, budget, depth + 1)?;
                Ok(Value::Erased(Box::new(payload)))
            }
            Input::Primitive(expected) => {
                if matches!(
                    value,
                    Value::Object { .. }
                        | Value::Erased(_)
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
                        field
                            .import_checked(module, value, budget, depth + 1)
                            .map_err(|error| {
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
