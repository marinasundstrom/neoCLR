//! Scope-local candidate indexes for an immutably borrowed module.
//! Indexes only narrow the search; signature, identity and access checks stay in
//! their existing callers. Loading/verification and execution each hold immutable
//! scopes; mutation between scopes rebuilds indexes rather than retaining stale rows.
use crate::metadata::Module;
use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
    rc::Rc,
};

struct Index {
    module: usize,
    types: HashMap<String, Rc<[usize]>>,
    functions: HashMap<String, Rc<[usize]>>,
    owners: OnceCell<HashMap<usize, Rc<[usize]>>>,
}

thread_local! {
    static ACTIVE: RefCell<Option<Rc<Index>>> = const { RefCell::new(None) };
}

fn grouped(names: impl Iterator<Item = String>) -> HashMap<String, Rc<[usize]>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, name) in names.enumerate() {
        groups.entry(name).or_default().push(index);
    }
    groups
        .into_iter()
        .map(|(name, indexes)| (name, indexes.into()))
        .collect()
}

pub(crate) fn with_module<R>(module: &Module, operation: impl FnOnce() -> R) -> R {
    let address = module as *const Module as usize;
    if ACTIVE.with(|active| {
        active
            .borrow()
            .as_ref()
            .is_some_and(|index| index.module == address)
    }) {
        return operation();
    }
    // The immutable borrow lasts through operation, including unwinding. No
    // address is dereferenced or retained after the outermost scope returns.
    let index = Rc::new(Index {
        module: address,
        owners: OnceCell::new(),
        types: grouped(
            module
                .types
                .iter()
                .map(|definition| definition.name.clone()),
        ),
        functions: grouped(
            module
                .functions
                .iter()
                .map(|definition| definition.name.clone()),
        ),
    });
    struct Restore(Option<Rc<Index>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            ACTIVE.with(|active| *active.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(ACTIVE.with(|active| active.replace(Some(index))));
    operation()
}

fn candidates(module: &Module, name: &str, types: bool) -> Option<Rc<[usize]>> {
    ACTIVE.with(|active| {
        let active = active.borrow();
        let index = active.as_ref()?;
        if index.module != module as *const Module as usize {
            return None;
        }
        let groups = if types {
            &index.types
        } else {
            &index.functions
        };
        Some(groups.get(name).cloned().unwrap_or_default())
    })
}

pub(crate) fn types(module: &Module, name: &str) -> Option<Rc<[usize]>> {
    candidates(module, name, true)
}

pub(crate) fn functions(module: &Module, name: &str) -> Option<Rc<[usize]>> {
    candidates(module, name, false)
}

/// Narrow methods by resolved owner, preserving module order and exact type lookup
/// semantics (including generic arity and array backing). Built only on demand.
/// Addresses are identity keys within the immutable scope, never dereferenced.
pub(crate) fn owned_functions(
    module: &Module,
    owner: &crate::metadata::TypeDef,
) -> Option<Rc<[usize]>> {
    ACTIVE.with(|active| {
        let active = active.borrow();
        let index = active.as_ref()?;
        if index.module != module as *const Module as usize {
            return None;
        }
        let owners = index.owners.get_or_init(|| {
            let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
            for (row, function) in module.functions.iter().enumerate() {
                if let Some(owner) = function
                    .owner
                    .as_ref()
                    .and_then(|ty| module.type_definition(ty))
                {
                    groups
                        .entry(owner as *const _ as usize)
                        .or_default()
                        .push(row);
                }
            }
            groups
                .into_iter()
                .map(|(owner, rows)| (owner, rows.into()))
                .collect()
        });
        Some(
            owners
                .get(&(owner as *const _ as usize))
                .cloned()
                .unwrap_or_default(),
        )
    })
}

/// Candidate rows only: callers must still check ownership and their contract.
/// Outside a matching scope, preserve the complete linear candidate sequence.
pub(crate) fn functions_for_owner<'a>(
    module: &'a Module,
    owner: &crate::metadata::TypeDef,
) -> impl Iterator<Item = &'a crate::metadata::Function> {
    let rows = owned_functions(module, owner);
    let count = rows
        .as_ref()
        .map_or(module.functions.len(), |rows| rows.len());
    (0..count).map(move |index| &module.functions[rows.as_ref().map_or(index, |rows| rows[index])])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assemble,
        metadata::{FunctionRef, Type},
    };

    fn module(value: i32) -> Module {
        assemble(&format!(
            ".module Example\n.function Main() -> Int32\nldc.i4 {value}\nret\n.end\n.entry Main\n"
        ))
        .unwrap()
    }

    #[test]
    fn owner_index_is_lazy_ordered_and_rebuilt_after_mutation() {
        let mut module = assemble(".module Owners\n.type Cell\n.end\n.type Cell<T>\n.end\n.function Main() -> Int32\nldc.i4 0\nret\n.end\n.entry Main\n").unwrap();
        let template = module.functions[0].clone();
        for owner in [
            "Cell<Int32>",
            "Cell",
            "Missing",
            "Cell",
            "Cell<Int32,String>",
        ] {
            let mut function = template.clone();
            function.owner = Some(crate::assembler::parse_type(owner).unwrap());
            module.functions.push(function);
        }
        let check = |module: &Module| {
            for definition in &module.types {
                let expected: Vec<_> = module
                    .functions
                    .iter()
                    .enumerate()
                    .filter_map(|(row, f)| {
                        f.owner
                            .as_ref()
                            .and_then(|ty| module.type_definition(ty))
                            .filter(|owner| std::ptr::eq(*owner, definition))
                            .map(|_| row)
                    })
                    .collect();
                assert_eq!(&*owned_functions(module, definition).unwrap(), expected);
            }
        };
        assert!(owned_functions(&module, &module.types[0]).is_none());
        with_module(&module, || {
            ACTIVE.with(|active| assert!(active.borrow().as_ref().unwrap().owners.get().is_none()));
            check(&module);
            let rows = owned_functions(&module, &module.types[0]).unwrap();
            assert_eq!(&*rows, &[2, 4]);
            assert!(Rc::ptr_eq(
                &rows,
                &owned_functions(&module, &module.types[0]).unwrap()
            ));
            let other = super::tests::module(1);
            let caught = std::panic::catch_unwind(|| {
                with_module(&other, || {
                    assert!(owned_functions(&module, &module.types[0]).is_none());
                    panic!("restore owner scope");
                })
            });
            assert!(caught.is_err());
            assert!(Rc::ptr_eq(
                &rows,
                &owned_functions(&module, &module.types[0]).unwrap()
            ));
            std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        assert!(owned_functions(&module, &module.types[0]).is_none());
                        with_module(&module, || check(&module));
                    })
                    .join()
                    .unwrap();
            });
        });
        module.functions[2].owner = Some(crate::assembler::parse_type("Cell<Int32>").unwrap());
        with_module(&module, || {
            check(&module);
            assert_eq!(&*owned_functions(&module, &module.types[0]).unwrap(), &[4]);
        });
    }

    #[test]
    fn scopes_restore_after_nested_modules_and_unwinding() {
        let first = module(1);
        let second = module(2);
        assert!(functions(&first, "Main").is_none());
        with_module(&first, || {
            assert_eq!(&*functions(&first, "Main").unwrap(), &[0]);
            assert!(functions(&second, "Main").is_none());
            let caught = std::panic::catch_unwind(|| {
                with_module(&second, || {
                    assert!(functions(&first, "Main").is_none());
                    panic!("unwind nested lookup");
                })
            });
            assert!(caught.is_err());
            assert_eq!(&*functions(&first, "Main").unwrap(), &[0]);
        });
        assert!(functions(&first, "Main").is_none());
    }

    #[test]
    fn mutations_between_scopes_and_duplicate_candidates_keep_linear_semantics() {
        let mut module = module(1);
        let target = FunctionRef {
            definition: None,
            name: "Main".into(),
            owner: None,
            instance: false,
            generic_arguments: vec![],
            parameters: vec![],
        };
        with_module(&module, || {
            assert_eq!(
                crate::vm::resolve(&module, &target).unwrap().returns,
                Type::Int32
            )
        });
        module.functions.push(module.functions[0].clone());
        let expected = crate::vm::resolve(&module, &target).unwrap_err();
        with_module(&module, || {
            assert_eq!(&*functions(&module, "Main").unwrap(), &[0, 1]);
            assert_eq!(
                crate::vm::resolve(&module, &target).unwrap_err().message,
                expected.message
            );
            assert!(functions(&module, "Absent").unwrap().is_empty());
        });
    }
    #[test]
    fn type_arity_missing_types_and_threads_match_uncached_lookup() {
        let module = assemble(".module Types\n.type Cell\n.end\n.type Cell<T>\n.end\n").unwrap();
        let types = [
            Type::from_name("Cell"),
            crate::assembler::parse_type("Cell<Int32>").unwrap(),
            crate::assembler::parse_type("Cell<Int32,String>").unwrap(),
            Type::from_name("Absent"),
        ];
        let expected: Vec<_> = types
            .iter()
            .map(|ty| {
                module
                    .type_definition(ty)
                    .map(|definition| serde_json::to_value(definition).unwrap())
            })
            .collect();
        with_module(&module, || {
            for (ty, expected) in types.iter().zip(&expected) {
                assert_eq!(
                    module
                        .type_definition(ty)
                        .map(|definition| serde_json::to_value(definition).unwrap()),
                    *expected
                );
            }
            std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        assert!(super::types(&module, "Cell").is_none());
                        with_module(&module, || {
                            assert_eq!(&*super::types(&module, "Cell").unwrap(), &[0, 1])
                        });
                    })
                    .join()
                    .unwrap();
            });
        });
    }
}
