//! Definition tokens are scoped to their descriptive module, never provider objects.
use crate::metadata::{Function, TypeDef};
use crate::{Fault, Module, TypeIdentity};

fn token(table: u32, row: usize) -> Result<i32, Fault> {
    if row == 0 || row > 0x00ff_ffff {
        return Err(Fault::new("metadata token row overflow"));
    }
    Ok(((table << 24) | row as u32) as i32)
}
pub(crate) fn definition<'a>(module: &'a Module, identity: &TypeIdentity) -> Option<&'a TypeDef> {
    let id = match identity {
        TypeIdentity::FunctionReturn(_)
        | TypeIdentity::Tuple(_)
        | TypeIdentity::MethodGenericParameter { .. }
        | TypeIdentity::Function { .. }
        | TypeIdentity::Union(_)
        | TypeIdentity::Intersection(_) => {
            return None;
        }
        TypeIdentity::Definition { definition, .. }
        | TypeIdentity::GenericParameter { definition, .. } => definition,
        TypeIdentity::Array(t)
        | TypeIdentity::ArrayRef(t)
        | TypeIdentity::Ptr(t)
        | TypeIdentity::ByRef(t)
        | TypeIdentity::ReadOnlyByRef(t)
        | TypeIdentity::InterfaceRef(t) => return definition(module, t),
    };
    if let Some(row) = crate::runtime_lookup::definition_row(module, id) {
        return row.map(|row| &module.types[row]);
    }
    module
        .types
        .iter()
        .find(|d| d.definition.as_ref() == Some(id))
}
pub(crate) fn type_token(module: &Module, identity: &TypeIdentity) -> Result<i32, Fault> {
    if !matches!(identity, TypeIdentity::Definition { .. }) {
        return Ok(0);
    }
    let d =
        definition(module, identity).ok_or_else(|| Fault::new("missing type token definition"))?;
    if let Some(origin) = &d.origin {
        return Ok(origin.token as i32);
    }
    token(
        2,
        d.definition
            .as_ref()
            .ok_or_else(|| Fault::new("missing type identity"))?
            .index as usize
            + 1,
    )
}
pub(crate) fn member(
    module: &Module,
    owner: &TypeDef,
    index: usize,
    property: bool,
) -> Result<i32, Fault> {
    if let Some(origin) = &owner.origin {
        // Legacy producers recorded only instance-field origins. Missing static
        // tokens are absent metadata, never invented CLI table rows.
        if !property
            && index >= owner.fields.len()
            && origin.field_tokens.len() == owner.fields.len()
        {
            return Ok(0);
        }
        return (if property {
            &origin.property_tokens
        } else {
            &origin.field_tokens
        })
        .get(index)
        .copied()
        .map(|t| t as i32)
        .ok_or_else(|| Fault::new("missing source member token"));
    }
    let id = owner
        .definition
        .as_ref()
        .ok_or_else(|| Fault::new("missing declaring type identity"))?;
    let prior: usize = module
        .types
        .iter()
        .filter(|d| {
            d.definition
                .as_ref()
                .is_some_and(|other| other.module == id.module && other.index < id.index)
        })
        .map(|d| {
            if property {
                d.properties.len()
            } else {
                d.fields.len() + d.static_fields.len() + d.literal_fields.len()
            }
        })
        .sum();
    token(if property { 0x17 } else { 4 }, prior + index + 1)
}
pub(crate) fn method(function: &Function) -> Result<i32, Fault> {
    if let Some(origin) = &function.origin {
        return Ok(origin.token as i32);
    }
    token(
        6,
        function
            .definition
            .as_ref()
            .ok_or_else(|| Fault::new("missing method identity"))?
            .index as usize
            + 1,
    )
}
pub(crate) fn parameter(module: &Module, function: &Function, index: usize) -> Result<i32, Fault> {
    if let Some(origin) = &function.origin {
        return origin
            .parameter_tokens
            .get(index)
            .copied()
            .map(|t| t as i32)
            .ok_or_else(|| Fault::new("missing parameter token"));
    }
    let id = function
        .definition
        .as_ref()
        .ok_or_else(|| Fault::new("missing method identity"))?;
    let prior: usize = module
        .functions
        .iter()
        .filter(|f| {
            f.definition
                .as_ref()
                .is_some_and(|other| other.module == id.module && other.index < id.index)
        })
        .map(|f| f.parameters.len())
        .sum();
    token(8, prior + index + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::TypeDefId;

    fn module() -> Module {
        let mut module = crate::assemble(".module Tokens\n.type Cell\n.end\n.function Main() -> Int32\nldc.i4 0\nret\n.end\n.entry Main\n").unwrap();
        let template = module.types[0].clone();
        module.types.clear();
        for (source, revision, index) in [
            ("A", "1", 0),
            ("B", "1", 0),
            ("A", "2", 0),
            ("A", "1", 9),
            ("A", "1", 0),
        ] {
            let mut definition = template.clone();
            definition.definition = Some(TypeDefId {
                module: source.into(),
                revision: Some(revision.into()),
                index,
            });
            definition.name = format!("Cell{}", module.types.len());
            module.types.push(definition);
        }
        module
    }
    fn identity(id: TypeDefId) -> TypeIdentity {
        TypeIdentity::Definition {
            definition: id,
            arguments: vec![],
        }
    }
    #[test]
    fn indexed_definitions_preserve_module_revision_first_row_and_missing_results() {
        let mut module = module();
        let mut queries: Vec<_> = module
            .types
            .iter()
            .map(|d| identity(d.definition.clone().unwrap()))
            .collect();
        queries.push(identity(TypeDefId {
            module: "Missing".into(),
            revision: None,
            index: 0,
        }));
        queries.push(TypeIdentity::Array(Box::new(queries[1].clone())));
        queries.push(TypeIdentity::Tuple(vec![]));
        let expected: Vec<_> = queries
            .iter()
            .map(|id| definition(&module, id).map(|d| d.name.clone()))
            .collect();
        crate::runtime_lookup::with_module(&module, || {
            for (id, expected) in queries.iter().zip(&expected) {
                assert_eq!(definition(&module, id).map(|d| d.name.clone()), *expected);
            }
            assert_eq!(definition(&module, &queries[4]).unwrap().name, "Cell0");
            let other = super::tests::module();
            let result = std::panic::catch_unwind(|| {
                crate::runtime_lookup::with_module(&other, || {
                    assert_eq!(definition(&module, &queries[0]).unwrap().name, "Cell0");
                    panic!("restore scoped identity index");
                })
            });
            assert!(result.is_err());
            assert_eq!(definition(&module, &queries[0]).unwrap().name, "Cell0");
            std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        assert!(
                            crate::runtime_lookup::definition_row(
                                &module,
                                module.types[0].definition.as_ref().unwrap()
                            )
                            .is_none()
                        );
                        crate::runtime_lookup::with_module(&module, || {
                            assert_eq!(definition(&module, &queries[0]).unwrap().name, "Cell0")
                        });
                    })
                    .join()
                    .unwrap();
            });
        });
        module.types.swap(0, 4);
        crate::runtime_lookup::with_module(&module, || {
            assert_eq!(definition(&module, &queries[0]).unwrap().name, "Cell4")
        });
    }
    #[test]
    fn token_encoding_and_source_origin_are_unchanged_by_indexing() {
        let mut module = module();
        module.types[0].origin = Some(
            serde_json::from_str(
                r#"{"assembly":"A","module":"A.dll","name":"Original","token":33554449}"#,
            )
            .unwrap(),
        );
        crate::runtime_lookup::with_module(&module, || {
            assert_eq!(
                type_token(
                    &module,
                    &identity(module.types[0].definition.clone().unwrap())
                )
                .unwrap(),
                0x02000011
            );
            assert_eq!(
                type_token(
                    &module,
                    &identity(module.types[3].definition.clone().unwrap())
                )
                .unwrap(),
                0x0200000a
            );
            assert_eq!(
                type_token(&module, &TypeIdentity::Tuple(vec![])).unwrap(),
                0
            );
            assert_eq!(token(6, 4).unwrap(), 0x06000004);
            assert!(token(2, 0).is_err());
            assert!(token(2, 0x01000000).is_err());
        });
    }
    #[test]
    #[ignore = "isolated lookup cost experiment; not a whole-program throughput test"]
    fn definition_lookup_cost() {
        use std::{hint::black_box, time::Instant};
        let mut module = module();
        let template = module.types[0].clone();
        module.types.clear();
        for index in 0..10000 {
            let mut definition = template.clone();
            definition.definition.as_mut().unwrap().index = index;
            definition.name = format!("Cell{index}");
            module.types.push(definition);
        }
        let query = identity(module.types.last().unwrap().definition.clone().unwrap());
        for trial in 0..4 {
            let linear = || {
                let start = Instant::now();
                for _ in 0..20000 {
                    black_box(definition(black_box(&module), black_box(&query)).unwrap());
                }
                start.elapsed().as_secs_f64()
            };
            let indexed = || {
                let start = Instant::now();
                crate::runtime_lookup::with_module(&module, || {
                    black_box(definition(&module, &query).unwrap());
                    let prepared = start.elapsed().as_secs_f64();
                    let start = Instant::now();
                    for _ in 0..20000 {
                        black_box(definition(black_box(&module), black_box(&query)).unwrap());
                    }
                    (prepared, start.elapsed().as_secs_f64())
                })
            };
            let (slow, (preparation, warm)) = if trial % 2 == 0 {
                (linear(), indexed())
            } else {
                let fast = indexed();
                (linear(), fast)
            };
            eprintln!(
                "DEFINITION_LOOKUP {{\"trial\":{trial},\"linearSeconds\":{slow},\"preparationSeconds\":{preparation},\"indexedSeconds\":{warm}}}"
            );
        }
    }
}
