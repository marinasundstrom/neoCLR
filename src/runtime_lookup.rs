//! Scope-local candidate indexes for an immutably borrowed module.
//! Candidate indexes and bounded successful interface views avoid repeated work;
//! signature, identity and access checks stay in their existing callers. Loading/verification and execution each hold immutable
//! scopes; mutation between scopes rebuilds indexes rather than retaining stale rows.
use crate::metadata::{Module, Type};
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
    definitions: OnceCell<HashMap<crate::metadata::TypeDefId, usize>>,
    interfaces: RefCell<InterfaceClosures>,
}

#[derive(Default)]
struct InterfaceClosures {
    views: HashMap<Type, Rc<[Type]>>,
    retained_views: usize,
}

const INTERFACE_OWNERS: usize = 1024;
const INTERFACE_VIEWS: usize = 4096;

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
        definitions: OnceCell::new(),
        interfaces: RefCell::new(InterfaceClosures::default()),
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

// Successful transitive views depend only on the complete owner type and the
// immutably borrowed module. Keep errors and traversal in interfaces::closure;
// do not hold a cache borrow while that traversal calls other metadata helpers.
pub(crate) fn interface_closure(module: &Module, owner: &Type) -> Option<Rc<[Type]>> {
    ACTIVE.with(|active| {
        let active = active.borrow();
        let index = active.as_ref()?;
        if index.module != module as *const Module as usize {
            return None;
        }
        index.interfaces.borrow().views.get(owner).cloned()
    })
}

pub(crate) fn remember_interface_closure(module: &Module, owner: &Type, views: &[Type]) {
    ACTIVE.with(|active| {
        let active = active.borrow();
        let Some(index) = active.as_ref() else {
            return;
        };
        if index.module != module as *const Module as usize {
            return;
        }
        let mut cache = index.interfaces.borrow_mut();
        if cache.views.len() >= INTERFACE_OWNERS
            || views.len() > INTERFACE_VIEWS.saturating_sub(cache.retained_views)
            || cache.views.contains_key(owner)
        {
            return;
        }
        cache.retained_views += views.len();
        cache.views.insert(owner.clone(), views.into());
    });
}

/// Exact source definition identity, including module revision. The outer None
/// means no matching immutable scope; Some(None) means the identity is absent.
/// Keep the first row for duplicate identities, matching the linear fallback.
pub(crate) fn definition_row(
    module: &Module,
    identity: &crate::metadata::TypeDefId,
) -> Option<Option<usize>> {
    ACTIVE.with(|active| {
        let active = active.borrow();
        let index = active.as_ref()?;
        if index.module != module as *const Module as usize {
            return None;
        }
        let definitions = index.definitions.get_or_init(|| {
            let mut rows = HashMap::new();
            for (row, definition) in module.types.iter().enumerate() {
                if let Some(identity) = &definition.definition {
                    rows.entry(identity.clone()).or_insert(row);
                }
            }
            rows
        });
        Some(definitions.get(identity).copied())
    })
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

    #[test]
    fn interface_views_preserve_substitution_errors_and_scope_lifetimes() {
        let mut module = assemble(
            ".module Views\n.interface Root<T>\n.end\n.interface Left<T>\n.implements Root<T>\n.end\n.interface Right<T>\n.implements Root<T>\n.end\n.interface Diamond<T>\n.implements Left<T>\n.implements Right<T>\n.end\n.interface Related\n.implements Root<Self>\n.end\n.type class Cell<T>\n.implements Diamond<T>\n.implements Related\n.end\n.interface Cycle\n.end\n",
        ).unwrap();
        module
            .types
            .iter_mut()
            .find(|t| t.name == "Cycle")
            .unwrap()
            .implements
            .push(Type::Named("Cycle".into()));
        let other =
            assemble(".module Other\n.interface Root<T>\n.end\n.type class Cell<T>\n.end\n")
                .unwrap();
        let parse = |s| crate::assembler::parse_type(s).unwrap();
        let owners = [
            parse("Cell<Int32>"),
            parse("Cell<String>"),
            parse("Related"),
            parse("Cycle"),
            parse("Missing"),
            parse("arrayref<Int32>"),
        ];
        let results: Vec<_> = owners
            .iter()
            .map(|ty| crate::interfaces::closure(&module, ty).map_err(|e| e.message))
            .collect();
        assert!(
            results[0]
                .as_ref()
                .unwrap()
                .contains(&parse("Root<Cell<Int32>>"))
        );
        assert!(results[2].as_ref().unwrap().contains(&parse("Root<Self>")));
        assert!(results[3].is_err());
        with_module(&module, || {
            for _ in 0..2 {
                for (ty, expected) in owners.iter().zip(&results) {
                    assert_eq!(
                        &crate::interfaces::closure(&module, ty).map_err(|e| e.message),
                        expected
                    );
                    assert_eq!(interface_closure(&module, ty).is_some(), expected.is_ok());
                }
            }
            with_module(&other, || {
                assert!(
                    crate::interfaces::closure(&other, &owners[0])
                        .unwrap()
                        .is_empty()
                );
                assert!(interface_closure(&module, &owners[0]).is_none());
            });
            assert_eq!(
                crate::interfaces::closure(&module, &owners[0]).unwrap(),
                *results[0].as_ref().unwrap()
            );
        });
        assert!(interface_closure(&module, &owners[0]).is_none());
        module
            .types
            .iter_mut()
            .find(|t| t.name == "Cell")
            .unwrap()
            .implements
            .clear();
        with_module(&module, || {
            assert!(
                crate::interfaces::closure(&module, &owners[0])
                    .unwrap()
                    .is_empty()
            );
        });
    }

    #[test]
    fn interface_view_limits_fall_back_without_changing_results() {
        let module = assemble(".module Limits\n.interface Root<T>\n.end\n").unwrap();
        with_module(&module, || {
            for index in 0..=INTERFACE_OWNERS {
                let ty = Type::Constructed {
                    definition: "Root".into(),
                    arguments: vec![Type::TypeParameter(index as u16)],
                };
                assert_eq!(
                    crate::interfaces::closure(&module, &ty).unwrap(),
                    vec![ty.clone()]
                );
                assert_eq!(
                    interface_closure(&module, &ty).is_some(),
                    index < INTERFACE_OWNERS
                );
            }
        });
        with_module(&module, || {
            let large = vec![Type::Int32; INTERFACE_VIEWS];
            remember_interface_closure(&module, &Type::Int32, &large);
            remember_interface_closure(&module, &Type::String, &[Type::String]);
            assert!(interface_closure(&module, &Type::String).is_none());
            assert!(interface_closure(&module, &Type::Int32).is_some());
        });
    }

    fn module(value: i32) -> Module {
        assemble(&format!(
            ".module Example\n.function Main() -> Int32\nldc.i4 {value}\nret\n.end\n.entry Main\n"
        ))
        .unwrap()
    }

    #[test]
    fn definition_index_is_lazy_and_does_not_build_owner_candidates() {
        let module = module(0);
        let identity = crate::metadata::TypeDefId {
            module: "absent".into(),
            revision: None,
            index: 0,
        };
        module.with_metadata_lookups(|module| {
            ACTIVE.with(|active| {
                assert!(
                    active
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .definitions
                        .get()
                        .is_none()
                )
            });
            assert_eq!(definition_row(module, &identity), Some(None));
            ACTIVE.with(|active| {
                let active = active.borrow();
                let index = active.as_ref().unwrap();
                assert!(index.definitions.get().is_some());
                assert!(index.owners.get().is_none());
            });
        });
        assert_eq!(definition_row(&module, &identity), None);
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
        module.with_metadata_lookups(|module| {
            ACTIVE.with(|active| assert!(active.borrow().as_ref().unwrap().owners.get().is_none()));
            check(module);
            let rows = owned_functions(module, &module.types[0]).unwrap();
            assert_eq!(&*rows, &[2, 4]);
            assert!(Rc::ptr_eq(
                &rows,
                &owned_functions(module, &module.types[0]).unwrap()
            ));
            let other = super::tests::module(1);
            let caught = std::panic::catch_unwind(|| {
                other.with_metadata_lookups(|_other| {
                    assert!(owned_functions(module, &module.types[0]).is_none());
                    panic!("restore owner scope");
                })
            });
            assert!(caught.is_err());
            assert!(Rc::ptr_eq(
                &rows,
                &owned_functions(module, &module.types[0]).unwrap()
            ));
            std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        assert!(owned_functions(module, &module.types[0]).is_none());
                        module.with_metadata_lookups(|module| check(module));
                    })
                    .join()
                    .unwrap();
            });
        });
        module.functions[2].owner = Some(crate::assembler::parse_type("Cell<Int32>").unwrap());
        module.with_metadata_lookups(|module| {
            check(module);
            assert_eq!(&*owned_functions(module, &module.types[0]).unwrap(), &[4]);
        });
    }

    #[test]
    fn scopes_restore_after_nested_modules_and_unwinding() {
        let first = module(1);
        let second = module(2);
        assert!(functions(&first, "Main").is_none());
        first.with_metadata_lookups(|first| {
            assert_eq!(&*functions(first, "Main").unwrap(), &[0]);
            let rows = functions(first, "Main").unwrap();
            first.with_metadata_lookups(|first| {
                assert!(Rc::ptr_eq(&rows, &functions(first, "Main").unwrap()));
            });
            assert!(functions(&second, "Main").is_none());
            let caught = std::panic::catch_unwind(|| {
                second.with_metadata_lookups(|_second| {
                    assert!(functions(first, "Main").is_none());
                    panic!("unwind nested lookup");
                })
            });
            assert!(caught.is_err());
            assert_eq!(&*functions(first, "Main").unwrap(), &[0]);
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
        module.with_metadata_lookups(|module| {
            assert_eq!(
                crate::vm::resolve(module, &target).unwrap().returns,
                Type::Int32
            )
        });
        module.functions.push(module.functions[0].clone());
        let expected = crate::vm::resolve(&module, &target).unwrap_err();
        module.with_metadata_lookups(|module| {
            assert_eq!(&*functions(module, "Main").unwrap(), &[0, 1]);
            assert_eq!(
                crate::vm::resolve(module, &target).unwrap_err().message,
                expected.message
            );
            assert!(functions(module, "Absent").unwrap().is_empty());
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
        module.with_metadata_lookups(|module| {
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
                        assert!(super::types(module, "Cell").is_none());
                        module.with_metadata_lookups(|module| {
                            assert_eq!(&*super::types(module, "Cell").unwrap(), &[0, 1])
                        });
                    })
                    .join()
                    .unwrap();
            });
        });
    }
}
